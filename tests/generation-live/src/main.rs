//! Explicitly invoked, one charged request, synthetic content only. Never part of cargo test.
#[path="../../../meetily/frontend/src-tauri/src/classcatch/core.rs"]mod core;
#[path="../../../meetily/frontend/src-tauri/src/classcatch/generation.rs"]mod generation;
#[path="../../../meetily/frontend/src-tauri/src/classcatch/prompts.rs"]mod prompts;
#[path="../../../meetily/frontend/src-tauri/src/classcatch/transport.rs"]mod transport;
use serde_json::{json,Value};use std::{path::Path,time::Instant};
#[tokio::main]async fn main(){if std::env::var("RUN_LLM_LIVE_TEST").as_deref()!=Ok("1"){eprintln!("Live test is opt-in and makes one API request.");std::process::exit(2)}if let Err(e)=run().await{eprintln!("{}",e);std::process::exit(1)}}
async fn run()->Result<(),String>{
 let root=Path::new("G:/MeetingAssistant");let cfg:Value=serde_json::from_slice(&std::fs::read(root.join("data/settings.json")).map_err(|_|"Settings unavailable")?).map_err(|_|"Settings invalid")?;
 let base=cfg["llm_base_url"].as_str().ok_or("Missing endpoint")?;
 if base.trim_end_matches('/')!="https://dashscope.aliyuncs.com/compatible-mode/v1"||cfg["processing_mode"]!="cloud"{return Err("Configured endpoint/mode differs from the intended test; no request sent".into())}
 let key=keyring::Entry::new("MeetingAssistant","llm").map_err(|_|"Credential storage unavailable")?.get_password().map_err(|_|"Saved summary credential unavailable")?;
 let paragraphs=[
 "今天讲条件概率与贝叶斯定理。条件概率P(A|B)表示已经知道B发生时A发生的概率。它与P(B|A)不同，交换条件和结果通常会改变数值。",
 "先验概率是在新证据到来前对事件概率的判断。后验概率是结合新证据之后更新的判断。这里的更新依靠概率计算，而不是只凭直觉。",
 "贝叶斯公式把似然与先验相乘，再除以证据的总体概率。分母用于归一化，保证不同可能性对应的后验概率总和为一。",
 "现在看医学检测示例。假设一万人中有百分之一患病，也就是一百人患病、九千九百人没有患病。这里只是课堂虚构的数字，不是医疗建议。",
 "假设检测的灵敏度是百分之九十：一百名患者中约九十人检测阳性，十人检测阴性。灵敏度回答患病者被检测出来的比例。",
 "假设特异度也是百分之九十：九千九百名未患病者中约八千九百一十人阴性，还有九百九十人假阳性。不能忽略这个大基数。",
 "于是所有阳性者约一千零八十人，其中真正患病的约九十人。阳性后患病的比例为九十除以一千零八十，大约百分之八点三。",
 "这个例子说明：疾病发生率较低时，即使检测灵敏度不低，阳性结果也不能直接等同于患病。理解这种差别需要同时考虑基础发生率与假阳性。",
 "最后区分三个概念：先验是更新前，似然衡量假设成立时观察到证据的可能性，后验是看到证据后。接下来课堂将用同样方法计算另一个虚构检测例子。"
 ];
 let evidence:Vec<core::Segment>=paragraphs.iter().enumerate().map(|(i,t)|core::Segment{id:format!("synthetic-long-segment-identifier-00000000-0000-4000-8000-{:012}",i),start_ms:i as u64*60_000,end_ms:i as u64*60_000+50_000,text:t.to_string(),is_final:true,revision:1}).collect();
 let req=core::CatchupRequest{anchor_ms:600_000,lookback_ms:600_000,mode:"ask".into(),question:"请用中文连贯总结本段课堂，约500字，说明概念关系与例子的计算过程。".into()};
 let(actual,evidence)=core::prepare_window(&evidence,&req,true)?;let(wire,aliases)=generation::model_evidence(&evidence,"ask");let limit=generation::output_budget("ask",false);
 let body=json!({"model":cfg["llm_model"],"messages":[{"role":"system","content":prompts::ASK_PROMPT},{"role":"user","content":json!({"request":actual,"transcript":wire}).to_string()}],"temperature":0.2,"max_tokens":limit,"response_format":{"type":"json_object"},"enable_thinking":false});
 let started=Instant::now();let response=transport::post_json(&transport::client("ask")?,&format!("{}/chat/completions",base.trim_end_matches('/')),&key,&body,120).await.map_err(|f|f.message)?;
 let finish=response["choices"][0]["finish_reason"].as_str().unwrap_or("unknown");if finish=="length"{return Err("Live model still truncated output; no success claimed".into())}
 let content=response["choices"][0]["message"]["content"].as_str().ok_or("No model content")?;let mut card:core::Card=serde_json::from_str(content).map_err(|_|"Live reply was not a complete card")?;
 generation::restore_references(&mut card,&aliases)?;core::validate_card_for_mode(&card,&evidence,"ask")?;
 let answer=card.answer.ok_or("Missing answer")?;
 let report=json!({"passed":true,"synthetic_evidence_only":true,"cloud_calls":1,"retries":0,"microphone_opened":false,"model":cfg["llm_model"],"finish_reason":finish,"max_output_tokens":limit,"completion_tokens":response["usage"]["completion_tokens"],"answer_chars":answer.text.chars().count(),"unique_answer_references":answer.segment_ids.len(),"reference_ids_restored":answer.segment_ids.iter().all(|id|evidence.iter().any(|s|&s.id==id)),"effective_anchor_ms":actual.anchor_ms,"elapsed_ms":started.elapsed().as_millis()});
 std::fs::write(root.join("logs/generation-live-test.json"),serde_json::to_vec_pretty(&report).unwrap()).map_err(|_|"Cannot save test report")?;println!("{}",report);Ok(())
}
