use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum BurnInError {
    #[error("Hardware failure: {0}")]
    HardwareFailure(String),
    
    #[error("Insufficient resources: {0}")]
    InsufficientResources(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Test timeout: {0}")]
    TestTimeout(String),
    
    #[error("System unstable: {0}")]
    SystemUnstable(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Test execution error: {0}")]
    TestExecutionError(String),
    
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

pub type Result<T> = std::result::Result<T, BurnInError>;
