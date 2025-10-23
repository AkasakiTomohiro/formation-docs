use std::sync::{Arc, OnceLock};

use super::file::FileSystem;

pub struct AppContext {
    pub file_system: Arc<dyn FileSystem>,
}

static APP_CONTEXT: OnceLock<AppContext> = OnceLock::new();

impl AppContext {
    pub fn init_global(ctx: AppContext) {
        if let Err(_) = APP_CONTEXT.set(ctx) {
            panic!("AppContext already initialized");
        }
    }

    pub fn global() -> &'static AppContext {
        APP_CONTEXT.get().expect("AppContext not initialized")
    }
}
