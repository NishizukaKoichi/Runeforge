#[cfg(feature = "std")]
use thiserror::Error;

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg_attr(feature = "std", derive(Error))]
#[derive(Debug)]
pub enum IoError {
    #[cfg_attr(feature = "std", error("File not found: {0}"))]
    NotFound(String),
    #[cfg_attr(feature = "std", error("Permission denied: {0}"))]
    PermissionDenied(String),
    #[cfg_attr(feature = "std", error("IO operation failed: {0}"))]
    OperationFailed(String),
}

#[cfg(not(feature = "std"))]
impl core::fmt::Display for IoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            IoError::NotFound(path) => write!(f, "File not found: {}", path),
            IoError::PermissionDenied(path) => write!(f, "Permission denied: {}", path),
            IoError::OperationFailed(msg) => write!(f, "IO operation failed: {}", msg),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
pub trait FileSystemPort: Send + Sync {
    async fn read(&self, path: &str) -> Result<Vec<u8>, IoError>;
    async fn write(&self, path: &str, data: &[u8]) -> Result<(), IoError>;
    async fn exists(&self, path: &str) -> Result<bool, IoError>;
    async fn delete(&self, path: &str) -> Result<(), IoError>;
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
pub trait NetworkPort: Send + Sync {
    async fn http_get(&self, url: &str) -> Result<Vec<u8>, IoError>;
    async fn http_post(&self, url: &str, body: &[u8]) -> Result<Vec<u8>, IoError>;
}
