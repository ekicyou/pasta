use crate::util::parsers;
use std::str::Utf8Error;
use std::sync::PoisonError;
use thiserror::Error;

pub type MyResult<T> = Result<T, MyError>;

#[derive(Clone, Eq, PartialEq, Debug, Error)]
pub enum MyError {
    #[error("Load error: {0}")]
    Load(String),

    #[error("Not initialized error")]
    NotInitialized,

    #[error("Poison error")]
    Poison,

    #[error("Shiori request parse error: '{0}'")]
    ParseRequest(Box<parsers::req::ParseError>),

    #[error("ANSI encoding error")]
    EncodeAnsi,
    #[error("UTF8 encoding error")]
    EncodeUtf8(#[from] Utf8Error),

    #[error("Script error: {message}")]
    Script { message: String },

    #[error("Invalid X-Pasta-Time header value: '{value}', reason: {reason}")]
    InvalidPastaTime { value: String, reason: String },
}

impl From<parsers::req::ParseError> for MyError {
    fn from(error: parsers::req::ParseError) -> MyError {
        MyError::ParseRequest(Box::new(error))
    }
}

impl<G> From<PoisonError<G>> for MyError {
    fn from(_error: PoisonError<G>) -> MyError {
        MyError::Poison
    }
}

impl From<pasta_lua::LoaderError> for MyError {
    fn from(error: pasta_lua::LoaderError) -> MyError {
        MyError::Load(format!("{}", error))
    }
}

impl From<pasta_lua::mlua::Error> for MyError {
    fn from(error: pasta_lua::mlua::Error) -> MyError {
        MyError::Script {
            message: format!("{}", error),
        }
    }
}

impl From<time::error::IndeterminateOffset> for MyError {
    fn from(error: time::error::IndeterminateOffset) -> MyError {
        MyError::Script {
            message: format!("Failed to get local time: {}", error),
        }
    }
}

impl MyError {
    /// Generate SHIORI 3.0 error response
    ///
    /// Format:
    /// ```text
    /// SHIORI/3.0 500 Internal Server Error\r\n
    /// Charset: UTF-8\r\n
    /// X-ERROR-REASON: <error message>\r\n
    /// \r\n
    /// ```
    pub fn to_shiori_response(&self) -> String {
        format!(
            "SHIORI/3.0 500 Internal Server Error\r\n\
             Charset: UTF-8\r\n\
             X-ERROR-REASON: {}\r\n\
             \r\n",
            self
        )
    }

    /// Generate SHIORI 3.0 bad request response
    pub fn to_shiori_400_response(&self) -> String {
        format!(
            "SHIORI/3.0 400 Bad Request\r\n\
             Charset: UTF-8\r\n\
             X-ERROR-REASON: {}\r\n\
             \r\n",
            self
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_pasta_time_error_message() {
        let err = MyError::InvalidPastaTime {
            value: "bad-value".to_string(),
            reason: "parse failed".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Invalid X-Pasta-Time header value: 'bad-value', reason: parse failed"
        );
    }

    #[test]
    fn to_shiori_400_response_format() {
        let err = MyError::InvalidPastaTime {
            value: "bad-value".to_string(),
            reason: "parse failed".to_string(),
        };
        let response = err.to_shiori_400_response();
        assert_eq!(
            response,
            "SHIORI/3.0 400 Bad Request\r\n\
             Charset: UTF-8\r\n\
             X-ERROR-REASON: Invalid X-Pasta-Time header value: 'bad-value', reason: parse failed\r\n\
             \r\n"
        );
    }

    #[test]
    fn to_shiori_400_response_does_not_contain_sender() {
        let err = MyError::InvalidPastaTime {
            value: "x".to_string(),
            reason: "y".to_string(),
        };
        let response = err.to_shiori_400_response();
        assert!(!response.contains("Sender:"));
    }

    #[test]
    fn existing_to_shiori_response_unchanged() {
        let err = MyError::NotInitialized;
        let response = err.to_shiori_response();
        assert_eq!(
            response,
            "SHIORI/3.0 500 Internal Server Error\r\n\
             Charset: UTF-8\r\n\
             X-ERROR-REASON: Not initialized error\r\n\
             \r\n"
        );
    }
}
