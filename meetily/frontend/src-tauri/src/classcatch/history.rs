//! Course lifecycle operations. Files are staged before database deletion, with
//! durable cleanup jobs so a crash or locked audio file cannot silently orphan them.
use std::path::{Path, PathBuf};
use sqlx::{Row, SqlitePool};

pub async fn migrate(db: &SqlitePool) -> Result<(), String> {
    let columns=sqlx::query("PRAGMA table_info(class_sessions)").fetch_all(db).await.map_err(|e|e.to_string())?;
    if !columns.iter().any(|r|r.get::<String,_>("name")=="archived") {
        sqlx::query("ALTER TABLE class_sessions ADD COLUMN archived INTEGER NOT NULL DEFAULT 0").execute(db).await.map_err(|e|e.to_string())?;
    }
    sqlx::query("CREATE TABLE IF NOT EXISTS class_file_cleanup (session_id TEXT PRIMARY KEY,path TEXT NOT NULL)").execute(db).await.map_err(|e|e.to_string())?;
    Ok(())
}

pub async fn rename(db:&SqlitePool,id:&str,title:&str)->Result<String,String> {
    let title=title.trim();
    if title.is_empty() || title.chars().count()>100 || title.chars().any(char::is_control) {
        return Err("课程名称须为1至100个字符，不能包含换行或控制字符".into());
    }
    let result=sqlx::query("UPDATE class_sessions SET title=? WHERE id=?").bind(title).bind(id).execute(db).await.map_err(|e|e.to_string())?;
    if result.rows_affected()==0 {return Err("课程不存在或已删除".into());}
    Ok(title.to_owned())
}

pub async fn course_folder(db:&SqlitePool,root:&Path,id:&str)->Result<PathBuf,String> {
    if id.is_empty() || !id.chars().all(|c|c.is_ascii_alphanumeric()||c=='-') {return Err("课程标识不合法".into());}
    let row=sqlx::query("SELECT audio_folder FROM class_sessions WHERE id=?").bind(id).fetch_optional(db).await.map_err(|e|e.to_string())?.ok_or("课程不存在或已删除")?;
    let folder:Option<String>=row.get("audio_folder");
    let folder=folder.ok_or("此课程尚未生成录音文件夹")?;
    // Only inspect the folder itself. Browsing must not recursively scan recordings.
    let root=std::fs::canonicalize(root).map_err(|_|"录音根目录暂不可用，请检查磁盘连接")?;
    let target=std::fs::canonicalize(&folder).map_err(|_|"录音文件夹不存在或磁盘未连接")?;
    let metadata=std::fs::symlink_metadata(&folder).map_err(|e|e.to_string())?;
    #[cfg(windows)] {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {return Err("录音文件夹是链接，请核实路径".into());}
    }
    if metadata.file_type().is_symlink() || !target.is_dir() || target.parent()!=Some(root.as_path())
        || !target.file_name().and_then(|v|v.to_str()).is_some_and(|v|v.starts_with(&format!("Class-{}_",id))) {
        return Err("录音路径不属于此课程的独立文件夹".into());
    }
    Ok(target)
}

fn check_tree(path:&Path)->Result<(),String> {
    let meta=std::fs::symlink_metadata(path).map_err(|e|e.to_string())?;
    let mut linked=meta.file_type().is_symlink();
    #[cfg(windows)] {
        use std::os::windows::fs::MetadataExt;
        linked |= meta.file_attributes() & 0x400 != 0;
    }
    if linked {return Err("课程目录含链接，已阻止删除，请人工核实".into());}
    if meta.is_dir() {
        for entry in std::fs::read_dir(path).map_err(|e|e.to_string())? {
            check_tree(&entry.map_err(|e|e.to_string())?.path())?;
        }
    }
    Ok(())
}

fn checked_folder(root:&Path, folder:&Path, name_prefix:&str)->Result<Option<PathBuf>,String> {
    // Missing recordings are allowed: their database records can still be removed.
    match std::fs::symlink_metadata(folder) {
        Err(e) if e.kind()==std::io::ErrorKind::NotFound=>return Ok(None),
        Err(e)=>return Err(e.to_string()),
        Ok(_)=>{}
    }
    check_tree(folder)?;
    let root=std::fs::canonicalize(root).map_err(|e|e.to_string())?;
    let target=std::fs::canonicalize(folder).map_err(|e|e.to_string())?;
    if target.parent()!=Some(root.as_path()) || !target.is_dir()
        || !target.file_name().and_then(|v|v.to_str()).is_some_and(|v|v.starts_with(name_prefix)) {
        return Err("录音路径不属于此课程的独立目录，已阻止删除".into());
    }
    Ok(Some(target))
}

