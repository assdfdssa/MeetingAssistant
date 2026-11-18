use std::collections::BTreeMap;

pub const IDS: [&str; 6] = ["catch_up", "ask", "layered", "connection_test", "local_test_system", "local_test_user"];
pub fn resolve<'a>(overrides:&'a BTreeMap<String,String>,id:&str,fallback:&'a str)->&'a str {
    overrides.get(id).filter(|s|!s.trim().is_empty()).map(String::as_str).unwrap_or(fallback)
}
pub fn validate(overrides: &BTreeMap<String, String>) -> Result<(), String> {
    for (id, text) in overrides {
        if !IDS.contains(&id.as_str()) { return Err(format!("未知提示词：{}", id)); }
        let limit = if id.contains("test") { 500 } else { 20000 };
        if text.trim().is_empty() || text.chars().count() > limit {
            return Err(format!("提示词 {} 不能为空，且不能超过 {} 字", id, limit));
        }
    }
    Ok(())
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn overrides_are_scoped_and_default_restores_without_affecting_other_modes(){
        let mut values=BTreeMap::from([("ask".into(),"自定义提问规则".into())]);
        assert_eq!(resolve(&values,"ask","默认提问"),"自定义提问规则");
        assert_eq!(resolve(&values,"layered","默认分层"),"默认分层");
        values.remove("ask");assert_eq!(resolve(&values,"ask","默认提问"),"默认提问");
    }
    #[test] fn rejects_unknown_blank_and_oversized_templates() {
        for (id, text) in [("unknown", "正文".into()), ("ask", "  ".into()), ("connection_test", "字".repeat(501))] {
            assert!(validate(&BTreeMap::from([(id.into(), text)])).is_err());
        }
        assert!(validate(&BTreeMap::new()).is_ok());
        assert!(validate(&BTreeMap::from([("ask".into(), "按课堂内容解释，返回JSON".into())])).is_ok());
    }
}
