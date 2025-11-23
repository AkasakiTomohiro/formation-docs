use crate::utils::context::file::FileSystem;
use crate::utils::{AppError, ConfigMigratable};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

const WORKSPACE_FILE_NAME: &str = "workspace.json";
const WORKSPACE_CONFIG_LATEST_VERSION: u32 = 1;

///
/// ワークスペースコンフィグ v1
///

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WorkspaceConfigV1 {
    pub version: u32,
    // stack_id: stack_file_nameのマッピング
    pub stacks: HashMap<String, String>,
    pub name: String,
    pub description: String,
}

impl WorkspaceConfigV1 {
    pub fn new(name: &str) -> Self {
        WorkspaceConfigV1 {
            version: 1,
            stacks: HashMap::new(),
            name: name.to_string().chars().take(256).collect(),
            description: "".to_string().chars().take(256).collect(),
        }
    }
}

impl ConfigMigratable for WorkspaceConfigV1 {
    type Latest = WorkspaceConfig;

    fn migrate_boxed(self: Box<Self>) -> Box<dyn ConfigMigratable<Latest = Self::Latest>> {
        Box::new(WorkspaceConfig {
            version: WORKSPACE_CONFIG_LATEST_VERSION,
            stacks: self.stacks,
            name: self.name,
            description: self.description,
        })
    }
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

///
/// ワークスペースコンフィグ 最新バージョン
///

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceConfig {
    pub version: u32,
    pub stacks: HashMap<String, String>,
    pub name: String,
    pub description: String,
}
impl WorkspaceConfig {
    pub fn new(name: &str) -> Self {
        let config: Box<dyn ConfigMigratable<Latest = WorkspaceConfig>> =
            Box::new(WorkspaceConfigV1::new(name));
        *config
            .migrate_boxed()
            .as_any()
            .downcast::<WorkspaceConfig>()
            .unwrap()
    }

    /// workspace.jsonを読み込む
    ///
    /// versionが最新でない場合はマイグレーションして最新版に変換し、保存する
    /// - 戻り値: 読み込んだWorkspaceConfig
    pub async fn read(
        file_system: Arc<dyn FileSystem>,
        workspace_directory: &str,
    ) -> Result<WorkspaceConfig, WorkspaceConfigError> {
        let version = read_config_version(file_system.clone(), workspace_directory).await?;
        let config_path = workspace_config_path(workspace_directory);

        match version {
            1 => {
                let config_json = file_system.read_file(&config_path).await?;
                let config_json = serde_json::from_str::<WorkspaceConfigV1>(&config_json)?;
                let boxed = Box::new(config_json);
                Ok(boxed.migrate_until_latest())
            }
            _ => {
                let name = Path::new(workspace_directory)
                    .file_name()
                    .and_then(|f| f.to_str());
                if name.is_none() {
                    return Err(WorkspaceConfigError::App(AppError::new(
                        "Invalid directory",
                    )));
                }
                let name = name.unwrap();
                let config_json = WorkspaceConfig::new(name);
                config_json
                    .write(file_system.clone(), workspace_directory)
                    .await?;
                Ok(config_json)
            }
        }
    }

