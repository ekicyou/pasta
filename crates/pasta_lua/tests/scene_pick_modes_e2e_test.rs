//! scene-pick-modes の端から端のテスト（task 7.1。要件 9.4）。
//!
//! 重み・順番・単語の重みを書いた辞書で一時ゴーストを作って `PastaLoader::load_with_config` で読み込み
//! （辞書の読み込み → 辞書確定）、`SHIORI.request` の応答と `@pasta_search` の検索で選ばれ方を確かめる。
//!
//! - 重み 5:2:1 の 1 巡の回数を、イベント・Call・完全一致・選択肢・ローカルシーンの道で（1.1・1.4）
//! - `＠シーン数` の数え方が重みで変わらない（1.5）
//! - 順番の巡回と最初への戻り・定義の順に左右されない・再読み込みと乱数の差し替えで最初から（2.1〜2.4・2.7・2.10）
//! - 条件で外れた番号の飛ばし（2.8）、混ざりの warn とシャッフル（3.1）
//! - 単語の重み 3 つの値の形 × グローバル・ローカル・アクター辞書・前方一致・重ね（5.3・5.5〜5.7）、値 300 個
//! - 各読み込みエラーの文に位置・値が入る（3.2・3.3・4.2・4.4・5.8）
//!
//! 重みの回数は 1 巡（重みの合計の回数）で数えるので、シャッフルの並びに依らない。
//! `PASTA_DEBUG` の中和は `common` の `#[ctor]` に従う。

mod common;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use pasta_lua::loader::LoaderError;
use pasta_lua::mlua::{Function, Table};
use pasta_lua::{PastaLoader, PastaLuaRuntime, RuntimeConfig};
use tracing_test::traced_test;

/// 結合フィクスチャ
const FIXTURE: &str = "\
＃ scene-pick-modes 端から端のフィクスチャ
＠色：赤＠３、「青」＠２、\\s[10]＠４、白
＠色_淡：桃＠２
＠重ね：甲＠２、乙、甲

％さくら
　＠表情：笑＠３、「怒」＠２、\\s[30]＠４

＃ ---- 重み ----
＊OnWeightEvent　＆重み：5
　さくら：E5
＊OnWeightEvent　＆重み：2
　さくら：E2
＊OnWeightEvent
　さくら：E1

＊OnCallWeight
　＞重み話

＊重み話　＆重み：５
　さくら：W5
＊重み話　＆weight：2
　さくら：W2
＊重み話
　さくら：W1

＊完全重み　＆重み：5
　さくら：X5
＊完全重み　＆重み：2.0
　さくら：X2
＊完全重み
　さくら：X1
＊完全重みの外
　さくら：X外

＊選択重み　＆重み：5
　さくら：C5
＊選択重み　＆重み：2
　さくら：C2
＊選択重み
　さくら：C1

＊OnLocalWeight
　＞小話

　・小話＆重み：2
　　さくら：L2
　・小話
　　さくら：L1

＃ ---- 順番 ----
＊OnOrderEvent　＆順番：２
　さくら：OE2
＊OnOrderEvent　＆順番：１
　さくら：OE1

＊OnCallOrder
　＞巡回

＊巡回　＆順番：３０
　さくら：巡C
＊巡回　＆順番：１０
　さくら：巡A
＊巡回　＆index：20
　さくら：巡B

＊OnCondOrder
　＞条件巡

＊条件巡　＆順番：１
　さくら：条1
＊条件巡　＆順番：２　＆＄＊鍵＝１
　さくら：条2
＊条件巡　＆順番：３
　さくら：条3

＊完全巡　＆順番：２
　さくら：完2
＊完全巡　＆順番：１
　さくら：完1
＊完全巡の外　＆順番：０
　さくら：完外

＊選択巡　＆順番：２
　さくら：選2
＊選択巡　＆順番：１
　さくら：選1

＊OnLocalOrder
　＞小巡

