use crate::utils::context::file::FileSystem;
use crate::utils::{AppError, ConfigMigratable};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

const APP_CONFIG_LATEST_VERSION: u32 = 1;
const APP_CONFIG_FILE_NAME: &str = "app_config.json";
pub const APP_CONFIG_DIRECTORY_NAME: &str = "formation-docs";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Language {
    En,
    Ja,
}

///
/// アプリコンフィグ v1
///

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AppConfigV1 {
    pub version: u32,
    pub workspaces: HashMap<String, String>,
    pub initialized: bool,
    pub initialized_at: String,
    pub cf_schema_downloaded_at: String,
    pub language: Language,
}

impl AppConfigV1 {
    pub fn new() -> Self {
        AppConfigV1 {
            version: 1,
            workspaces: HashMap::new(),
            initialized: false,
            initialized_at: "".to_string(),
            cf_schema_downloaded_at: "".to_string(),
            language: Language::En,
        }
    }
}

impl ConfigMigratable for AppConfigV1 {
    type Latest = AppConfig;

    fn migrate_boxed(self: Box<Self>) -> Box<dyn ConfigMigratable<Latest = Self::Latest>> {
        Box::new(AppConfig {
            version: APP_CONFIG_LATEST_VERSION,
            workspaces: self.workspaces,
            initialized: self.initialized,
            initialized_at: self.initialized_at,
            cf_schema_downloaded_at: self.cf_schema_downloaded_at,
            language: self.language,
        })
    }
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

///
/// アプリコンフィグ 最新バージョン
///

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub version: u32,
    pub workspaces: HashMap<String, String>,
    pub initialized: bool,
    pub initialized_at: String,
    pub cf_schema_downloaded_at: String,
    pub language: Language,
}
impl AppConfig {
    pub fn new() -> Self {
        let config: Box<dyn ConfigMigratable<Latest = AppConfig>> = Box::new(AppConfigV1::new());
        *config
            .migrate_boxed()
            .as_any()
            .downcast::<AppConfig>()
            .unwrap()
    }

    /// app_config.jsonを読み込む
    ///
    /// versionが最新でない場合はマイグレーションして最新版に変換し、保存する
    /// - 戻り値: 読み込んだAppConfig
    pub async fn read(file_system: Arc<dyn FileSystem>) -> Result<AppConfig, AppConfigError> {
        let version = read_config_version(file_system.clone()).await?;
        let config_path = app_config_path(file_system.clone())?;
        match version {
            1 => {
                let config_json = file_system.read_file(&config_path).await?;
                if let Ok(config_json) = serde_json::from_str::<AppConfigV1>(&config_json) {
                    let boxed = Box::new(config_json);
                    Ok(boxed.migrate_until_latest())
                } else {
                    let config_json = AppConfig::new();
                    Ok(AppConfig::write(config_json, file_system).await?)
                }
            }
            _ => {
                let config_json = AppConfig::new();
                Ok(AppConfig::write(config_json, file_system).await?)
            }
        }
    }

    /// app_config.jsonへ書き込む
    pub async fn write(
        config: AppConfig,
        file_system: Arc<dyn FileSystem>,
    ) -> Result<AppConfig, AppConfigError> {
        // AppConfigでシリアライズ時にエラーが発生しうる属性（Infinity、NaN等）がないためunwrapしてもpanicは発生しない
        let app_config_json = serde_json::to_string::<AppConfig>(&config).unwrap();
        let app_config_path = app_config_path(file_system.clone())?;

        // ディレクトリが存在しないことがあるため作成する
        file_system
            .create_dir_all(app_config_path.parent().unwrap())
            .await?;

        file_system
            .write_file(&app_config_path, app_config_json.as_bytes())
            .await?;
        Ok(config)
    }
}

impl ConfigMigratable for AppConfig {
    type Latest = Self;

