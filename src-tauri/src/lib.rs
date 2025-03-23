use std::path::Path;
use std::path::PathBuf;
use tauri::WebviewWindowBuilder;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn open_workspace(
    handle: tauri::AppHandle,
    id: &str,
    name: &str,
    directory: &str,
) -> Result<bool, ()> {
    let path = Path::new(directory);
    log::info!("Open: {}", path.to_str().unwrap());
    if !path.exists() {
        return Ok(false);
    }
    let window = WebviewWindowBuilder::new(
        &handle,
        name,
        tauri::WebviewUrl::App(PathBuf::from(format!("workspaces/{}", id))),
    )
    .title(name)
    .build()
    .expect("failed to create new window");
    window.show().expect("failed to show window");
    return Ok(true);
}

#[tauri::command]
async fn create_workspace(file_path: &str, content: &str) -> Result<String, ()> {
    let path = Path::new(file_path);
    log::info!("Create: {}", path.to_str().unwrap());
    if path.exists() {
        let content = read_file(file_path).await.expect("failed to read file");
        return Ok(content.to_string());
    }
    save_file(file_path, content)
        .await
        .expect("failed to save file");
    return Ok(content.to_string());
}

#[tauri::command]
async fn read_file(file_path: &str) -> Result<String, ()> {
    let path = Path::new(file_path);
    log::info!("Read: {}", path.to_str().unwrap());
    if !path.exists() {
        return Err(());
    }
    let content = std::fs::read_to_string(path).expect("failed to read file");
    return Ok(content);
}

#[tauri::command]
async fn save_file(file_path: &str, content: &str) -> Result<(), ()> {
    let path = Path::new(file_path);
    log::info!("Save: {}", path.to_str().unwrap());
    std::fs::write(path, content).expect("failed to write file");
    return Ok(());
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            open_workspace,
            read_file,
            save_file,
            create_workspace
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