　・小巡＆順番：２
　　さくら：LO2
　・小巡＆順番：１
　　さくら：LO1

＃ ---- 混ざり ----
＊OnCallMixed
　＞混ざり

＊混ざり　＆順番：1
　さくら：M1
＊混ざり
　さくら：M2

＃ ---- ローカルの単語 ----
＊単語の場
　＠色：朝＠３、「昼」＠２、\\s[20]＠４
　さくら：＠色
";

/// 一時ゴーストのフォルダを作る（読み込みはしない）。
fn write_ghost(base: &Path, pasta: &str) {
    let pasta_file = base.join("dic/test/scene_pick_modes.pasta");
    std::fs::create_dir_all(pasta_file.parent().unwrap()).unwrap();
    std::fs::write(&pasta_file, pasta).unwrap();
    std::fs::write(base.join("pasta.toml"), "[loader]\ndebug_mode = true\n").unwrap();

    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for sub in ["pasta_scripts", "scriptlibs"] {
        let dst = base.join(sub);
        std::fs::create_dir_all(&dst).unwrap();
        common::copy_dir_recursive(&crate_root.join(sub), &dst).unwrap();
    }
}

fn load(base: &Path) -> PastaLuaRuntime {
    PastaLoader::load_with_config(base, RuntimeConfig::new()).expect("ghost must load")
}

fn load_ghost(base: &Path) -> PastaLuaRuntime {
    write_ghost(base, FIXTURE);
    load(base)
}