    /// workspace.jsonへ書き込む
    ///
    /// - `workspace_directory`: ワークスペースディレクトリ
    pub async fn write(
        &self,
        file_system: Arc<dyn FileSystem>,
        workspace_directory: &str,
    ) -> Result<(), WorkspaceConfigError> {
        let workspace_config_json = serde_json::to_string::<WorkspaceConfig>(self)?;
        let workspace_config_path = workspace_config_path(workspace_directory);
        file_system
            .write_file(&workspace_config_path, workspace_config_json.as_bytes())
            .await?;
        return Ok(());
    }
}

impl ConfigMigratable for WorkspaceConfig {
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
pub enum WorkspaceConfigError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// workspace.jsonのパスを取得
fn workspace_config_path(workspace_directory: &str) -> PathBuf {
    return PathBuf::from(workspace_directory).join(WORKSPACE_FILE_NAME);
}

/// workspace.jsonのバージョンを取得
async fn read_config_version(
    file_system: Arc<dyn FileSystem>,
    workspace_directory: &str,
) -> Result<u32, WorkspaceConfigError> {
    let config_path = workspace_config_path(workspace_directory);
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
mod workspace_config_v1_tests {
    use super::*;

    /// WorkspaceConfigV1::new で構造体が正しく作成されること
    #[test]
    fn workspace_config_v1_new() {
        // ######### 準備 #########

        // ######### 実行 #########
        let result = WorkspaceConfigV1::new("test");

        // ######### 検証 #########
        assert_eq!(result.version, 1);
        assert_eq!(result.stacks, HashMap::new());
        assert_eq!(result.name, "test");
        assert_eq!(result.description, "");
    }

    /// WorkspaceConfigV1::migrate_boxed で最新バージョンにマイグレーションされること
    #[test]
    fn workspace_config_v1_migrate_boxed() {
        // ######### 準備 #########
        let mut config = WorkspaceConfigV1::new("test");
        config
            .stacks
            .insert("stack1".to_string(), "stack1.json".to_string());
        config.description = "This is a test workspace".to_string();

        // ######### 実行 #########
        let migrated = Box::new(config.clone()).migrate_boxed();

        // ######### 検証 #########
        assert!(migrated.is_latest());

        let downcast = migrated.as_any().downcast::<WorkspaceConfig>().unwrap();
        assert_eq!(downcast.version, WORKSPACE_CONFIG_LATEST_VERSION);
        assert_eq!(downcast.stacks, config.stacks);
        assert_eq!(downcast.name, config.name);
        assert_eq!(downcast.description, config.description);
    }

    /// WorkspaceConfigV1::as_any を使ってダウンキャストできること
    #[test]
    fn workspace_config_v1_as_any() {
        // ######### 準備 #########
        let config = WorkspaceConfigV1::new("test");

        // ######### 実行 #########
        let any_box = Box::new(config).as_any();
        let downcast = any_box.downcast::<WorkspaceConfigV1>();

        // ######### 検証 #########
        assert!(downcast.is_ok());
    }
}

#[cfg(test)]
#[coverage(off)]
mod workspace_config_tests {
    use super::*;

    /// WorkspaceConfig::new で構造体が正しく作成されること
    #[test]
    fn workspace_config_new() {
        // ######### 準備 #########

        // ######### 実行 #########
        let result = WorkspaceConfig::new("test");

        // ######### 検証 #########
        assert_eq!(result.version, WORKSPACE_CONFIG_LATEST_VERSION);
        assert_eq!(result.stacks, HashMap::new());
        assert_eq!(result.name, "test");
        assert_eq!(result.description, "");
    }

    /// WorkspaceConfig::migrate_boxed で最新バージョンにマイグレーションされること
    #[test]
    fn workspace_config_migrate_boxed() {
        // ######### 準備 #########
        let mut config = WorkspaceConfig::new("test");
        config
            .stacks
            .insert("stack1".to_string(), "stack1.json".to_string());
        config.description = "This is a test workspace".to_string();

        // ######### 実行 #########
        let migrated = Box::new(config.clone()).migrate_boxed();

        // ######### 検証 #########
        assert!(migrated.is_latest());

        let downcast = migrated.as_any().downcast::<WorkspaceConfig>().unwrap();
        assert_eq!(downcast.version, WORKSPACE_CONFIG_LATEST_VERSION);
        assert_eq!(downcast.stacks, config.stacks);
        assert_eq!(downcast.name, config.name);
        assert_eq!(downcast.description, config.description);
    }

    /// WorkspaceConfig::as_any を使ってダウンキャストできること
    #[test]
    fn workspace_config_as_any() {
        // ######### 準備 #########
        let config = WorkspaceConfig::new("test");

        // ######### 実行 #########
        let any_box = Box::new(config).as_any();
        let downcast = any_box.downcast::<WorkspaceConfig>();

        // ######### 検証 #########
        assert!(downcast.is_ok());
    }

    /// WorkspaceConfig::is_latest が true を返すこと
    #[test]
    fn workspace_config_is_latest() {
        // ######### 準備 #########
        let config = WorkspaceConfig::new("test");

        // ######### 実行 #########
        let is_latest = config.is_latest();

        // ######### 検証 #########
        assert!(is_latest);
    }

    mod read_func {
        use super::*;
        use crate::utils::context::file::MockFileSystem;

        /// version: 1 であるとき、WorkspaceConfig::read が workspace.json を正しく読み込むこと
        #[tokio::test]
        async fn workspace_config_read() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // 関数のモック
            mock_file_system.expect_path_exists().return_const(true);
            mock_file_system.expect_read_file().returning(|_path| {
                let config_json = r#"
                {
                    "version": 1,
                    "stacks": {
                        "stack1": "stack1.json",
                        "stack2": "stack2.json"
                    },
                    "name": "test_workspace",
                    "description": "This is a test workspace"
                }
                "#;
                Ok(config_json.to_string())
            });

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = WorkspaceConfig::read(file_system, "directory/workspace.json").await;

            // ######### 検証 #########
            assert!(result.is_ok());
            let config = result.unwrap();
            assert_eq!(config.version, WORKSPACE_CONFIG_LATEST_VERSION);
            assert_eq!(
                config.stacks.get("stack1"),
                Some(&"stack1.json".to_string())
            );
            assert_eq!(
                config.stacks.get("stack2"),
                Some(&"stack2.json".to_string())
            );
            assert_eq!(config.name, "test_workspace");
            assert_eq!(config.description, "This is a test workspace");
        }

        /// version: [存在しないバージョン] であるとき、WorkspaceConfig::read が workspace.json を新規作成すること
        #[tokio::test]
        async fn workspace_config_read_nonexistent_version() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // 関数のモック
            mock_file_system.expect_path_exists().return_const(true);
            mock_file_system.expect_read_file().returning(|_path| {
                let config_json = r#"
                {
                    "version": 99999,
                    "stacks": {
                        "stack1": "stack1.json",
                        "stack2": "stack2.json"
                    },
                    "name": "test_workspace",
                    "description": "This is a test workspace"
                }
                "#;
                Ok(config_json.to_string())
            });
            mock_file_system
                .expect_write_file()
                .returning(|_path, _data| Ok(()));

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = WorkspaceConfig::read(file_system, "directory/workspace.json").await;

            // ######### 検証 #########
            assert!(result.is_ok());
            let config = result.unwrap();
            assert_eq!(config.version, WORKSPACE_CONFIG_LATEST_VERSION);
            assert_eq!(config.stacks, HashMap::new());
            assert_eq!(config.name, "workspace.json");
            assert_eq!(config.description, "");
        }

        /// version: [u64でない] とき、WorkspaceConfig::read が workspace.json を新規作成すること
        #[tokio::test]
        async fn workspace_config_read_invalid_version_type() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // 関数のモック
            mock_file_system.expect_path_exists().return_const(true);
            mock_file_system.expect_read_file().returning(|_path| {
                let config_json = r#"
                {
                    "version": "invalid_version",
                    "stacks": {
                        "stack1": "stack1.json",
                        "stack2": "stack2.json"
                    },
                    "name": "test_workspace",
                    "description": "This is a test workspace"
                }
                "#;
                Ok(config_json.to_string())
            });
            mock_file_system
                .expect_write_file()
                .returning(|_path, _data| Ok(()));

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = WorkspaceConfig::read(file_system, "directory/workspace.json").await;

            // ######### 検証 #########
            assert!(result.is_ok());
            let config = result.unwrap();
            assert_eq!(config.version, WORKSPACE_CONFIG_LATEST_VERSION);
            assert_eq!(config.stacks, HashMap::new());
            assert_eq!(config.name, "workspace.json");
            assert_eq!(config.description, "");
        }

