use crate::util::parsers;
use std::str::Utf8Error;
use std::sync::PoisonError;
use thiserror::Error;

pub type MyResult<T> = Result<T, MyError>;

#[derive(Clone, Eq, PartialEq, Debug, Error)]
pub enum MyError {
    #[error("others error")]
    #[allow(dead_code)]
    Others,

    #[error("load error: {0}")]
    Load(String),

    #[error("not initialized error")]
    NotInitialized,

    #[error("Poison error")]
    Poison,

    #[error("Shiori request parse error for '{0}'")]
    ParseRequest(Box<parsers::req::ParseError>),

    #[error("ANSI encoding error")]
    EncodeAnsi,
    #[error("UTF8 encoding error")]
    EncodeUtf8(Utf8Error),

    #[error("script error: {}", message)]
    Script { message: String },
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
impl From<Utf8Error> for MyError {
    fn from(error: Utf8Error) -> MyError {
        MyError::EncodeUtf8(error)
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

impl MyError {
    #[allow(dead_code)]
    pub fn script_error(message: String) -> MyError {
        MyError::Script { message }
    }

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
}
