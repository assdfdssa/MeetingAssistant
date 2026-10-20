#[path = "../../meetily/frontend/src-tauri/src/classcatch/history.rs"]
mod history;

#[cfg(test)]
mod tests {
 use super::history;
 use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
 use std::path::PathBuf;
 async fn setup()->(SqlitePool,PathBuf) {
  let db=SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
  for sql in ["CREATE TABLE class_sessions(id TEXT PRIMARY KEY,title TEXT DEFAULT '课堂',status TEXT,audio_folder TEXT)","CREATE TABLE class_segments(session_id TEXT,payload TEXT)","CREATE TABLE class_events(session_id TEXT,payload TEXT)","CREATE TABLE class_cards(session_id TEXT,payload TEXT)"]{sqlx::query(sql).execute(&db).await.unwrap();}
  history::migrate(&db).await.unwrap();history::migrate(&db).await.unwrap();
  let root=std::env::temp_dir().join(format!("classcatch-delete-test-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
  std::fs::create_dir_all(&root).unwrap();(db,root)
 }
 async fn seed(db:&SqlitePool,root:&std::path::Path,id:&str)->PathBuf {
  let folder=root.join(format!("Class-{}_test",id));std::fs::create_dir_all(&folder).unwrap();
  std::fs::write(folder.join("audio.mp4"),b"original audio").unwrap();
  std::fs::write(folder.join("transcripts.json"),b"original words").unwrap();
  sqlx::query("INSERT INTO class_sessions(id,status,audio_folder)VALUES(?,'completed',?)").bind(id).bind(folder.to_string_lossy().as_ref()).execute(db).await.unwrap();
  for table in ["class_segments","class_events","class_cards"]{sqlx::query(&format!("INSERT INTO {} VALUES(?,'original')",table)).bind(id).execute(db).await.unwrap();}
  folder
 }
 async fn count(db:&SqlitePool,table:&str)->i64 {sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {}",table)).fetch_one(db).await.unwrap()}
 #[tokio::test]
 async fn archive_preserves_files_and_rows_restore_works(){
  let(db,root)=setup().await;let folder=seed(&db,&root,"one").await;
  history::manage(&db,&root,"one","archive").await.unwrap();
  assert_eq!(std::fs::read(folder.join("audio.mp4")).unwrap(),b"original audio");
  assert_eq!(sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM class_sessions WHERE archived=0").fetch_one(&db).await.unwrap(),0);
  assert_eq!(count(&db,"class_segments").await,1);
  history::manage(&db,&root,"one","restore").await.unwrap();
  assert_eq!(sqlx::query_scalar::<_,i64>("SELECT archived FROM class_sessions").fetch_one(&db).await.unwrap(),0);
  drop(db);std::fs::remove_dir_all(root).unwrap();
 }
 #[tokio::test]
 async fn delete_removes_all_course_data_but_preserves_other_courses(){
  let(db,root)=setup().await;let folder=seed(&db,&root,"one").await;let other=seed(&db,&root,"two").await;
  assert!(history::manage(&db,&root,"one","delete").await.unwrap().is_none());assert!(!folder.exists());assert!(other.join("audio.mp4").exists());
  for table in ["class_sessions","class_segments","class_events","class_cards"]{assert_eq!(count(&db,table).await,1);}
  assert_eq!(count(&db,"class_file_cleanup").await,0);
  drop(db);std::fs::remove_dir_all(root).unwrap();
 }
 #[tokio::test]
 async fn outside_root_and_recording_are_rejected(){
  let(db,root)=setup().await;let folder=seed(&db,&root,"one").await;
  sqlx::query("UPDATE class_sessions SET status='recording'").execute(&db).await.unwrap();
  assert!(history::manage(&db,&root,"one","archive").await.is_err());assert!(history::manage(&db,&root,"one","delete").await.is_err());
  sqlx::query("UPDATE class_sessions SET status='completed'").execute(&db).await.unwrap();
  assert!(history::manage(&db,&folder,"one","delete").await.is_err());assert!(folder.join("audio.mp4").exists());
  drop(db);std::fs::remove_dir_all(root).unwrap();
 }
 #[tokio::test]
 async fn database_failure_restores_staged_recordings(){
  let(db,root)=setup().await;let folder=seed(&db,&root,"one").await;
  sqlx::query("CREATE TRIGGER reject_delete BEFORE DELETE ON class_sessions BEGIN SELECT RAISE(ABORT,'test failure'); END").execute(&db).await.unwrap();
  assert!(history::manage(&db,&root,"one","delete").await.is_err());assert!(folder.join("audio.mp4").exists());assert_eq!(count(&db,"class_segments").await,1);
  drop(db);std::fs::remove_dir_all(root).unwrap();
 }
 #[tokio::test]
 async fn durable_cleanup_retry_and_missing_audio(){
  let(db,root)=setup().await;let folder=seed(&db,&root,"one").await;std::fs::remove_dir_all(&folder).unwrap();
  history::manage(&db,&root,"one","delete").await.unwrap();assert_eq!(count(&db,"class_sessions").await,0);
  let staged=root.join(".delete-two");std::fs::create_dir(&staged).unwrap();std::fs::write(staged.join("audio.mp4"),b"test").unwrap();
  sqlx::query("INSERT INTO class_file_cleanup VALUES('two',?)").bind(staged.to_string_lossy().as_ref()).execute(&db).await.unwrap();
  assert!(history::cleanup_pending(&db,&staged).await.is_err());assert!(staged.exists());assert_eq!(count(&db,"class_file_cleanup").await,1);
  history::cleanup_pending(&db,&root).await.unwrap();assert!(!staged.exists());assert_eq!(count(&db,"class_file_cleanup").await,0);
  drop(db);std::fs::remove_dir_all(root).unwrap();
 }
 #[tokio::test]
 async fn shared_directory_is_protected(){
  let(db,root)=setup().await;let folder=seed(&db,&root,"one").await;seed(&db,&root,"two").await;
  sqlx::query("UPDATE class_sessions SET audio_folder=? WHERE id='two'").bind(folder.to_string_lossy().as_ref()).execute(&db).await.unwrap();
  assert!(history::manage(&db,&root,"one","delete").await.is_err());assert!(folder.join("audio.mp4").exists());assert_eq!(count(&db,"class_sessions").await,2);
  drop(db);std::fs::remove_dir_all(root).unwrap();
 }
 #[tokio::test]
 async fn rename_preserves_files_and_archive_and_rejects_bad_names(){
  let(db,root)=setup().await;let folder=seed(&db,&root,"one").await;
  history::manage(&db,&root,"one","archive").await.unwrap();
  assert_eq!(history::rename(&db,"one","  英语听力  ").await.unwrap(),"英语听力");
  assert_eq!(sqlx::query_scalar::<_,i64>("SELECT archived FROM class_sessions").fetch_one(&db).await.unwrap(),1);
  assert_eq!(history::course_folder(&db,&root,"one").await.unwrap(),std::fs::canonicalize(&folder).unwrap());
  for bad in ["".to_string()," ".to_string(),"a\nb".to_string(),"a".repeat(101)] {assert!(history::rename(&db,"one",&bad).await.is_err());}
  assert!(history::rename(&db,"missing","valid").await.is_err());assert!(folder.join("audio.mp4").exists());
  drop(db);std::fs::remove_dir_all(root).unwrap();
 }
 #[tokio::test]
 async fn active_course_can_be_renamed_and_folder_paths_are_checked(){
  let(db,root)=setup().await;let folder=seed(&db,&root,"one").await;
  sqlx::query("UPDATE class_sessions SET status='recording'").execute(&db).await.unwrap();
  history::rename(&db,"one","Live renamed").await.unwrap();
  assert!(history::course_folder(&db,&root,"one").await.is_ok());
  assert!(history::course_folder(&db,&folder,"one").await.is_err());
  assert!(history::course_folder(&db,&root,"missing").await.is_err());
  sqlx::query("UPDATE class_sessions SET audio_folder=NULL").execute(&db).await.unwrap();
  assert!(history::course_folder(&db,&root,"one").await.is_err());
  drop(db);std::fs::remove_dir_all(root).unwrap();
 }
}