/// `SHIORI.request` に GET のリクエスト（SecurityLevel: local）を渡し、200 の Value を返す。
fn request(runtime: &PastaLuaRuntime, id: &str, refs: &[&str]) -> String {
    let lua = runtime.lua();
    let req = lua.create_table().unwrap();
    req.set("id", id).unwrap();
    req.set("method", "get").unwrap();
    req.set("version", 30).unwrap();
    req.set("security_level", "local").unwrap();
    req.set("status", "idle").unwrap();
    let reference = lua.create_table().unwrap();
    for (i, value) in refs.iter().enumerate() {
        reference.set(i, *value).unwrap();
    }
    req.set("reference", reference).unwrap();
    req.set("dic", lua.create_table().unwrap()).unwrap();

    let shiori: Table = lua
        .load(r#"return require("pasta.shiori.entry")"#)
        .eval()
        .unwrap();
    let response = shiori
        .get::<Function>("request")
        .unwrap()
        .call::<String>(req)
        .expect("SHIORI.request must return a response string");
    assert_eq!(
        response.lines().next(),
        Some("SHIORI/3.0 200 OK"),
        "200 expected: {response:?}"
    );
    response
        .lines()
        .find_map(|l| l.strip_prefix("Value: "))
        .unwrap_or_else(|| panic!("Value line expected: {response:?}"))
        .to_string()
}

/// 応答に入っている印（`marks` のどれか 1 つ）を返す。
fn mark_of(value: &str, marks: &[&'static str]) -> &'static str {
    let found: Vec<_> = marks.iter().filter(|m| value.contains(**m)).collect();
    assert_eq!(found.len(), 1, "exactly one of {marks:?} expected: {value}");
    found[0]
}

/// `n` 回選び、印ごとの回数を返す。
fn tally(n: usize, mut pick: impl FnMut() -> String) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for _ in 0..n {
        *counts.entry(pick()).or_insert(0) += 1;
    }
    counts
}

fn counts(pairs: &[(&str, usize)]) -> HashMap<String, usize> {
    pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

/// `@pasta_search` のメソッドを呼び、1 つ目の戻り値（文字列）を返す。
fn search(runtime: &PastaLuaRuntime, method: &str, args: &str) -> String {
    runtime
        .lua()
        .load(format!(
            r#"local v = require("@pasta_search"):{method}({args}); return v"#
        ))
        .eval::<String>()
        .unwrap_or_else(|e| panic!("{method}({args}): {e}"))
}

fn count_scene(runtime: &PastaLuaRuntime, method: &str, name: &str) -> i64 {
    runtime
        .lua()
        .load(format!(
            r#"return require("@pasta_search"):{method}("{name}")"#
        ))
        .eval::<i64>()
        .unwrap()
}

/// 1.1・1.4: 5:2:1 の重みが、1 巡（8 回）でイベント・Call・完全一致・選択肢の道に 5・2・1 回出る。
/// ローカルシーンでも 2:1。重みの書き方は全角・`weight`・整数の小数を混ぜる。
#[test]
fn weight_round_counts_on_every_path() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());

    let marks = ["E5", "E2", "E1"];
    let got = tally(8, || {
        mark_of(&request(&runtime, "OnWeightEvent", &[]), &marks).into()
    });
    assert_eq!(got, counts(&[("E5", 5), ("E2", 2), ("E1", 1)]), "event");

    let marks = ["W5", "W2", "W1"];
    let got = tally(8, || {
        mark_of(&request(&runtime, "OnCallWeight", &[]), &marks).into()
    });
    assert_eq!(got, counts(&[("W5", 5), ("W2", 2), ("W1", 1)]), "call");

    let marks = ["C5", "C2", "C1"];
    let got = tally(8, || {
        mark_of(
            &request(&runtime, "OnChoiceSelectEx", &["選ぶ", "選択重み"]),
            &marks,
        )
        .into()
    });
    assert_eq!(got, counts(&[("C5", 5), ("C2", 2), ("C1", 1)]), "choice");

    // 完全一致の検索は `完全重みの外` を集めず、重みだけが効く
    let got = tally(8, || {
        search(&runtime, "search_scene_exact", r#""完全重み""#)
    });
    assert_eq!(
        got,
        counts(&[("完全重み_1", 5), ("完全重み_2", 2), ("完全重み_3", 1)]),
        "exact"
    );

    let marks = ["L2", "L1"];
    let got = tally(3, || {
        mark_of(&request(&runtime, "OnLocalWeight", &[]), &marks).into()
    });
    assert_eq!(got, counts(&[("L2", 2), ("L1", 1)]), "local");
}

/// 1.5: `＠シーン数` の数え方（count_scene・count_scene_exact）は重みで変わらない。
#[test]
fn scene_count_ignores_weight() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());

    assert_eq!(count_scene(&runtime, "count_scene", "重み話"), 3);
    assert_eq!(count_scene(&runtime, "count_scene_exact", "重み話"), 3);
    // 前方一致は `完全重みの外` も 1 と数える
    assert_eq!(count_scene(&runtime, "count_scene", "完全重み"), 4);
    assert_eq!(count_scene(&runtime, "count_scene_exact", "完全重み"), 3);
}

