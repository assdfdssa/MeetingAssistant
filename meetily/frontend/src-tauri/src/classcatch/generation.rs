use super::core::{Card,Segment};
use std::collections::{HashMap,HashSet};

pub fn output_budget(mode:&str,local:bool)->i32{
    match (mode,local){("ask",false)=>4096,("layered",false)=>6144,("ask",true)=>2200,("layered",true)=>3600,_=>1600}
}

/// New learning responses have no sentence-level citations. Keep legacy cards readable.
pub fn clear_references(card:&mut Card){
    card.headline.segment_ids.clear();
    for f in &mut card.key_points{f.segment_ids.clear();}
    if let Some(f)=&mut card.current_topic{f.segment_ids.clear();}
    if let Some(f)=&mut card.answer{f.segment_ids.clear();}
    for l in &mut card.layers{l.classroom.segment_ids.clear();}
    card.tasks.clear();
}

/// Long local UUIDs need not consume model output tokens. Restore them before validation/storage.
pub fn model_evidence(evidence:&[Segment],mode:&str)->(Vec<Segment>,HashMap<String,String>){
    if !["ask","layered"].contains(&mode){return(evidence.to_vec(),HashMap::new());}
    let mut aliases=HashMap::new();
    let rows=evidence.iter().enumerate().map(|(i,s)|{let mut row=s.clone();row.id=format!("S{}",i+1);aliases.insert(row.id.clone(),s.id.clone());row}).collect();
    (rows,aliases)
}
pub fn restore_references(card:&mut Card,aliases:&HashMap<String,String>)->Result<(),String>{
    if aliases.is_empty(){return Ok(());}
    let restore=|refs:&mut Vec<String>|->Result<(),String>{
        let mut seen=HashSet::new();let mut out=Vec::new();
        for id in refs.iter(){let real=aliases.get(id).ok_or("模型引用了不存在的课堂片段，请重试")?;
            if seen.insert(real.clone()){out.push(real.clone());}}
        if out.len()>8{return Err("模型引用条目过多，请缩小范围后重试".into());}
        *refs=out;Ok(())
    };
    restore(&mut card.headline.segment_ids)?;
    for f in &mut card.key_points{restore(&mut f.segment_ids)?;}
    if let Some(f)=&mut card.current_topic{restore(&mut f.segment_ids)?;}
    if let Some(f)=&mut card.answer{restore(&mut f.segment_ids)?;}
    for t in &mut card.tasks{restore(&mut t.segment_ids)?;}
    for l in &mut card.layers{restore(&mut l.classroom.segment_ids)?;}
    Ok(())
}
#[cfg(test)]mod tests{
    use super::*;use super::super::core::{validate_card_for_mode,Fact};
    fn fixture()->Vec<Segment>{(0..3).map(|i|Segment{id:format!("classroom-long-uuid-{i}-confirmed-segment"),start_ms:i*1000,end_ms:(i+1)*1000,text:"示例课堂内容".into(),is_final:true,revision:1}).collect()}
    fn card()->Card{Card{headline:Fact{text:"标题".into(),segment_ids:vec!["S1".into()]},key_points:vec![],current_topic:None,tasks:vec![],explanation:None,answer:Some(Fact{text:"一段有依据的回答".into(),segment_ids:vec!["S1".into(),"S1".into(),"S2".into()]}),layers:vec![]}}
    #[test]fn citation_free_answer_accepts_no_refs_but_still_requires_content(){let mut c=card();c.answer.as_mut().unwrap().segment_ids=(0..100).map(|i|format!("S{i}")).collect();clear_references(&mut c);assert!(c.answer.as_ref().unwrap().segment_ids.is_empty());assert!(validate_card_for_mode(&c,&fixture(),"ask").is_ok());assert!(validate_card_for_mode(&c,&fixture(),"catch_up").is_err());c.answer.as_mut().unwrap().text.clear();assert!(validate_card_for_mode(&c,&fixture(),"ask").is_err());}
    #[test]fn aliases_round_trip_and_duplicate_refs_are_removed(){let original=fixture();let(wire,map)=model_evidence(&original,"ask");assert_eq!(wire[0].id,"S1");assert_eq!(wire[0].text,original[0].text);let mut c=card();restore_references(&mut c,&map).unwrap();assert_eq!(c.answer.as_ref().unwrap().segment_ids,vec![original[0].id.clone(),original[1].id.clone()]);assert!(validate_card_for_mode(&c,&original,"ask").is_ok());}
    #[test]fn unknown_reference_is_not_silently_accepted(){let(_,map)=model_evidence(&fixture(),"ask");let mut c=card();c.answer.as_mut().unwrap().segment_ids.push("S999".into());assert!(restore_references(&mut c,&map).is_err());}
    #[test]fn catchup_and_local_budgets_are_preserved(){let rows=fixture();let(wire,map)=model_evidence(&rows,"catch_up");assert!(map.is_empty());assert_eq!(wire[0].id,rows[0].id);assert_eq!(output_budget("catch_up",false),1600);assert_eq!(output_budget("ask",true),2200);assert_eq!(output_budget("layered",true),3600);assert_eq!(output_budget("ask",false),4096);assert_eq!(output_budget("layered",false),6144);}
}
