//! span の正解のテスト（振る舞いを固めるテスト）。
//!
//! AST のすべての span の行と桁が、バイトの位置から pest の
//! `Position::line_col` で出した値と一致することを確かめる。
//! `#[ignore]` の書き出しテストは、直す前と後の AST を比べるための控えを作る。

use pasta_dsl::parser::parse_str;
use std::path::{Path, PathBuf};

/// リポジトリの `crates/` と `.kiro/` の下の `.pasta` を集める。
/// ルートから歩くと別のワークツリーの複製まで拾うので、2 つの下だけを歩く。
fn repo_pasta_files() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut out = Vec::new();
    for dir in ["crates", ".kiro"] {
        walk(&root.join(dir), &mut out);
    }
    out.sort();
    out
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name();
            if name != "target" && name != "node_modules" {
                walk(&path, out);
            }
        } else if path.extension().is_some_and(|e| e == "pasta") {
            out.push(path);
        }
    }
}

/// `{:?}` から `Span { … }` の 6 つの数を拾う。
fn collect_spans(debug: &str) -> Vec<[usize; 6]> {
    const FIELDS: [&str; 6] = [
        "start_line: ",
        "start_col: ",
        "end_line: ",
        "end_col: ",
        "start_byte: ",
        "end_byte: ",
    ];
    let mut spans = Vec::new();
    for chunk in debug.split("Span { ").skip(1) {
        let body = &chunk[..chunk.find('}').expect("Span の閉じ括弧")];
        let mut vals = [0usize; 6];
        for (v, field) in vals.iter_mut().zip(FIELDS) {
            let rest = &body[body.find(field).expect(field) + field.len()..];
            let end = rest
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(rest.len());
            *v = rest[..end].parse().unwrap();
        }
        spans.push(vals);
    }
    spans
}

/// 1 つの辞書の span をすべて確かめ、確かめた数を返す。解析に失敗したら None。
fn check_source(source: &str, name: &str) -> Option<usize> {
    let file = parse_str(source, name).ok()?;
    // parse_str は BOM を外した文字列でバイトの位置を数える
    let src = source.strip_prefix('\u{FEFF}').unwrap_or(source);
    let mut count = 0;
    for [sl, sc, el, ec, sb, eb] in collect_spans(&format!("{:?}", file.items)) {
        if [sl, sc, el, ec, sb, eb] == [0; 6] {
            continue;
        }
        let start = pest::Position::new(src, sb).unwrap().line_col();
        let end = pest::Position::new(src, eb).unwrap().line_col();
        assert_eq!((sl, sc), start, "{name}: start of byte {sb}..{eb}");
        assert_eq!((el, ec), end, "{name}: end of byte {sb}..{eb}");
        count += 1;
    }
    Some(count)
}

#[test]
fn span_matches_pest_line_col_for_repo_dictionaries() {
    let files = repo_pasta_files();
    assert!(!files.is_empty(), "no .pasta files found");
    let mut total = 0;
    for path in &files {
        let source = std::fs::read_to_string(path).unwrap();
        total += check_source(&source, &path.display().to_string()).unwrap_or(0);
    }
    assert!(total > 0, "no spans collected");
}

#[test]
fn span_matches_pest_line_col_for_edge_dictionaries() {
    let cases = [
        (
            "crlf",
            "＊挨拶\r\n  さくら：こんにちは\r\n  ＄x＝（１＋２）\n",
        ),
        ("bom", "\u{FEFF}＊挨拶\n  さくら：こんにちは\n"),
        (
            "emoji",
            "＊挨拶\n  さくら：🎈ＡＢＣ🎈こんにちは\n  ＄x＝（１＋２）\n",
        ),
        // 文法は単独の `\r` を改行と読み、pest の line_col は 1 桁と数える
        (
            "lone_cr",
            "＊挨拶\n  さくら：こんにちは\r  うにゅう：へえ\n",
        ),
        // 行は改行で終わる文法なので、最後の行は空白だけにする
        ("no_trailing_newline", "＊挨拶\n  さくら：こんにちは\n  "),
    ];
    for (name, source) in cases {
        let count = check_source(source, name).unwrap_or_else(|| panic!("{name}: parse failed"));
        assert!(count > 0, "{name}: no spans collected");
    }
}

/// 直す前と後の AST を比べるための書き出し（`PASTA_AST_DUMP_DIR` のフォルダへ）。
#[test]
#[ignore]
fn dump_ast_to_dir() {
    let dir = PathBuf::from(std::env::var("PASTA_AST_DUMP_DIR").expect("PASTA_AST_DUMP_DIR"));
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = root.canonicalize().unwrap();
    for path in repo_pasta_files() {
        let path = path.canonicalize().unwrap();
        let rel = path.strip_prefix(&root).unwrap();
        let source = std::fs::read_to_string(&path).unwrap();
        // 名前は相対パスで固定する（Err の文言に絶対パスを混ぜない）
        let name = rel.to_string_lossy().replace('\\', "/");
        let text = match parse_str(&source, &name) {
            Ok(file) => format!("{:#?}\n", file.items),
            Err(e) => format!("Err({e:?})\n"),
        };
        let out = dir.join(format!("{name}.txt"));
        std::fs::create_dir_all(out.parent().unwrap()).unwrap();
        std::fs::write(out, text).unwrap();
    }
}
