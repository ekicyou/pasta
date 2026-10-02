//! 同梱バルーン判定モジュール
//!
//! 配布フォルダの `install.txt`・バルーンの `descript.txt` を読むだけで、ファイルは書きません。

use std::collections::{HashMap, HashSet};
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

/// 値を要素列に正規化し検証する。`/`・`\` のどちらも区切りとみなし、空要素と `.` は捨てる。
/// 検査は 空 → `directory` に区切り → 先頭区切り → `:` → `..` → 正規化後の空 → `profile`・`var`
/// の順で、最初に該当した理由でエラーにする。
fn normalize_value(spec: &BalloonSpec) -> io::Result<Vec<String>> {
    let value = spec.value.trim();
    let invalid = |reason: &str| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "install.txt: invalid value for {}: \"{value}\" ({reason})",
                spec.key
            ),
        )
    };
    let is_sep = |c: char| c == '/' || c == '\\';

    if value.is_empty() {
        return Err(invalid("empty"));
    }
    if !spec.hierarchical_allowed && value.contains(is_sep) {
        return Err(invalid("path separator is not allowed in directory"));
    }
    // 先頭の区切りはルート・UNC、`:` はドライブ名・ドライブ相対・代替ストリーム
    if value.starts_with(is_sep) || value.contains(':') {
        return Err(invalid("absolute path"));
    }
    let comps: Vec<String> = value
        .split(is_sep)
        .filter(|c| !c.is_empty() && *c != ".")
        .map(str::to_string)
        .collect();
    if comps.iter().any(|c| c == "..") {
        return Err(invalid("parent directory reference"));
    }
    if comps.is_empty() {
        return Err(invalid("empty"));
    }
    if comps
        .iter()
        .any(|c| c.eq_ignore_ascii_case("profile") || c.eq_ignore_ascii_case("var"))
    {
        return Err(invalid("excluded folder (profile/var)"));
    }
    Ok(comps)
}

/// 要素ごとに親フォルダを `read_dir` し、実在のフォルダ名へ解決して `/` で結ぶ。
/// 名前が完全一致するフォルダ（シンボリックリンクでないもの）を採り、無ければ
/// `親.join(要素).is_dir()` が真（ファイルシステムが同じとみなす）の場合に限り、
/// 小文字化（Unicode）して一致するフォルダの実在名を採る。
/// `is_dir()` だけでは採らない（Windows では `bal.`・`...` なども真になるため）。
/// 結合には検証済みの要素と `read_dir` の実在名だけを使うので、配布フォルダの外を指さない。
fn resolve_existing_dir(root: &Path, comps: &[String], key: &str) -> io::Result<String> {
    let not_found = || {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "install.txt: bundled balloon folder \"{}\" ({key}) does not exist in the release folder",
                comps.join("/")
            ),
        )
    };

    let mut dir = root.to_path_buf();
    let mut names = Vec::with_capacity(comps.len());
    for comp in comps {
        // file_type() はリンクをたどらないので、シンボリックリンク（ジャンクション含む）は is_dir() が偽
        let mut subdirs = Vec::new();
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir()
                && let Ok(name) = entry.file_name().into_string()
            {
                subdirs.push(name);
            }
        }
        let name = match subdirs.iter().find(|n| *n == comp) {
            Some(name) => name,
            None if dir.join(comp).is_dir() => {
                let lower = comp.to_lowercase();
                subdirs
                    .iter()
                    .find(|n| n.to_lowercase() == lower)
                    .ok_or_else(not_found)?
            }
            None => return Err(not_found()),
        };
        dir.push(name);
        names.push(name.clone());
    }
    Ok(names.join("/"))
}

/// 解決後のフォルダ（.0）が同じ指定は最初の 1 件だけ残す。重なり検出より前に行う。
fn dedup_dirs(dirs: &mut Vec<(String, String)>) {
    let mut seen = HashSet::new();
    dirs.retain(|(dir, _)| seen.insert(dir.clone()));
}

