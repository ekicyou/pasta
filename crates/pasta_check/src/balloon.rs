//! 同梱バルーン判定モジュール
//!
//! 配布フォルダの `install.txt`・バルーンの `descript.txt` を読むだけで、ファイルは書きません。

use std::collections::HashMap;
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

/// バルーン指定 1 件。key は採用した行のキー（エラー表示用）。
struct BalloonSpec {
    key: String,
    value: String,
    hierarchical_allowed: bool,
}

/// 番号なし → balloon0 → balloon1 … の順に探す（キーの重複・欠番の後ろの指定はエラー）。
/// `P.source.directory` の行があれば（値が空でも）それを、無ければ `P.directory` を採る。
/// 番号付きは両方の行が無い最初の番号で打ち切り、それより後ろの番号の指定はエラーにする。
/// エラーの優先順: 重複キー → 先頭に 0 の付いた番号 → 欠番の後ろの指定（それぞれ行順で最初のもの）。
fn find_balloon_specs(kv: &[(String, String)]) -> io::Result<Vec<BalloonSpec>> {
    let invalid =
        |msg: String| io::Error::new(io::ErrorKind::InvalidData, format!("install.txt: {msg}"));

    // ベースウェアはキーを文字列で引くため、番号も書かれた文字列のまま扱う。
    // (番号の文字列（番号なしは ""）, source.directory か) → (キー, 値)
    let mut found = HashMap::new();
    // 番号付きの指定の (番号の文字列, キー)。行順。
    let mut numbered = Vec::new();
    for (key, value) in kv {
        let Some((num, source)) = parse_spec_key(key) else {
            continue;
        };
        if found.insert((num, source), (key, value)).is_some() {
            return Err(invalid(format!("duplicate key {key}")));
        }
        if !num.is_empty() {
            numbered.push((num, key));
        }
    }

    if let Some((_, key)) = numbered
        .iter()
        .find(|(num, _)| num.len() > 1 && num.starts_with('0'))
    {
        return Err(invalid(format!(
            "{key} is never read by the baseware (numbers must not have leading zeros)"
        )));
    }

    let take = |num: &str| {
        let (source, (key, value)) = [true, false]
            .into_iter()
            .find_map(|source| found.get(&(num, source)).map(|kv| (source, kv)))?;
        Some(BalloonSpec {
            key: key.to_string(),
            value: value.to_string(),
            hierarchical_allowed: source,
        })
    };

    let mut specs: Vec<BalloonSpec> = take("").into_iter().collect();
    let mut cut: u32 = 0;
    while let Some(spec) = take(&cut.to_string()) {
        specs.push(spec);
        cut += 1;
    }

    // 読まれた番号は 0..cut だけ。u32 に収まらない番号は打ち切った番号より後ろとみなす。
    if let Some((_, key)) = numbered
        .iter()
        .find(|(num, _)| num.parse::<u32>().map_or(true, |n| n > cut))
    {
        return Err(invalid(format!(
            "{key} is never read by the baseware because balloon{cut} is missing (numbered entries must not have gaps)"
        )));
    }
    Ok(specs)
}