pub async fn cleanup_pending(db:&SqlitePool,root:&Path)->Result<(),String> {
    let rows=sqlx::query("SELECT session_id,path FROM class_file_cleanup").fetch_all(db).await.map_err(|e|e.to_string())?;
    let mut failed=false;
    for row in rows {
        let id:String=row.get("session_id");
        let path=PathBuf::from(row.get::<String,_>("path"));
        let expected=format!(".delete-{}",id);
        let result=checked_folder(root,&path,&expected).and_then(|p|{
            if let Some(p)=p {std::fs::remove_dir_all(p).map_err(|e|e.to_string())?;}
            Ok(())
        });
        if result.is_ok() {
            sqlx::query("DELETE FROM class_file_cleanup WHERE session_id=?").bind(id).execute(db).await.map_err(|e|e.to_string())?;
        } else {failed=true;}
    }
    if failed {Err("记录已删除，部分文件正在被占用或路径校验失败；清理任务已保留，下次启动会重试".into())} else {Ok(())}
}

pub async fn manage(db:&SqlitePool,root:&Path,id:&str,action:&str)->Result<Option<String>,String> {
    if !["archive","restore","delete"].contains(&action) {return Err("未知课程操作".into());}
    let row=sqlx::query("SELECT audio_folder,status FROM class_sessions WHERE id=?").bind(id).fetch_optional(db).await.map_err(|e|e.to_string())?.ok_or("课程不存在或已删除")?;
    if ["recording","paused","stopping"].contains(&row.get::<String,_>("status").as_str()) {return Err("请先结束并保存此课程".into());}
    if action!="delete" {
        sqlx::query("UPDATE class_sessions SET archived=? WHERE id=?").bind(action=="archive").bind(id).execute(db).await.map_err(|e|e.to_string())?;
        return Ok(None);
    }
    if id.is_empty() || !id.chars().all(|c|c.is_ascii_alphanumeric()||c=='-') {return Err("课程标识不合法".into());}
    let folder=row.get::<Option<String>,_>("audio_folder");
    let source=match folder {
        Some(ref f)=>{
            let shared:i64=sqlx::query_scalar("SELECT COUNT(*) FROM class_sessions WHERE audio_folder=? AND id<>?").bind(f).bind(id).fetch_one(db).await.map_err(|e|e.to_string())?;
            if shared>0 {return Err("其他课程也在引用此目录，已阻止删除".into());}
            checked_folder(root,Path::new(f),&format!("Class-{}_",id))?
        },None=>None
    };
    let staged=root.join(format!(".delete-{}",id));
    if source.is_some() && staged.exists() {return Err("此课程有待处理的清理目录，请重新启动后重试".into());}
    let mut tx=db.begin().await.map_err(|e|e.to_string())?;
    if let Some(ref path)=source {std::fs::rename(path,&staged).map_err(|e|format!("录音文件可能正在播放或被占用，未删除课程：{}",e))?;}
    let deleted:Result<(),sqlx::Error>=async {
        for table in ["class_segments","class_events","class_cards"] {
            sqlx::query(&format!("DELETE FROM {} WHERE session_id=?",table)).bind(id).execute(&mut *tx).await?;
        }
        sqlx::query("DELETE FROM class_sessions WHERE id=?").bind(id).execute(&mut *tx).await?;
        if source.is_some() {sqlx::query("INSERT INTO class_file_cleanup(session_id,path)VALUES(?,?)").bind(id).bind(staged.to_string_lossy().as_ref()).execute(&mut *tx).await?;}
        Ok(())
    }.await;
    if let Err(e)=deleted {
        let _=tx.rollback().await;
        if let Some(ref path)=source {std::fs::rename(&staged,path).map_err(|restore|format!("数据库删除失败：{}；录音移回失败：{}；文件仍在 {}",e,restore,staged.display()))?;}
        return Err(e.to_string());
    }
    if let Err(e)=tx.commit().await {
        if let Some(ref path)=source {let _=std::fs::rename(&staged,path);}
        return Err(format!("课程删除提交失败，请重新打开核实：{}",e));
    }
    Ok(cleanup_pending(db,root).await.err())
}