/// (解決後のフォルダ, キー) の列で、ghost/master・他バルーンとの入れ子を検出する（重複除去済みとする）。
/// 要素単位の前方一致で判定し、同じ・上位・配下を重なりとする。`ghost/master` との比較だけは
/// 要素ごとに ASCII の大文字小文字を無視し、同梱バルーンどうしは実在名の完全一致で比べる。
fn check_overlaps(dirs: &[(String, String)]) -> io::Result<()> {
    let overlap = |dir: &str, key: &str, with: String| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("install.txt: bundled balloon folder \"{dir}\" ({key}) overlaps with {with}"),
        )
    };
    let nested = |a: &str, b: &str, eq: fn(&str, &str) -> bool| {
        a.split('/').zip(b.split('/')).all(|(x, y)| eq(x, y))
    };

    for (i, (dir, key)) in dirs.iter().enumerate() {
        if nested(dir, "ghost/master", |x, y| x.eq_ignore_ascii_case(y)) {
            return Err(overlap(dir, key, "ghost/master".to_string()));
        }
        if let Some((other, other_key)) = dirs[..i]
            .iter()
            .find(|(other, _)| nested(dir, other, |x, y| x == y))
        {
            return Err(overlap(
                dir,
                key,
                format!("bundled balloon folder \"{other}\" ({other_key})"),
            ));
        }
    }
    Ok(())
}

/// 同梱バルーンの判定結果。ファイルは一切書かない。
pub(crate) struct BalloonPlan {
    /// 同梱バルーンのフォルダ（配布フォルダからの相対パス、`/` 区切り、実在のフォルダ名）。
    /// 探索順（番号なし → 0 → 1 …）で並び、重複は除去済み。
    pub dirs: Vec<String>,
    /// 警告文（接頭辞なし。表示は呼び出し側が `Warning: ` を付ける）。
    pub warnings: Vec<String>,
}

