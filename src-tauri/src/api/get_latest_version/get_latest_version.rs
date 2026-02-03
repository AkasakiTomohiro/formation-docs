use crate::utils::context::app_context::AppContext;

pub const LATEST_VERSION_URL: &str = "https://raw.githubusercontent.com/AkasakiTomohiro/formation-docs/refs/heads/feature/version-check/version.json";

pub async fn get_latest_version(ctx: &AppContext) -> Option<String> {
    // version.jsonを取得
    let latest_version_json = ctx.http_client.get(LATEST_VERSION_URL.to_string()).await;
    if latest_version_json.is_err() {
        return None;
    }
    return match serde_json::from_slice::<serde_json::Value>(&latest_version_json.unwrap()) {
        Ok(json) => {
            let version = json.get("version");
            if version.is_none() {
                return None;
            }
            let version = version.unwrap();
            if !version.is_string() {
                return None;
            }
            return Some(version.as_str().unwrap().to_string());
        }
        Err(err) => {
            return None;
        }
    };
}
