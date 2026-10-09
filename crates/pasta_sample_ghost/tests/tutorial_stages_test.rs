//! 入門ガイドの段階検証（hello-pasta-tutorial-stages）。
//!
//! 段階表の正本 `STAGES.md` の 2 つの表（`## 段階表`・`## 検証イベント表`）を読み、
//! 配布辞書 `ghosts/hello-pasta/ghost/master/dic/` との対応を検査する。
//!
//! 表は見出しの行で探し、列見出しの名前で列を引く。見出しが無い・列が足りない・
//! 段階番号が数値でないなど形が崩れていれば、0 件として素通りせず panic（テストの失敗）にする。

use std::collections::BTreeSet;
use std::path::PathBuf;

/// Neutralize ambient DAP debug env vars before any test thread starts.
///
/// The developer session may export `PASTA_DEBUG=1` / `PASTA_DEBUG_PORT=9276`.
/// Loads in this binary would then bind that single fixed port and collide with
/// `AddrInUse`. Same guard as `self_deploy_integration_test.rs`; running inside a
/// `#[ctor]` (before `main`, single-threaded) makes `remove_var` race-free.
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

/// 段階表の 1 行（テストが読む列だけ）
struct StageRow {
    stage: u8,
    file: Option<String>, // "—" なら None
}

/// 検証イベント表の 1 行
struct VerifyEvent {
    stage: u8,
    id: String,
    references: Vec<(u8, String)>, // (Reference 番号, 値)
}

/// 表のセルに書く「無し」の記号
const NONE_MARK: &str = "—";

/// 見出し `heading` の直後の最初の GFM 表を、列見出しと行（セルの並び）に分けて返す。
fn find_table(markdown: &str, heading: &str) -> (Vec<String>, Vec<Vec<String>>) {
    let mut lines = markdown
        .lines()
        .skip_while(|l| l.trim_end() != heading)
        .skip(1);
    let first = loop {
        match lines.next() {
            None => panic!("{heading}: 見出しが無いか、見出しの後に表が無い"),
            Some(l) if l.starts_with('|') => break l,
            Some(l) if l.starts_with('#') => panic!("{heading}: 次の見出し `{l}` までに表が無い"),
            Some(_) => {}
        }
    };
    let header = split_row(first);
    let separator = lines
        .next()
        .unwrap_or_else(|| panic!("{heading}: 表の区切り行が無い"));
    assert!(
        separator.starts_with('|')
            && separator
                .chars()
                .all(|c| matches!(c, '|' | '-' | ':' | ' ')),
        "{heading}: 列見出しの次が区切り行でない: `{separator}`"
    );
    let rows: Vec<Vec<String>> = lines
        .take_while(|l| l.starts_with('|'))
        .map(split_row)
        .collect();
    assert!(!rows.is_empty(), "{heading}: 表に行が無い");
    for row in &rows {
        assert_eq!(
            row.len(),
            header.len(),
            "{heading}: 列の数が列見出しと合わない行: {row:?}"
        );
    }
    (header, rows)
}

fn split_row(line: &str) -> Vec<String> {
    let inner = line.trim().trim_start_matches('|').trim_end_matches('|');
    inner.split('|').map(|c| c.trim().to_string()).collect()
}

/// 列見出しの名前で列の位置を引く。無ければ panic。
fn column(header: &[String], name: &str, heading: &str) -> usize {
    header
        .iter()
        .position(|h| h == name)
        .unwrap_or_else(|| panic!("{heading}: 列 `{name}` が無い（列見出し: {header:?}）"))
}

/// セル全体を囲むバッククォートを外す（`` `01-boot.pasta` `` → `01-boot.pasta`）。
fn unquote(cell: &str) -> &str {
    cell.strip_prefix('`')
        .and_then(|c| c.strip_suffix('`'))
        .unwrap_or(cell)
}

fn parse_stage(cell: &str, heading: &str) -> u8 {
    cell.parse()
        .unwrap_or_else(|_| panic!("{heading}: 段階 `{cell}` が整数でない"))
}

/// `## 段階表` 直後の表を読む。見出し名で列を引く。形が崩れていれば panic（テストの失敗）
fn parse_stage_table(markdown: &str) -> Vec<StageRow> {
    const HEADING: &str = "## 段階表";
    let (header, rows) = find_table(markdown, HEADING);
    let stage_col = column(&header, "段階", HEADING);
    let file_col = column(&header, "追加するファイル", HEADING);
    rows.iter()
        .map(|row| {
            let file = unquote(&row[file_col]);
            StageRow {
                stage: parse_stage(&row[stage_col], HEADING),
                file: (file != NONE_MARK).then(|| file.to_string()),
            }
        })
        .collect()
}