/// キーがバルーン指定（`balloon[<数字>].source.directory`・`balloon[<数字>].directory`）なら
/// (番号の文字列（番号なしは ""）, source.directory か) を返す。キーは小文字化済みとする。
fn parse_spec_key(key: &str) -> Option<(&str, bool)> {
    let (num, field) = key.strip_prefix("balloon")?.split_once('.')?;
    let source = match field {
        "source.directory" => true,
        "directory" => false,
        _ => return None,
    };
    num.bytes()
        .all(|b| b.is_ascii_digit())
        .then_some((num, source))
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

    fn specs_of(kv: &[(&str, &str)]) -> io::Result<Vec<(String, String, bool)>> {
        let specs = find_balloon_specs(&pairs(kv))?;
        Ok(specs
            .into_iter()
            .map(|s| (s.key, s.value, s.hierarchical_allowed))
            .collect())
    }

    fn spec(key: &str, value: &str, hierarchical_allowed: bool) -> (String, String, bool) {
        (key.to_string(), value.to_string(), hierarchical_allowed)
    }

    fn assert_invalid(result: io::Result<Vec<(String, String, bool)>>, expected: &str) {
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), expected);
    }

    #[test]
    fn test_find_balloon_specs_prefers_source_directory() {
        let specs = specs_of(&[
            ("balloon.directory", "a"),
            ("balloon.source.directory", "extra/b"),
        ])
        .unwrap();
        assert_eq!(
            specs,
            vec![spec("balloon.source.directory", "extra/b", true)]
        );

        // 値が空でも source.directory の行があればそれを採る
        let specs =
            specs_of(&[("balloon.source.directory", ""), ("balloon.directory", "a")]).unwrap();
        assert_eq!(specs, vec![spec("balloon.source.directory", "", true)]);
    }

    #[test]
    fn test_find_balloon_specs_falls_back_to_directory() {
        let specs = specs_of(&[("type", "ghost"), ("balloon.directory", "a")]).unwrap();
        assert_eq!(specs, vec![spec("balloon.directory", "a", false)]);
    }

    #[test]
    fn test_find_balloon_specs_none_when_no_spec() {
        let specs = specs_of(&[
            ("type", "ghost"),
            ("directory", "hello"),
            ("homeurl", "x"),
            ("homeurl", "y"),
        ])
        .unwrap();
        assert!(specs.is_empty());
    }

    #[test]
    fn test_find_balloon_specs_unnumbered_then_numbered_in_order() {
        let specs = specs_of(&[
            ("balloon1.directory", "c"),
            ("balloon0.source.directory", "b"),
            ("balloon.directory", "a"),
        ])
        .unwrap();
        assert_eq!(
            specs,
            vec![
                spec("balloon.directory", "a", false),
                spec("balloon0.source.directory", "b", true),
                spec("balloon1.directory", "c", false),
            ]
        );
    }

    #[test]
    fn test_find_balloon_specs_numbered_without_unnumbered() {
        let specs = specs_of(&[("balloon0.directory", "a")]).unwrap();
        assert_eq!(specs, vec![spec("balloon0.directory", "a", false)]);
    }

    #[test]
    fn test_find_balloon_specs_rejects_spec_after_gap() {
        assert_invalid(
            specs_of(&[("balloon0.directory", "a"), ("balloon2.directory", "c")]),
            "install.txt: balloon2.directory is never read by the baseware because balloon1 is missing (numbered entries must not have gaps)",
        );
        // balloon0 が無ければ番号なしがあっても balloon1 以降は読まれない
        assert_invalid(
            specs_of(&[
                ("balloon.directory", "a"),
                ("balloon1.source.directory", "b"),
            ]),
            "install.txt: balloon1.source.directory is never read by the baseware because balloon0 is missing (numbered entries must not have gaps)",
        );
        // u32 に収まらない番号も打ち切った番号より後ろの指定として扱う
        assert_invalid(
            specs_of(&[
                ("balloon0.directory", "a"),
                ("balloon99999999999999999999.directory", "b"),
            ]),
            "install.txt: balloon99999999999999999999.directory is never read by the baseware because balloon1 is missing (numbered entries must not have gaps)",
        );
    }

    #[test]
    fn test_find_balloon_specs_rejects_leading_zero_number() {
        let expected = "install.txt: balloon01.directory is never read by the baseware (numbers must not have leading zeros)";
        assert_invalid(
            specs_of(&[("balloon0.directory", "a"), ("balloon01.directory", "b")]),
            expected,
        );
        // balloon1 と balloon01 は別のキーで、重複ではなく先頭 0 のエラーになる
        assert_invalid(
            specs_of(&[
                ("balloon0.directory", "a"),
                ("balloon1.directory", "b"),
                ("balloon01.directory", "c"),
            ]),
            expected,
        );
    }

    #[test]
    fn test_find_balloon_specs_ignores_key_case() {
        let kv = kv_of(b"charset,UTF-8\r\nBalloon.Source.Directory,a\r\nBALLOON0.DIRECTORY,b\r\n")
            .unwrap();
        let specs: Vec<_> = find_balloon_specs(&kv)
            .unwrap()
            .into_iter()
            .map(|s| (s.key, s.value, s.hierarchical_allowed))
            .collect();
        assert_eq!(
            specs,
            vec![
                spec("balloon.source.directory", "a", true),
                spec("balloon0.directory", "b", false),
            ]
        );
    }

    #[test]
    fn test_find_balloon_specs_rejects_duplicate_key() {
        let kv = kv_of(b"charset,UTF-8\r\nballoon.directory,a\r\nBalloon.Directory,b\r\n").unwrap();
        let err = find_balloon_specs(&kv).err().unwrap();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(
            err.to_string(),
            "install.txt: duplicate key balloon.directory"
        );

        assert_invalid(
            specs_of(&[
                ("balloon0.source.directory", "a"),
                ("balloon0.source.directory", "a"),
            ]),
            "install.txt: duplicate key balloon0.source.directory",
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
