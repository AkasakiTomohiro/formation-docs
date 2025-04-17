use crate::utils::AppError;
use crate::utils::CommandResult;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct Stack {
    name: String,
}

impl Stack {
    fn new(name: String) -> Self {
        Stack { name }
    }
}

#[derive(Debug, Error)]
pub enum LoadStacksError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    // FIXME: globmatchのエラーを含める
    // #[error("globmatch error: {0}")]
    // Globmatch(#[from] globmatch::Error),
}

fn load_stacks(workspace_directory: &str) -> Result<Vec<Stack>, LoadStacksError> {
    let builder = globmatch::Builder::new("./*.template.json").build(workspace_directory);
    let builder = match builder {
        Ok(builder) => builder,
        Err(_) => {
            return Err(LoadStacksError::App(AppError::new("Failed to load stacks")));
        }
    };
    let paths: Vec<_> = builder.into_iter().flatten().collect();
    let mut stacks = Vec::new();
    for path in paths {
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        stacks.push(Stack::new(filename.replace(".template.json", "")));
    }

    return Ok(stacks);
}

#[tauri::command(rename_all = "snake_case")]
pub fn load_stacks_command(
    workspace_directory: &str,
) -> Result<CommandResult<Vec<Stack>>, CommandResult> {
    return match load_stacks(workspace_directory) {
        Ok(stacks) => Ok(CommandResult::success(stacks)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}
