//! 入門ガイドの段階検証（hello-pasta-tutorial-stages）。
//!
//! 段階表の正本 `STAGES.md` の 2 つの表（`## 段階表`・`## 検証イベント表`）を読み、
//! 配布辞書 `ghosts/hello-pasta/ghost/master/dic/` との対応を検査する。
//!
//! 表は見出しの行で探し、列見出しの名前で列を引く。見出しが無い・列が足りない・
//! 段階番号が数値でないなど形が崩れていれば、0 件として素通りせず panic（テストの失敗）にする。
//!
//! 段階 N の検証は `master/` を tempdir へコピーしてから行い、コミット済みのゴーストを
//! その場では読み込まない（自己展開の `profile/` は tempdir の中にだけできる）。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use pasta_lua::PastaLuaRuntime;
use pasta_lua::loader::PastaLoader;
use tempfile::TempDir;

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

fn master_dir() -> PathBuf {
    crate_dir().join("ghosts/hello-pasta/ghost/master")
}

fn dic_dir() -> PathBuf {
    master_dir().join("dic")
}

fn read_stages_md() -> String {
    std::fs::read_to_string(crate_dir().join("STAGES.md")).expect("STAGES.md を読めない")
}

/// `src` を `dst` へ再帰コピーする。`profile/`（自己展開先・保存データ）と `pasta.dll`
/// （ローダーに不要）は持ち越さない。`self_deploy_integration_test.rs` の
/// `copy_tree_skip_profile` と同じ処理に `pasta.dll` の除外を足したもの。
fn copy_master(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).expect("create dst dir");
    for entry in std::fs::read_dir(src).expect("read src dir").flatten() {
        let path = entry.path();
        let name = entry.file_name();
        if entry.metadata().expect("metadata").is_dir() {
            if name != "profile" {
                copy_master(&path, &dst.join(&name));
            }
        } else if name != "pasta.dll" {
            std::fs::copy(&path, dst.join(&name)).expect("copy file");
        }
    }
}

/// master/ を tempdir へコピーし（profile/ と pasta.dll を除く）、
/// dic/ から段階 n より後のファイルを消す
fn assemble_stage(rows: &[StageRow], n: u8) -> TempDir {
    let temp = TempDir::new().expect("tempdir");
    copy_master(&master_dir(), temp.path());
    for row in rows.iter().filter(|r| r.stage > n) {
        if let Some(file) = &row.file {
            std::fs::remove_file(temp.path().join("dic").join(file))
                .unwrap_or_else(|e| panic!("stage {n}: dic/{file} を消せない: {e}"));
        }
    }
    temp
}

/// SHIORI.request を Lua から呼び、応答文字列を返す
fn shiori_request(runtime: &PastaLuaRuntime, id: &str, references: &[(u8, String)]) -> String {
    // 値は長括弧で埋め込む（引用符が入っても壊れない）。閉じ括弧が入る値は表に書かない。
    let lua_str = |v: &str| {
        assert!(!v.contains("]=]"), "Reference の値 `{v}` に `]=]` がある");
        format!("[=[{v}]=]")
    };
    let reference: String = references
        .iter()
        .map(|(n, v)| format!("[{n}] = {}, ", lua_str(v)))
        .collect();
    let script = format!(
        r#"local SHIORI = require("pasta.shiori.entry")
return SHIORI.request({{ id = {id}, method = "get", version = 30, charset = "UTF-8",
    sender = "SSP", reference = {{ {reference}}}, dic = {{}} }})"#,
        id = lua_str(id),
    );
    match runtime.exec(&script) {
        Ok(v) => v
            .as_string()
            .and_then(|s| s.to_str().ok().map(|s| s.to_string()))
            .unwrap_or_else(|| panic!("{id}: SHIORI.request が文字列を返さない: {v:?}")),
        Err(e) => format!("(Lua error) {e}"),
    }
}

/// 204 では素通りさせないイベント。9 段目の OnChoiceSelectEx は Reference2 が登録名でない
/// `OnMouseDoubleClick` で、グローバル前方一致のフォールバックで `＊おやつの話` に届いて
/// 200 になる。ジャンプ先が壊れると 204 に落ちて検査を素通りするので、ここだけ 200 と
/// 空でない Value を必須にする（tasks.md Implementation Notes「3.2 で」）。
const MUST_SPEAK: &[&str] = &["OnChoiceSelectEx"];

/// 200 OK かつ空でない Value 行、または 204 No Content であることを確かめる。
/// 違えば panic（メッセージに stage・file・id・応答全文を含める）
fn assert_responds(stage: u8, file: &str, id: &str, response: &str) {
    let status = response.lines().next().unwrap_or("");
    let has_value = response.lines().any(|l| {
        l.strip_prefix("Value:")
            .is_some_and(|v| !v.trim().is_empty())
    });
    let ok = match status {
        "SHIORI/3.0 200 OK" => has_value,
        "SHIORI/3.0 204 No Content" => !MUST_SPEAK.contains(&id),
        _ => false,
    };
    let expected = if MUST_SPEAK.contains(&id) {
        "200 with Value"
    } else {
        "200 with Value or 204"
    };
    assert!(
        ok,
        "stage {stage} ({file}): {id} returned `{status}` (expected {expected}):\n{response}"
    );
}

