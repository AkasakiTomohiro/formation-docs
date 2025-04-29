use dashmap::DashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct WindowState {
    pub workspace_directory: String,
}

static WINDOW_STATE: LazyLock<DashMap<String, WindowState>> = LazyLock::new(DashMap::new);

pub fn get_window_state(window_id: String) -> Option<WindowState> {
    WINDOW_STATE
        .get(window_id.as_str())
        .map(|state| state.clone())
}

pub fn set_window_state(window_id: String, state: WindowState) {
    WINDOW_STATE.insert(window_id, state);
}
