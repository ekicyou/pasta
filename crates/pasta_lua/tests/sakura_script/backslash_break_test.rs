//! vertical-balloon-lines 1.2（決定 9）: 単位にならない `\` の直後に pasta が入れる改行が、
//! エスケープの `\\` に化けて改行として働かなくなる形が、DSL から届くことを示すテスト。
//!
//! アクション行に単独の `\` は書けない（パースエラー）が、単語の値・関数の返り値は単独の `\` を
//! 台詞に運べる。その直後に budoux の改行（2.3 で守る）や、組み立ての区切り（4.1 で守る）が続くと、
//! `\` と区切りの頭の `\` がつながって `\\` になる。守り方は各タスクが決めるので、ここでは
//! 「区切りの直前の `\` の並びが奇数個（＝区切りの `\` がエスケープに食われていない）」だけを見る。
//! 結論と根拠は research.md の「単独の `\` と pasta が入れる改行（1.2・決定 9）」に書いた。

use crate::common;

use common::{create_temp_with_pasta, value_as_str};
use pasta_lua::PastaLuaRuntime;
use pasta_lua::loader::PastaLoader;
use tempfile::TempDir;

/// 台詞の幅を 4（全角 2 字）にしたアクター。
const TOML: &str = "\n[actor.\"さくら\"]\nspot = 0\nbudoux = [4]\n";

const DIC: &str = r#"＠記号：天気天気\

＊単語の後の台詞
　さくら：＠記号
　さくら：今日は

＊関数の後の台詞
```lua
function SCENE.返す(act)
    return "天気天気\\"
end
```
　さくら：＠返す（）今日は

＊単語の後の選択肢
　さくら：＠記号
　＠？次

　・次
　　さくら：つぎ
"#;

fn load_ghost() -> (TempDir, PastaLuaRuntime) {
    let temp = create_temp_with_pasta(DIC);
    let toml = temp.path().join("pasta.toml");
    let base = std::fs::read_to_string(&toml).unwrap();
    std::fs::write(&toml, base + TOML).unwrap();
    let runtime =
        PastaLoader::load(temp.path()).unwrap_or_else(|e| panic!("ゴーストの読み込みに失敗: {e}"));
    (temp, runtime)
}

fn fire(runtime: &PastaLuaRuntime, id: &str) -> String {
    let script = format!(
        r#"local EVENT = require "pasta.shiori.event"
return EVENT.fire({{ id = "{id}", method = "get", version = 30 }})"#
    );
    let value = runtime.exec(&script).expect("EVENT.fire に失敗");
    value_as_str(&value).expect("応答が文字列でない")
}

/// `brk`（`\` を除いた区切り。`n`・`n[150]`）の直前の `\` の並びの長さを、出てきた所ごとに返す。
fn backslash_runs_before(out: &str, brk: &str) -> Vec<usize> {
    out.match_indices(brk)
        .map(|(i, _)| out[..i].bytes().rev().take_while(|&b| b == b'\\').count())
        .filter(|&n| n > 0)
        .collect()
}

/// 区切りが 1 つ以上あり、どれも直前の `\` が奇数個（区切りの `\` が生きている）。
fn assert_breaks_alive(out: &str, brk: &str) {
    let runs = backslash_runs_before(out, brk);
    assert!(!runs.is_empty(), "区切り {brk:?} が無い: {out:?}");
    assert!(
        runs.iter().all(|n| n % 2 == 1),
        "区切り {brk:?} の `\\` がエスケープの `\\\\` に化けている（直前の `\\` の数 {runs:?}）: {out:?}"
    );
}

/// budoux の改行が、単語・関数の返り値の末尾の単独の `\` の直後に入っても、改行として働く（2.3 で守った）。
#[test]
fn budoux_break_after_lone_backslash_stays_a_break() {
    let (_temp, runtime) = load_ghost();
    // どちらの経路も確かめるため、片方で止めずに両方の出力を見てから判定する
    let outs: Vec<String> = ["単語の後の台詞", "関数の後の台詞"]
        .iter()
        .map(|id| fire(&runtime, id))
        .collect();
    for out in &outs {
        assert_breaks_alive(out, "n");
    }
}

/// 字の後の選択肢の前の区切り `\n[150]` が、単語の末尾の単独の `\` の直後に入る（今も起きる）。
#[test]
#[ignore = "1.2 で DSL から届くと確かめた形。4.1（組み立ての区切りの層）で守って ignore を外す"]
fn choice_break_after_lone_backslash_stays_a_break() {
    let (_temp, runtime) = load_ghost();
    assert_breaks_alive(&fire(&runtime, "単語の後の選択肢"), "n[150]");
}
