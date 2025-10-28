use async_trait::async_trait;
use std::path::Path;
use tokio::{fs::ReadDir, io};

#[async_trait]
pub trait FileSystem: Send + Sync {
    async fn read_file(&self, path: &Path) -> io::Result<String>;
    async fn write_file(&self, path: &Path, contents: &[u8]) -> io::Result<()>;
    async fn remove_file(&self, path: &Path) -> io::Result<()>;
    async fn copy_file(&self, from: &Path, to: &Path) -> Result<u64, std::io::Error>;
    // async fn copy_file_stream<'a, R, W>(&self, from: &'a mut R, to: &'a mut W) -> io::Result<u64>
    // where
    //     R: io::AsyncRead + Unpin + ?Sized,
    //     W: io::AsyncWrite + Unpin + ?Sized;
    async fn read_dir(&self, path: &Path) -> io::Result<ReadDir>;
    async fn create_dir_all(&self, path: &Path) -> io::Result<()>;
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

    // FIXME:
    // async fn copy_file_stream<'a, R, W>(&self, from: &'a mut R, to: &'a mut W) -> io::Result<u64>
    // where
    //     R: io::AsyncRead + Unpin + ?Sized,
    //     W: io::AsyncWrite + Unpin + ?Sized,
    // {
    //     return io::copy(from, to).await;
    // }

    async fn read_dir(&self, path: &Path) -> io::Result<ReadDir> {
        return tokio::fs::read_dir(path).await;
    }

    async fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        return tokio::fs::create_dir_all(path).await;
    }
}