/// `## 検証イベント表` 直後の表を読む
fn parse_verify_events(markdown: &str) -> Vec<VerifyEvent> {
    const HEADING: &str = "## 検証イベント表";
    let (header, rows) = find_table(markdown, HEADING);
    let stage_col = column(&header, "段階", HEADING);
    let id_col = column(&header, "イベント", HEADING);
    let ref_col = column(&header, "Reference", HEADING);
    rows.iter()
        .map(|row| VerifyEvent {
            stage: parse_stage(&row[stage_col], HEADING),
            id: unquote(&row[id_col]).to_string(),
            references: parse_references(unquote(&row[ref_col])),
        })
        .collect()
}

/// `0=むらさき, 2=えも？？` を `[(0, "むらさき"), (2, "えも？？")]` に。`—` は空。
fn parse_references(cell: &str) -> Vec<(u8, String)> {
    if cell == NONE_MARK {
        return Vec::new();
    }
    cell.split(", ")
        .map(|pair| {
            let (n, value) = pair
                .split_once('=')
                .unwrap_or_else(|| panic!("Reference `{pair}` が `番号=値` の形でない"));
            let n = n
                .parse()
                .unwrap_or_else(|_| panic!("Reference `{pair}` の番号が整数でない"));
            (n, value.to_string())
        })
        .collect()
}

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn dic_dir() -> PathBuf {
    crate_dir().join("ghosts/hello-pasta/ghost/master/dic")
}

/// 段階表の「追加するファイル」の集合 ＝ `dic/*.pasta` の集合、各ファイル名の番号 ＝ 段階、
/// 1 段 1 ファイルで 13 段目だけファイルが無い（Req 2.3・4.3）。
#[test]
fn stage_table_matches_dic_files() {
    let markdown =
        std::fs::read_to_string(crate_dir().join("STAGES.md")).expect("STAGES.md を読めない");
    let rows = parse_stage_table(&markdown);

    let stages: Vec<u8> = rows.iter().map(|r| r.stage).collect();
    assert_eq!(
        stages,
        (1..=13).collect::<Vec<u8>>(),
        "段階表の段階は 1〜13 を順に 1 行ずつ"
    );

    for row in &rows {
        match (row.stage, &row.file) {
            (13, None) => {}
            (13, Some(f)) => panic!("13 段目は辞書の差分を持たないのにファイル `{f}` がある"),
            (n, None) => panic!("{n} 段目に追加するファイルが無い"),
            (n, Some(f)) => {
                let name = f
                    .strip_prefix(&format!("{n:02}-"))
                    .and_then(|s| s.strip_suffix(".pasta"))
                    .unwrap_or_else(|| {
                        panic!("{n} 段目のファイル `{f}` が `{n:02}-name.pasta` の形でない")
                    });
                assert!(
                    !name.is_empty() && name.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
                    "{n} 段目のファイル `{f}` の name が ASCII 小文字（とハイフン）でない"
                );
            }
        }
    }

    let table_files: BTreeSet<&str> = rows.iter().filter_map(|r| r.file.as_deref()).collect();
    let dic_files: BTreeSet<String> = std::fs::read_dir(dic_dir())
        .expect("dic/ を読めない")
        .map(|e| e.expect("dic/ の項目を読めない").file_name())
        .filter_map(|n| n.into_string().ok())
        .filter(|n| n.ends_with(".pasta"))
        .collect();
    let dic_files: BTreeSet<&str> = dic_files.iter().map(String::as_str).collect();
    assert_eq!(
        table_files, dic_files,
        "段階表の「追加するファイル」と dic/*.pasta が一致しない（左: 段階表、右: dic/）"
    );

    let events = parse_verify_events(&markdown);
    for e in &events {
        assert!(
            (1..=12).contains(&e.stage),
            "検証イベント表の段階 {} は 1〜12 でない",
            e.stage
        );
        assert!(
            e.id.starts_with("On"),
            "検証イベント表のイベント `{}` がイベント ID でない",
            e.id
        );
        assert!(
            e.references.iter().all(|(_, v)| !v.is_empty()),
            "{} の Reference に空の値がある",
            e.id
        );
    }
}
