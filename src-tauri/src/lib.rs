mod api;
mod command;
mod utils;

use std::path::Path;
use std::path::PathBuf;
use tauri::WebviewWindowBuilder;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command(rename_all = "snake_case")]
async fn open_workspace_command(
    handle: tauri::AppHandle,
    id: &str,
    name: &str,
    directory: &str,
) -> Result<bool, ()> {
    let window = if id == "main" {
        WebviewWindowBuilder::new(
            &handle,
            "main".to_string(),
            tauri::WebviewUrl::App(PathBuf::from("workspaces")),
        )
    } else {
        let path = Path::new(directory);
        log::info!("Open: {}", path.to_str().unwrap());
        if !path.exists() {
            return Ok(false);
        }
        WebviewWindowBuilder::new(
            &handle,
            format!("workspace-{}", id),
            tauri::WebviewUrl::App(PathBuf::from(format!("workspaces/{}", id))),
        )
    }
    .title(name)
    .build()
    .expect("failed to create new window");
    window.show().expect("failed to show window");
    return Ok(true);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            open_workspace_command,
            command::app_config::read_app_config_command,
            command::app_config::delete_workspace_from_app_config_command,
            command::workspace::create_workspace_command,
            command::workspace::load_workspace_command,
            command::workspace::update_workspace_command,
            command::workspace::load_workspaces_command,
            command::resource_provider::setup_app_command,
            command::stack::load_stacks_command,
            command::stack::delete_stack_command,
            command::stack::import_stack_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
