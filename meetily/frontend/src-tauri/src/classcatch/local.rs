//! Explicit, offline-only fallback using already installed models. No downloads,
//! no API calls, and no automatic mode changes are performed here.
use super::{core::Segment,dsp::Downsampler,service::*};
use crate::audio::recording_state::AudioChunk;
use base64::Engine as _;
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use std::{path::PathBuf,sync::{Arc,atomic::{AtomicBool,Ordering}}};
use tauri::{AppHandle,Emitter,Manager,Runtime};
use tokio::{sync::{Mutex,OnceCell},io::{AsyncBufReadExt,AsyncWriteExt,BufReader},process::{Child,ChildStdin,ChildStdout,Command}};

const ASR:&str="qwen3-asr-1.7b-openvino";
const LLM:&str="qwen3.5:4b";
static TESTED:AtomicBool=AtomicBool::new(false);
static ENGINE:OnceCell<Arc<OpenVinoEngine>>=OnceCell::const_new();
static WORK:Mutex<()>=Mutex::const_new(());

#[derive(Clone,Serialize,Deserialize)]
pub struct Paths {pub asr_base:PathBuf,pub llm_root:PathBuf,#[serde(default="default_runtime")]pub runtime_root:PathBuf,#[serde(default="default_device")]pub device:String}
fn default_runtime()->PathBuf{project_root().join("tools/openvino-python")}
fn default_device()->String{"CPU".into()}
fn worker_script(p:&Paths)->PathBuf{p.runtime_root.parent().unwrap_or(&p.runtime_root).join("openvino-asr.py")}
fn paths<R:Runtime>(_app:&AppHandle<R>)->Result<Paths,String>{
    if let Ok(raw)=std::fs::read(data_dir().join("local-models.json")) {
        return serde_json::from_slice(&raw).map_err(|_|"本地模型路径记录无效".into());
    }
    Ok(Paths{asr_base:project_root().join("models"),llm_root:project_root(),runtime_root:default_runtime(),device:default_device()})
}
fn helper()->Option<PathBuf>{
    let parent=std::env::current_exe().ok()?.parent()?.to_path_buf();
    ["llama-helper-x86_64-pc-windows-msvc.exe","llama-helper.exe"].iter().map(|n|parent.join(n)).find(|p|p.is_file())
}
pub fn status<R:Runtime>(app:&AppHandle<R>)->Result<Value,String>{
    let p=paths(app)?;let asr_dir=p.asr_base.join(ASR);
    // Pinned GenAI 2026.4.1 / Optimum Intel 2.2 use the encoder-decoder IR format.
    let required=["openvino_encoder_model.xml","openvino_encoder_model.bin","openvino_decoder_model.xml","openvino_decoder_model.bin","openvino_tokenizer.xml","openvino_tokenizer.bin","openvino_detokenizer.xml","openvino_detokenizer.bin","config.json","preprocessor_config.json"];
    let asr_files=required.iter().all(|n|std::fs::metadata(asr_dir.join(n)).map(|m|m.len()>0).unwrap_or(false));
    let runtime_ready=p.runtime_root.join("python.exe").is_file()&&worker_script(&p).is_file();
    let llm=p.llm_root.join("models/summary/Qwen3.5-4B-Q4_K_M.gguf");
    let llm_file=std::fs::metadata(&llm).map(|m|m.len()==2740937888).unwrap_or(false);
    Ok(json!({"asr_directory":asr_dir,"runtime_directory":p.runtime_root,"device":p.device,"summary_file":llm,"helper_file":helper(),"asr_files_ready":asr_files,"summary_file_ready":llm_file,"files_ready":asr_files&&runtime_ready&&llm_file&&helper().is_some(),"tested":TESTED.load(Ordering::SeqCst),"language_notice":"Qwen3-ASR 1.7B + OpenVINO：支持中文、英文和中英混合；分段转写，完全本地运行。"}))
}
struct Worker{child:Child,input:ChildStdin,output:BufReader<ChildStdout>}
pub struct OpenVinoEngine{paths:Paths,worker:Mutex<Option<Worker>>}
impl OpenVinoEngine{
    async fn start(&self)->Result<Worker,String>{
        if !["CPU","GPU"].contains(&self.paths.device.as_str()){return Err("OpenVINO 设备必须为 CPU 或 GPU".into());}
        std::fs::create_dir_all(project_root().join("logs")).map_err(|e|e.to_string())?;
        let log=std::fs::OpenOptions::new().create(true).append(true).open(project_root().join("logs/openvino-worker.log")).map_err(|e|e.to_string())?;
        let mut command=Command::new(self.paths.runtime_root.join("python.exe"));
        command.arg("-u").arg(worker_script(&self.paths)).arg("--model").arg(self.paths.asr_base.join(ASR)).arg("--device").arg(&self.paths.device).arg("--cache").arg(project_root().join("models/openvino-cache"))
            .env("PYTHONIOENCODING","utf-8").env("HF_HUB_OFFLINE","1").env("HF_HUB_DISABLE_TELEMETRY","1").env("DO_NOT_TRACK","1")
            .stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(log).kill_on_drop(true);
        #[cfg(windows)]command.creation_flags(0x08000000);
        let mut child=command.spawn().map_err(|e|format!("OpenVINO 无法启动：{}",e))?;
        let input=child.stdin.take().ok_or("OpenVINO 输入不可用")?;let output=BufReader::new(child.stdout.take().ok_or("OpenVINO 输出不可用")?);
        let mut worker=Worker{child,input,output};let reply=Self::reply(&mut worker,300).await?;
        if reply["ready"]!=true{return Err(reply["error"].as_str().unwrap_or("OpenVINO 加载失败").into());}Ok(worker)
    }
    async fn reply(worker:&mut Worker,seconds:u64)->Result<Value,String>{
        let mut line=String::new();
        let n=tokio::time::timeout(std::time::Duration::from_secs(seconds),worker.output.read_line(&mut line)).await.map_err(|_|"OpenVINO 推理超时；不会切换云端".to_string())?.map_err(|e|e.to_string())?;
        if n==0{return Err("OpenVINO 进程已退出，请查看 logs/openvino-worker.log".into());}
        serde_json::from_str(&line).map_err(|_|"OpenVINO 返回格式无效，请查看诊断日志".into())
    }
    async fn ensure_ready(&self)->Result<(),String>{let mut worker=self.worker.lock().await;if worker.is_none(){*worker=Some(self.start().await?);}Ok(())}
    async fn release(&self){if let Some(mut worker)=self.worker.lock().await.take(){let _=worker.child.kill().await;let _=worker.child.wait().await;}}
    async fn transcribe_audio(&self,audio:Vec<f32>)->Result<String,String>{
        let mut slot=self.worker.lock().await;
        if slot.is_none(){*slot=Some(self.start().await?);}
        let worker=slot.as_mut().unwrap();
        let bytes=audio.iter().flat_map(|s|((s.clamp(-1.,1.)*32767.) as i16).to_le_bytes()).collect::<Vec<_>>();
        let request=json!({"pcm16":base64::engine::general_purpose::STANDARD.encode(bytes)}).to_string()+"\n";
        let result:Result<String,String>=async{
            worker.input.write_all(request.as_bytes()).await.map_err(|e|e.to_string())?;
            worker.input.flush().await.map_err(|e|e.to_string())?;
            let reply=Self::reply(worker,90).await?;
            if let Some(error)=reply["error"].as_str(){return Err(error.into());}
            reply["text"].as_str().map(str::to_string).ok_or("OpenVINO 未返回转写文本".into())
        }.await;
        if result.is_err(){if let Some(mut failed)=slot.take(){let _=failed.child.kill().await;}}
        result
    }
}
pub async fn prepare<R:Runtime>(app:&AppHandle<R>)->Result<Arc<OpenVinoEngine>,String>{
    if status(app)?["files_ready"]!=true {return Err("本地模型或推理程序不完整，请先在设置中检测；不会自动联网或下载".into());}
    let p=paths(app)?;
    let engine=ENGINE.get_or_init(||async{Arc::new(OpenVinoEngine{paths:p,worker:Mutex::new(None)})}).await.clone();
    engine.ensure_ready().await?;Ok(engine)
}
pub async fn generate<R:Runtime>(app:&AppHandle<R>,system:&str,user:&str,max_tokens:i32)->Result<String,String>{
    let _guard=WORK.lock().await;
    let p=paths(app)?;
    let result=crate::summary::summary_engine::client::generate_with_builtin_limits(&p.llm_root,LLM,system,user,None,Some((max_tokens,8192))).await;
    result.map_err(|e|format!("本地摘要失败：{}（不会自动切换云端）",e))
}
pub async fn test<R:Runtime>(app:&AppHandle<R>)->Result<Value,String>{
    let engine=prepare(app).await?;
    // This runs actual ASR inference, but captures no microphone and sends no data.
    let result:Result<(),String>=async{
        engine.transcribe_audio(vec![0f32;16000]).await.map_err(|e|format!("本地语音推理测试失败：{}",e))?;
        let prompt_settings=load_settings();
        let reply=generate(app,effective_prompt(&prompt_settings,"local_test_system"),effective_prompt(&prompt_settings,"local_test_user"),32).await?;
        if !reply.to_ascii_uppercase().contains("OK") {return Err("本地摘要模型已运行，但未通过简短响应测试，请查看诊断记录".into());}
        Ok(())
    }.await;
    // A diagnostic run in cloud mode must not retain gigabytes of unused model memory.
    if load_settings().processing_mode!="local"{engine.release().await;}
    result?;
    TESTED.store(true,Ordering::SeqCst);status(app)
}
pub fn tested()->bool{TESTED.load(Ordering::SeqCst)}
pub async fn release(){if let Some(engine)=ENGINE.get(){engine.release().await;}}

async fn transcribe<R:Runtime>(app:&AppHandle<R>,id:&str,engine:&OpenVinoEngine,audio:Vec<f32>,start:u64,end:u64){
    if audio.is_empty(){return;}
    let rms=(audio.iter().map(|s|s*s).sum::<f32>()/audio.len() as f32).sqrt();
    if rms<0.002 {return;}
    let result=engine.transcribe_audio(audio).await;
    match result {
        Ok(text) if !text.trim().is_empty()=>{
            let segment=Segment{id:uuid::Uuid::new_v4().to_string(),start_ms:start,end_ms:end,text:text.trim().to_string(),is_final:true,revision:1};
            let save:Result<(),String>=async{
                sqlx::query("INSERT INTO class_segments(session_id,id,payload) SELECT ?,?,? WHERE EXISTS(SELECT 1 FROM class_sessions WHERE id=?)")
                    .bind(id).bind(&segment.id).bind(serde_json::to_string(&segment).map_err(|e|e.to_string())?).bind(id).execute(database().await?).await.map_err(|e|e.to_string())?;
                sqlx::query("UPDATE class_sessions SET elapsed_ms=MAX(elapsed_ms,?) WHERE id=?").bind(end as i64).bind(id).execute(database().await?).await.map_err(|e|e.to_string())?;
                Ok(())
            }.await;
            match save {Ok(())=>{
                let _=app.emit("class-transcript",json!({"session_id":id,"segment":segment}));
                let _=app.emit("transcript-update",crate::audio::transcription::worker::TranscriptUpdate{text:segment.text.clone(),timestamp:chrono::Local::now().format("%H:%M:%S").to_string(),source:"Local OpenVINO Qwen3-ASR".into(),sequence_id:uuid::Uuid::new_v4().as_u128() as u64,chunk_start_time:start as f64/1000.,is_partial:false,confidence:0.,audio_start_time:start as f64/1000.,audio_end_time:end as f64/1000.,duration:end.saturating_sub(start) as f64/1000.});
            },Err(e)=>{let _=app.emit("class-asr-status",json!({"state":"error","message":format!("本地转写保存失败：{}",e)}));}}
        },
        Ok(_)=>{},
        Err(e)=>{let _=app.emit("class-asr-status",json!({"state":"error","message":format!("本地识别失败：{}；录音仍保存，不会自动联网",e)}));}
    }
}
pub async fn run<R:Runtime>(app:AppHandle<R>,mut upstream:tokio::sync::mpsc::UnboundedReceiver<AudioChunk>){
    let Some(id)=app.state::<ClassroomState>().active.lock().ok().and_then(|s|s.as_ref().map(|s|s.id.clone()))else{return;};
    let engine=match prepare(&app).await {Ok(e)=>e,Err(e)=>{let _=app.emit("class-asr-status",json!({"state":"error","message":e}));return;}};
    let _=app.emit("class-asr-status",json!({"state":"connected","message":"OpenVINO 本地识别 · 中英混合 · 不上传音频"}));
    // A bounded reserve absorbs inference bursts without blocking the separate recorder.
    let(tx,mut rx)=tokio::sync::mpsc::channel::<AudioChunk>(256);
    let bridge_app=app.clone();let bridge_id=id.clone();let bridge=tokio::spawn(async move{
        while let Some(chunk)=upstream.recv().await {if tx.try_send(chunk).is_err(){super::cloud::report_gap(&bridge_app,&bridge_id,"本地识别处理落后，部分转写可能缺失；原始录音仍保存，不会自动联网").await;}}
    });
    let mut ds=Downsampler::new();let mut buffer=Vec::new();let(mut start,mut end)=(0,0);
    while let Some(chunk)=rx.recv().await{
        let pcm=match ds.pcm(&chunk.data,chunk.sample_rate){Ok(v)=>v,Err(e)=>{let _=app.emit("class-asr-status",json!({"state":"error","message":e}));continue;}};
        let samples=pcm.chunks_exact(2).map(|b|i16::from_le_bytes([b[0],b[1]]) as f32/32768.).collect::<Vec<_>>();
        let chunk_end=(chunk.timestamp*1000.).max(0.) as u64;let chunk_start=chunk_end.saturating_sub(samples.len() as u64*1000/16000);
        // A class break or dropped queue must not join distant audio into one utterance.
        if !buffer.is_empty()&&chunk_start>end+1000 {transcribe(&app,&id,&engine,std::mem::take(&mut buffer),start,end).await;}
        if buffer.is_empty(){start=chunk_start;}end=chunk_end;buffer.extend(samples);
        if buffer.len()>=16000*8 {transcribe(&app,&id,&engine,std::mem::take(&mut buffer),start,end).await;}
    }
    transcribe(&app,&id,&engine,buffer,start,end).await;let _=bridge.await;
    let _=app.emit("class-asr-status",json!({"state":"stopped","message":"本地识别已结束；未调用云端"}));
}
