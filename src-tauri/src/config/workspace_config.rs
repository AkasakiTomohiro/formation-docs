use crate::utils::AppError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;

const WORKSPACE_FILE_NAME: &str = "workspace.json";

///
/// 旧バージョンのコンフィグを最新版に変換するための仕組み
/// TODO：共通化できるようになったらUtilsに移動
///

/// 「最新版に変換できる」トレイト
pub trait WorkspaceConfigMigratable: Any {
    /// 次のバージョンの型（最新版なら Self を返す）
    fn migrate_boxed(self: Box<Self>) -> Box<dyn WorkspaceConfigMigratable>;

    /// 今が最新版なら true
    fn is_latest(&self) -> bool;

    /// Any型としてダウンキャストできるようにする
    fn as_any(self: Box<Self>) -> Box<dyn Any>;

    /// 最新版になるまでマイグレーション
    fn migrate_until_latest(self: Box<Self>) -> WorkspaceConfig
    where
        Self: Sized,
    {
        let mut current: Box<dyn WorkspaceConfigMigratable> = self;
        while !current.is_latest() {
            current = current.migrate_boxed();
        }
        // 最後は WorkspaceConfig に downcast
        *current
            .as_any()
            .downcast::<WorkspaceConfig>()
            .expect("must be Conf at latest")
    }
}

///
/// ワークスペースコンフィグ v1
///

#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceConfigV1 {
    pub version: u32,
    pub name: String,
    pub description: String,
}

impl WorkspaceConfigMigratable for WorkspaceConfigV1 {
    fn migrate_boxed(self: Box<Self>) -> Box<dyn WorkspaceConfigMigratable> {
        Box::new(WorkspaceConfigV2 {
            version: 2,
            stacks: HashMap::new(),
            name: self.name,
            description: self.description,
        })
    }
    fn is_latest(&self) -> bool {
        false
    }
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

///
/// ワークスペースコンフィグ v2
///

#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceConfigV2 {
    pub version: u32,
    // stack_id: stack_file_nameのマッピング
    pub stacks: HashMap<String, String>,
    pub name: String,
    pub description: String,
}

impl WorkspaceConfigV2 {
    pub fn new(name: &str) -> Self {
        WorkspaceConfigV2 {
            version: 2,
            stacks: HashMap::new(),
            name: name.to_string().chars().take(256).collect(),
            description: "".to_string().chars().take(256).collect(),
        }
    }
}

impl WorkspaceConfigMigratable for WorkspaceConfigV2 {
    fn migrate_boxed(self: Box<Self>) -> Box<dyn WorkspaceConfigMigratable> {
        Box::new(WorkspaceConfig {
            version: 2,
            stacks: self.stacks,
            name: self.name,
            description: self.description,
        })
    }
    fn is_latest(&self) -> bool {
        false
    }
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

///
/// ワークスペースコンフィグ 最新バージョン
///

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub version: u32,
    pub stacks: HashMap<String, String>,
    pub name: String,
    pub description: String,
}
impl WorkspaceConfig {
    pub fn new(name: &str) -> Self {
        let config: Box<dyn WorkspaceConfigMigratable> = Box::new(WorkspaceConfigV2::new(name));
        *config
            .migrate_boxed()
            .as_any()
            .downcast::<WorkspaceConfig>()
            .unwrap()
    }
}

impl WorkspaceConfigMigratable for WorkspaceConfig {
    fn migrate_boxed(self: Box<Self>) -> Box<dyn WorkspaceConfigMigratable> {
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
pub fn workspace_config_path(workspace_directory: &str) -> Result<PathBuf, WorkspaceConfigError> {
    let workspace_path = PathBuf::from(workspace_directory).join(WORKSPACE_FILE_NAME);
    if !workspace_path.exists() {
        return Err(WorkspaceConfigError::App(AppError::new(
            "Failed to find Workspace",
        )));
    }
    return Ok(workspace_path);
}

/// workspace.jsonのバージョンを取得
async fn read_config_version(workspace_directory: &str) -> Result<u32, WorkspaceConfigError> {
    let config_path = workspace_config_path(workspace_directory)?;
    match config_path.exists() {
        true => {
            let config_json = fs::read_to_string(&config_path).await?;
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

/// workspace.jsonへ書き込む
///
/// - `new_app_config`: 書き込む内容
async fn write_workspace_config(
    workspace_directory: &str,
    new_workspace_config: &WorkspaceConfig,
) -> Result<(), WorkspaceConfigError> {
    let workspace_config_json = serde_json::to_string::<WorkspaceConfig>(new_workspace_config)?;
    let workspace_config_path = workspace_config_path(workspace_directory)?;
    fs::write(workspace_config_path, workspace_config_json).await?;
    return Ok(());
}

/// workspace.jsonを読み込む
///
/// versionが最新でない場合はマイグレーションして最新版に変換し、保存する
/// - 戻り値: 読み込んだWorkspaceConfig
pub async fn read_workspace_config(
    workspace_directory: &str,
) -> Result<WorkspaceConfig, WorkspaceConfigError> {
    let version = read_config_version(workspace_directory).await?;
    let config_path = workspace_config_path(workspace_directory)?;
    let config_json = fs::read_to_string(&config_path).await?;

    match version {
        1 => {
            let config_json = serde_json::from_str::<WorkspaceConfigV1>(&config_json)?;
            let boxed = Box::new(config_json);
            let config: WorkspaceConfig = boxed.migrate_until_latest();
            write_workspace_config(workspace_directory, &config).await?;
            Ok(config)
        }
        2 => {
            let config_json = serde_json::from_str::<WorkspaceConfigV2>(&config_json)?;
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
            write_workspace_config(workspace_directory, &config_json).await?;
            Ok(config_json)
        }
    }
}
