use std::sync::Arc;

use super::clock::{Clock, SystemClock};
use super::file::{FileSystem, LocalFileSystem};
use super::http_client::{HttpClient, RealHttpClient};

pub struct AppContext {
    pub file_system: Arc<dyn FileSystem>,
    pub http_client: Arc<dyn HttpClient>,
    pub clock: Arc<dyn Clock>,
}

#[coverage(off)]
impl AppContext {
    pub fn new() -> Self {
        Self {
            file_system: Arc::new(LocalFileSystem {}),
            http_client: Arc::new(RealHttpClient {}),
            clock: Arc::new(SystemClock {}),
        }
    }
}
