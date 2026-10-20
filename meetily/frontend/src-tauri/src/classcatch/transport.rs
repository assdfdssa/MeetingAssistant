//! Summary transport only: no audio, no automatic retries, no credential logging.
use serde_json::Value;
use std::time::Duration;

pub fn timeout_seconds(mode:&str)->u64{match mode{"ask"=>120,"layered"=>180,_=>60}}
pub fn client(mode:&str)->Result<reqwest::Client,String>{
    reqwest::Client::builder().redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(10)).timeout(Duration::from_secs(timeout_seconds(mode)))
        .build().map_err(|_|"无法创建摘要网络连接".into())
}
#[derive(Debug)]
pub struct Failure{pub kind:&'static str,pub message:String,pub http_status:Option<u16>,pub request_id:Option<String>}
fn network(error:reqwest::Error,seconds:u64,phase:&'static str)->Failure{
    let(kind,message)=if error.is_timeout()&&error.is_connect(){
        ("connect_timeout","摘要服务连接超时，请检查当前网络或代理；录音仍在继续".to_string())
    }else if error.is_timeout(){
        ("response_timeout",format!("摘要服务在{}秒内未返回完整结果，请稍后重试或缩小时间范围；录音仍在继续",seconds))
    }else if error.is_connect(){
        ("connect_error","无法连接摘要服务，请检查当前网络、DNS或代理；录音仍在继续".to_string())
    }else{
        (phase,"摘要响应接收中断，请检查网络后重试；录音仍在继续".to_string())
    };
    Failure{kind,message,http_status:None,request_id:None}
}
fn request_id(headers:&reqwest::header::HeaderMap)->Option<String>{
    ["x-request-id","x-dashscope-request-id"].iter().find_map(|name|headers.get(*name).and_then(|v|v.to_str().ok())
        .filter(|s|!s.is_empty()&&s.len()<=128&&s.bytes().all(|b|b.is_ascii_alphanumeric()||b"-_.".contains(&b))).map(str::to_string))
}
pub async fn post_json(client:&reqwest::Client,url:&str,key:&str,body:&Value,seconds:u64)->Result<Value,Failure>{
    let response=client.post(url).bearer_auth(key).json(body).send().await.map_err(|e|network(e,seconds,"request_error"))?;
    let status=response.status().as_u16();let id=request_id(response.headers());
    if !(200..300).contains(&status){
        let detail=match status{
            401=>"鉴权失败，请检查设置中的摘要API Key",
            403=>"访问被拒绝，请检查模型权限、地域和访问限制",
            429=>"请求或额度受限，请稍后重试并检查额度",
            500..=599=>"服务暂时异常，请稍后重试",
            _=>"请求被拒绝，请检查服务地址、模型名称和所选范围",
        };
        return Err(Failure{kind:"http_error",message:format!("摘要{}（HTTP {}）；录音仍在继续",detail,status),http_status:Some(status),request_id:id});
    }
    // Reading the body can also time out; do not misreport that as invalid JSON.
    let bytes=response.bytes().await.map_err(|e|{let mut f=network(e,seconds,"body_error");f.http_status=Some(status);f.request_id=id.clone();f})?;
    serde_json::from_slice(&bytes).map_err(|_|Failure{kind:"invalid_response",message:"摘要服务返回了无法解析的数据，请重试；录音仍在继续".into(),http_status:Some(status),request_id:id})
}
