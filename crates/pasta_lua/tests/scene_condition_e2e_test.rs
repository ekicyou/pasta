//! scene-condition-attributes の端から端のテスト（task 5.1）。
//!
//! 条件・優先を書いた辞書で一時ゴーストを作って `PastaLoader::load_with_config` で読み込み
//! （辞書の読み込み → 辞書確定）、`SHIORI.request` に GET のリクエストを渡して、
//! `@pasta_search` の絞り込みから Lua の Call・イベント・選択肢の応答までを確かめる（10.3・10.4）。
//!
//! - `＊会話` → `＞雑談_` の経路（2.3）、`＄気分＝「高」` の後の `＞告白`（2.4）
//! - Call の条件漏れは止まる表記・名前が無いときは今の表記・イベントは 204（3.1・3.3・3.4）、
//!   時報の名前の通り抜け（2.5・3.6）
//! - 並び: 外れた候補も見て使い、巡の始め直しの後に出る（4.3〜4.6）、優先の独占（5.1）
//! - `＊OnUpdateComplete　＆優先：1　＆＄ｒ０＝「changed」` が転記の行なしで選ばれ（6.1）、
//!   チェイントークの続きの変数は上書きされない（6.5）
//! - 選択肢の飛び先の条件漏れ（2.6・3.2）と `OnPastaCallFunction` の条件漏れ（2.8）
//!
//! 並びの順は `set_scene_selector(0, 1, 2)`（シャッフルしない固定の並び）で決める。
//! `PASTA_DEBUG` の中和は `common` の `#[ctor]` に従う。

mod common;

use std::path::{Path, PathBuf};

use pasta_lua::mlua::{Function, Table};
use pasta_lua::{PastaLoader, PastaLuaRuntime, RuntimeConfig};

/// 結合フィクスチャ
const FIXTURE: &str = "\
＃ scene-condition-attributes 端から端のフィクスチャ
＊会話
　＞雑談_

＊雑談_朝　＆＄＊時間帯＝「朝」
　さくら：朝の雑談

＊雑談_夜　＆＄＊時間帯＝「夜」
　さくら：夜の雑談

＊OnTestMood
　＄気分＝「高」
　＞告白

＊告白　＆＄気分＝「高」
　さくら：大好き

＊告白　＆＄気分＝「低」
　さくら：ふつう

＊OnTestCallUnmet
　さくら：前の行
　＞秘密
　さくら：後の行

＊OnTestCallMissing
　さくら：前の行
　＞無い名前
　さくら：後の行

＊秘密　＆＄＊鍵＝１
　さくら：秘密の話

＊OnTestEventUnmet　＆＄＊鍵＝１
　さくら：鍵のあるときだけ

＊時報15　＆＄＊鍵＝１
　さくら：鍵の時報

＊OnHour15
　さくら：ふつうの時報

＊OnTestList
　＞並び

＊並び　＆＄＊段＝１
　さくら：並びA

＊並び
　さくら：並びB

＊並び
　さくら：並びC

＊OnTestPriority
　＞独占

＊独占　＆優先：1
　さくら：独占X

＊独占
　さくら：独占Y

＊OnUpdateComplete　＆優先：1　＆＄ｒ０＝「changed」
　さくら：更新した＄ｒ０
　＞チェイントーク
　さくら：続きは＄ｒ０

＊OnUpdateComplete
　さくら：更新なし
";

/// 一時ゴーストを作って読み込み、並びを固定し、続きを流すハンドラを置く。
fn load_ghost(base: &Path) -> PastaLuaRuntime {
    let pasta_file = base.join("dic/test/scene_condition.pasta");
    std::fs::create_dir_all(pasta_file.parent().unwrap()).unwrap();
    std::fs::write(&pasta_file, FIXTURE).unwrap();
    std::fs::write(base.join("pasta.toml"), "[loader]\ndebug_mode = true\n").unwrap();

    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for sub in ["pasta_scripts", "scriptlibs"] {
        let dst = base.join(sub);
        std::fs::create_dir_all(&dst).unwrap();
        common::copy_dir_recursive(&crate_root.join(sub), &dst).unwrap();
    }

    let runtime =
        PastaLoader::load_with_config(base, RuntimeConfig::new()).expect("ghost must load");
    // 並びはシャッフルせず登録順。続きは OnTalk の時機と同じく STORE.co_scene を返して流す
    runtime
        .exec(
            r#"
            require("@pasta_search"):set_scene_selector(0, 1, 2)
            local REG = require("pasta.shiori.event.register")
            REG.OnTestContinue = function() return require("pasta.store").co_scene end
            "#,
        )
        .unwrap();
    runtime
}