/// 配布フォルダ直下の install.txt から同梱バルーンを判定する。
/// - install.txt が無い: Ok(空の BalloonPlan)
/// - 判定に関わる問題: Err(io::Error)（kind は InvalidData。メッセージにファイル・キー・値を含める）
pub(crate) fn plan_bundled_balloons(release_dir: &Path) -> io::Result<BalloonPlan> {
    // 宣言の確認（read_utf8_kv）は指定の有無より先に行う
    let kv = match read_utf8_kv(&release_dir.join("install.txt"), "install.txt") {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Vec::new(),
        r => r?,
    };

    let mut dirs = Vec::new();
    for spec in find_balloon_specs(&kv)? {
        let comps = normalize_value(&spec)?;
        dirs.push((
            resolve_existing_dir(release_dir, &comps, &spec.key)?,
            spec.key,
        ));
    }
    // 解決 → 重複除去 → 重なり検出の順（逆だと同じフォルダが重なりと誤判定される）
    dedup_dirs(&mut dirs);
    check_overlaps(&dirs)?;

    let mut warnings = Vec::new();
    for (dir, _) in &dirs {
        let path = release_dir.join(dir).join("descript.txt");
        if !path.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("bundled balloon \"{dir}\": descript.txt not found"),
            ));
        }
        let kv = read_utf8_kv(&path, &format!("{dir}/descript.txt"))?;
        let homeurl = kv.iter().find(|(k, _)| k == "homeurl");
        if homeurl.is_none_or(|(_, v)| v.is_empty()) {
            warnings.push(format!(
                "bundled balloon \"{dir}\": descript.txt has no homeurl; the balloon cannot be network-updated"
            ));
        }
    }

    Ok(BalloonPlan {
        dirs: dirs.into_iter().map(|(dir, _)| dir).collect(),
        warnings,
    })
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

    fn normalize(key: &str, value: &str, hierarchical_allowed: bool) -> io::Result<Vec<String>> {
        normalize_value(&BalloonSpec {
            key: key.to_string(),
            value: value.to_string(),
            hierarchical_allowed,
        })
    }

    #[test]
    fn test_normalize_value_accepts_hierarchical_paths() {
        let key = "balloon.source.directory";
        assert_eq!(
            normalize(key, r"extra\bal1", true).unwrap(),
            ["extra", "bal1"]
        );
        assert_eq!(
            normalize(key, "extra/bal1/", true).unwrap(),
            ["extra", "bal1"]
        );
        assert_eq!(normalize(key, "./a//b", true).unwrap(), ["a", "b"]);
        assert_eq!(
            normalize("balloon.directory", "bal1", false).unwrap(),
            ["bal1"]
        );
    }

    #[test]
    fn test_normalize_value_rejects_each_reason() {
        let src = "balloon0.source.directory";
        let dir = "balloon.directory";
        for (key, value, hierarchical_allowed, reason) in [
            (src, "", true, "empty"),
            (src, "/abs", true, "absolute path"),
            (src, r"\\server\x", true, "absolute path"),
            (src, r"C:\x", true, "absolute path"),
            (src, "C:x", true, "absolute path"),
            (src, "a/../b", true, "parent directory reference"),
            (src, ".", true, "empty"),
            (src, "./", true, "empty"),
            (
                dir,
                "a/b",
                false,
                "path separator is not allowed in directory",
            ),
            (
                dir,
                r"a\b",
                false,
                "path separator is not allowed in directory",
            ),
            (
                dir,
                "/abs",
                false,
                "path separator is not allowed in directory",
            ),
            (src, "profile/bal", true, "excluded folder (profile/var)"),
            (src, "x/Var/bal", true, "excluded folder (profile/var)"),
            (dir, "PROFILE", false, "excluded folder (profile/var)"),
            // 最初に該当した理由でエラーにする
            (src, "/../x", true, "absolute path"),
            (src, "../profile", true, "parent directory reference"),
            (
                dir,
                "profile/x",
                false,
                "path separator is not allowed in directory",
            ),
        ] {
            let err = normalize(key, value, hierarchical_allowed).unwrap_err();
            assert_eq!(err.kind(), io::ErrorKind::InvalidData);
            assert_eq!(
                err.to_string(),
                format!("install.txt: invalid value for {key}: \"{value}\" ({reason})")
            );
        }
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

    fn resolve(root: &Path, comps: &[&str]) -> io::Result<String> {
        let comps: Vec<String> = comps.iter().map(|c| c.to_string()).collect();
        resolve_existing_dir(root, &comps, "balloon.source.directory")
    }

    fn assert_not_exist(result: io::Result<String>, folder: &str) {
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(
            err.to_string(),
            format!(
                "install.txt: bundled balloon folder \"{folder}\" (balloon.source.directory) does not exist in the release folder"
            )
        );
    }

    #[test]
    fn test_resolve_existing_dir_joins_existing_names() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("extra/bal1")).unwrap();
        assert_eq!(
            resolve(temp.path(), &["extra", "bal1"]).unwrap(),
            "extra/bal1"
        );
        assert_eq!(resolve(temp.path(), &["extra"]).unwrap(), "extra");
    }

    #[test]
    fn test_resolve_existing_dir_rejects_missing_or_file() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("extra")).unwrap();
        fs::write(temp.path().join("bal"), b"").unwrap();
        fs::write(temp.path().join("extra/bal1"), b"").unwrap();
        assert_not_exist(resolve(temp.path(), &["emo2-kakukaku"]), "emo2-kakukaku");
        // 同名のファイルはフォルダではない
        assert_not_exist(resolve(temp.path(), &["bal"]), "bal");
        assert_not_exist(resolve(temp.path(), &["extra", "bal1"]), "extra/bal1");
        assert_not_exist(resolve(temp.path(), &["nope", "bal1"]), "nope/bal1");
    }

    #[test]
    fn test_resolve_existing_dir_rejects_names_only_the_os_resolves() {
        // Windows では `bal.`・`bal `・`...` も is_dir() が真になるが、実在名と一致しないので不在とする
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("bal")).unwrap();
        for name in ["bal.", "bal ", "..."] {
            assert_not_exist(resolve(temp.path(), &[name]), name);
        }
        assert_not_exist(resolve(temp.path(), &["bal", "..."]), "bal/...");
    }

    #[cfg(unix)]
    #[test]
    fn test_resolve_existing_dir_rejects_symlink() {
        let temp = TempDir::new().unwrap();
        let real = temp.path().join("real");
        fs::create_dir(&real).unwrap();
        std::os::unix::fs::symlink(&real, temp.path().join("bal")).unwrap();
        assert_not_exist(resolve(temp.path(), &["bal"]), "bal");
    }

    #[cfg(windows)]
    #[test]
    fn test_resolve_existing_dir_rejects_junction() {
        // ジャンクションは権限なしで作れる: cmd /C mklink /J <link> <target>
        let temp = TempDir::new().unwrap();
        let real = temp.path().join("real");
        fs::create_dir(&real).unwrap();
        let status = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(temp.path().join("bal"))
            .arg(&real)
            .status()
            .expect("failed to spawn cmd for mklink");
        assert!(status.success(), "mklink /J failed");
        assert_not_exist(resolve(temp.path(), &["bal"]), "bal");
        assert_not_exist(resolve(temp.path(), &["BAL"]), "BAL");
    }

    #[cfg(windows)]
    #[test]
    fn test_resolve_existing_dir_uses_existing_full_width_case_on_windows() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("ｂａｌ")).unwrap();
        assert_eq!(resolve(temp.path(), &["ＢＡＬ"]).unwrap(), "ｂａｌ");
    }

    #[cfg(windows)]
    #[test]
    fn test_resolve_existing_dir_uses_existing_case_on_windows() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("extra/emo2-kakukaku")).unwrap();
        assert_eq!(
            resolve(temp.path(), &["EMO2-KAKUKAKU"]).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(
            resolve(temp.path(), &["Extra", "EMO2-KAKUKAKU"]).unwrap(),
            "extra/emo2-kakukaku"
        );
        fs::create_dir(temp.path().join("emo2-kakukaku")).unwrap();
        assert_eq!(
            resolve(temp.path(), &["EMO2-KAKUKAKU"]).unwrap(),
            "emo2-kakukaku"
        );
    }

    fn dirs(v: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs(v)
    }

    #[test]
    fn test_dedup_dirs_keeps_first() {
        let mut d = dirs(&[
            ("a", "balloon.directory"),
            ("b", "balloon0.directory"),
            ("a", "balloon1.source.directory"),
        ]);
        dedup_dirs(&mut d);
        assert_eq!(
            d,
            dirs(&[("a", "balloon.directory"), ("b", "balloon0.directory")])
        );
    }

    #[test]
    fn test_check_overlaps_rejects_ghost_master() {
        for dir in ["ghost", "ghost/master", "ghost/master/x", "Ghost/MASTER"] {
            let err = check_overlaps(&dirs(&[
                ("ok", "balloon.directory"),
                (dir, "balloon0.source.directory"),
            ]))
            .unwrap_err();
            assert_eq!(err.kind(), io::ErrorKind::InvalidData);
            assert_eq!(
                err.to_string(),
                format!(
                    "install.txt: bundled balloon folder \"{dir}\" (balloon0.source.directory) overlaps with ghost/master"
                )
            );
        }
    }

    #[test]
    fn test_check_overlaps_rejects_nested_balloons() {
        let expected = "install.txt: bundled balloon folder \"a/b\" (balloon1.source.directory) overlaps with bundled balloon folder \"a\" (balloon0.source.directory)";
        let err = check_overlaps(&dirs(&[
            ("a", "balloon0.source.directory"),
            ("a/b", "balloon1.source.directory"),
        ]))
        .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(err.to_string(), expected);

        let err = check_overlaps(&dirs(&[
            ("x/a/b", "balloon0.source.directory"),
            ("x/a", "balloon1.source.directory"),
        ]))
        .unwrap_err();
        assert_eq!(
            err.to_string(),
            "install.txt: bundled balloon folder \"x/a\" (balloon1.source.directory) overlaps with bundled balloon folder \"x/a/b\" (balloon0.source.directory)"
        );
    }

    #[test]
    fn test_check_overlaps_accepts_separate_folders() {
        check_overlaps(&dirs(&[
            ("a", "balloon.directory"),
            ("ab", "balloon0.directory"),
            ("ghosts", "balloon1.directory"),
            ("ghost2/master", "balloon2.source.directory"),
            ("x/a", "balloon3.source.directory"),
            ("ghost/masterx", "balloon4.source.directory"),
        ]))
        .unwrap();
        check_overlaps(&[]).unwrap();
    }

    /// 配布フォルダを作り、(相対パス, 内容) のファイルを置く。
    fn release_with(files: &[(&str, &[u8])]) -> TempDir {
        let temp = TempDir::new().unwrap();
        for (rel, bytes) in files {
            let path = temp.path().join(rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, bytes).unwrap();
        }
        temp
    }

    fn plan(root: &Path) -> io::Result<(Vec<String>, Vec<String>)> {
        plan_bundled_balloons(root).map(|p| (p.dirs, p.warnings))
    }

    fn plan_err(root: &Path) -> String {
        let err = plan_bundled_balloons(root).err().unwrap();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        err.to_string()
    }

    const INSTALL_BAL: &[u8] = b"charset,UTF-8\r\ntype,ghost\r\nballoon.source.directory,bal\r\n";
    const DESCRIPT_OK: &[u8] =
        b"charset,UTF-8\r\ntype,balloon\r\nHomeURL,https://example.com/bal/\r\n";

    #[test]
    fn test_plan_empty_without_install_txt() {
        let temp = release_with(&[("ghost/master/descript.txt", b"charset,UTF-8\r\n")]);
        assert_eq!(plan(temp.path()).unwrap(), (vec![], vec![]));
    }

    #[test]
    fn test_plan_empty_without_spec() {
        let temp = release_with(&[(
            "install.txt",
            b"charset,UTF-8\r\ntype,ghost\r\ndirectory,hello\r\n",
        )]);
        assert_eq!(plan(temp.path()).unwrap(), (vec![], vec![]));
    }

    #[test]
    fn test_plan_checks_charset_before_spec() {
        // 指定が無くても宣言の無い install.txt はエラー
        let temp = release_with(&[("install.txt", b"type,ghost\r\ndirectory,hello\r\n")]);
        assert_eq!(
            plan_err(temp.path()),
            "install.txt is not UTF-8: the first line must be \"charset,UTF-8\" (pasta_check supports UTF-8 only)"
        );
    }

    #[test]
    fn test_plan_returns_dir_without_warning_when_homeurl_present() {
        let temp = release_with(&[
            ("install.txt", INSTALL_BAL),
            ("bal/descript.txt", DESCRIPT_OK),
        ]);
        assert_eq!(
            plan(temp.path()).unwrap(),
            (vec!["bal".to_string()], vec![])
        );
    }

    #[test]
    fn test_plan_warns_when_homeurl_missing_or_empty() {
        for descript in [
            &b"charset,UTF-8\r\ntype,balloon\r\n"[..],
            b"charset,UTF-8\r\nhomeurl,\r\n",
            b"charset,UTF-8\r\nhomeurl,   \r\n",
            // 2 行目以降の homeurl は見ない（最初の行を採る）
            b"charset,UTF-8\r\nhomeurl,\r\nhomeurl,https://example.com/\r\n",
        ] {
            let temp = release_with(&[
                (
                    "install.txt",
                    b"charset,UTF-8\r\nballoon.source.directory,extra/emo2-kakukaku\r\n",
                ),
                ("extra/emo2-kakukaku/descript.txt", descript),
            ]);
            assert_eq!(
                plan(temp.path()).unwrap(),
                (
                    vec!["extra/emo2-kakukaku".to_string()],
                    vec![
                        "bundled balloon \"extra/emo2-kakukaku\": descript.txt has no homeurl; the balloon cannot be network-updated"
                            .to_string()
                    ]
                )
            );
        }
    }

    #[test]
    fn test_plan_rejects_missing_descript_txt() {
        let temp = release_with(&[
            (
                "install.txt",
                b"charset,UTF-8\r\nballoon.directory,emo2-kakukaku\r\n",
            ),
            ("emo2-kakukaku/readme.txt", b""),
        ]);
        assert_eq!(
            plan_err(temp.path()),
            "bundled balloon \"emo2-kakukaku\": descript.txt not found"
        );
        // 同名のフォルダはファイルではない
        fs::create_dir(temp.path().join("emo2-kakukaku/descript.txt")).unwrap();
        assert_eq!(
            plan_err(temp.path()),
            "bundled balloon \"emo2-kakukaku\": descript.txt not found"
        );
    }

    #[test]
    fn test_plan_rejects_non_utf8_descript_txt() {
        let temp = release_with(&[
            (
                "install.txt",
                b"charset,UTF-8\r\nballoon.directory,emo2-kakukaku\r\n",
            ),
            (
                "emo2-kakukaku/descript.txt",
                b"charset,Shift_JIS\r\nhomeurl,x\r\n",
            ),
        ]);
        assert_eq!(
            plan_err(temp.path()),
            "emo2-kakukaku/descript.txt is not UTF-8: the first line must be \"charset,UTF-8\" (pasta_check supports UTF-8 only)"
        );
    }

    #[test]
    fn test_plan_ignores_install_txt_in_subfolders() {
        // バルーン自身の install.txt（指定・宣言なし）も ghost/master の install.txt も判定に使わない
        let temp = release_with(&[
            (
                "bal/install.txt",
                b"type,balloon\r\nballoon.directory,ghost\r\n",
            ),
            (
                "ghost/master/install.txt",
                b"charset,UTF-8\r\nballoon.directory,bal\r\n",
            ),
            ("bal/descript.txt", b"charset,UTF-8\r\n"),
        ]);
        assert_eq!(plan(temp.path()).unwrap(), (vec![], vec![]));

        fs::write(temp.path().join("install.txt"), INSTALL_BAL).unwrap();
        fs::write(temp.path().join("bal/descript.txt"), DESCRIPT_OK).unwrap();
        assert_eq!(
            plan(temp.path()).unwrap(),
            (vec!["bal".to_string()], vec![])
        );
    }

    #[test]
    fn test_plan_dedups_specs_pointing_to_same_folder() {
        // 区切りの違いだけの指定は 1 件になり、警告も 1 件
        let temp = release_with(&[
            (
                "install.txt",
                b"charset,UTF-8\r\nballoon.source.directory,extra\\bal\r\nballoon0.source.directory,extra/bal/\r\n",
            ),
            ("extra/bal/descript.txt", b"charset,UTF-8\r\n"),
        ]);
        let (dirs, warnings) = plan(temp.path()).unwrap();
        assert_eq!(dirs, ["extra/bal"]);
        assert_eq!(warnings.len(), 1);
    }

    #[cfg(windows)]
    #[test]
    fn test_plan_dedups_case_variants_on_windows() {
        let temp = release_with(&[
            (
                "install.txt",
                b"charset,UTF-8\r\nballoon.source.directory,extra\\Bal\r\nballoon0.source.directory,EXTRA/bal\r\n",
            ),
            ("extra/bal/descript.txt", DESCRIPT_OK),
        ]);
        assert_eq!(
            plan(temp.path()).unwrap(),
            (vec!["extra/bal".to_string()], vec![])
        );
    }

    #[test]
    fn test_plan_keeps_search_order() {
        let temp = release_with(&[
            (
                "install.txt",
                b"charset,UTF-8\r\nballoon1.directory,b\r\nballoon0.directory,a\r\nballoon.directory,c\r\n",
            ),
            ("a/descript.txt", DESCRIPT_OK),
            ("b/descript.txt", DESCRIPT_OK),
            ("c/descript.txt", b"charset,UTF-8\r\n"),
        ]);
        assert_eq!(
            plan(temp.path()).unwrap(),
            (
                vec!["c".to_string(), "a".to_string(), "b".to_string()],
                vec![
                    "bundled balloon \"c\": descript.txt has no homeurl; the balloon cannot be network-updated"
                        .to_string()
                ]
            )
        );
    }

    /// 配布フォルダの (相対パス, ファイルなら内容) をすべて集める。
    fn snapshot(root: &Path) -> std::collections::BTreeMap<std::path::PathBuf, Option<Vec<u8>>> {
        let mut out = std::collections::BTreeMap::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                let rel = path.strip_prefix(root).unwrap().to_path_buf();
                if path.is_dir() {
                    out.insert(rel, None);
                    stack.push(path);
                } else {
                    out.insert(rel, Some(fs::read(&path).unwrap()));
                }
            }
        }
        out
    }

    #[test]
    fn test_plan_does_not_change_release_folder() {
        let ok = release_with(&[
            ("install.txt", INSTALL_BAL),
            ("bal/descript.txt", b"charset,UTF-8\r\n"),
            ("bal/install.txt", b"type,balloon\r\n"),
            ("ghost/master/descript.txt", b"charset,UTF-8\r\nname,x\n"),
        ]);
        let ng = release_with(&[
            ("install.txt", INSTALL_BAL),
            ("bal/readme.txt", b"no descript\r\n"),
        ]);
        for (temp, ok) in [(ok, true), (ng, false)] {
            let before = snapshot(temp.path());
            assert_eq!(plan_bundled_balloons(temp.path()).is_ok(), ok);
            assert_eq!(snapshot(temp.path()), before);
        }
    }
}