/// 段階表の「追加するファイル」の集合 ＝ `dic/*.pasta` の集合、各ファイル名の番号 ＝ 段階、
/// 1 段 1 ファイルで 13 段目だけファイルが無い（Req 2.3・4.3）。
#[test]
fn stage_table_matches_dic_files() {
    let markdown = read_stages_md();
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

/// N = 1〜12 で段階の辞書を組み立てて実ローダーで読み込み、OnBoot と検証イベント表の
/// 該当行を送って応答を確かめる（Req 1.6・2.1・2.6・3.9・4.1・4.2・4.2a・4.4・4.5・4.6）。
/// 仮想イベント（ランダムトーク・時報）は送らない。
#[test]
fn every_stage_loads_and_responds() {
    let markdown = read_stages_md();
    let rows = parse_stage_table(&markdown);
    let events = parse_verify_events(&markdown);

    // 13 段目は辞書の差分を持たない（手順だけの段階）ので、ファイルのある 1〜12 段目を回す
    for (n, file) in rows
        .iter()
        .filter_map(|r| Some((r.stage, r.file.as_deref()?)))
    {
        let ghost = assemble_stage(&rows, n);
        let runtime = PastaLoader::load(ghost.path())
            .unwrap_or_else(|e| panic!("stage {n} ({file}): load failed: {e}"));

        let response = shiori_request(&runtime, "OnBoot", &[]);
        assert_responds(n, file, "OnBoot", &response);
        for e in events.iter().filter(|e| e.stage == n) {
            let response = shiori_request(&runtime, &e.id, &e.references);
            assert_responds(n, file, &e.id, &response);
        }
    }
}

/// `GLOBAL` のキー。Call は GLOBAL を先に引くので、同じ名前のシーンは呼べない
/// （`pasta_lua` の `pasta/global.lua`・`pasta/shiori/entry.lua` が定義する）。
const GLOBAL_KEYS: &[&str] = &["yield", "チェイントーク", "ゴースト終了", "close_ghost"];

/// `pasta_shiori` の e2e が足すシーン（`KickE2EProbe`・`GatePrevScene` など）の接頭辞
const E2E_PREFIXES: &[&str] = &["Kick", "Gate"];

/// 行頭の `＊名前` / `*名前` の名前を集める。単独の `＊` と Lua ブロックの中は除く。
/// 名前は宣言行への属性の付記（`＆`）・行末コメント（`＃`）・空白の手前まで。
fn global_scene_names(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut fence = 0; // 開いている Lua ブロックのフェンスの本数（0 は外）
    for line in source.lines() {
        let ticks = line.chars().take_while(|&c| c == '`').count();
        if ticks >= 3 && (fence == 0 || ticks == fence) {
            fence = if fence == 0 { ticks } else { 0 };
            continue;
        }
        if fence > 0 {
            continue;
        }
        let Some(rest) = line.strip_prefix('＊').or_else(|| line.strip_prefix('*')) else {
            continue;
        };
        let name = rest
            .split(|c: char| c.is_whitespace() || matches!(c, '＆' | '&' | '＃' | '#'))
            .next()
            .unwrap_or("");
        if !name.is_empty() {
            names.push(name.to_string());
        }
    }
    assert_eq!(fence, 0, "Lua ブロックが閉じていない");
    names
}

#[test]
fn global_scene_names_skip_lua_and_trailers() {
    let source = "＊A＆作者：x\r\n＊\r\n*B ＃c\r\n```lua\r\n＊C\r\n```\r\n＊D\r\n````\r\n```\r\n＊E\r\n````\r\n    ・F\r\n";
    assert_eq!(global_scene_names(source), ["A", "B", "D"]);
}

/// 全 `dic/*.pasta` のグローバルシーン名について、異なる 2 つの名前のどちらも他方で始まらず、
/// `Kick`・`Gate` で始まらず、`GLOBAL` のキーと同じでないことを確かめる（Req 2.5・5.3）。
/// `＊会話` は別名で OnTalk になるが、書いた名前 `会話` のまま検査する。
#[test]
fn scene_names_do_not_prefix_collide() {
    let mut names = BTreeSet::new();
    for entry in std::fs::read_dir(dic_dir()).expect("dic/ を読めない") {
        let path = entry.expect("dic/ の項目を読めない").path();
        if path.extension().is_some_and(|e| e == "pasta") {
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{} を読めない: {e}", path.display()));
            names.extend(global_scene_names(&source));
        }
    }
    assert!(!names.is_empty(), "dic/ にグローバルシーンが無い");

    let mut problems = Vec::new();
    for a in &names {
        for b in names
            .iter()
            .filter(|b| *b != a && b.starts_with(a.as_str()))
        {
            problems.push(format!(
                "`＊{b}` が `＊{a}` で始まる（前方一致の候補に混ざる）"
            ));
        }
        if let Some(p) = E2E_PREFIXES.iter().find(|p| a.starts_with(*p)) {
            problems.push(format!(
                "`＊{a}` が `{p}` で始まる（pasta_shiori の e2e と衝突）"
            ));
        }
        if GLOBAL_KEYS.contains(&a.as_str()) {
            problems.push(format!("`＊{a}` が GLOBAL のキーと同じ"));
        }
    }
    assert!(
        problems.is_empty(),
        "シーン名の衝突（シーン名: {names:?}）:\n{}",
        problems.join("\n")
    );
}