/// `SHIORI.request` に GET のリクエスト（SecurityLevel: local）を渡し、応答の文字列を返す。
/// `date` を渡すと、その Unix 時刻と時（X-Pasta-Time 付きで、時計の飛びとして数えない）を載せる。
fn request_at(
    runtime: &PastaLuaRuntime,
    id: &str,
    refs: &[&str],
    date: Option<(i64, i64)>,
) -> String {
    let lua = runtime.lua();
    let req = lua.create_table().unwrap();
    req.set("id", id).unwrap();
    req.set("method", "get").unwrap();
    req.set("version", 30).unwrap();
    req.set("security_level", "local").unwrap();
    req.set("status", "idle").unwrap();
    let dic = lua.create_table().unwrap();
    let reference = lua.create_table().unwrap();
    for (i, value) in refs.iter().enumerate() {
        reference.set(i, *value).unwrap();
    }
    req.set("reference", reference).unwrap();
    if let Some((unix, hour)) = date {
        dic.set("X-Pasta-Time", "1").unwrap();
        let d = lua.create_table().unwrap();
        for (k, v) in [
            ("unix", unix),
            ("year", 2023),
            ("month", 12),
            ("day", 15),
            ("hour", hour),
            ("min", 0),
            ("sec", 0),
            ("ns", 0),
            ("yday", 349),
            ("ordinal", 349),
            ("wday", 5),
            ("num_days_from_sunday", 5),
        ] {
            d.set(k, v).unwrap();
        }
        req.set("date", d).unwrap();
    }
    req.set("dic", dic).unwrap();

    let shiori: Table = lua
        .load(r#"return require("pasta.shiori.entry")"#)
        .eval()
        .unwrap();
    shiori
        .get::<Function>("request")
        .unwrap()
        .call::<String>(req)
        .expect("SHIORI.request must return a response string")
}

fn request(runtime: &PastaLuaRuntime, id: &str, refs: &[&str]) -> String {
    request_at(runtime, id, refs, None)
}

/// 200 OK であることを確かめ、Value の行の値を返す。
fn value_of_200(response: &str) -> String {
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

fn assert_204(response: &str) {
    assert_eq!(
        response.lines().next(),
        Some("SHIORI/3.0 204 No Content"),
        "204 expected: {response:?}"
    );
}

/// 保存変数に値を入れる（`＄＊name＝value` と同じ）。
fn set_save(runtime: &PastaLuaRuntime, name: &str, lua_value: &str) {
    runtime
        .exec(&format!(r#"require("pasta.save")["{name}"] = {lua_value}"#))
        .unwrap();
}

/// 2.3: OnTalk の別名の `＊会話` から `＞雑談_` が前方一致で集めた候補を、保存変数の条件で絞る。
#[test]
fn kaiwa_calls_zatsudan_prefix_filtered_by_condition() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());

    set_save(&runtime, "時間帯", r#""夜""#);
    for _ in 0..4 {
        let value = value_of_200(&request(&runtime, "OnTalk", &[]));
        assert!(value.contains("夜の雑談"), "night talk: {value}");
        assert!(
            !value.contains("朝の雑談"),
            "morning talk filtered: {value}"
        );
    }

    set_save(&runtime, "時間帯", r#""朝""#);
    let value = value_of_200(&request(&runtime, "OnTalk", &[]));
    assert!(value.contains("朝の雑談"), "morning talk: {value}");
}

/// 2.4: 呼び出し元が `＄気分＝「高」` を入れてから `＞告白` すると、同じローカル変数で絞る。
#[test]
fn caller_local_variable_selects_callee() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());

    for _ in 0..3 {
        let value = value_of_200(&request(&runtime, "OnTestMood", &[]));
        assert!(value.contains("大好き"), "high mood scene: {value}");
        assert!(!value.contains("ふつう"), "low mood filtered: {value}");
    }
}

/// 3.1・3.4: Call の条件漏れは止まる表記で後の行を出さず、名前が無いときは今の表記で次の行へ進む。
#[test]
fn call_unmet_stops_and_not_found_keeps_notation() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());

    let response = request(&runtime, "OnTestCallUnmet", &[]);
    let value = value_of_200(&response);
    assert!(
        value.contains("【条件に合うシーンが無い：＞秘密（＊OnTestCallUnmet）】"),
        "unmet stop notice: {value}"
    );
    assert!(!value.contains("後の行"), "must stop at the call: {value}");
    assert!(
        !value.contains("シーンが見つからない"),
        "unmet differs from not found: {value}"
    );

    let value = value_of_200(&request(&runtime, "OnTestCallMissing", &[]));
    assert!(
        value.contains("【シーンが見つからない：＞無い名前】"),
        "not found notice: {value}"
    );
    assert!(value.contains("後の行"), "not found continues: {value}");
    assert!(
        !value.contains("条件に合うシーンが無い"),
        "not found differs from unmet: {value}"
    );

    // 条件を満たせば同じ Call が通る
    set_save(&runtime, "鍵", "1");
    let value = value_of_200(&request(&runtime, "OnTestCallUnmet", &[]));
    assert!(
        value.contains("秘密の話") && value.contains("後の行"),
        "met call: {value}"
    );
}

/// 2.5・3.3: イベントのシーンが条件漏れなら 204（失敗表記を出さない）。満たせば選ばれる。
#[test]
fn event_unmet_returns_204() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());

    assert_204(&request(&runtime, "OnTestEventUnmet", &[]));

    set_save(&runtime, "鍵", "1");
    let value = value_of_200(&request(&runtime, "OnTestEventUnmet", &[]));
    assert!(value.contains("鍵のあるときだけ"), "met event: {value}");
}

