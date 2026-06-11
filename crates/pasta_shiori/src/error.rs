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

    // ====================================================================
    // G1 (3.35): Display contract for previously untested variants.
    // X-ERROR-REASON embeds `{self}` directly, so these strings are part of
    // the observable SHIORI 500/400 response surface.
    // ====================================================================

    #[test]
    fn display_contract_remaining_variants() {
        assert_eq!(
            MyError::Load("boot failed".to_string()).to_string(),
            "Load error: boot failed"
        );
        assert_eq!(MyError::Poison.to_string(), "Poison error");
        assert_eq!(MyError::EncodeAnsi.to_string(), "ANSI encoding error");
        assert_eq!(
            MyError::Script {
                message: "boom".to_string()
            }
            .to_string(),
            "Script error: boom"
        );
    }

    // ====================================================================
    // G1 (3.35): From conversions (previously all untested).
    // ====================================================================

    #[test]
    fn from_utf8_error_maps_to_encode_utf8() {
        // Build the invalid sequence at runtime (a literal would trip the
        // invalid_from_utf8 lint — producing the error is the whole point).
        let invalid: Vec<u8> = vec![0xC0];
        let utf8_err = std::str::from_utf8(&invalid).unwrap_err();
        let err = MyError::from(utf8_err);
        assert!(matches!(err, MyError::EncodeUtf8(_)));
        assert_eq!(err.to_string(), "UTF8 encoding error");
    }

    #[test]
    fn from_poison_error_maps_to_poison() {
        let poison = std::sync::PoisonError::new(());
        let err = MyError::from(poison);
        assert_eq!(err, MyError::Poison);
    }

    #[test]
    fn from_mlua_error_maps_to_script_with_message() {
        let lua_err = pasta_lua::mlua::Error::RuntimeError("lua boom".to_string());
        let err = MyError::from(lua_err);
        match err {
            MyError::Script { ref message } => {
                assert!(
                    message.contains("lua boom"),
                    "Script message should contain the mlua error text: {message}"
                );
            }
            other => panic!("Expected MyError::Script, got {other:?}"),
        }
    }

    #[test]
    fn from_loader_error_maps_to_load_with_message() {
        let loader_err =
            pasta_lua::LoaderError::ConfigNotFound(std::path::PathBuf::from("missing-dir"));
        let err = MyError::from(loader_err);
        match err {
            MyError::Load(ref msg) => {
                assert_eq!(msg, "Configuration file not found: 'missing-dir'");
            }
            other => panic!("Expected MyError::Load, got {other:?}"),
        }
    }

    #[test]
    fn from_parse_error_maps_to_parse_request_with_display_prefix() {
        use crate::util::parsers::req::{Parser, Rule};
        use pest::Parser as _;
        let pest_err = Parser::parse(Rule::req, "NOT A SHIORI REQUEST").unwrap_err();
        let err = MyError::from(pest_err);
        assert!(matches!(err, MyError::ParseRequest(_)));
        assert!(
            err.to_string().starts_with("Shiori request parse error: '"),
            "Display should use the documented prefix: {err}"
        );
    }
}
