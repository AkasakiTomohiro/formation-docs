use crate::utils::context::file::FileSystem;
use crate::utils::{AppError, ConfigMigratable};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

const STACK_META_CONFIG_LATEST_VERSION: u32 = 1;

///
/// スタックメタコンフィグ v1
///

#[derive(Debug, Serialize, Deserialize)]
pub struct StackMetaConfigResource {
    pub description: String,
    pub reasons: HashMap<String, String>,
}
#[derive(Debug, Serialize, Deserialize)]
struct StackMetaConfigV1 {
    pub version: u32,
    pub name: String,
    pub description: String,
    pub resources: HashMap<String, StackMetaConfigResource>,
}

impl StackMetaConfigV1 {
    pub fn new(name: &str) -> Self {
        StackMetaConfigV1 {
            version: 1,
            name: name.to_string(),
            description: "".to_string(),
            resources: HashMap::new(),
        }
    }
}

impl ConfigMigratable for StackMetaConfigV1 {
    type Latest = StackMetaConfig;

    fn migrate_boxed(self: Box<Self>) -> Box<dyn ConfigMigratable<Latest = Self::Latest>> {
        Box::new(StackMetaConfig {
            version: STACK_META_CONFIG_LATEST_VERSION,
            name: self.name,
            description: self.description,
            resources: self.resources,
        })
    }
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

///
/// スタックメタコンフィグ 最新バージョン
///

#[derive(Debug, Serialize, Deserialize)]
pub struct StackMetaConfig {
    pub version: u32,
    pub name: String,
    pub description: String,
    pub resources: HashMap<String, StackMetaConfigResource>,
}

impl StackMetaConfig {
    pub fn new(name: &str) -> Self {
        let config: Box<dyn ConfigMigratable<Latest = StackMetaConfig>> =
            Box::new(StackMetaConfigV1::new(name));
        *config
            .migrate_boxed()
            .as_any()
            .downcast::<StackMetaConfig>()
            .expect("must be StackMeta at latest")
    }

    /// ${スタック名}.meta.jsonを読み込む
    ///
    /// versionが最新でない場合はマイグレーションして最新版に変換し、保存する
    /// - 戻り値: 読み込んだStackMetaConfig
    ///
    /// - `workspace_directory`: ワークスペースディレクトリ
    /// - `stack_name`: スタック名
    pub async fn read(
        file_system: Arc<dyn FileSystem>,
        workspace_directory: &str,
        stack_name: &str,
    ) -> Result<StackMetaConfig, StackMetaConfigError> {
        let version =
            read_config_version(file_system.clone(), workspace_directory, stack_name).await?;
        let config_path = stack_meta_config_path(workspace_directory, &stack_name);

        match version {
            1 => {
                let config_json = file_system.read_file(&config_path).await?;
                let config_json = serde_json::from_str::<StackMetaConfigV1>(&config_json)?;
                let boxed = Box::new(config_json);
                Ok(boxed.migrate_until_latest())
            }
            _ => {
                let config_json = StackMetaConfig::new(stack_name);
                Ok(StackMetaConfig::write(
                    config_json,
                    file_system.clone(),
                    workspace_directory,
                    stack_name,
                )
                .await?)
            }
        }
    }

