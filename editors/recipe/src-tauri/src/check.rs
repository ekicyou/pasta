//! ゴースト全体の検査（pasta_check の集め＋核・列の UTF-16 化）。

use pasta_check::ghost_files::collect;
use pasta_check::validate::{FileText, Report, TomlSource, validate_ghost};
use std::path::Path;

/// ghost_files::collect → validate_ghost → 各 Finding の column を「その行のコードポイントの列 → UTF-16 の列（1 始まり）」に直す
pub fn check_ghost(master: &Path) -> Result<Report, String> {
    let (input, files) = collect(master)?;
    let mut report = validate_ghost(&input, &files);
    for f in &mut report.findings {
        // 辞書は parse_str が BOM を外した本文で数える。pasta.toml は本文そのままで数える
        let text = if f.path == "pasta.toml" {
            match &input.pasta_toml {
                TomlSource::Text { text } => Some(text.as_str()),
                _ => None,
            }
        } else {
            match files.get(&f.path) {
                Some(FileText::Text { text }) => {
                    Some(text.strip_prefix('\u{feff}').unwrap_or(text))
                }
                _ => None,
            }
        };
        if let (Some(text), Some(line), Some(column)) = (text, f.line, f.column) {
            f.column = Some(utf16_column(text, line, column));
        }
    }
    Ok(report)
}

/// 1 始まりの行・コードポイントの列を、その行の UTF-16 の列（1 始まり）に直す。行が無ければ元のまま
fn utf16_column(text: &str, line: u32, column: u32) -> u32 {
    let Some(row) = text.split('\n').nth(line.saturating_sub(1) as usize) else {
        return column;
    };
    let row = row.strip_suffix('\r').unwrap_or(row);
    let units: usize = row
        .chars()
        .take(column.saturating_sub(1) as usize)
        .map(char::len_utf16)
        .sum();
    units as u32 + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use pasta_check::validate::Finding;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn utf16_column_counts_surrogate_pairs_twice() {
        // 😀 は UTF-16 で 2 単位なので、その次（コードポイントの 3 列目）は 4 列目
        assert_eq!(utf16_column("a😀b", 1, 3), 4);
        assert_eq!(utf16_column("a😀b", 1, 1), 1);
    }

    #[test]
    fn utf16_column_full_width_is_one_unit() {
        assert_eq!(utf16_column("＊挨拶\n  ＠＠＠", 2, 4), 4);
    }

    #[test]
    fn utf16_column_ignores_crlf_and_picks_the_line() {
        assert_eq!(utf16_column("x\r\n😀😀y\r\n", 2, 3), 5);
        // 行の終わりの先（改行の位置）も \r を数えない
        assert_eq!(utf16_column("😀\r\nz", 1, 2), 3);
    }

    #[test]
    fn utf16_column_keeps_value_without_the_line() {
        assert_eq!(utf16_column("a", 5, 7), 7);
    }

    /// pasta_check validate と同じ集め＋核の結果
    fn validate_report(master: &Path) -> Report {
        let (input, files) = collect(master).unwrap();
        validate_ghost(&input, &files)
    }

    /// 列を除いた行（validate の出力の行の、列のほかの中身）
    fn lines_without_column(report: &Report) -> Vec<String> {
        let mut r = report.clone();
        r.findings.iter_mut().for_each(|f| f.column = None);
        r.lines()
    }

    fn key(f: &Finding) -> (String, String, String, Option<u32>, String) {
        (
            format!("{:?}", f.code),
            format!("{:?}", f.severity),
            f.path.clone(),
            f.line,
            f.message.clone(),
        )
    }

    fn assert_same_as_validate(master: &Path) -> Report {
        let got = check_ghost(master).unwrap();
        let want = validate_report(master);
        assert_eq!(
            got.findings.iter().map(key).collect::<Vec<_>>(),
            want.findings.iter().map(key).collect::<Vec<_>>()
        );
        assert_eq!(lines_without_column(&got), lines_without_column(&want));
        got
    }

    #[test]
    fn hello_pasta_matches_validate() {
        let master = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master");
        assert!(master.is_dir(), "{}", master.display());
        let got = assert_same_as_validate(&master);
        assert_eq!(got.stats.errors, 0);
    }

    #[test]
    fn ghost_with_errors_warnings_and_surrogates_matches_validate() {
        let tmp = tempfile::tempdir().unwrap();
        let master = tmp.path().join("ghost").join("master");
        fs::create_dir_all(master.join("dic")).unwrap();
        fs::write(master.join("pasta.toml"), "").unwrap();
        // 誤り（優先の値）: BOM・CRLF のファイルの 2 行目、サロゲートペア（𠮷）2 つの後ろの ＆
        fs::write(
            master.join("dic/a.pasta"),
            "\u{feff}＊挨拶\r\n＊𠮷𠮷　＆優先：高い\r\n  さくら：はい。\r\n",
        )
        .unwrap();
        // 注意（飛び先が無い）
        fs::write(master.join("dic/b.pasta"), "＊別\n  ＞どこにも無い\n").unwrap();

        let got = assert_same_as_validate(&master);
        let want = validate_report(&master);
        assert!(
            got.stats.errors > 0 && got.stats.warnings > 0,
            "{:?}",
            got.lines()
        );

        // ＆ はコードポイントで 5 列目、UTF-16 で 7 列目
        let at = |r: &Report| {
            r.findings
                .iter()
                .find(|f| f.path == "dic/a.pasta" && f.line == Some(2))
                .map(|f| f.column)
                .expect("サロゲートペアの行に誤りがある")
        };
        assert_eq!((at(&want), at(&got)), (Some(5), Some(7)));
    }

    #[test]
    fn collect_failure_returns_reason() {
        let tmp = tempfile::tempdir().unwrap();
        let err = check_ghost(&tmp.path().join("nothing")).unwrap_err();
        assert!(err.contains("を読めません"), "{err}");
    }
}
