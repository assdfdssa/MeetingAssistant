//! Additive crash recovery. Inputs and checkpoints are never overwritten/deleted.
use super::core::Segment;
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use std::{path::{Path,PathBuf},process::Command,collections::BTreeMap};

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct RepairReport {
    pub job_id:String,pub session_id:String,pub status:String,pub audio_file:String,
    pub audio_duration_ms:u64,pub expected_audio_ms:u64,pub missing_tail_ms:u64,
    pub checkpoint_count:usize,pub used_checkpoint_count:usize,
    pub confirmed_segments:usize,pub imported_segments:usize,pub unconfirmed_segments:usize,
    pub warnings:Vec<String>,pub report_file:String,pub transcript_file:String,
}
pub struct RepairOutput {pub report:RepairReport,pub imported:Vec<Segment>}

fn error(e:impl std::fmt::Display)->String{e.to_string()}
fn checked(path:&Path,parent:&Path)->Result<PathBuf,String>{
    let meta=std::fs::symlink_metadata(path).map_err(error)?;
    let mut link=meta.file_type().is_symlink();
    #[cfg(windows)] {use std::os::windows::fs::MetadataExt;link|=meta.file_attributes()&0x400!=0;}
    let actual=std::fs::canonicalize(path).map_err(error)?;
    if link||actual.parent()!=Some(parent){return Err("修复文件路径含链接或超出课程目录，已停止".into());}
    Ok(actual)
}
fn read_json(folder:&Path,name:&str)->Result<Option<Value>,String>{
    let path=folder.join(name);if !path.exists(){return Ok(None);}
    let path=checked(&path,folder)?;
    Ok(serde_json::from_slice(&std::fs::read(path).map_err(error)?).ok())
}
pub fn preferred_audio(folder:&Path)->Result<Option<PathBuf>,String>{
    let Some(v)=read_json(folder,"repair-state.json")? else{return Ok(None);};
    let name=v["audio_file"].as_str().ok_or("修复记录缺少音频路径")?;
    if !(name=="audio.mp4"||name.starts_with("audio-recovered-")&&name.ends_with(".mp4"))||Path::new(name).components().count()!=1 {
        return Err("修复音频路径无效".into());
    }
    Ok(Some(checked(&folder.join(name),folder)?))
}
pub fn recovered_duration(folder:&Path)->Result<Option<u64>,String>{
    Ok(read_json(folder,"repair-state.json")?.and_then(|v|v["audio_duration_ms"].as_u64()))
}
fn command(ffmpeg:&Path)->Command{
    let mut c=Command::new(ffmpeg);
    #[cfg(windows)] {use std::os::windows::process::CommandExt;c.creation_flags(0x08000000);}
    c
}
/// Full decoding, not just a header check. FFmpeg reports microsecond timestamps.
pub fn verify_audio(ffmpeg:&Path,path:&Path)->Result<u64,String>{
    let output=command(ffmpeg).args(["-nostdin","-v","error","-xerror","-i"]).arg(path)
        .args(["-map","0:a:0","-progress","pipe:1","-nostats","-f","null","-"]).output().map_err(error)?;
    if !output.status.success(){return Err("音频未通过完整解码核验".into());}
    let duration=String::from_utf8_lossy(&output.stdout).lines().filter_map(|s|s.strip_prefix("out_time_us=")?.parse::<i64>().ok()).max().unwrap_or(0);
    if duration<=0 {return Err("音频为空或没有可解码的音轨".into());}
    Ok(duration as u64/1000)
}
fn chunk_number(name:&str)->Option<u32>{
    let n=name.strip_prefix("audio_chunk_")?.strip_suffix(".mp4")?;
    if n.is_empty()||!n.chars().all(|c|c.is_ascii_digit()){return None;}n.parse().ok()
}

pub fn import_export(existing:&[Segment],export:&Value)->Vec<Segment>{
    let mut imported=Vec::new();
    for row in export["segments"].as_array().into_iter().flatten(){
        // Legacy saver exports final updates; explicitly partial rows stay partial.
        if row["is_final"]==false {continue;}
        let(Some(text),Some(start),Some(end))=(row["text"].as_str(),row["audio_start_time"].as_f64(),row["audio_end_time"].as_f64())else{continue;};
        if text.trim().is_empty()||!start.is_finite()||!end.is_finite()||start<0.||end<=start {continue;}
        let(start_ms,end_ms)=((start*1000.).round() as u64,(end*1000.).round() as u64);
        // A final database revision wins over an older export at the same bounds.
        if existing.iter().chain(imported.iter()).any(|s:&Segment|s.is_final&&s.start_ms.abs_diff(start_ms)<=5&&s.end_ms.abs_diff(end_ms)<=5){continue;}
        imported.push(Segment{id:format!("recovered:{}",uuid::Uuid::new_v4()),start_ms,end_ms,text:text.into(),is_final:true,revision:1});
    }
    imported
}

