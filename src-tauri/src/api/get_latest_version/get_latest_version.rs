use crate::utils::context::app_context::AppContext;

pub const LATEST_VERSION_URL: &str = "https://raw.githubusercontent.com/AkasakiTomohiro/formation-docs/refs/heads/master/version.json";

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

#[cfg(test)]
#[coverage(off)]
mod get_latest_version_tests {
    use std::sync::Arc;

    use bytes::Bytes;

    use crate::{
        api::get_latest_version::get_latest_version::get_latest_version,
        utils::context::{
            app_context::AppContext,
            file::MockFileSystem,
            http_client::{HttpClientError, MockHttpClient},
        },
    };

    /// latest_versionの取得に成功すること
    #[tokio::test]
    async fn success() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mut mock_http_client = MockHttpClient::new();

        let json = r#"{ "version": "1.1.1" }"#;
        mock_http_client
            .expect_get()
            .returning(move |_| Ok(Bytes::from(json)));

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let result = get_latest_version(&app_context).await;

        // ######### 検証 #########
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "1.1.1");
    }

    /// version.jsonの取得に失敗した場合、Noneを返すこと
    #[tokio::test]
    async fn http_client_get_err() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mut mock_http_client = MockHttpClient::new();

        mock_http_client.expect_get().returning(move |_| {
            Err(HttpClientError::Reqwest(
                "http client get error".to_string(),
            ))
        });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let result = get_latest_version(&app_context).await;

        // ######### 検証 #########
        assert!(result.is_none());
    }

    /// serde_json::from_sliceに失敗した場合、Noneを返すこと
    #[tokio::test]
    async fn from_slice_err() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mut mock_http_client = MockHttpClient::new();

        let json = "not json";
        mock_http_client
            .expect_get()
            .returning(move |_| Ok(Bytes::from(json)));

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let result = get_latest_version(&app_context).await;

        // ######### 検証 #########
        assert!(result.is_none());
    }

    /// version.jsonにversionプロパティが存在しない場合、Noneを返すこと
    #[tokio::test]
    async fn version_property_missing_err() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mut mock_http_client = MockHttpClient::new();

        let json = r#"{ "noVersion": "1.1.1" }"#;
        mock_http_client
            .expect_get()
            .returning(move |_| Ok(Bytes::from(json)));

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let result = get_latest_version(&app_context).await;

        // ######### 検証 #########
        assert!(result.is_none());
    }

    /// versionが文字列でない場合、Noneを返すこと
    #[tokio::test]
    async fn version_not_string_err() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mut mock_http_client = MockHttpClient::new();

        let json = r#"{ "version": 123 }"#;
        mock_http_client
            .expect_get()
            .returning(move |_| Ok(Bytes::from(json)));

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let result = get_latest_version(&app_context).await;

        // ######### 検証 #########
        assert!(result.is_none());
    }
}
