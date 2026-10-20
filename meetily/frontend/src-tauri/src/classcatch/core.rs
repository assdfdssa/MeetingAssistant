use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub fn validate_processing_mode_change(previous:&str,next:&str,active:bool,busy:bool,acknowledged:bool)->Result<(),String>{
    if !["cloud","local"].contains(&next){return Err("未知处理方式".into());}
    if previous!=next {
        if active||busy{return Err("请先结束录课并等待当前摘要完成，再手动切换处理方式".into());}
        if next=="local"&&!acknowledged{return Err("请主动确认使用本地模型，失败时不会自动联网".into());}
    }
    Ok(())
}

#[test]
fn local_mode_requires_explicit_idle_selection(){
    assert!(validate_processing_mode_change("cloud","local",false,false,false).is_err());
    assert!(validate_processing_mode_change("cloud","local",true,false,true).is_err());
    assert!(validate_processing_mode_change("cloud","local",false,true,true).is_err());
    assert!(validate_processing_mode_change("cloud","local",false,false,true).is_ok());
    assert!(validate_processing_mode_change("local","cloud",false,false,false).is_ok());
    assert!(validate_processing_mode_change("local","cloud",true,false,false).is_err());
    assert!(validate_processing_mode_change("cloud","cloud",true,true,false).is_ok());
    assert!(validate_processing_mode_change("cloud","auto",false,false,true).is_err());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub id: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
    pub is_final: bool,
    pub revision: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatchupRequest {
    pub anchor_ms: u64,
    pub lookback_ms: u64,
    pub mode: String,
    #[serde(default)]
    pub question: String,
}

impl CatchupRequest {
    pub fn validate(&self) -> Result<(), String> {
        if !(10_000..=3_600_000).contains(&self.lookback_ms) {
            return Err("时间范围应为10秒至60分钟".into());
        }
        if !["catch_up", "tasks", "explain", "ask", "layered"].contains(&self.mode.as_str()) {
            return Err("未知补听模式".into());
        }
        if self.question.chars().count() > 2000 { return Err("问题过长".into()); }
        Ok(())
    }
}

pub fn select_window(segments: &[Segment], req: &CatchupRequest) -> Vec<Segment> {
    let start = req.anchor_ms.saturating_sub(req.lookback_ms);
    segments.iter().filter(|s| s.is_final && !s.text.trim().is_empty()
        && s.end_ms > start && s.start_ms < req.anchor_ms).cloned().collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub text: String,
    pub segment_ids: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub action: String,
    pub due_original: Option<String>,
    pub submission: Option<String>,
    pub evidence_quote: String,
    pub segment_ids: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub headline: Fact,
    pub key_points: Vec<Fact>,
    pub current_topic: Option<Fact>,
    pub tasks: Vec<Task>,
    pub explanation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer: Option<Fact>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layers: Vec<LearningLayer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningLayer {
    pub title: String,
    pub classroom: Fact,
    pub explanation: String,
    pub example: Option<String>,
    pub relation: Option<String>,
}

pub fn validate_card(card: &Card, evidence: &[Segment]) -> Result<(), String> {
    let ids: HashSet<&str> = evidence.iter().map(|s| s.id.as_str()).collect();
    let check = |text: &str, refs: &[String]| -> Result<(), String> {
        if text.trim().is_empty() || text.chars().count() > 1200 {
            return Err("模型返回空内容或过长内容".into());
        }
        if refs.is_empty() || refs.iter().any(|s| !ids.contains(s.as_str())) {
            return Err("模型引用了不存在的课堂片段，请重试".into());
        }
        Ok(())
    };
    check(&card.headline.text, &card.headline.segment_ids)?;
    if card.key_points.len() > 3 || card.tasks.len() > 30 {
        return Err("返回条目超出约定范围".into());
    }
    for fact in &card.key_points { check(&fact.text, &fact.segment_ids)?; }
    if let Some(fact) = &card.current_topic { check(&fact.text, &fact.segment_ids)?; }
    for task in &card.tasks {
        check(&task.action, &task.segment_ids)?;
        if task.evidence_quote.trim().is_empty() || !evidence.iter()
            .filter(|s| task.segment_ids.contains(&s.id))
            .any(|s| s.text.contains(&task.evidence_quote)) {
            return Err("任务原话无法在引用的转写中找到，请回放核实".into());
        }
        // Dates must remain in the original transcript, never inferred from model memory.
        if let Some(due) = &task.due_original {
            if due.trim().is_empty() || !evidence.iter().filter(|s| task.segment_ids.contains(&s.id)).any(|s| s.text.contains(due)) {
                return Err("截止时间不是引用片段中的原话".into());
            }
        }
        if let Some(submission) = &task.submission {
            if submission.trim().is_empty() || !evidence.iter().filter(|s| task.segment_ids.contains(&s.id)).any(|s| s.text.contains(submission)) {
                return Err("提交方式不是引用片段中的原话".into());
            }
        }
    }
    if card.explanation.as_ref().is_some_and(|s| s.chars().count() > 2400) {
        return Err("解释过长，请缩小范围".into());
    }
    if let Some(answer) = &card.answer { check(&answer.text, &answer.segment_ids)?; }
    if card.layers.len()>6 {return Err("分层重点过多，请缩小范围".into());}
    for layer in &card.layers {
        if layer.title.trim().is_empty() || layer.title.chars().count()>100 {return Err("分层重点标题无效".into());}
        check(&layer.classroom.text,&layer.classroom.segment_ids)?;
        if layer.explanation.trim().is_empty() || layer.explanation.chars().count()>1800
            || layer.example.as_ref().is_some_and(|s|s.chars().count()>1200)
            || layer.relation.as_ref().is_some_and(|s|s.chars().count()>1200) {
            return Err("分层解释为空或过长，请重试".into());
        }
    }
    Ok(())
}

pub fn validate_card_for_mode(card:&Card,evidence:&[Segment],mode:&str)->Result<(),String>{
    validate_card(card,evidence)?;
    if mode=="ask" && card.answer.is_none(){return Err("模型未返回提问正文，请重试".into());}
    if mode=="layered" && card.layers.is_empty(){return Err("模型未返回可展开重点，请重试".into());}
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn seg(id: &str, start_ms: u64, end_ms: u64, is_final: bool) -> Segment {
        Segment { id:id.into(), start_ms, end_ms, text:"下周三交报告，周五不用交".into(), is_final, revision:1 }
    }
    #[test]
    fn anchor_is_click_time_and_not_last_transcript() {
        let req = CatchupRequest { anchor_ms: 600_000, lookback_ms:60_000, mode:"catch_up".into(), question:String::new() };
        assert!(select_window(&[seg("old", 200_000,210_000,true)], &req).is_empty());
    }
    #[test]
    fn keeps_full_window_but_excludes_future_and_partial() {
        let req = CatchupRequest { anchor_ms:600_000, lookback_ms:300_000, mode:"tasks".into(), question:String::new() };
        let rows=vec![seg("before",1,299_999,true),seg("boundary",299_000,301_000,true),seg("middle",400_000,420_000,true),seg("future",601_000,602_000,true),seg("partial",500_000,501_000,false)];
        assert_eq!(select_window(&rows,&req).iter().map(|s|s.id.as_str()).collect::<Vec<_>>(),vec!["boundary","middle"]);
    }
    #[test]
    fn rejects_invented_deadlines_and_ids() {
        let mut card=Card { headline:Fact{text:"布置报告".into(),segment_ids:vec!["a".into()]},key_points:vec![],current_topic:None,tasks:vec![Task{action:"交报告".into(),due_original:Some("下周三".into()),submission:None,evidence_quote:"下周三交报告".into(),segment_ids:vec!["a".into()]}],explanation:None,answer:None,layers:vec![] };
        assert!(validate_card(&card,&[seg("a",1,10,true)]).is_ok());
        card.tasks[0].due_original=Some("明天".into());
        assert!(validate_card(&card,&[seg("a",1,10,true)]).is_err());
        card.tasks.clear(); card.headline.segment_ids=vec!["invented".into()];
        assert!(validate_card(&card,&[seg("a",1,10,true)]).is_err());
    }
    #[test]
    fn legacy_cards_still_load_and_new_modes_require_their_content(){
        let raw=r#"{"headline":{"text":"主题","segment_ids":["a"]},"key_points":[],"current_topic":null,"tasks":[],"explanation":null}"#;
        let mut card:Card=serde_json::from_str(raw).unwrap();let evidence=vec![seg("a",1,10,true)];
        assert!(validate_card_for_mode(&card,&evidence,"catch_up").is_ok());
        assert!(validate_card_for_mode(&card,&evidence,"ask").is_err());
        assert!(validate_card_for_mode(&card,&evidence,"layered").is_err());
        card.answer=Some(Fact{text:"所选课堂片段说明下周三交报告，周五不用交。".into(),segment_ids:vec!["a".into()]});
        assert!(validate_card_for_mode(&card,&evidence,"ask").is_ok());
        card.layers.push(LearningLayer{title:"作业时间".into(),classroom:card.headline.clone(),explanation:"白话解释".into(),example:None,relation:None});
        assert!(validate_card_for_mode(&card,&evidence,"layered").is_ok());
        card.layers[0].classroom.segment_ids=vec!["invented".into()];
        assert!(validate_card_for_mode(&card,&evidence,"layered").is_err());
    }
}
