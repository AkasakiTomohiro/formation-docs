use std::sync::Arc;

use super::file::{FileSystem, LocalFileSystem};

pub struct AppContext {
    pub file_system: Arc<dyn FileSystem>,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            file_system: Arc::new(LocalFileSystem {}),
        }
    }
}