/// 2.1〜2.3・2.7・2.10: 番号の順に巡り、最後の次は最初へ戻る。定義の順（30・10・20）に左右されない。
/// イベント・Call・完全一致・選択肢・ローカルシーンの道で同じ。
#[test]
fn order_cycles_on_every_path() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());

    let marks = ["OE1", "OE2"];
    let seq: Vec<_> = (0..3)
        .map(|_| mark_of(&request(&runtime, "OnOrderEvent", &[]), &marks))
        .collect();
    assert_eq!(seq, ["OE1", "OE2", "OE1"], "event");

    let marks = ["巡A", "巡B", "巡C"];
    let seq: Vec<_> = (0..4)
        .map(|_| mark_of(&request(&runtime, "OnCallOrder", &[]), &marks))
        .collect();
    assert_eq!(seq, ["巡A", "巡B", "巡C", "巡A"], "call");

    let marks = ["選1", "選2"];
    let seq: Vec<_> = (0..3)
        .map(|_| {
            mark_of(
                &request(&runtime, "OnChoiceSelectEx", &["選ぶ", "選択巡"]),
                &marks,
            )
        })
        .collect();
    assert_eq!(seq, ["選1", "選2", "選1"], "choice");

    // 完全一致は番号 0 の `完全巡の外` を集めない
    let seq: Vec<_> = (0..3)
        .map(|_| search(&runtime, "search_scene_exact", r#""完全巡""#))
        .collect();
    assert_eq!(seq, ["完全巡_2", "完全巡_1", "完全巡_2"], "exact");

    let marks = ["LO1", "LO2"];
    let seq: Vec<_> = (0..3)
        .map(|_| mark_of(&request(&runtime, "OnLocalOrder", &[]), &marks))
        .collect();
    assert_eq!(seq, ["LO1", "LO2", "LO1"], "local");
}

/// 2.4: 辞書の再読み込み・乱数の差し替えで、覚えた番号が消えて最初からに戻る。
#[test]
fn order_restarts_after_reload_and_selector_reset() {
    let temp = tempfile::TempDir::new().unwrap();
    let marks = ["巡A", "巡B", "巡C"];
    let runtime = load_ghost(temp.path());
    let pick = |rt: &PastaLuaRuntime| mark_of(&request(rt, "OnCallOrder", &[]), &marks);
    assert_eq!(pick(&runtime), "巡A");
    assert_eq!(pick(&runtime), "巡B");
    drop(runtime);

    // 同じゴーストを読み込み直す
    let runtime = load(temp.path());
    assert_eq!(pick(&runtime), "巡A", "reload restarts");
    assert_eq!(pick(&runtime), "巡B");

    runtime
        .exec(r#"require("@pasta_search"):set_scene_selector()"#)
        .unwrap();
    assert_eq!(pick(&runtime), "巡A", "selector reset restarts");
}

/// 2.8: 条件で外れた番号は飛ばされ、満たせば次の巡で選ばれる。
#[test]
fn order_skips_unmet_number() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());
    let marks = ["条1", "条2", "条3"];
    let pick = || mark_of(&request(&runtime, "OnCondOrder", &[]), &marks);

    assert_eq!(pick(), "条1");
    assert_eq!(pick(), "条3", "2 is unmet and skipped");
    assert_eq!(pick(), "条1", "back to the first");

    runtime.exec(r#"require("pasta.save")["鍵"] = 1"#).unwrap();
    assert_eq!(pick(), "条2", "2 is met now");
    assert_eq!(pick(), "条3");
}

/// 3.1: 順番の有る・無いが混ざると、探した名前と理由の warn を出してシャッフル（乱数の差し替えの並び）で選ぶ。
#[traced_test]
#[test]
fn mixed_order_warns_and_shuffles() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());
    // シャッフルの並びは 2 つ目を先に（番号の順なら M1）
    runtime
        .exec(r#"require("@pasta_search"):set_scene_selector(1, 0)"#)
        .unwrap();

    let value = request(&runtime, "OnCallMixed", &[]);
    assert_eq!(mark_of(&value, &["M1", "M2"]), "M2", "shuffled pick");
    assert!(logs_contain("pasta_lua::search"));
    assert!(logs_contain("name=混ざり"));
    assert!(logs_contain("順番の有るものと無いものが混ざっている"));
}

