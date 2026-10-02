//! 同梱バルーン判定モジュール
//!
//! 配布フォルダの `install.txt`・バルーンの `descript.txt` を読むだけで、ファイルは書きません。

use std::fs;
use std::io;
use std::path::Path;

/// UTF-8 の `key,value` ファイルを読む。BOM を除き、1 行目が `charset,UTF-8` であることを確かめ、
/// 2 行目以降を (小文字化したキー, trim した値) の列で返す。カンマの無い行は無視する。
/// 重複キーは出現順のまま残す（重複の扱いは呼び出し側が決める）。
/// 文字コードの変換は行わない。`label` はエラー文に出すファイル名。
fn read_utf8_kv(path: &Path, label: &str) -> io::Result<Vec<(String, String)>> {
    let not_utf8 = |reason: &str| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{label} is not UTF-8: {reason}"),
        )
    };

    let bytes = fs::read(path)?;
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
    let text =
        std::str::from_utf8(bytes).map_err(|_| not_utf8("contains invalid UTF-8 byte sequence"))?;

    let split = |line: &str| {
        line.split_once(',')
            .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string()))
    };
    let mut lines = text.split('\n');

    let charset_ok = lines
        .next()
        .and_then(split)
        .is_some_and(|(k, v)| k == "charset" && v.eq_ignore_ascii_case("UTF-8"));
    if !charset_ok {
        return Err(not_utf8(
            "the first line must be \"charset,UTF-8\" (pasta_check supports UTF-8 only)",
        ));
    }

    Ok(lines.filter_map(split).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn kv_of(bytes: &[u8]) -> io::Result<Vec<(String, String)>> {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("install.txt");
        fs::write(&path, bytes).unwrap();
        read_utf8_kv(&path, "install.txt")
    }

    fn pairs(v: &[(&str, &str)]) -> Vec<(String, String)> {
        v.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn assert_not_utf8(result: io::Result<Vec<(String, String)>>) -> String {
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        let msg = err.to_string();
        assert!(msg.starts_with("install.txt is not UTF-8: "), "{msg}");
        msg
    }

    #[test]
    fn test_read_utf8_kv_accepts_charset_and_reads_rest() {
        let kv = kv_of(b"charset,UTF-8\r\ntype,ghost\r\nBalloon.Directory , foo \r\nno comma\r\nhomeurl,a,b\r\n")
            .unwrap();
        assert_eq!(
            kv,
            pairs(&[
                ("type", "ghost"),
                ("balloon.directory", "foo"),
                ("homeurl", "a,b"),
            ])
        );
    }

    #[test]
    fn test_read_utf8_kv_keeps_duplicate_keys_in_order() {
        let kv = kv_of(b"charset,UTF-8\nhomeurl,a\nHOMEURL,b\n").unwrap();
        assert_eq!(kv, pairs(&[("homeurl", "a"), ("homeurl", "b")]));
    }

    #[test]
    fn test_read_utf8_kv_accepts_case_bom_and_spaces() {
        assert!(kv_of(b"Charset,UTF-8\r\n").unwrap().is_empty());
        assert!(kv_of(b"\xEF\xBB\xBFcharset,UTF-8\r\n").unwrap().is_empty());
        assert!(kv_of(b"  CHARSET , utf-8  \r\n").unwrap().is_empty());
        assert!(kv_of(b"charset,UTF-8").unwrap().is_empty());
    }

    #[test]
    fn test_read_utf8_kv_rejects_missing_or_other_charset() {
        for bytes in [
            &b"type,ghost\r\ncharset,UTF-8\r\n"[..],
            b"charset,Shift_JIS\r\n",
            b"charset\r\n",
            b"",
        ] {
            let msg = assert_not_utf8(kv_of(bytes));
            assert_eq!(
                msg,
                "install.txt is not UTF-8: the first line must be \"charset,UTF-8\" (pasta_check supports UTF-8 only)"
            );
        }
    }

    #[test]
    fn test_read_utf8_kv_rejects_invalid_bytes() {
        let msg = assert_not_utf8(kv_of(b"charset,UTF-8\r\nname,\x82\xA0\r\n"));
        assert!(
            msg.ends_with("contains invalid UTF-8 byte sequence"),
            "{msg}"
        );
    }

    #[test]
    fn test_read_utf8_kv_uses_label_in_message() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("descript.txt");
        fs::write(&path, b"charset,Shift_JIS\r\n").unwrap();
        let err = read_utf8_kv(&path, "emo2-kakukaku/descript.txt").unwrap_err();
        assert!(
            err.to_string()
                .starts_with("emo2-kakukaku/descript.txt is not UTF-8: "),
            "{err}"
        );
    }
}
