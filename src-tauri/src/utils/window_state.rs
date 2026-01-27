use dashmap::DashMap;
use std::sync::LazyLock;
use tauri::Window;

#[derive(Debug, Clone)]
pub struct WindowState {
    pub workspace_directory: String,
}

static WINDOW_STATE: LazyLock<DashMap<String, WindowState>> = LazyLock::new(DashMap::new);

#[coverage(off)]
pub fn get_window_state(window: Window) -> Option<WindowState> {
    let window_id = window.label().to_string();
    WINDOW_STATE
        .get(window_id.as_str())
        .map(|state| state.clone())
}

#[coverage(off)]
pub fn set_window_state(window_id: String, state: WindowState) {
    WINDOW_STATE.insert(window_id, state);
}