        /// workspace.json が存在しないとき、WorkspaceConfig::read が workspace.json を新規作成すること
        #[tokio::test]
        async fn workspace_config_read_nonexistent_workspace_json() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // 関数のモック
            mock_file_system.expect_path_exists().return_const(false);
            mock_file_system
                .expect_write_file()
                .returning(|_path, _data| Ok(()));

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = WorkspaceConfig::read(file_system, "directory/workspace.json").await;

            // ######### 検証 #########
            assert!(result.is_ok());
            let config = result.unwrap();
            assert_eq!(config.version, WORKSPACE_CONFIG_LATEST_VERSION);
            assert_eq!(config.stacks, HashMap::new());
            assert_eq!(config.name, "workspace.json");
            assert_eq!(config.description, "");
        }

        /// 1回目の read_file がエラーのとき、WorkspaceConfig::read がエラーを返すこと
        #[tokio::test]
        async fn workspace_config_first_read_file_error() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // 関数のモック
            mock_file_system.expect_path_exists().return_const(true);
            mock_file_system.expect_read_file().returning(|_path| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "File not found",
                ))
            });

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = WorkspaceConfig::read(file_system, "directory/workspace.json").await;

            // ######### 検証 #########
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "io error: File not found");
        }

        /// 2回目の read_file がエラーのとき、WorkspaceConfig::read がエラーを返すこと
        #[tokio::test]
        async fn workspace_config_second_read_file_error() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // 関数のモック
            mock_file_system.expect_path_exists().return_const(true);

            let mut call_count = 0;
            mock_file_system
                .expect_read_file()
                .times(2)
                .returning_st(move |_path| {
                    call_count += 1;
                    match call_count {
                        // 1回目の呼び出しは正常
                        1 => {
                            let app_config_json = r#"{
                                "version": 1,
                                "stacks": {
                                    "stack1": "stack1.json",
                                    "stack2": "stack2.json"
                                },
                                "name": "test_workspace",
                                "description": "This is a test workspace"
                            }"#;
                            Ok(app_config_json.to_string())
                        }
                        // 2回目の呼び出しでErrを返す
                        2 => Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "read_file error",
                        )),
                        _ => panic!("Unexpected call"),
                    }
                });

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = WorkspaceConfig::read(file_system, "directory/workspace.json").await;

            // ######### 検証 #########
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "io error: read_file error");
        }

        /// 1回目の serde_json::from_str がエラーのとき、WorkspaceConfig::read がエラーを返すこと
        #[tokio::test]
        async fn workspace_config_first_from_str_error() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // 関数のモック
            mock_file_system.expect_path_exists().return_const(true);
            mock_file_system.expect_read_file().returning(|_path| {
                // 無効なJSONを返す
                let invalid_json = r#"
                {
                    "version": 1,
                    "stacks": {
                        "stack1": "stack1.json",
                        "stack2": "stack2.json",
                    },
                    "name": "test_workspace",
                    "description": "This is a test workspace"
                }
                "#;
                Ok(invalid_json.to_string())
            });

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = WorkspaceConfig::read(file_system, "directory/workspace.json").await;

            // ######### 検証 #########
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().starts_with("json error:"));
        }

        /// 2回目の serde_json::from_str がエラーのとき、WorkspaceConfig::read がエラーを返すこと
        #[tokio::test]
        async fn workspace_config_second_from_str_error() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // 関数のモック
            mock_file_system.expect_path_exists().return_const(true);

            let mut call_count = 0;
            mock_file_system
                .expect_read_file()
                .times(2)
                .returning(move |_path| {
                    call_count += 1;
                    match call_count {
                        // 1回目の呼び出しは正常
                        1 => {
                            let app_config_json = r#"{
                                "version": 1,
                                "stacks": {
                                    "stack1": "stack1.json",
                                    "stack2": "stack2.json"
                                },
                                "name": "test_workspace",
                                "description": "This is a test workspace"
                            }"#;
                            Ok(app_config_json.to_string())
                        }
                        // 2回目の呼び出しで無効なJSONを返す
                        2 => {
                            let invalid_json = r#"{
                                "version": 1,
                                "stacks": {
                                    "stack1": "stack1.json",
                                    "stack2": "stack2.json",
                                },
                                "name": "test_workspace",
                                "description": "This is a test workspace"
                            }"#;
                            Ok(invalid_json.to_string())
                        }
                        _ => panic!("Unexpected call"),
                    }
                });

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = WorkspaceConfig::read(file_system, "directory/workspace.json").await;

            // ######### 検証 #########
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().starts_with("json error:"));
        }

        /// workspace.json が存在せず、引数 workspace_directory が不正なとき、WorkspaceConfig::read がエラーを返すこと
        #[tokio::test]
        async fn workspace_config_invalid_workspace_directory() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // 関数のモック
            mock_file_system.expect_path_exists().return_const(false);

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = WorkspaceConfig::read(file_system, "").await;

            // ######### 検証 #########
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "app error: Invalid directory"
            );
        }

        /// write がエラーのとき、WorkspaceConfig::read がエラーを返すこと
        #[tokio::test]
        async fn workspace_config_write_error() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // 関数のモック
            mock_file_system.expect_path_exists().return_const(false);
            mock_file_system
                .expect_write_file()
                .returning(|_path, _data| {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "write_file error",
                    ))
                });

            let file_system = Arc::new(mock_file_system);

            // ######### 実行 #########
            let result = WorkspaceConfig::read(file_system, "directory/workspace.json").await;

            // ######### 検証 #########
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "io error: write_file error"
            );
        }
    }

    mod write_func {
        use super::*;
        use crate::utils::context::file::MockFileSystem;

        /// WorkspaceConfig::write が workspace.json を書き込むこと
        #[tokio::test]
        async fn workspace_config_write() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // 関数のモック
            mock_file_system
                .expect_write_file()
                .returning(|_path, _data| Ok(()));

            let file_system = Arc::new(mock_file_system);
            let config = WorkspaceConfig::new("test_workspace");

            // ######### 実行 #########
            let result = config.write(file_system, "directory/workspace.json").await;

            // ######### 検証 #########
            assert!(result.is_ok());
        }

        /// serde_json::to_string がエラーになるパターンは通常起こりえないためスキップ

        /// write_file がエラーのとき、WorkspaceConfig::write がエラーを返すこと
        #[tokio::test]
        async fn workspace_config_write_file_error() {
            // ######### 準備 #########
            let mut mock_file_system = MockFileSystem::new();

            // 関数のモック
            mock_file_system
                .expect_write_file()
                .returning(|_path, _data| {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "write_file error",
                    ))
                });

            let file_system = Arc::new(mock_file_system);
            let config = WorkspaceConfig::new("test_workspace");

            // ######### 実行 #########
            let result = config.write(file_system, "directory/workspace.json").await;

            // ######### 検証 #########
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "io error: write_file error"
            );
        }
    }
}