    fn migrate_boxed(self: Box<Self>) -> Box<dyn ConfigMigratable<Latest = Self::Latest>> {
        self
    }
    fn is_latest(&self) -> bool {
        true
    }
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

///
/// アプリコンフィグ エラー
///

#[derive(Debug, Error)]
pub enum AppConfigError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// app_config.jsonのパスを取得
fn app_config_path(file_system: Arc<dyn FileSystem>) -> Result<PathBuf, AppConfigError> {
    let dir = file_system
        .config_local_dir()
        .ok_or(AppConfigError::App(AppError::new(
            "Failed to get local config directory",
        )))?;

    return Ok(dir
        .join(APP_CONFIG_DIRECTORY_NAME)
        .join(APP_CONFIG_FILE_NAME));
}

/// app_config.jsonのバージョンを取得
async fn read_config_version(file_system: Arc<dyn FileSystem>) -> Result<u32, AppConfigError> {
    let config_path = app_config_path(file_system.clone())?;
    match file_system.path_exists(&config_path) {
        true => {
            let config_json = file_system.read_file(&config_path).await?;
            let config_json: Value = serde_json::from_str(&config_json)?;
            let version = config_json
                .get("version")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;
            Ok(version)
        }
        false => Ok(0),
    }
}

#[cfg(test)]
#[coverage(off)]
/// AppConfigV1構造体用のテスト
mod app_config_v1_tests {
    use crate::utils::ConfigMigratable;

    /// AppConfigV1の初期生成データが正しいことを確認
    #[test]
    fn app_config_v1_new() {
        // ######### 準備 #########

        // ######### 実行 #########
        let config_v1 = super::AppConfigV1::new();

        // ######### 検証 #########
        assert_eq!(config_v1.version, 1);
        assert!(config_v1.workspaces.is_empty());
        assert!(config_v1.initialized == false);
        assert!(config_v1.initialized_at.is_empty());
        assert!(config_v1.is_latest() == false);
    }

    /// AppConfigV1をマイグレーションしたときに、AppConfigに変換できることを確認
    #[test]
    fn app_config_v1_migrate_boxed() {
        // ######### 準備 #########
        let mut config_v1 = super::AppConfigV1::new();
        config_v1
            .workspaces
            .insert("default".to_string(), "Default Workspace".to_string());
        config_v1.initialized_at = "2024-01-01T00:00:00Z".to_string();

        // ######### 実行 #########
        let boxed = Box::new(config_v1.clone()).migrate_boxed();

        // ######### 検証 #########
        let downcasted = boxed.as_any().downcast::<super::AppConfig>();

        // downcastに成功していること
        assert!(downcasted.is_ok());

        // AppConfigV1がAppConfigに変換されていること
        let migrate_config = downcasted.unwrap();
        assert!(migrate_config.is_latest());
        assert_eq!(migrate_config.version, super::APP_CONFIG_LATEST_VERSION);
        assert_eq!(migrate_config.workspaces, config_v1.workspaces);
        assert_eq!(migrate_config.initialized, config_v1.initialized);
        assert_eq!(migrate_config.initialized_at, config_v1.initialized_at);
    }

    /// AppConfigV1をas_anyでダウンキャストできることを確認
    #[test]
    fn app_config_v1_as_any() {
        // ######### 準備 #########
        let mut config_v1 = super::AppConfigV1::new();
        config_v1
            .workspaces
            .insert("default".to_string(), "Default Workspace".to_string());
        config_v1.initialized_at = "2024-01-01T00:00:00Z".to_string();

        // ######### 実行 #########
        let boxed = Box::new(config_v1.clone()).as_any();

        // ######### 検証 #########
        let downcasted = boxed.downcast::<super::AppConfigV1>();

        // downcastに成功していること
        assert!(downcasted.is_ok());
    }
}

#[cfg(test)]
#[coverage(off)]
/// AppConfig構造体用のテスト
mod app_config {
    use super::*;
    use crate::utils::ConfigMigratable;

    // AppConfigの初期生成データが正しいことを確認
    #[test]
    fn app_config_new() {
        // ######### 準備 #########

        // ######### 実行 #########
        let config = AppConfig::new();

        // ######### 検証 #########
        assert_eq!(config.version, APP_CONFIG_LATEST_VERSION);
        assert!(config.workspaces.is_empty());
        assert!(config.initialized == false);
        assert!(config.initialized_at.is_empty());
        assert!(config.is_latest());
    }

