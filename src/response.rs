use serde::Deserialize;

use super::errors::Error;

#[derive(Debug, Deserialize)]
#[serde(tag = "stat", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DuoResponse<T> {
    Ok {
        response: T,
    },
    Fail {
        code: u64,
        message: String,
        message_detail: Option<String>,
    },
}

impl<T> DuoResponse<T> {
    pub(crate) fn ok(self) -> Result<T, Error> {
        match self {
            DuoResponse::Ok { response } => Ok(response),
            DuoResponse::Fail {
                code,
                message,
                message_detail,
            } => Err(Error::ApiRequestFailed {
                code,
                message,
                message_detail,
            }),
        }
    }
}
