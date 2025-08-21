#[cfg(feature = "std")]
use thiserror::Error;

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg_attr(feature = "std", derive(Error))]
#[derive(Debug)]
pub enum EnvError {
    #[cfg_attr(feature = "std", error("Environment variable not found: {0}"))]
    NotFound(String),
    #[cfg_attr(feature = "std", error("Invalid value for environment variable {0}: {1}"))]
    InvalidValue(String, String),
}

#[cfg(not(feature = "std"))]
impl core::fmt::Display for EnvError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            EnvError::NotFound(var) => write!(f, "Environment variable not found: {}", var),
            EnvError::InvalidValue(var, val) => write!(f, "Invalid value for environment variable {}: {}", var, val),
        }
    }
}

pub trait EnvironmentPort: Send + Sync {
    fn get_var(&self, key: &str) -> Result<String, EnvError>;
    fn set_var(&self, key: &str, value: &str);
    fn remove_var(&self, key: &str);
    fn current_dir(&self) -> Result<String, EnvError>;
    fn args(&self) -> Vec<String>;
}