/// 3.6: 時報は `時報15` が条件漏れなら次の名前 `OnHour15` を試す。満たせば `時報15`。
#[test]
fn hour_signal_falls_through_unmet_name() {
    // 2023-12-15 14:00 の正時の Unix 時刻と、その 1 時間後
    const T14: i64 = 1_702_648_800;
    const T15: i64 = T14 + 3600;
    const T16: i64 = T15 + 3600;

    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());

    // 1 回目で次の正時を決めるだけ（発行しない）
    request_at(&runtime, "OnSecondChange", &[], Some((T14, 14)));
    let value = value_of_200(&request_at(
        &runtime,
        "OnSecondChange",
        &[],
        Some((T15, 15)),
    ));
    assert!(value.contains("ふつうの時報"), "fell through: {value}");
    assert!(
        !value.contains("条件に合うシーンが無い"),
        "no notice: {value}"
    );

    set_save(&runtime, "鍵", "1");
    let value = value_of_200(&request_at(
        &runtime,
        "OnSecondChange",
        &[],
        Some((T16, 15)),
    ));
    assert!(value.contains("鍵の時報"), "met hour scene: {value}");
}

/// 4.3〜4.6: 外れた候補も見て使い（その巡では満たしても出ない）、巡の始め直しの後に出る。
#[test]
fn unmet_seen_candidate_is_consumed_and_returns_next_round() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());

    let pick = || value_of_200(&request(&runtime, "OnTestList", &[]));

    // 並び A（段＝1）・B・C。A は外れて使ったものになり、B が出る
    assert!(pick().contains("並びB"));
    // ここで A が条件を満たしても、今の巡では A をもう見ない
    set_save(&runtime, "段", "1");
    assert!(pick().contains("並びC"));
    // 巡を全部見終えたので新しい巡で A から見る
    let value = pick();
    assert!(value.contains("並びA"), "A after round restart: {value}");

    // 新しい巡の残りは並びの順に B
    let value = pick();
    assert!(value.contains("並びB"), "next in the round: {value}");
}

/// 5.1・5.7: 優先の高い候補が条件を満たす間は、巡を使い切っても低い段へ下りない。
#[test]
fn priority_monopolizes_selection() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());

    for _ in 0..4 {
        let value = value_of_200(&request(&runtime, "OnTestPriority", &[]));
        assert!(value.contains("独占X"), "priority scene only: {value}");
        assert!(!value.contains("独占Y"), "low tier never: {value}");
    }
}

/// 6.1・6.5: 転記の行なしで `＄ｒ０` の条件が効いて選ばれ、続きの変数は次のイベントで上書きされない。
#[test]
fn event_scene_condition_on_reference_with_auto_transfer() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());

    let value = value_of_200(&request(&runtime, "OnUpdateComplete", &["none"]));
    assert!(value.contains("更新なし"), "fallback scene: {value}");

    let value = value_of_200(&request(&runtime, "OnUpdateComplete", &["changed"]));
    assert!(
        value.contains("更新したchanged"),
        "conditioned scene: {value}"
    );
    assert!(!value.contains("続きは"), "beat 2 must wait: {value}");

    let value = value_of_200(&request(&runtime, "OnTestContinue", &["other"]));
    assert!(
        value.contains("続きはchanged"),
        "continuation keeps its own var: {value}"
    );

    // ＆優先：1 なので、条件を満たす間は巡を使い切っても毎回この場面
    let value = value_of_200(&request(&runtime, "OnUpdateComplete", &["changed"]));
    assert!(value.contains("更新したchanged"), "priority again: {value}");
}

/// 2.6・3.2: 選択肢の飛び先が条件漏れなら止まる表記。満たせば飛び先へ進む。
#[test]
fn choice_target_unmet_stops() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());

    let value = value_of_200(&request(
        &runtime,
        "OnChoiceSelectEx",
        &["秘密を聞く", "秘密"],
    ));
    assert!(
        value.contains("【条件に合うシーンが無い：OnChoiceSelectEx ＞秘密"),
        "choice unmet notice: {value}"
    );

    set_save(&runtime, "鍵", "1");
    let value = value_of_200(&request(
        &runtime,
        "OnChoiceSelectEx",
        &["秘密を聞く", "秘密"],
    ));
    assert!(value.contains("秘密の話"), "choice met: {value}");
}

/// 2.8: `OnPastaCallFunction` の条件漏れは止めずに条件漏れの表記の応答。満たせば呼ぶ。
#[test]
fn call_function_unmet_replies_notice() {
    let temp = tempfile::TempDir::new().unwrap();
    let runtime = load_ghost(temp.path());

    let value = value_of_200(&request(&runtime, "OnPastaCallFunction", &["秘密"]));
    assert!(
        value.contains("【条件に合うシーンが無い：OnPastaCallFunction ＞秘密】"),
        "call function unmet notice: {value}"
    );

    set_save(&runtime, "鍵", "1");
    let value = value_of_200(&request(&runtime, "OnPastaCallFunction", &["秘密"]));
    assert!(value.contains("秘密の話"), "call function met: {value}");
}
