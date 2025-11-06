use async_trait::async_trait;
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use tokio::{fs::ReadDir, io};

#[mockall::automock]
#[async_trait]
pub trait FileSystem: Send + Sync {
    async fn read_file(&self, path: &Path) -> io::Result<String>;
    async fn write_file(&self, path: &Path, contents: &[u8]) -> io::Result<()>;
    async fn remove_file(&self, path: &Path) -> io::Result<()>;
    async fn copy_file(&self, from: &Path, to: &Path) -> Result<u64, std::io::Error>;
    fn copy_file_stream(
        &self,
        from: &mut (dyn Read + Send),
        to: &mut (dyn Write + Send),
    ) -> std::io::Result<u64>;
    fn touch_and_open_file(&self, path: &Path) -> std::io::Result<File>;
    async fn read_dir(&self, path: &Path) -> io::Result<ReadDir>;
    async fn create_dir_all(&self, path: &Path) -> io::Result<()>;
    fn create_dir_all_sync(&self, path: &Path) -> std::io::Result<()>;
    fn config_local_dir(&self) -> Option<PathBuf>;
    fn path_exists(&self, path: &Path) -> bool;
}

pub struct LocalFileSystem;
#[async_trait]
impl FileSystem for LocalFileSystem {
    async fn read_file(&self, path: &Path) -> io::Result<String> {
        return tokio::fs::read_to_string(path).await;
    }

    async fn write_file(&self, path: &Path, contents: &[u8]) -> io::Result<()> {
        return tokio::fs::write(path, contents).await;
    }

    async fn remove_file(&self, path: &Path) -> io::Result<()> {
        return tokio::fs::remove_file(path).await;
    }

    async fn copy_file(&self, from: &Path, to: &Path) -> Result<u64, std::io::Error> {
        return tokio::fs::copy(from, to).await;
    }

    fn copy_file_stream(
        &self,
        from: &mut (dyn Read + Send),
        to: &mut (dyn Write + Send),
    ) -> std::io::Result<u64> {
        return std::io::copy(from, to);
    }

    fn touch_and_open_file(&self, path: &Path) -> std::io::Result<File> {
        return std::fs::File::create(path);
    }

    async fn read_dir(&self, path: &Path) -> io::Result<ReadDir> {
        return tokio::fs::read_dir(path).await;
    }

    async fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        return tokio::fs::create_dir_all(path).await;
    }

    fn create_dir_all_sync(&self, path: &Path) -> std::io::Result<()> {
        return std::fs::create_dir_all(path);
    }

    fn config_local_dir(&self) -> Option<PathBuf> {
        return dirs::config_local_dir();
    }

    fn path_exists(&self, path: &Path) -> bool {
        path.exists()
    }
}
