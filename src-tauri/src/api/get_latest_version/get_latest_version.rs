use crate::utils::context::app_context::AppContext;

pub const LATEST_VERSION_URL: &str = "https://raw.githubusercontent.com/AkasakiTomohiro/formation-docs/refs/heads/feature/version-check/version.json";

pub async fn get_latest_version(ctx: &AppContext) -> Option<String> {
    // version.jsonを取得
    let response = ctx
        .http_client
        .get(LATEST_VERSION_URL.to_string())
        .await
        .ok()?;

    // 取得結果をデシリアライズ
    let json = serde_json::from_slice::<serde_json::Value>(&response).ok()?;

    // versionを返す
    return Some(json.get("version")?.as_str()?.to_string());
}