    /// AppConfigをマイグレーションしたときに、同じデータが返ってくることを確認
    #[test]
    fn app_config_migrate_boxed() {
        // ######### 準備 #########
        let mut config = AppConfig::new();
        config
            .workspaces
            .insert("default".to_string(), "Default Workspace".to_string());
        config.initialized_at = "2024-01-01T00:00:00Z".to_string();

        // ######### 実行 #########
        let boxed = Box::new(config.clone()).migrate_boxed();

        // ######### 検証 #########
        let downcasted = boxed.as_any().downcast::<super::AppConfig>();

        // downcastに成功していること
        assert!(downcasted.is_ok());

        // AppConfigがAppConfigに変換されていること
        let migrate_config = downcasted.unwrap();
        assert!(migrate_config.is_latest());
        assert_eq!(migrate_config.version, config.version);
        assert_eq!(migrate_config.workspaces, config.workspaces);
        assert_eq!(migrate_config.initialized, config.initialized);
        assert_eq!(migrate_config.initialized_at, config.initialized_at);
    }

    mod write_func {
        use crate::utils::context::file::MockFileSystem;
        use std::io::{Error, ErrorKind};
        use std::path::PathBuf;
        use std::str::FromStr;
        use std::sync::Arc;

        /// versionが1のときの呼び出しでAppConfig::writeがErrを返す場合、AppConfig::readがErrを返すことを確認
        #[tokio::test]
        async fn app_config_read_err_first_write() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // config_local_dirのモック
            mock_file_system
                .expect_config_local_dir()
                .times(3)
                .returning_st({
                    let mut call_count = 0;
                    move || {
                        call_count += 1;
                        match call_count {
                            1 => Some(PathBuf::from_str("config_local_dir_path").unwrap()),
                            2 => Some(PathBuf::from_str("config_local_dir_path").unwrap()),
                            3 => None, // 3回目の呼び出しでNoneを返す
                            _ => panic!("Unexpected call"),
                        }
                    }
                });

            // path_existsのモック
            mock_file_system.expect_path_exists().return_const(true);

            // read_fileのモック
            let mut call_count = 0;
            mock_file_system
                .expect_read_file()
                .times(2)
                .returning_st(move |_path| {
                    call_count += 1;
                    match call_count {
                        1 => {
                            let app_config_json = r#"{
                                "version": 1,
                                "workspaces": {},
                                "initialized_at": ""
                            }"#;
                            Ok(app_config_json.to_string())
                        }
                        2 => {
                            // 2回目の呼び出しで不正な文字列を返す
                            let app_config_json = r#"invalid json"#;
                            Ok(app_config_json.to_string())
                        }
                        _ => panic!("Unexpected call"),
                    }
                });

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = super::AppConfig::read(file_system).await;

            // ######### 検証 #########
            // readの戻り値がErrであること
            assert!(result.is_err());
        }

        /// versionが1でないときの呼び出しでAppConfig::writeがErrを返す場合、AppConfig::readがErrを返すことを確認
        #[tokio::test]
        async fn app_config_read_err_second_write() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // config_local_dirのモック
            mock_file_system
                .expect_config_local_dir()
                .times(3)
                .returning_st({
                    let mut call_count = 0;
                    move || {
                        call_count += 1;
                        match call_count {
                            1 => Some(PathBuf::from_str("config_local_dir_path").unwrap()),
                            2 => Some(PathBuf::from_str("config_local_dir_path").unwrap()),
                            3 => None, // 3回目の呼び出しでNoneを返す
                            _ => panic!("Unexpected call"),
                        }
                    }
                });

            // path_existsのモック
            mock_file_system.expect_path_exists().return_const(true);

