use async_imap::error::Error as ImapError;
use thiserror::Error;

pub type EmailResult<T> = Result<T, EmailError>;

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("connect error: {0}")]
    ConnectError(std::io::Error),
    #[error("tls error: {0}")]
    TlsError(async_native_tls::Error),
    #[error("imap client error: {0}")]
    Imap(ImapError),
    #[error("login failed: {0}")]
    LoginFailed(ImapError),
}

impl From<ImapError> for EmailError {
    fn from(e: ImapError) -> Self {
        Self::Imap(e)
    }
}

impl From<std::io::Error> for EmailError {
    fn from(e: std::io::Error) -> Self {
        Self::ConnectError(e)
    }
}

impl From<async_native_tls::Error> for EmailError {
    fn from(e: async_native_tls::Error) -> Self {
        Self::TlsError(e)
    }
}