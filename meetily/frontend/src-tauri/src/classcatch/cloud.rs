//! Alibaba Qwen-ASR Realtime adapter. Audio is already archived by Meetily.
use super::dsp::{Downsampler,Span,map_time,map_time_end};
use super::{core::Segment, service::*};
use crate::audio::{recording_state::AudioChunk, transcription::worker::TranscriptUpdate};
use base64::{engine::general_purpose::STANDARD, Engine};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::{collections::HashMap, time::Duration};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tokio_tungstenite::{connect_async, tungstenite::{client::IntoClientRequest, http::HeaderValue, Message}};

fn setup_message(settings:&Settings)->Value {
    let mut transcription=json!({});
    if !settings.vocabulary.trim().is_empty(){transcription["corpus"]=json!({"text":settings.vocabulary});}
    json!({"event_id":uuid::Uuid::new_v4().to_string(),"type":"session.update","session":{
        "input_audio_format":"pcm","sample_rate":16000,"input_audio_transcription":transcription,
        "turn_detection":{"type":"server_vad","threshold":0.2,"silence_duration_ms":800}}})
}
async fn connect(settings:&Settings)->Result<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,String> {
    let key=secret("asr")?;
    let mut url=url::Url::parse("wss://dashscope.aliyuncs.com/api-ws/v1/realtime").map_err(|e|e.to_string())?;
    url.query_pairs_mut().append_pair("model",&settings.asr_model);
    let mut req=url.as_str().into_client_request().map_err(|_|"识别连接参数无效".to_string())?;
    req.headers_mut().insert("Authorization",HeaderValue::from_str(&format!("Bearer {}",key)).map_err(|_|"密钥格式不正确".to_string())?);
    let (mut socket,_)=tokio::time::timeout(Duration::from_secs(15),connect_async(req)).await
        .map_err(|_|"阿里识别连接超时".to_string())?.map_err(|_|"阿里识别连接失败，请检查北京地域密钥、网络和模型权限".to_string())?;
    socket.send(Message::Text(setup_message(settings).to_string())).await.map_err(|_|"发送识别配置失败".to_string())?;
    tokio::time::timeout(Duration::from_secs(15),async {
        while let Some(message)=socket.next().await {
            let message=message.map_err(|_|"识别配置连接中断".to_string())?;
            if let Message::Text(text)=message {
                let v:Value=serde_json::from_str(&text).map_err(|_|"识别服务返回格式错误".to_string())?;
                match v["type"].as_str().unwrap_or("") {
                    "session.updated"=>return Ok(()),
                    "error"=>return Err(format!("识别配置被拒绝：{}",v["error"]["code"].as_str().unwrap_or("unknown"))),
                    _=>{}
                }
            }
        }
        Err("识别服务在配置完成前断开".into())
    }).await.map_err(|_|"等待识别配置确认超时".to_string())??;
    Ok(socket)
}
pub async fn test_connection(settings:&Settings)->Result<(),String> {
    let mut socket=connect(settings).await?;
    socket.send(Message::Text(json!({"type":"session.finish","event_id":uuid::Uuid::new_v4().to_string()}).to_string())).await.map_err(|_|"关闭测试会话失败".to_string())?;
    let _=socket.close(None).await;Ok(())
}

pub(super) async fn report_gap<R:Runtime>(app:&AppHandle<R>,id:&str,message:&str){
    let class_ms=app.state::<ClassroomState>().active.lock().ok()
        .and_then(|g|g.as_ref().map(|s|s.started.elapsed().as_millis() as u64)).unwrap_or(0);
    let payload=json!({"type":"class.asr.gap","class_ms":class_ms,"message":message});
    if let Ok(db)=database().await {
        if sqlx::query("INSERT INTO class_events(session_id,payload) SELECT ?,? WHERE EXISTS(SELECT 1 FROM class_sessions WHERE id=?)").bind(id).bind(payload.to_string()).bind(id).execute(db).await.is_err(){
            let _=app.emit("class-asr-status",json!({"state":"error","message":"转写缺口记录保存失败，请核对磁盘"}));
        }
    }
    let _=app.emit("class-asr-status",json!({"state":"gap","message":message}));
}

