#[path="../../meetily/frontend/src-tauri/src/classcatch/transport.rs"]
mod transport;
#[cfg(test)]
mod tests{
 use super::transport::*;
 use std::{io::{Read,Write},net::TcpListener,time::Duration};
 fn server(status:u16,delay_ms:u64,body:&'static str,partial:bool)->String{
  let socket=TcpListener::bind("127.0.0.1:0").unwrap();let url=format!("http://{}",socket.local_addr().unwrap());
  std::thread::spawn(move||{let(mut s,_)=socket.accept().unwrap();let mut b=[0;4096];let _=s.read(&mut b);
   if !partial{std::thread::sleep(Duration::from_millis(delay_ms));}
   let headers=format!("HTTP/1.1 {} Test\r\nContent-Type: application/json\r\nx-request-id: safe-request-123\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",status,body.len());
   let _=s.write_all(headers.as_bytes());let _=s.flush();
   if partial{std::thread::sleep(Duration::from_millis(delay_ms));}let _=s.write_all(body.as_bytes());
  });url
 }
 fn client()->reqwest::Client{reqwest::Client::builder().no_proxy().timeout(Duration::from_millis(120)).build().unwrap()}
 #[test]fn budgets_are_mode_specific(){assert_eq!(timeout_seconds("catch_up"),60);assert_eq!(timeout_seconds("ask"),120);assert_eq!(timeout_seconds("layered"),180);assert!(super::transport::client("ask").is_ok());}
 #[tokio::test]async fn valid_json(){let v=post_json(&client(),&server(200,0,"{\"ok\":true}",false),"test-key-not-real",&serde_json::json!({}),120).await.unwrap();assert_eq!(v["ok"],true);}
 #[tokio::test]async fn header_timeout_is_clear(){let e=post_json(&client(),&server(200,400,"{}",false),"test-key-not-real",&serde_json::json!({}),120).await.unwrap_err();assert_eq!(e.kind,"response_timeout");assert!(e.message.contains("120秒"));assert!(!e.message.contains("test-key"));}
 #[tokio::test]async fn body_timeout_is_not_json_error(){let e=post_json(&client(),&server(200,400,"{}",true),"test-key-not-real",&serde_json::json!({}),120).await.unwrap_err();assert_eq!(e.kind,"response_timeout");assert_eq!(e.http_status,Some(200));}
 #[tokio::test]async fn http_errors_keep_safe_request_id(){for code in [401,403,429,503]{let e=post_json(&client(),&server(code,0,"{}",false),"test-key-not-real",&serde_json::json!({}),120).await.unwrap_err();assert_eq!(e.kind,"http_error");assert_eq!(e.http_status,Some(code));assert_eq!(e.request_id.as_deref(),Some("safe-request-123"));assert!(e.message.contains(&code.to_string()));}}
 #[tokio::test]async fn invalid_response_is_separate(){let e=post_json(&client(),&server(200,0,"not json",false),"test-key-not-real",&serde_json::json!({}),120).await.unwrap_err();assert_eq!(e.kind,"invalid_response");}
 #[tokio::test]async fn refused_connection_is_separate(){let s=TcpListener::bind("127.0.0.1:0").unwrap();let url=format!("http://{}",s.local_addr().unwrap());drop(s);let e=post_json(&reqwest::Client::builder().no_proxy().timeout(Duration::from_secs(10)).build().unwrap(),&url,"test-key-not-real",&serde_json::json!({}),120).await.unwrap_err();assert_eq!(e.kind,"connect_error");}
}