pub fn run(folder:&Path,ffmpeg:&Path,id:&str,old_status:&str,existing:Vec<Segment>,job:&str)->Result<RepairOutput,String>{
    let folder=std::fs::canonicalize(folder).map_err(error)?;
    let repairs=folder.join(".repairs");
    if repairs.exists(){checked(&repairs,&folder)?;}else{std::fs::create_dir(&repairs).map_err(error)?;}
    let job_dir=repairs.join(job);std::fs::create_dir(&job_dir).map_err(error)?;
    let job_dir=std::fs::canonicalize(job_dir).map_err(error)?;
    // Small source files are snapshotted before any change. Audio inputs remain intact.
    for name in ["metadata.json","transcripts.json","audio-timeline.jsonl","repair-state.json"] {
        let path=folder.join(name);if path.exists(){std::fs::copy(checked(&path,&folder)?,job_dir.join(format!("original-{}",name))).map_err(error)?;}
    }
    let mut warnings=Vec::new();
    let metadata=read_json(&folder,"metadata.json")?.unwrap_or_else(||json!({}));
    let export=read_json(&folder,"transcripts.json")?;
    if export.is_none(){warnings.push("本地转写导出缺失或无效，保留数据库中的转写".into());}
    let imported=export.as_ref().map(|v|import_export(&existing,v)).unwrap_or_default();
    let mut final_segments=existing.iter().filter(|s|s.is_final).cloned().collect::<Vec<_>>();final_segments.extend(imported.clone());final_segments.sort_by_key(|s|s.start_ms);
    let partial=existing.iter().filter(|s|!s.is_final).count();
    if partial>0 {warnings.push(format!("保留{}条未确认转写，不将它们自动确认为老师原话",partial));}
    let mut expected=0u64;let mut invalid_timeline=0;
    let timeline=folder.join("audio-timeline.jsonl");
    if timeline.exists(){
        for line in std::fs::read_to_string(checked(&timeline,&folder)?).map_err(error)?.lines(){
            match serde_json::from_str::<Value>(line).ok().and_then(|v|v["audio_end_ms"].as_f64()){
                Some(n) if n.is_finite()&&n>=0.=>expected=expected.max(n.round() as u64),_=>invalid_timeline+=1,
            }
        }
    }
    if invalid_timeline>0{warnings.push(format!("时间映射含{}条无效记录，已保留原文件",invalid_timeline));}
    if expected==0{expected=metadata["duration_seconds"].as_f64().filter(|n|n.is_finite()&&*n>0.).map(|s|(s*1000.) as u64).unwrap_or(0);}
    let mut chunks=BTreeMap::new();
    let checkpoints=folder.join(".checkpoints");
    if checkpoints.exists(){
        let checkpoints=checked(&checkpoints,&folder)?;
        for entry in std::fs::read_dir(&checkpoints).map_err(error)?{
            let entry=entry.map_err(error)?;let name=entry.file_name().to_string_lossy().to_string();
            if let Some(n)=chunk_number(&name){let path=checked(&entry.path(),&checkpoints)?;if chunks.insert(n,path).is_some(){return Err("检查点编号重复，请人工核实；原文件未修改".into());}}
        }
    }
    let mut audio=match preferred_audio(&folder){Ok(p)=>p,Err(_)=>{warnings.push("上次修复音频缺失或路径无效，将重新核验检查点".into());None}};
    if audio.is_none()&&folder.join("audio.mp4").exists(){audio=Some(checked(&folder.join("audio.mp4"),&folder)?);}
    let existing_duration=audio.as_ref().and_then(|p|verify_audio(ffmpeg,p).ok());
    let(mut used,mut prefix_duration)=(0usize,0u64);
    let audio_duration;
    if let Some(duration)=existing_duration {
        audio_duration=duration;
    }else{
        if audio.is_some(){warnings.push("原合并音频无效，改用检查点生成新文件；原音频保留".into());}
        let mut names=Vec::new();
        for (expected_index,(&n,path)) in chunks.iter().enumerate(){
            if n!=expected_index as u32 {warnings.push(format!("缺少第{}个检查点，仅恢复连续前段，后续原文件保留",expected_index));break;}
            match verify_audio(ffmpeg,path){
                Ok(duration)=>{prefix_duration+=duration;names.push(path.file_name().unwrap().to_string_lossy().to_string());used+=1;},
                Err(_)=>{warnings.push(format!("{}损坏，仅恢复其前面的连续音频，全部检查点保留",path.file_name().unwrap().to_string_lossy()));break;}
            }
        }
        if names.is_empty(){return Err("没有可连续恢复并通过核验的录音检查点；原文件和中断状态保留".into());}
        let concat=names.iter().map(|name|format!("file '../../.checkpoints/{}'\n",name)).collect::<String>();
        std::fs::write(job_dir.join("concat.txt"),concat).map_err(error)?;
        let result=command(ffmpeg).current_dir(&job_dir).args(["-nostdin","-v","error","-f","concat","-safe","0","-i","concat.txt","-map","0:a:0","-c","copy","-movflags","+faststart","-n","merged.mp4"]).output().map_err(error)?;
        if !result.status.success(){std::fs::write(job_dir.join("ffmpeg-error.log"),&result.stderr).map_err(error)?;return Err("检查点合并失败，诊断和原文件均已保留，中断状态未改变".into());}
        audio_duration=verify_audio(ffmpeg,&job_dir.join("merged.mp4"))?;
        if audio_duration.abs_diff(prefix_duration)>(used as u64*100).max(1500){return Err("合并音频时长与检查点不一致，保留中断状态供人工核实".into());}
        let published=folder.join(format!("audio-recovered-{}.mp4",job));
        std::fs::rename(job_dir.join("merged.mp4"),&published).map_err(error)?;audio=Some(published);
    }
    let missing=expected.saturating_sub(audio_duration);
    let incomplete=missing>1500||used>0&&used<chunks.len()||invalid_timeline>0||expected==0&&old_status!="completed";
    if missing>1500{warnings.push(format!("已落盘录音比时间映射少约{:.1}秒，通常是中断前尚未写入的尾部，无法凭检查点补回",missing as f64/1000.));}
    if expected==0&&old_status!="completed"{warnings.push("缺少可靠的总时长记录，无法保证覆盖整节课堂".into());}
    let status=if incomplete{"recovered_partial"}else if old_status=="completed"{"completed"}else{"recovered"};
    let audio=audio.ok_or("修复后没有有效音频")?;
    let audio_name=audio.file_name().unwrap().to_string_lossy().to_string();
    let transcript_file=job_dir.join("transcripts-recovered.json");
    std::fs::write(&transcript_file,serde_json::to_vec_pretty(&json!({"version":"classcatch-recovery-1","segments":final_segments,"unconfirmed_segments":existing.iter().filter(|s|!s.is_final).collect::<Vec<_>>()})).map_err(error)?).map_err(error)?;
    let report=RepairReport{job_id:job.into(),session_id:id.into(),status:status.into(),audio_file:audio.to_string_lossy().into(),audio_duration_ms:audio_duration,expected_audio_ms:expected,missing_tail_ms:missing,checkpoint_count:chunks.len(),used_checkpoint_count:used,confirmed_segments:final_segments.len(),imported_segments:imported.len(),unconfirmed_segments:partial,warnings,report_file:job_dir.join("report.json").to_string_lossy().into(),transcript_file:transcript_file.to_string_lossy().into()};
    std::fs::write(&report.report_file,serde_json::to_vec_pretty(&report).map_err(error)?).map_err(error)?;
    {
        let temp=job_dir.join("repair-state.json");std::fs::write(&temp,serde_json::to_vec_pretty(&json!({"audio_file":audio_name,"audio_duration_ms":audio_duration,"report_file":report.report_file})).map_err(error)?).map_err(error)?;
        std::fs::rename(temp,folder.join("repair-state.json")).map_err(error)?;
    }
    let mut updated=metadata;
    if !updated.is_object(){updated=json!({});}
    updated["status"]=json!(status);updated["audio_file"]=json!(audio_name);updated["duration_seconds"]=json!(audio_duration as f64/1000.);updated["recovered_at"]=json!(chrono::Local::now().to_rfc3339());
    let temp=job_dir.join("metadata-updated.json");std::fs::write(&temp,serde_json::to_vec_pretty(&updated).map_err(error)?).map_err(error)?;
    std::fs::rename(temp,folder.join("metadata.json")).map_err(error)?;
    Ok(RepairOutput{report,imported})
}