pub async fn run<R:Runtime>(app:AppHandle<R>,mut upstream:tokio::sync::mpsc::UnboundedReceiver<AudioChunk>) {
    if load_settings().processing_mode=="local" {super::local::run(app,upstream).await;return;}
    let id=match app.state::<ClassroomState>().active.lock().ok().and_then(|s|s.as_ref().map(|s|s.id.clone())) {Some(id)=>id,None=>return};
    let (tx,mut rx)=tokio::sync::mpsc::channel::<AudioChunk>(8);
    let bridge_app=app.clone();
    let bridge_id=id.clone();
    let bridge=tokio::spawn(async move {
        while let Some(chunk)=upstream.recv().await {
            if tx.try_send(chunk).is_err(){report_gap(&bridge_app,&bridge_id,"网络处理落后，部分实时音频未发送；原始录音继续保存，请稍后回放").await;}
        }
    });
    let mut attempts=0u32;
    'connections: loop {
        let settings=load_settings();
        let mut socket=match connect(&settings).await {Ok(socket)=>socket,Err(message)=>{
            let _=app.emit("class-asr-status",json!({"state":"reconnecting","message":message}));
            report_gap(&app,&id,"识别连接中断，部分音频尚未转写；请保留录音并核对原文").await;
            attempts+=1;tokio::time::sleep(Duration::from_secs((attempts as u64).min(8))).await;
            if rx.is_closed(){
                let _=app.emit("class-asr-status",json!({"state":"gap","message":"已停止录课；网络未恢复，末尾音频未完成转写，请保留录音回放"}));
                break;
            }continue;
        }};
        attempts=0;
        let connection_id=uuid::Uuid::new_v4().to_string();
        let mut ds=Downsampler::new();let mut spans:Vec<Span>=vec![];let mut samples_sent=0u64;
        let mut bounds:HashMap<String,(u64,u64)>=HashMap::new();let mut revisions:HashMap<String,u32>=HashMap::new();
        let mut finishing=false;let mut rotating=false;let mut finishing_at=std::time::Instant::now();
        let opened=std::time::Instant::now();
        let mut heartbeat=tokio::time::interval(Duration::from_secs(1));
        let _=app.emit("class-asr-status",json!({"state":"connected","message":"阿里实时识别已连接"}));
        loop {
            tokio::select! {
                chunk=rx.recv(),if !finishing=>{
                    match chunk {
                        Some(chunk)=>{
                            let pcm=match ds.pcm(&chunk.data,chunk.sample_rate){Ok(p)=>p,Err(message)=>{let _=app.emit("class-asr-status",json!({"state":"error","message":message}));continue;}};
                            let begin=samples_sent*1000/16000;samples_sent+=(pcm.len()/2) as u64;let end=samples_sent*1000/16000;
                            let class_end=(chunk.timestamp*1000.).max(0.) as u64;
                            spans.push(Span{asr_start:begin,asr_end:end,class_start:class_end.saturating_sub(end-begin),class_end});
                            // Keep provider messages below one second; pace at actual capture rate.
                            for piece in pcm.chunks(6400) {
                                let msg=json!({"event_id":uuid::Uuid::new_v4().to_string(),"type":"input_audio_buffer.append","audio":STANDARD.encode(piece)});
                                if tokio::time::timeout(Duration::from_secs(5),socket.send(Message::Text(msg.to_string()))).await.map_or(true,|r|r.is_err()) {
                                    report_gap(&app,&id,"识别推流中断，正在重连；录音继续保存").await;continue 'connections;
                                }
                            }
                        }
                        None=>{finishing=true;finishing_at=std::time::Instant::now();let _=socket.send(Message::Text(json!({"type":"session.finish","event_id":uuid::Uuid::new_v4().to_string()}).to_string())).await;}
                    }
                }
                msg=socket.next()=>{
                    let text=match msg {Some(Ok(Message::Text(t)))=>t,Some(Ok(Message::Ping(p)))=>{let _=socket.send(Message::Pong(p)).await;continue;},Some(Ok(_))=>continue,_=>{if finishing && !rotating {break 'connections;}if !rotating{report_gap(&app,&id,"识别连接意外关闭，未确认转写可能缺失，请核对录音").await;}continue 'connections;}};
                    let event:Value=match serde_json::from_str(&text){Ok(v)=>v,Err(_)=>continue};
                    if let Ok(db)=database().await {if let Err(e)=sqlx::query("INSERT INTO class_events(session_id,payload) SELECT ?,? WHERE EXISTS(SELECT 1 FROM class_sessions WHERE id=?)").bind(&id).bind(&text).bind(&id).execute(db).await {let _=app.emit("class-asr-status",json!({"state":"error","message":format!("实时事件保存失败：{}",e)}));}}
                    let item=event["item_id"].as_str().unwrap_or("").to_string();
                    match event["type"].as_str().unwrap_or("") {
                        "input_audio_buffer.speech_started"=>{bounds.entry(item).or_default().0=event["audio_start_ms"].as_u64().unwrap_or(samples_sent/16);},
                        "input_audio_buffer.speech_stopped"=>{bounds.entry(item).or_default().1=event["audio_end_ms"].as_u64().unwrap_or(samples_sent/16);},
                        "conversation.item.input_audio_transcription.text"|"conversation.item.input_audio_transcription.completed"=>{
                            let final_result=event["type"].as_str().unwrap_or("").ends_with(".completed");
                            let transcript=if final_result{event["transcript"].as_str().unwrap_or("").to_string()}
                                else{format!("{}{}",event["text"].as_str().unwrap_or(""),event["stash"].as_str().unwrap_or(""))};
                            if transcript.trim().is_empty(){continue;}
                            let (start,end)=bounds.get(&item).copied().unwrap_or((samples_sent.saturating_sub(16000)/16,samples_sent/16));
                            let revision=revisions.entry(item.clone()).or_default();*revision+=1;
                            let segment=Segment{id:format!("{}:{}",connection_id,item),start_ms:map_time(&spans,start),end_ms:map_time_end(&spans,if end>start{end}else{samples_sent/16}),text:transcript.clone(),is_final:final_result,revision:*revision};
                            if let Ok(db)=database().await {
                                if let Err(e)=sqlx::query("INSERT OR REPLACE INTO class_segments(session_id,id,payload) SELECT ?,?,? WHERE EXISTS(SELECT 1 FROM class_sessions WHERE id=?)").bind(&id).bind(&segment.id).bind(serde_json::to_string(&segment).unwrap_or_default()).bind(&id).execute(db).await {let _=app.emit("class-asr-status",json!({"state":"error","message":format!("转写保存失败：{}",e)}));}
                                let _=sqlx::query("UPDATE class_sessions SET elapsed_ms=MAX(elapsed_ms,?) WHERE id=?").bind(segment.end_ms as i64).bind(&id).execute(db).await;
                            }
                            let _=app.emit("class-transcript",json!({"session_id":id,"segment":segment}));
                            if final_result {
                                let sequence=uuid::Uuid::new_v4().as_u128() as u64;
                                let _=app.emit("transcript-update",TranscriptUpdate{text:transcript,timestamp:chrono::Local::now().format("%H:%M:%S").to_string(),source:"Aliyun Qwen-ASR".into(),sequence_id:sequence,chunk_start_time:segment.start_ms as f64/1000.,is_partial:false,confidence:0.,audio_start_time:segment.start_ms as f64/1000.,audio_end_time:segment.end_ms as f64/1000.,duration:segment.end_ms.saturating_sub(segment.start_ms) as f64/1000.});
                            }
                        }
                        "error"|"conversation.item.input_audio_transcription.failed"=>{let _=app.emit("class-asr-status",json!({"state":"error","message":format!("识别服务错误：{}。录音继续保存。",event["error"]["code"].as_str().unwrap_or("unknown"))}));if !finishing {continue 'connections;}},
                        "session.finished"=>{if finishing && !rotating {break 'connections;}continue 'connections;},
                        _=>{}
                    }
                }
                _=heartbeat.tick()=>{
                    if finishing && finishing_at.elapsed()>Duration::from_secs(15){let _=app.emit("class-asr-status",json!({"state":"gap","message":"部分转写尚未确认，录音已保留"}));if rotating {continue 'connections;}break 'connections;}
                    if !finishing && opened.elapsed()>Duration::from_secs(45*60){finishing=true;rotating=true;finishing_at=std::time::Instant::now();let _=socket.send(Message::Text(json!({"type":"session.finish","event_id":uuid::Uuid::new_v4().to_string()}).to_string())).await;}
                }
            }
        }
    }
    bridge.abort();
    let _=app.emit("class-asr-status",json!({"state":"stopped","message":"实时识别已结束"}));
}

