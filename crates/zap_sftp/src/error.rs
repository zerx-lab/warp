//! SFTP 协议层错误类型定义
//!
//! 定义 SftpError 和 SftpChannelError 两种错误枚举，
//! 覆盖连接、认证、超时、权限等错误场景。
//! author: logic
//! date: 2026-05-31

use thiserror::Error;

/// SFTP 协议级错误
#[derive(Debug, Error)]
pub enum SftpError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SSH2 error: {0}")]
    Ssh2(#[from] ssh2::Error),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Operation timed out")]
    Timeout,

    #[error("File not found: {0}")]
    NoSuchFile(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Operation failed: {0}")]
    General(String),
}

/// SFTP 通道错误
#[derive(Debug, Error)]
pub enum SftpChannelError {
    #[error("SFTP error: {0}")]
    Sftp(#[from] SftpError),

    #[error("Failed to send request: {0}")]
    SendFailed(String),

    #[error("Failed to receive response: {0}")]
    RecvFailed(String),
}

impl From<ssh2::Error> for SftpChannelError {
    fn from(e: ssh2::Error) -> Self {
        SftpChannelError::Sftp(SftpError::Ssh2(e))
    }
}