/// 5.3・5.5〜5.7: 単語の重みが 3 つの値の形 × グローバル・ローカル・アクター辞書で効く。
/// 前方一致で集めた値（`色_淡`）も重みの回数だけ乗り、同じ値の重ねは足し合わせる。
#[test]
fn word_weight_on_every_dictionary() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());

    // グローバル: `色` は前方一致で `色_淡` も集める（3+2+4+1+2 = 12）
    let got = tally(12, || search(&runtime, "search_word", r#""色""#));
    assert_eq!(
        got,
        counts(&[("赤", 3), ("青", 2), ("\\s[10]", 4), ("白", 1), ("桃", 2)]),
        "global + prefix"
    );

    let got = tally(4, || search(&runtime, "search_word", r#""重ね""#));
    assert_eq!(got, counts(&[("甲", 3), ("乙", 1)]), "same value adds up");

    let got = tally(9, || {
        search(&runtime, "search_word", r#""色", "単語の場_1""#)
    });
    assert_eq!(
        got,
        counts(&[("朝", 3), ("昼", 2), ("\\s[20]", 4)]),
        "local"
    );

    let got = tally(9, || {
        search(&runtime, "search_word", r#""表情", "__actor_さくら__""#)
    });
    assert_eq!(
        got,
        counts(&[("笑", 3), ("怒", 2), ("\\s[30]", 4)]),
        "actor"
    );
}

/// 値 300 個の単語が読み込め、1 巡で 300 個がそろう。
#[test]
fn word_with_300_values_loads() {
    let values: Vec<String> = (1..=300).map(|i| format!("値{i}")).collect();
    let temp = tempfile::TempDir::new().unwrap();
    write_ghost(
        temp.path(),
        &format!(
            "＠多い：{}\n\n＊OnBoot\n　さくら：起動\n",
            values.join("、")
        ),
    );
    let runtime = load(temp.path());

    let got = tally(300, || search(&runtime, "search_word", r#""多い""#));
    assert_eq!(got.len(), 300);
    assert!(got.values().all(|n| *n == 1), "{got:?}");
    assert!(got.contains_key("値300"));
}

/// 辞書を読み込み、失敗の文を返す（通れば panic）。
fn load_error_of(pasta: &str) -> String {
    let temp = tempfile::TempDir::new().unwrap();
    write_ghost(temp.path(), pasta);
    match PastaLoader::load_with_config(temp.path(), RuntimeConfig::new()) {
        Ok(_) => panic!("must fail to load: {pasta}"),
        Err(LoaderError::PartialTranspileError { failures, .. }) => {
            assert_eq!(failures.len(), 1, "{failures:?}");
            failures[0].error.clone()
        }
        Err(other) => other.to_string(),
    }
}

/// 3.2・3.3・4.2・4.4・5.8: 各読み込みエラーで読み込みが止まり、文に位置・値が入る。
#[test]
fn load_errors_stop_with_position_and_value() {
    for (pasta, expected) in [
        (
            "＊挨拶\n　＆重み：0\n　さくら：あ\n",
            &["`＆重み` の値は 1 以上の整数で書く: `0`", "[L2:"][..],
        ),
        (
            "＊挨拶\n　＆順番：最初\n　さくら：あ\n",
            &["`＆順番` の値は整数で書く: `最初`", "[L2:"],
        ),
        (
            "＊挨拶\n　＆重み：2\n　＆順番：1\n　さくら：あ\n",
            &["`＆重み` と `＆順番` は同じシーンに書けない", "[L3:"],
        ),
        (
            "＊挨拶\n　＆順番：１０\n　さくら：あ\n＊挨拶\n　＆順番：１０\n　さくら：い\n",
            &[
                "`＆順番` の番号 10 が同じ名前のシーン `挨拶` に 2 つ以上ある",
                "挨拶_1::__start__",
                "挨拶_2::__start__",
            ],
        ),
        (
            "＠挨拶：やあ＠０\n",
            &[
                "単語 `挨拶` の値 `やあ＠０` の重みは 1 以上の整数で書きます",
                "scene_pick_modes.pasta at 1:5",
            ],
        ),
        (
            "＠挨拶：＠３\n",
            &[
                "単語 `挨拶` の値 `＠３` は重みの `＠数` を除くと空になります",
                "scene_pick_modes.pasta at 1:5",
            ],
        ),
    ] {
        let error = load_error_of(pasta);
        for e in expected {
            assert!(error.contains(e), "{pasta:?} -> {error}\n  missing: {e}");
        }
    }
}