            // read_fileのモック
            mock_file_system.expect_read_file().returning(|_path| {
                let app_config_json = r#"{
                    "version": 2,
                    "workspaces": {},
                    "initialized_at": ""
                }"#;
                Ok(app_config_json.to_string())
            });

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = super::AppConfig::read(file_system).await;

            // ######### 検証 #########
            // readの戻り値がErrであること
            assert!(result.is_err());
        }

        /// AppConfig::writeが正常に動作することを確認
        #[tokio::test]
        async fn app_config_write_normal() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // config_local_dirのモック
            mock_file_system
                .expect_config_local_dir()
                .return_const(Some(
                    PathBuf::from_str("parent/config_local_dir_path").unwrap(),
                ));

            // create_dir_allのモック
            mock_file_system
                .expect_create_dir_all()
                .returning(|_path| Ok(()));

            // write_fileのモック
            mock_file_system
                .expect_write_file()
                .returning(|_path, _contents| Ok(()));

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = super::AppConfig::write(super::AppConfig::new(), file_system).await;

            // ######### 検証 #########
            // readの戻り値がOkであること
            assert!(result.is_ok());
        }

        // config_local_dirが取得できない場合、AppConfig::writeがErrを返すことを確認
        #[tokio::test]
        async fn app_config_write_err_config_local_dir() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // config_local_dirのモック
            mock_file_system
                .expect_config_local_dir()
                .return_const(None);

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = super::AppConfig::write(super::AppConfig::new(), file_system).await;

            // ######### 検証 #########
            // readの戻り値がErrであること
            assert!(result.is_err());
        }

        // create_dir_allがErrを返す場合、AppConfig::writeがErrを返すことを確認
        #[tokio::test]
        async fn app_config_write_err_create_dir_all() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // config_local_dirのモック
            mock_file_system
                .expect_config_local_dir()
                .return_const(Some(
                    PathBuf::from_str("parent/config_local_dir_path").unwrap(),
                ));

            // create_dir_allのモック
            mock_file_system
                .expect_create_dir_all()
                .returning(|_path| Err(Error::new(ErrorKind::Other, "create_dir_all error")));

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = super::AppConfig::write(super::AppConfig::new(), file_system).await;

            // ######### 検証 #########
            // readの戻り値がErrであること
            assert!(result.is_err());
        }

        // write_fileがErrを返す場合、AppConfig::writeがErrを返すことを確認
        #[tokio::test]
        async fn app_config_write_err_write_file() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // config_local_dirのモック
            mock_file_system
                .expect_config_local_dir()
                .return_const(Some(
                    PathBuf::from_str("parent/config_local_dir_path").unwrap(),
                ));

            // create_dir_allのモック
            mock_file_system
                .expect_create_dir_all()
                .returning(|_path| Ok(()));

            // write_fileのモック
            mock_file_system
                .expect_write_file()
                .returning(|_path, _contents| {
                    Err(Error::new(ErrorKind::Other, "write_file error"))
                });

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = super::AppConfig::write(super::AppConfig::new(), file_system).await;

            // ######### 検証 #########
            // readの戻り値がErrであること
            assert!(result.is_err());
        }
    }

    mod read_func {
        use super::super::*;
        use crate::utils::context::file::MockFileSystem;
        use std::io::{Error, ErrorKind};
        use std::path::PathBuf;
        use std::str::FromStr;
        use std::sync::Arc;

        /// app_configが存在するバージョンであるとき、AppConfig::readが正しく動作することを確認
        #[tokio::test]
        async fn app_config_read_normal() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // config_local_dirのモック
            mock_file_system
                .expect_config_local_dir()
                .return_const(Some(PathBuf::from_str("config_local_dir_path").unwrap()));

            // path_existsのモック
            mock_file_system.expect_path_exists().return_const(true);

            // read_fileのモック
            mock_file_system.expect_read_file().returning(|_path| {
                let app_config_json = r#"{
                    "version": 1,
                    "workspaces": {},
                    "initialized": false,
                    "initialized_at": "",
                    "cf_schema_downloaded_at": ""
                }"#;
                Ok(app_config_json.to_string())
            });

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = AppConfig::read(file_system).await;

            // ######### 検証 #########
            // readの戻り値がOkであること
            assert!(result.is_ok());

            // readの戻り値の内容が期待通りであること
            let app_config = result.unwrap();
            assert_eq!(app_config.version, APP_CONFIG_LATEST_VERSION);
            assert!(app_config.workspaces.is_empty());
            assert!(app_config.initialized == false);
            assert!(app_config.initialized_at.is_empty());
            assert!(app_config.cf_schema_downloaded_at.is_empty());
        }

        /// app_configが存在しない場合、AppConfig::readが正しく動作することを確認
        #[tokio::test]
        async fn app_config_read_no_exist() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // config_local_dirのモック
            mock_file_system
                .expect_config_local_dir()
                .return_const(Some(PathBuf::from_str("config_local_dir_path").unwrap()));

            // path_existsのモック
            mock_file_system.expect_path_exists().return_const(false);

            // write_fileのモック
            mock_file_system
                .expect_write_file()
                .returning(|_path, _contents| Ok(()));

            // create_dir_allのモック
            mock_file_system
                .expect_create_dir_all()
                .returning(|_path| Ok(()));

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = AppConfig::read(file_system).await;

            // ######### 検証 #########
            // readの戻り値がOkであること
            assert!(result.is_ok());

            // readの戻り値の内容が期待通りであること
            let app_config = result.unwrap();
            assert_eq!(app_config.version, APP_CONFIG_LATEST_VERSION);
            assert!(app_config.workspaces.is_empty());
            assert!(app_config.initialized == false);
            assert!(app_config.initialized_at.is_empty());
        }

        /// app_configが想定外のバージョンであるとき、AppConfig::readが正しく動作することを確認
        #[tokio::test]
        async fn app_config_read_unexpected_version() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // config_local_dirのモック
            mock_file_system
                .expect_config_local_dir()
                .return_const(Some(PathBuf::from_str("config_local_dir_path").unwrap()));

            // path_existsのモック
            mock_file_system.expect_path_exists().return_const(true);

            // read_fileのモック
            mock_file_system.expect_read_file().returning(|_path| {
                let app_config_json = r#"{
                    "version": -1,
                    "workspaces": {},
                    "initialized": false,
                    "initialized_at": ""
                }"#;
                Ok(app_config_json.to_string())
            });

            // write_fileのモック
            mock_file_system
                .expect_write_file()
                .returning(|_path, _contents| Ok(()));

            // create_dir_allのモック
            mock_file_system
                .expect_create_dir_all()
                .returning(|_path| Ok(()));

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = AppConfig::read(file_system).await;

            // ######### 検証 #########
            // readの戻り値がOkであること
            assert!(result.is_ok());

            // readの戻り値の内容が期待通りであること
            let app_config = result.unwrap();
            assert_eq!(app_config.version, APP_CONFIG_LATEST_VERSION);
            assert!(app_config.workspaces.is_empty());
            assert!(app_config.initialized == false);
            assert!(app_config.initialized_at.is_empty());
        }

        /// app_configが適切な形でないとき、AppConfig::readが正しく動作することを確認
        #[tokio::test]
        async fn app_config_read_invalid() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // config_local_dirのモック
            mock_file_system
                .expect_config_local_dir()
                .return_const(Some(PathBuf::from_str("config_local_dir_path").unwrap()));

            // path_existsのモック
            mock_file_system.expect_path_exists().return_const(true);

            // read_fileのモック
            mock_file_system.expect_read_file().returning(|_path| {
                let app_config_json = r#"{
                    "version": 1,
                    "workspaces": {},
                    "initialized_at": ""
                }"#;
                Ok(app_config_json.to_string())
            });

            // write_fileのモック
            mock_file_system
                .expect_write_file()
                .returning(|_path, _contents| Ok(()));

            // create_dir_allのモック
            mock_file_system
                .expect_create_dir_all()
                .returning(|_path| Ok(()));

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = AppConfig::read(file_system).await;

            // ######### 検証 #########
            // readの戻り値がOkであること
            assert!(result.is_ok());

            // readの戻り値の内容が期待通りであること
            let app_config = result.unwrap();
            assert_eq!(app_config.version, APP_CONFIG_LATEST_VERSION);
            assert!(app_config.workspaces.is_empty());
            assert!(app_config.initialized == false);
            assert!(app_config.initialized_at.is_empty());
        }

        /// 1回目の呼び出しでconfig_local_dirが取得できない場合、AppConfig::readがErrを返すことを確認
        #[tokio::test]
        async fn app_config_read_first_err_config_local_dir() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // config_local_dirのモック
            mock_file_system
                .expect_config_local_dir()
                .return_const(None);

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = AppConfig::read(file_system).await;

            // ######### 検証 #########
            // readの戻り値がErrであること
            assert!(result.is_err());
        }

        /// 1回目の呼び出しでread_fileがErrを返す場合、AppConfig::readがErrを返すことを確認
        #[tokio::test]
        async fn app_config_read_first_err_read_file() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // config_local_dirのモック
            mock_file_system
                .expect_config_local_dir()
                .return_const(Some(PathBuf::from_str("config_local_dir_path").unwrap()));

            // path_existsのモック
            mock_file_system.expect_path_exists().return_const(true);

            // read_fileのモック
            mock_file_system
                .expect_read_file()
                .returning(|_path| Err(Error::new(ErrorKind::Other, "read_file error")));

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = AppConfig::read(file_system).await;

            // ######### 検証 #########
            // readの戻り値がErrであること
            assert!(result.is_err());
        }

        /// serde_json::from_strがErrを返す場合、AppConfig::readがErrを返すことを確認
        #[tokio::test]
        async fn app_config_read_err_json_from_str() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // config_local_dirのモック
            mock_file_system
                .expect_config_local_dir()
                .return_const(Some(PathBuf::from_str("config_local_dir_path").unwrap()));

            // path_existsのモック
            mock_file_system.expect_path_exists().return_const(true);

            // read_fileのモック
            mock_file_system.expect_read_file().returning(|_path| {
                let app_config_json = r#"invalid json"#;
                Ok(app_config_json.to_string())
            });

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = AppConfig::read(file_system).await;

            // ######### 検証 #########
            // readの戻り値がErrであること
            assert!(result.is_err());
        }

        /// 2回目の呼び出しでconfig_local_dirが取得できない場合、AppConfig::readがErrを返すことを確認
        #[tokio::test]
        async fn app_config_read_second_err_config_local_dir() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // config_local_dirのモック
            mock_file_system
                .expect_config_local_dir()
                .times(2)
                .returning_st({
                    let mut call_count = 0;
                    move || {
                        call_count += 1;
                        match call_count {
                            1 => Some(PathBuf::from_str("config_local_dir_path").unwrap()),
                            2 => None, // 2回目の呼び出しでNoneを返す
                            _ => panic!("Unexpected call"),
                        }
                    }
                });

            // path_existsのモック
            mock_file_system.expect_path_exists().return_const(true);

            // read_fileのモック
            mock_file_system.expect_read_file().returning(|_path| {
                let app_config_json = r#"{
                    "version": 1,
                    "workspaces": {},
                    "initialized_at": ""
                }"#;
                Ok(app_config_json.to_string())
            });

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = AppConfig::read(file_system).await;

            // ######### 検証 #########
            // readの戻り値がErrであること
            assert!(result.is_err());
        }

        /// 2回目の呼び出しでread_fileがErrを返す場合、AppConfig::readがErrを返すことを確認
        #[tokio::test]
        async fn app_config_read_second_err_read_file() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // config_local_dirのモック
            mock_file_system
                .expect_config_local_dir()
                .return_const(Some(PathBuf::from_str("config_local_dir_path").unwrap()));

            // path_existsのモック
            mock_file_system.expect_path_exists().return_const(true);

            // read_fileのモック
            let mut call_count = 0;
            mock_file_system
                .expect_read_file()
                .times(2)
                .returning_st(move |_path| {
                    call_count += 1;
                    match call_count {
                        1 => {
                            let app_config_json = r#"{
                                "version": 1,
                                "workspaces": {},
                                "initialized_at": ""
                            }"#;
                            Ok(app_config_json.to_string())
                        }
                        2 => Err(Error::new(ErrorKind::Other, "read_file error")), // 2回目の呼び出しでErrを返す
                        _ => panic!("Unexpected call"),
                    }
                });

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = AppConfig::read(file_system).await;

            // ######### 検証 #########
            // readの戻り値がErrであること
            assert!(result.is_err());
        }
    }
}
