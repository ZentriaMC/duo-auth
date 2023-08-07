use thiserror::Error;

use super::StdError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid API domain '{domain}': {cause}")]
    InvalidApiDomain { domain: String, cause: StdError },

    #[error("API request failed: {message} ({code})")]
    ApiRequestFailed {
        code: u64,
        message: String,
        message_detail: Option<String>,
    },

    #[error("Unspecified error")]
    Unspecified(#[from] StdError),
}

impl Error {
    pub(crate) fn unspecified<E: Into<StdError>>(err: E) -> Self {
        Self::Unspecified(err.into())
    }
}
