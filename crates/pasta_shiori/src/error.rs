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

/// mlua のエラー Display が末尾に付ける、スタックトレース開始行のマーカー。
const STACK_TRACEBACK_MARKER: &str = "stack traceback:";

/// `X-ERROR-REASON` ヘッダ値を単一行に整形する（要件 4.6 / 4.11）。
///
/// (1) `stack traceback:` の行以降を捨てる。全文は既存の error ログに残る。
/// (2) 残りを CR / LF で分割し、各行を trim、空行を捨て、半角スペース 1 個で連結する。
///
/// 改行を含まない入力は無変換で返すため、既存応答はバイト不変（要件 3.5）。
/// `no file '…'` の候補パス列は落とさず、長さ上限も設けない。非 ASCII は変換しない（要件 2.3）。
fn single_line(message: &str) -> String {
    if !message.contains(['\r', '\n']) {
        return message.to_string();
    }
    let body = match message.find(STACK_TRACEBACK_MARKER) {
        Some(index) => &message[..index],
        None => message,
    };
    body.split(['\r', '\n'])
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
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
            single_line(&self.to_string())
        )
    }

    /// Generate SHIORI 3.0 bad request response
    pub fn to_shiori_400_response(&self) -> String {
        format!(
            "SHIORI/3.0 400 Bad Request\r\n\
             Charset: UTF-8\r\n\
             X-ERROR-REASON: {}\r\n\
             \r\n",
            single_line(&self.to_string())
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

    // ====================================================================
    // タスク 2.1: `X-ERROR-REASON` 値の単一行化（`single_line`）
    // 要件 2.3 / 3.5 / 4.6 / 4.11、design.md `ErrorResponse`
    // ====================================================================

    /// LF 区切りの複数行を半角スペース 1 個で連結する（4.6）。
    #[test]
    fn single_line_joins_lf_separated_lines() {
        assert_eq!(single_line("first\nsecond\nthird"), "first second third");
    }

    /// CRLF 区切りでも空行を生まずに連結し、末尾改行も落とす（4.6）。
    #[test]
    fn single_line_joins_crlf_separated_lines() {
        assert_eq!(single_line("first\r\nsecond\r\n"), "first second");
    }

    /// 単独 CR 区切りも改行として扱う（4.6）。
    #[test]
    fn single_line_joins_cr_separated_lines() {
        assert_eq!(single_line("first\rsecond"), "first second");
    }

    /// タブ字下げの継続行は trim し、連続改行による空行は捨てる（4.6）。
    #[test]
    fn single_line_trims_tab_indented_continuation_lines() {
        let message = "module 'x' not found:\n\tno field package.preload['x']\n\n\tno file 'a.lua'";
        assert_eq!(
            single_line(message),
            "module 'x' not found: no field package.preload['x'] no file 'a.lua'"
        );
    }

    /// `stack traceback:` の行以降を捨てる（4.11）。
    #[test]
    fn single_line_drops_stack_traceback_and_after() {
        let message = "runtime error: boom\n\tdetail line\nstack traceback:\n\t[C]: in function 'require'\n\tentry.lua:1: in main chunk";
        assert_eq!(single_line(message), "runtime error: boom detail line");
    }

    /// 字下げ付きの `stack traceback:` でも同様に落とす（4.11）。
    #[test]
    fn single_line_drops_indented_stack_traceback() {
        let message = "runtime error: boom\n\tstack traceback:\n\t[C]: in ?";
        assert_eq!(single_line(message), "runtime error: boom");
    }

    /// 入力が丸ごとトレースバックなら空文字になる（4.11）。
    #[test]
    fn single_line_traceback_only_becomes_empty() {
        assert_eq!(single_line("stack traceback:\n\t[C]: in ?"), "");
    }

    /// 空白のみの複数行入力は空文字になる（4.6）。
    #[test]
    fn single_line_whitespace_only_becomes_empty() {
        assert_eq!(single_line("  \n\t\r\n "), "");
    }

    /// 改行を含まない入力はバイト不変で返す（3.5）。
    #[test]
    fn single_line_without_newline_is_byte_identical() {
        for message in [
            "Not initialized error",
            "Invalid X-Pasta-Time header value: 'bad-value', reason: parse failed",
            "  leading and trailing spaces  ",
            "",
        ] {
            assert_eq!(
                single_line(message).as_bytes(),
                message.as_bytes(),
                "改行無し入力はバイト不変であるべき: {message:?}"
            );
        }
    }

    /// 非 ASCII 文字は変換・欠落させない（2.3）。
    #[test]
    fn single_line_preserves_non_ascii() {
        let message = "モジュール 'テスト' が見つかりません:\n\tno file 'C:/ゴースト/テスト.lua'";
        assert_eq!(
            single_line(message),
            "モジュール 'テスト' が見つかりません: no file 'C:/ゴースト/テスト.lua'"
        );
    }

    /// `no file '…'` の候補パス列は落とさず、長さ上限による切り詰めもしない（4.11）。
    #[test]
    fn single_line_keeps_all_candidate_paths_without_truncation() {
        let mut message = String::from("module 'a.b' not found:");
        for i in 0..200 {
            message.push_str(&format!(
                "\n\tno file 'C:/very/long/search/path/number{i}/a/b.lua'"
            ));
        }
        let result = single_line(&message);
        for i in 0..200 {
            assert!(
                result.contains(&format!("number{i}/a/b.lua")),
                "候補パス {i} が欠落している"
            );
        }
        assert!(
            !result.contains('\n') && !result.contains('\r'),
            "単一行化後に改行が残っている"
        );
    }

    /// 複数行メッセージでも 500 応答のヘッダ構造が壊れない（4.6 / 4.11）。
    #[test]
    fn to_shiori_response_multiline_message_is_single_line() {
        let err =
            MyError::Load("failed:\n\tno file 'x.lua'\nstack traceback:\n\t[C]: in ?".to_string());
        let response = err.to_shiori_response();
        assert_eq!(
            response,
            "SHIORI/3.0 500 Internal Server Error\r\n\
             Charset: UTF-8\r\n\
             X-ERROR-REASON: Load error: failed: no file 'x.lua'\r\n\
             \r\n"
        );
    }

    /// 複数行メッセージでも 400 応答のヘッダ構造が壊れない（4.6）。
    #[test]
    fn to_shiori_400_response_multiline_message_is_single_line() {
        let err = MyError::InvalidPastaTime {
            value: "bad\nvalue".to_string(),
            reason: "parse\r\nfailed".to_string(),
        };
        let response = err.to_shiori_400_response();
        assert_eq!(
            response,
            "SHIORI/3.0 400 Bad Request\r\n\
             Charset: UTF-8\r\n\
             X-ERROR-REASON: Invalid X-Pasta-Time header value: 'bad value', reason: parse failed\r\n\
             \r\n"
        );
    }
}
