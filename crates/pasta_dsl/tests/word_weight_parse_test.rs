//! 単語の値の後ろの `＠整数`（重み）のパーステスト
//!
//! Requirements 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.8, 9.2

use pasta_dsl::parser::*;

fn parse(src: &str) -> PastaFile {
    parse_str(src, "test.pasta").unwrap()
}

/// ファイルレベルの単語定義（1 つ目）
fn global_word(src: &str) -> KeyWords {
    parse(src)
        .items
        .into_iter()
        .find_map(|item| match item {
            FileItem::GlobalWord(kw) => Some(kw),
            _ => None,
        })
        .unwrap()
}

fn words(line: &str) -> Vec<String> {
    global_word(&format!("{line}\n")).words
}

fn err(line: &str) -> String {
    match parse_str(&format!("{line}\n"), "test.pasta") {
        Ok(_) => panic!("エラーにならない: {line}"),
        Err(e) => e.to_string(),
    }
}

fn rep(v: &str, n: usize) -> Vec<String> {
    vec![v.to_string(); n]
}

#[test]
fn three_value_forms_full_and_half_width() {
    // 囲まない値
    assert_eq!(words("＠w：こんにちは＠３"), rep("こんにちは", 3));
    assert_eq!(words("@w:hello@3"), rep("hello", 3));
    assert_eq!(words("＠w：a＠12"), rep("a", 12));
    assert_eq!(words("＠w：a@１２"), rep("a", 12));
    // 「」で囲んだ値（中の読点も値）
    assert_eq!(words("＠w：「こんにちは、」＠３"), rep("こんにちは、", 3));
    assert_eq!(words("＠w：\"hi\"@2"), rep("hi", 2));
    // さくらスクリプトの値
    assert_eq!(words("＠w：\\s[10]＠２"), rep("\\s[10]", 2));
    assert_eq!(words("＠w：\\s[10]@2"), rep("\\s[10]", 2));
}

#[test]
fn weight_with_following_items_and_spaces() {
    assert_eq!(
        words("＠挨拶：こんにちは＠３、やあ"),
        vec!["こんにちは", "こんにちは", "こんにちは", "やあ"]
    );
    assert_eq!(words("＠w：a＠２ 、 b"), vec!["a", "a", "b"]);
    assert_eq!(words("＠w：a＠２、"), rep("a", 2));
    assert_eq!(words("＠w：「a」＠２ ＃メモ"), rep("a", 2));
}

#[test]
fn only_trailing_weight_of_unfenced_value_is_read() {
    assert_eq!(words("＠w：a＠3b"), vec!["a＠3b"]);
    assert_eq!(words("＠w：a＠３ ＃メモ"), vec!["a＠３ ＃メモ"]);
    assert_eq!(words("＠w：＠３abc"), vec!["＠３abc"]);
    assert_eq!(words("＠w：a＠"), vec!["a＠"]);
    assert_eq!(words("＠w：a＠２＠３"), rep("a＠２", 3));
    assert_eq!(words("＠w：mail@example.com"), vec!["mail@example.com"]);
}

#[test]
fn repeated_values_add_up() {
    assert_eq!(words("＠w：A＠２、A"), rep("A", 3));
    assert_eq!(words("＠w：A、B＠２、A＠２"), vec!["A", "B", "B", "A", "A"]);
}

#[test]
fn weight_applies_to_local_and_actor_words() {
    let file = parse("＊会話\n　＠w：a＠２\n　Alice：やあ\n\n％さくら\n　＠表情：\\s[0]＠３\n");
    let mut scene_words = None;
    let mut actor_words = None;
    for item in file.items {
        match item {
            FileItem::GlobalSceneScope(scene) => scene_words = Some(scene.words[0].words.clone()),
            FileItem::ActorScope(actor) => actor_words = Some(actor.words[0].words.clone()),
            _ => {}
        }
    }
    assert_eq!(scene_words.unwrap(), rep("a", 2));
    assert_eq!(actor_words.unwrap(), rep("\\s[0]", 3));
}

#[test]
fn zero_weight_is_syntax_error() {
    for (line, value) in [
        ("＠挨拶：a＠０", "a＠０"),
        ("＠挨拶：a＠00", "a＠00"),
        ("＠挨拶：「a」＠０", "「a」＠０"),
    ] {
        let e = err(line);
        assert!(e.contains("test.pasta:1:"), "{e}");
        assert!(e.contains("`挨拶`"), "{e}");
        assert!(e.contains(&format!("`{value}`")), "{e}");
        assert!(e.contains("1 以上"), "{e}");
    }
}

#[test]
fn empty_value_is_syntax_error() {
    for (line, value) in [
        ("＠挨拶：＠３", "＠３"),
        ("＠挨拶：「」＠３", "「」＠３"),
        ("＠挨拶：\"\"@3", "\"\"@3"),
        ("＠挨拶：a、＠３", "＠３"),
    ] {
        let e = err(line);
        assert!(e.contains("`挨拶`"), "{e}");
        assert!(e.contains(&format!("`{value}`")), "{e}");
        assert!(e.contains("空"), "{e}");
        // `＠３` を文字として出す書き方を示す
        assert!(e.contains("「」"), "{e}");
    }
}

#[test]
fn error_names_all_keys() {
    let e = err("＠挨拶、あいさつ：a＠０");
    assert!(e.contains("`挨拶、あいさつ`"), "{e}");
}

/// 並びの数に収まらない重み（usize を超える・並べられない）は構文エラー
#[test]
fn too_large_weight_is_syntax_error() {
    #[cfg(target_pointer_width = "64")]
    let cases = ["18446744073709551616", "18446744073709551615"];
    #[cfg(target_pointer_width = "32")]
    let cases = ["4294967296", "4294967295"];
    for n in cases {
        let e = err(&format!("＠挨拶：a＠{n}"));
        assert!(e.contains("`挨拶`"), "{e}");
        assert!(e.contains(&format!("`a＠{n}`")), "{e}");
        assert!(e.contains("大きすぎ"), "{e}");
    }
}

/// `＠数` が無い行は今と同じ構文木（9.2）
#[test]
fn lines_without_weight_are_unchanged() {
    let kw = global_word("＠w：a、「b、c」、\\s[1]、「」、x＠y、 d  \n");
    assert_eq!(kw.names, vec!["w"]);
    assert_eq!(kw.words, vec!["a", "b、c", "\\s[1]", "", "x＠y", "d  "]);
    assert!(kw.weight_spans.is_empty());
}

#[test]
fn weight_spans_in_written_order() {
    let src = "＠w：a＠３、「b」@12、c\n";
    let kw = global_word(src);
    assert_eq!(
        kw.words,
        vec![
            "a", "a", "a", "b", "b", "b", "b", "b", "b", "b", "b", "b", "b", "b", "b", "c"
        ]
    );
    let texts: Vec<&str> = kw
        .weight_spans
        .iter()
        .map(|s| &src[s.start_byte..s.end_byte])
        .collect();
    assert_eq!(texts, vec!["＠３", "@12"]);
    let first = &kw.weight_spans[0];
    assert_eq!(
        (first.start_line, first.start_col, first.end_col),
        (1, 5, 7)
    );
    for s in &kw.weight_spans {
        assert!(kw.span.start_byte <= s.start_byte && s.end_byte <= kw.span.end_byte);
    }
}
