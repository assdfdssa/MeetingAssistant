#[path = "../../meetily/frontend/src-tauri/src/classcatch/core.rs"]
mod core;
#[path = "../../meetily/frontend/src-tauri/src/classcatch/repair.rs"]
mod repair;

#[cfg(test)]
mod tests {
 use super::*;
 use serde_json::json;
 use std::{path::PathBuf,process::Command};
 fn setup()->(PathBuf,PathBuf){
  let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../build").canonicalize().unwrap().join(format!("repair-fixture-{}",uuid::Uuid::new_v4()));
  std::fs::create_dir(&root).unwrap();std::fs::create_dir(root.join(".checkpoints")).unwrap();
  let ffmpeg=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../build/native/debug/ffmpeg.exe").canonicalize().unwrap();(root,ffmpeg)
 }
 fn chunk(root:&std::path::Path,ffmpeg:&std::path::Path,n:u32){
  let status=Command::new(ffmpeg).args(["-nostdin","-v","error","-f","lavfi","-i","sine=frequency=440:sample_rate=48000","-t","0.5","-ac","1","-c:a","aac","-n"]).arg(root.join(".checkpoints").join(format!("audio_chunk_{:03}.mp4",n))).status().unwrap();assert!(status.success());
 }
 #[test]
 fn full_decode_partial_tail_and_idempotent_repair_preserve_originals(){
  let(root,ffmpeg)=setup();chunk(&root,&ffmpeg,0);chunk(&root,&ffmpeg,1);
  std::fs::write(root.join("metadata.json"),br#"{"status":"recording","meeting_name":"original"}"#).unwrap();
  std::fs::write(root.join("audio-timeline.jsonl"),"{\"audio_end_ms\":4000}\n").unwrap();
  std::fs::write(root.join("transcripts.json"),json!({"segments":[{"id":"one","text":"Recovered final words","audio_start_time":0.1,"audio_end_time":0.4},{"id":"p","text":"Do not promote","audio_start_time":0.5,"audio_end_time":0.9,"is_final":false}]}).to_string()).unwrap();
  let original=std::fs::read(root.join(".checkpoints/audio_chunk_000.mp4")).unwrap();
  let result=repair::run(&root,&ffmpeg,"one","interrupted",vec![],"first").unwrap();
  assert_eq!(result.report.status,"recovered_partial");assert_eq!(result.report.imported_segments,1);assert!(result.report.missing_tail_ms>2500);
  assert_eq!(std::fs::read(root.join(".checkpoints/audio_chunk_000.mp4")).unwrap(),original);
  assert!(root.join(".repairs/first/original-metadata.json").is_file());
  assert!(repair::verify_audio(&ffmpeg,&PathBuf::from(&result.report.audio_file)).unwrap()>900);
  let second=repair::run(&root,&ffmpeg,"one","recovered_partial",result.imported,"second").unwrap();
  assert_eq!(second.report.imported_segments,0);assert_eq!(second.report.audio_file,result.report.audio_file);
 }
 #[test]
 fn corrupt_first_chunk_never_publishes_audio_or_changes_metadata(){
  let(root,ffmpeg)=setup();std::fs::write(root.join(".checkpoints/audio_chunk_000.mp4"),b"broken mp4").unwrap();
  let original=br#"{"status":"recording"}"#;std::fs::write(root.join("metadata.json"),original).unwrap();
  assert!(repair::run(&root,&ffmpeg,"one","interrupted",vec![],"broken").is_err());
  assert!(!root.join("repair-state.json").exists());assert_eq!(std::fs::read(root.join("metadata.json")).unwrap(),original);
 }
 #[test]
 fn missing_middle_never_shifts_later_audio_into_wrong_time(){
  let(root,ffmpeg)=setup();chunk(&root,&ffmpeg,0);chunk(&root,&ffmpeg,2);
  std::fs::write(root.join("audio-timeline.jsonl"),"{\"audio_end_ms\":3000}\n").unwrap();
  let result=repair::run(&root,&ffmpeg,"one","interrupted",vec![],"gap").unwrap();
  assert_eq!(result.report.used_checkpoint_count,1);assert_eq!(result.report.checkpoint_count,2);assert_eq!(result.report.status,"recovered_partial");assert!(root.join(".checkpoints/audio_chunk_002.mp4").is_file());
 }
 #[test]
 fn numeric_order_and_final_database_revision_win(){
  let(root,ffmpeg)=setup();for n in 0..11{chunk(&root,&ffmpeg,n);}
  std::fs::write(root.join("audio-timeline.jsonl"),"{\"audio_end_ms\":5500}\n").unwrap();
  let result=repair::run(&root,&ffmpeg,"one","interrupted",vec![],"ordered").unwrap();assert_eq!(result.report.used_checkpoint_count,11);assert_eq!(result.report.status,"recovered");
  let existing=vec![core::Segment{id:"a".into(),start_ms:100,end_ms:500,text:"New final revision".into(),is_final:true,revision:2}];
  let exported=json!({"segments":[{"text":"Older revision","audio_start_time":0.1,"audio_end_time":0.5}]});assert!(repair::import_export(&existing,&exported).is_empty());
 }
 #[test]
 fn escaped_manifest_path_is_rejected(){
  let(root,_)=setup();std::fs::write(root.join("repair-state.json"),br#"{"audio_file":"audio-recovered-../outside.mp4"}"#).unwrap();assert!(repair::preferred_audio(&root).is_err());
 }
}