    /// ${スタック名}.meta.jsonへ書き込む
    ///
    /// - `workspace_directory`: ワークスペースディレクトリ
    /// - `stack_name`: スタック名
    pub async fn write(
        config: StackMetaConfig,
        file_system: Arc<dyn FileSystem>,
        workspace_directory: &str,
        stack_name: &str,
    ) -> Result<StackMetaConfig, StackMetaConfigError> {
        let config_path = stack_meta_config_path(workspace_directory, stack_name);
        let config_json = serde_json::to_string(&config)?;
        file_system
            .write_file(&config_path, config_json.as_bytes())
            .await?;
        return Ok(config);
    }
}

impl ConfigMigratable for StackMetaConfig {
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

#[derive(Debug, Error)]
pub enum StackMetaConfigError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// ${スタック名}.meta.jsonのパスを取得
fn stack_meta_config_path(workspace_directory: &str, stack_name: &str) -> PathBuf {
    let meta_path = PathBuf::from(workspace_directory)
        .join(stack_name)
        .with_extension("meta.json");
    let meta_path = PathBuf::from(&meta_path);
    return meta_path;
}

/// ${スタック名}.meta.jsonのバージョンを取得
async fn read_config_version(
    file_system: Arc<dyn FileSystem>,
    workspace_directory: &str,
    stack_name: &str,
) -> Result<u32, StackMetaConfigError> {
    let meta_path = stack_meta_config_path(workspace_directory, stack_name);
    match file_system.path_exists(&meta_path) {
        true => {
            let config_json = file_system.read_file(&meta_path).await?;
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
/// StackMetaConfigV1構造体用のテスト
mod stack_meta_config_v1_tests {
    use super::*;

    /// StackMetaConfigV1の初期生成データが正しいことを確認
    #[test]
    fn stack_meta_config_v1_new() {
        // ######### 準備 #########
        let name = "ConfigName";

        // ######### 実行 #########
        let config_v1 = StackMetaConfigV1::new(name);

        // ######### 検証 #########
        assert_eq!(config_v1.version, 1);
        assert_eq!(config_v1.name, name.to_string());
        assert_eq!(config_v1.description, "".to_string());
        assert_eq!(config_v1.reasons.is_empty(), true);
    }

    /// StackMetaConfigV1をマイグレーションしたときに、StackMetaConfigに変換できることを確認
    #[test]
    fn stack_meta_config_v1_migrate() {
        // ######### 準備 #########
        let name = "ConfigName";
        let config_v1 = StackMetaConfigV1::new(name);

        // ######### 実行 #########
        let boxed: Box<dyn ConfigMigratable<Latest = StackMetaConfig>> = Box::new(config_v1);
        let migrated = boxed.migrate_boxed();

        // V1 -> StackMetaConfigへマイグレーション
        let config_latest = migrated
            .as_any()
            .downcast::<StackMetaConfig>()
            .expect("must be StackMetaConfig at latest");

        // ######### 検証 #########
        assert_eq!(config_latest.version, STACK_META_CONFIG_LATEST_VERSION);
        assert_eq!(config_latest.name, name.to_string());
        assert_eq!(config_latest.description, "".to_string());
        assert_eq!(config_latest.reasons.is_empty(), true);
        assert_eq!(config_latest.descriptions.is_empty(), true);
    }

    #[test]
    fn stack_meta_config_v1_as_any() {
        // ######### 準備 #########
        let name = "ConfigName";
        let config_v1 = StackMetaConfigV1::new(name);

        // ######### 実行 #########
        let boxed: Box<dyn ConfigMigratable<Latest = StackMetaConfig>> = Box::new(config_v1);
        let any_boxed = boxed.as_any();
        let downcasted = any_boxed
            .downcast::<StackMetaConfigV1>()
            .expect("must be StackMetaConfigV1");

        // ######### 検証 #########
        assert_eq!(downcasted.version, 1);
        assert_eq!(downcasted.name, name.to_string());
        assert_eq!(downcasted.description, "".to_string());
        assert_eq!(downcasted.reasons.is_empty(), true);
    }
}

#[cfg(test)]
#[coverage(off)]
/// StackMetaConfig構造体用のテスト
mod stack_meta_config_tests {
    use super::*;

    #[test]
    fn stack_meta_config_new() {
        // ######### 準備 #########
        let name = "ConfigName";

        // ######### 実行 #########
        let config_latest = StackMetaConfig::new(name);

        // ######### 検証 #########
        assert_eq!(config_latest.version, STACK_META_CONFIG_LATEST_VERSION);
        assert_eq!(config_latest.name, name.to_string());
        assert_eq!(config_latest.description, "".to_string());
        assert_eq!(config_latest.reasons.is_empty(), true);
    }

    #[test]
    fn stack_meta_config_migrate_boxed() {
        // ######### 準備 #########
        let name = "ConfigName";
        let config_latest = StackMetaConfig::new(name);

        // ######### 実行 #########
        let boxed: Box<dyn ConfigMigratable<Latest = StackMetaConfig>> = Box::new(config_latest);

        // ######### 検証 #########
        let migrated = boxed.migrate_boxed();
        let downcasted = migrated.as_any().downcast::<StackMetaConfig>();

        // downcastに成功していること
        assert!(downcasted.is_ok());

        // StackMetaConfigがStackMetaConfigに変換されていること
        let config_latest = downcasted.unwrap();
        assert_eq!(config_latest.version, STACK_META_CONFIG_LATEST_VERSION);
        assert_eq!(config_latest.name, name.to_string());
        assert_eq!(config_latest.description, "".to_string());
        assert_eq!(config_latest.reasons.is_empty(), true);
    }

    mod write_func {
        use super::super::*;
        use crate::utils::context::file::MockFileSystem;

        #[tokio::test]
        async fn stack_meta_config_write_normal() {
            // ######### 準備 #########
            let name = "ConfigName";
            let workspace_dir = "workspace_dir";
            let stack_name = "stack_name";
            let mut mock_file_system = MockFileSystem::new();

            // returningはvitestのimplmentationと同様の動作をする関数
            mock_file_system
                .expect_write_file()
                .returning(|_path, _contents| Ok(()));

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = StackMetaConfig::write(
                StackMetaConfig::new(name),
                file_system,
                workspace_dir,
                stack_name,
            )
            .await;

            // ######### 検証 #########
            // readの戻り値がOkであること
            assert!(result.is_ok());
        }
    }

    mod read_func {
        use super::super::*;
        use crate::utils::context::file::MockFileSystem;
        use tokio::io::{Error, ErrorKind};

        #[tokio::test]
        async fn stack_meta_config_read_normal() {
            // ######### 準備 #########
            let workspace_dir = "workspace_dir";
            let stack_name: &str = "stack_name";
            let mut mock_file_system = MockFileSystem::new();

            mock_file_system
                .expect_path_exists()
                .returning(|_path| true);

            mock_file_system.expect_read_file().returning(move |_path| {
                let config_json = r#"{
                    "version": 1,
                    "name": "name",
                    "description": "description",
                    "reasons": {},
                    "descriptions": {}
                }"#;
                Ok(config_json.to_string())
            });

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = StackMetaConfig::read(file_system, workspace_dir, stack_name).await;

            // ######### 検証 #########
            // readの戻り値がOkであること
            assert!(result.is_ok());

            // readの戻り値の内容が期待通りであること
            let config = result.unwrap();
            assert_eq!(config.version, STACK_META_CONFIG_LATEST_VERSION);
            assert_eq!(config.name, "name".to_string());
            assert_eq!(config.description, "description".to_string());
            assert_eq!(config.reasons.is_empty(), true);
            assert_eq!(config.descriptions.is_empty(), true);
        }

        #[tokio::test]
        async fn stack_meta_config_read_other_version() {
            // ######### 準備 #########
            let workspace_dir = "workspace_dir";
            let stack_name: &str = "stack_name";
            let mut mock_file_system = MockFileSystem::new();

            mock_file_system.expect_path_exists().return_const(true);

            mock_file_system.expect_read_file().returning(move |_path| {
                let config_json = r#"{
                    "version": 0,
                    "name": "name",
                    "description": "description",
                    "reasons": {}
                }"#;
                Ok(config_json.to_string())
            });

            mock_file_system
                .expect_write_file()
                .returning(|_path, _contents| Ok(()));

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = StackMetaConfig::read(file_system, workspace_dir, stack_name).await;

            // ######### 検証 #########
            // readの戻り値がOkであること
            assert!(result.is_ok());

            // readの戻り値の内容が期待通りであること
            let config = result.unwrap();
            assert_eq!(config.version, STACK_META_CONFIG_LATEST_VERSION);
            assert_eq!(config.name, stack_name.to_string());
            assert_eq!(config.description, "".to_string());
            assert_eq!(config.reasons.is_empty(), true);
            assert_eq!(config.descriptions.is_empty(), true);
        }

        #[tokio::test]
        async fn stack_meta_config_read_file_not_exist() {
            // ######### 準備 #########
            let workspace_dir = "workspace_dir";
            let stack_name: &str = "stack_name";
            let mut mock_file_system = MockFileSystem::new();

            mock_file_system.expect_path_exists().return_const(false);

            mock_file_system
                .expect_write_file()
                .returning(|_path, _contents| Ok(()));

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = StackMetaConfig::read(file_system, workspace_dir, stack_name).await;

            // ######### 検証 #########
            // readの戻り値がOkであること
            assert!(result.is_ok());

            // readの戻り値の内容が期待通りであること
            let config = result.unwrap();
            assert_eq!(config.version, STACK_META_CONFIG_LATEST_VERSION);
            assert_eq!(config.name, stack_name.to_string());
            assert_eq!(config.description, "".to_string());
            assert_eq!(config.reasons.is_empty(), true);
            assert_eq!(config.descriptions.is_empty(), true);
        }

        #[tokio::test]
        async fn read_config_version_read_file_err() {
            // ######### 準備 #########
            let workspace_dir = "workspace_dir";
            let stack_name: &str = "stack_name";
            let mut mock_file_system = MockFileSystem::new();

            mock_file_system
                .expect_path_exists()
                .returning(|_path| true);

            mock_file_system
                .expect_read_file()
                .returning(|_path| Err(Error::new(ErrorKind::Other, "read_file error")));

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = StackMetaConfig::read(file_system, workspace_dir, stack_name).await;

            // ######### 検証 #########
            // readの戻り値がErrであること
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn read_from_str_err() {
            // ######### 準備 #########
            let workspace_dir = "workspace_dir";
            let stack_name: &str = "stack_name";
            let mut mock_file_system = MockFileSystem::new();

            mock_file_system
                .expect_path_exists()
                .returning(|_path| true);

            // read_fileのモック
            let mut call_count = 0;
            mock_file_system
                .expect_read_file()
                .times(2)
                .returning_st(move |_path| {
                    call_count += 1;
                    match call_count {
                        1 => {
                            let config_json = r#"{
                                "version": 1,
                                "name": "name",
                                "description": "description",
                                "reasons": {}
                            }"#;
                            Ok(config_json.to_string())
                        }
                        2 => {
                            let config_json = r#"invalid json"#;
                            Ok(config_json.to_string())
                        }
                        _ => panic!("Unexpected call"),
                    }
                });

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = StackMetaConfig::read(file_system, workspace_dir, stack_name).await;

            // ######### 検証 #########
            // readの戻り値がErrであること
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn read_config_version_from_str_err() {
            // ######### 準備 #########
            let workspace_dir = "workspace_dir";
            let stack_name: &str = "stack_name";
            let mut mock_file_system = MockFileSystem::new();

            mock_file_system
                .expect_path_exists()
                .returning(|_path| true);

            // read_fileのモック
            mock_file_system.expect_read_file().returning(move |_path| {
                let config_json = r#"invalid json"#;
                Ok(config_json.to_string())
            });

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = StackMetaConfig::read(file_system, workspace_dir, stack_name).await;

            // ######### 検証 #########
            // readの戻り値がErrであること
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn read_config_read_second_err_read_file() {
            // ######### 準備 #########
            let workspace_dir = "workspace_dir";
            let stack_name: &str = "stack_name";
            let mut mock_file_system = MockFileSystem::new();

            mock_file_system
                .expect_path_exists()
                .returning(|_path| true);

            // read_fileのモック
            let mut call_count = 0;
            mock_file_system
                .expect_read_file()
                .times(2)
                .returning_st(move |_path| {
                    call_count += 1;
                    match call_count {
                        1 => {
                            let config_json = r#"{
                                "version": 1,
                                "name": "name",
                                "description": "description",
                                "reasons": {}
                            }"#;
                            Ok(config_json.to_string())
                        }
                        2 => Err(Error::new(ErrorKind::Other, "read_file error")), // 2回目の呼び出しでErrを返す
                        _ => panic!("Unexpected call"),
                    }
                });

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = StackMetaConfig::read(file_system, workspace_dir, stack_name).await;

            // ######### 検証 #########
            // readの戻り値がErrであること
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn app_config_read_unexpected_version_write_file_err() {
            // ######### 準備 #########
            let workspace_dir = "workspace_dir";
            let stack_name: &str = "stack_name";
            let mut mock_file_system = MockFileSystem::new();

            mock_file_system
                .expect_path_exists()
                .returning(|_path| false);

            mock_file_system
                .expect_write_file()
                .returning(|_path, _contents| {
                    Err(Error::new(ErrorKind::Other, "write_file error"))
                });

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = StackMetaConfig::read(file_system, workspace_dir, stack_name).await;

            // ######### 検証 #########
            // readの戻り値がErrであること
            assert!(result.is_err());
        }
    }
}
