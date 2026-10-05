//! Call の実行の正しさ（文脈の復元・選択肢のジャンプ先・動的コールの失敗）の SHIORI 経由 E2E テスト
//!
//! 仕様 `call-execution-correctness`（requirements 8.1–8.3）。
//!
//! フィクスチャ `tests/fixtures/call_execution_correctness/` の辞書を読み込み、シーンを SHIORI
//! リクエストのイベント ID（シーン関数フォールバック）で起動して、応答のさくらスクリプトと
//! ゴーストのログの警告を確かめる。フィクスチャは埋め込みの標準ランタイムと本番 entry.lua を通す
//! （理由は pasta.toml のコメント）。シーンを足すときは `dic/` に `.pasta` を置くだけでよい。

mod common;

use common::copy_fixture_to_temp;
use common::response::ShioriResponse;
use pasta::{PastaShiori, Shiori};

/// 1 つのゴーストに、順にリクエストを送り、各応答の Value とゴーストのログファイルの中身を返す。
///
/// `requests` の各要素は `GET SHIORI/3.0` 行・`Charset`・`Sender` に続くヘッダ行（`\r\n` 区切り）。
/// 例: `"ID: OnFoo"`、`"ID: OnChoiceSelectEx\r\nReference0: …"`。各応答が 200 か 204 であることを
/// 確かめる（204 は Value が無いので空文字列を返す。話すことが無い OnSecondChange が 204 になる）。
/// ロガーは non_blocking で書くため、ゴーストを drop して書き出しを待ってからログを読む。
fn run(requests: &[&str]) -> (Vec<String>, String) {
    run_with_extra_dic(None, requests)
}

/// `run` に、読み込み前のフィクスチャへ足す辞書（`dic/extra.pasta` の中身）を指定できるようにしたもの。
/// 共有のフィクスチャに置くとほかのテストの挙動を変えてしまうシーン（`＊OnChoiceSelectEx` など）に使う。
fn run_with_extra_dic(extra_dic: Option<&str>, requests: &[&str]) -> (Vec<String>, String) {
    let temp = copy_fixture_to_temp("call_execution_correctness");
    if let Some(extra) = extra_dic {
        std::fs::write(temp.path().join("dic/extra.pasta"), extra).expect("write extra dic");
    }
    let values = {
        let mut shiori = PastaShiori::default();
        assert!(
            shiori
                .load(0, temp.path().as_os_str())
                .expect("SHIORI load should not error"),
            "SHIORI load should return true"
        );
        requests
            .iter()
            .map(|headers| {
                let raw = shiori
                    .request(format!(
                        "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\n{headers}\r\n\r\n"
                    ))
                    .expect("request should succeed");
                let resp = ShioriResponse::parse(&raw).expect("response should parse");
                if resp.status_code == 204 {
                    return String::new();
                }
                assert_eq!(
                    resp.status_code, 200,
                    "{headers} must not be 500: {} {:?}",
                    resp.status_text, resp.value
                );
                resp.value.expect("Value header must exist")
            })
            .collect()
    };
    let log = std::fs::read_to_string(temp.path().join("profile/pasta/logs/pasta.log"))
        .unwrap_or_default();
    (values, log)
}

/// イベント ID 1 つを送り、応答の Value とログを返す
fn fire(event_id: &str) -> (String, String) {
    let (mut values, log) = run(&[&format!("ID: {event_id}")]);
    (values.remove(0), log)
}

/// Lua 側（`@pasta_log`）が出した警告の行
fn lua_warnings(log: &str) -> Vec<&str> {
    log.lines()
        .filter(|l| l.contains(" WARN pasta_lua::runtime::log:"))
        .collect()
}

/// 土台: 最小の辞書（1 シーン 1 行）が SHIORI 経由で話し、Lua 側の警告は出ない
#[test]
fn test_minimal_scene_talks() {
    let (value, log) = fire("OnMinimal");
    assert_eq!(value, r"\p[0]こんにちは\e");
    assert!(
        log.contains("SHIORI.load called successfully"),
        "log must be captured:\n{log}"
    );
    assert!(lua_warnings(&log).is_empty(), "no warning expected:\n{log}");
}

// ---------------------------------------------------------------------------
// 特性化テスト（tasks 1.2）: 変更前のコードで成功し、以後の変更で落ちないことの基準
// 辞書: dic/characterization.pasta
// ---------------------------------------------------------------------------

/// 4.1: 末尾の Call（動的コール `＞＠次（）`、Lua のカウンタ）だけで 10 万回つないでも
/// 深さの上限によるエラーにならず、最後のシーンまで実行される
#[test]
fn test_tail_call_chain_100k_completes() {
    let (value, log) = fire("OnTailChain");
    assert_eq!(value, r"\p[0]完走\e");
    assert!(lua_warnings(&log).is_empty(), "no warning expected:\n{log}");
}

/// 4.4: 同じグローバルシーンの配下のローカルシーンを Call したときの応答（途中の Call と
/// 末尾の Call）。前方一致するグローバルシーンよりローカルシーンが選ばれる
#[test]
fn test_call_local_scene_in_same_global() {
    let (value, log) = fire("OnSameGlobalCall");
    assert_eq!(value, r"\p[0]［ローカル］締め\e");
    assert!(lua_warnings(&log).is_empty(), "no warning expected:\n{log}");
}

/// 4.2: 末尾の Call で別のグローバルシーンへ移った後は、移った先のシーンの文脈で解決される
/// （単語参照は移った先のローカル単語、Call は移った先の配下のローカルシーン）
#[test]
fn test_tail_call_to_other_global_resolves_in_callee_context() {
    let (value, log) = fire("OnTailToOther");
    assert_eq!(value, r"\p[0]元先の単語先の後半\e");
    assert!(lua_warnings(&log).is_empty(), "no warning expected:\n{log}");
}

/// 2.6: 別のグローバルシーンへの途中の Call を含まないシーンが出した選択肢を、既定の自動
/// ルーティングで選んだときのジャンプ先。出したシーンの配下のローカルシーンが先に選ばれ、
/// 無ければグローバルシーンへフォールバックする。`\q` の文字列そのものは確かめない（2.4 で変わる）
#[test]
fn test_choice_routing_without_mid_call() {
    let select = |title: &str, id: &str| {
        format!("ID: OnChoiceSelectEx\r\nReference0: {title}\r\nReference1: {id}")
    };
    for (title, id, expected) in [
        ("甲", "選択肢甲", r"\p[0]メニューの甲\e"),
        ("乙", "選択肢乙", r"\p[0]グローバルの乙\e"),
    ] {
        let (values, log) = run(&["ID: OnChoiceMenu", &select(title, id)]);
        assert!(values[0].contains("どれ？"), "menu: {}", values[0]);
        assert_eq!(values[1], expected, "choice {id}");
        assert!(lua_warnings(&log).is_empty(), "no warning expected:\n{log}");
    }
}

/// 4.8: ローカルシーンの最後の行が Call のとき、呼ばれた側の戻り値がそのシーンの戻り値になる。
/// Lua の関数が `act:call` でローカルシーンを呼び、その戻り値をアクション行で出力する
#[test]
fn test_tail_call_return_value_propagates() {
    let (value, log) = fire("OnTailReturn");
    assert_eq!(value, r"\p[0]末尾の戻り値\e");
    assert!(lua_warnings(&log).is_empty(), "no warning expected:\n{log}");
}

// ---------------------------------------------------------------------------
// 途中の Call と式の関数呼び出しから戻った後の文脈（tasks 4.1）
// 辞書: dic/mid_call_context.pasta。呼び出し元と呼ばれた側に同名のローカル単語・ローカルシーン・
// 関数を置き、出力でどちらの文脈で解決されたかを見分ける。
// ---------------------------------------------------------------------------

/// イベント ID 1 つを送り、応答が `expected` で Lua 側の警告が出ないことを確かめる
fn assert_fires(event_id: &str, expected: &str) {
    let (value, log) = fire(event_id);
    assert_eq!(value, expected, "{event_id}");
    assert!(lua_warnings(&log).is_empty(), "no warning expected:\n{log}");
}

/// 1.11（U28）・1.1–1.3・1.7・1.8: `＞挨拶` → `＞別グローバル` → `＞挨拶` の 2 回とも呼び出し元の
/// ローカルシーンを実行し、戻った後の単語参照（アクター付き・代入）・関数呼び出し（アクター付き・代入）・
/// 動的コールも呼び出し元で解決される。呼ばれた側の行は呼ばれた側で解決される
#[test]
fn test_u28_mid_call_restores_caller_context() {
    assert_fires(
        "OnU28Twice",
        r"\p[0]Ａ挨拶Ｂ単語Ｂ関数Ａ挨拶Ａ単語Ａ単語Ａ関数Ａ関数Ａ挨拶終\e",
    );
}

/// 1.4: 入れ子（A→B→C）。戻った行はそれぞれ自分のシーンで解決される
#[test]
fn test_nested_mid_calls_restore_each_level() {
    assert_fires("OnNestedMid", r"\p[0]ＣＢＡ\e");
}

/// 1.5: 呼ばれた側が `＞チェイントーク` で中断し、次の OnTalk の機会に再開して最後まで実行した後、
/// 呼び出し元の続きが呼び出し元で解決される
#[test]
fn test_mid_call_restores_after_chain_talk_resume() {
    let second_change = |time: &str| {
        format!("ID: OnSecondChange\r\nStatus: idle\r\nReference0: 1\r\nX-Pasta-Time: {time}")
    };
    let (values, log) = run(&[
        // 次回トーク時刻を 12:05:10 に決める（話すことは無いので 204）
        &second_change("2025-07-15T12:05:00Z"),
        "ID: OnChainResumeMid",
        // 次の OnTalk の機会 → 中断点から再開
        &second_change("2025-07-15T12:05:10Z"),
    ]);
    assert_eq!(values[0], "");
    assert_eq!(values[1], r"\p[0]先\e");
    assert_eq!(values[2], r"\p[0]再開先元\e");
    assert!(lua_warnings(&log).is_empty(), "no warning expected:\n{log}");
}

/// 1.10: 途中の Call のターゲットが Lua の関数で、その中から別のグローバルシーンが呼ばれた後も、
/// 次の行は呼び出し元で解決される
#[test]
fn test_mid_call_to_lua_function_calling_other_global() {
    assert_fires("OnLuaTargetMid", r"\p[0]先元\e");
}

/// 4.3・4.2: G→A→(末尾)B→G。A の末尾の Call の後は B の文脈で、G に戻った行は G で解決される
#[test]
fn test_mid_call_then_tail_call_returns_to_caller_context() {
    assert_fires("OnMidThenTail", r"\p[0]ＡＢＧ\e");
}

/// 3.1: 式の関数呼び出しが 5 段目（グローバル辞書の前方一致）で別のグローバルシーンを実行した後、
/// 同じ行の残りと次の行（単語参照・Call）が呼び出し元で解決される
#[test]
fn test_expr_fn_stage5_global_scene_restores_context() {
    assert_fires("OnExprStage5", r"\p[0]先元元元ローカル終\e");
}

/// 3.4: `＠＊関数（…）` の中から別のグローバルシーンを呼んだ後、同じ行の残りと次の行が呼び出し元で
/// 解決される
#[test]
fn test_global_fn_calling_other_global_restores_context() {
    assert_fires("OnGlobalFnCall", r"\p[0]先元元\e");
}

/// 1.8: 別のグローバルシーンへの途中の Call が動的コール（`＞＄g`）でも、次の行は呼び出し元で解決される
#[test]
fn test_dynamic_mid_call_to_other_global_restores_context() {
    assert_fires("OnDynamicMid", r"\p[0]先元\e");
}

/// 1.6: 途中の Call のターゲットが `GLOBAL` の関数で、その中から別のグローバルシーンが呼ばれた後も、
/// 次の行は呼び出し元で解決される
#[test]
fn test_mid_call_to_global_function_calling_other_global() {
    assert_fires("OnGlobalTargetMid", r"\p[0]先元\e");
}

/// 1.1・1.3: 別のグローバルシーンへの途中の Call から戻った後、動的単語参照（`＠＄変数`）と動的関数
/// 呼び出し（`＠＄変数（…）`）が呼び出し元で解決される（呼ばれた側では同じ書き方が呼ばれた側に解決される）
#[test]
fn test_dynamic_refs_after_mid_call_resolve_in_caller_context() {
    assert_fires("OnDynRefAfterMid", r"\p[0]先語先関数元語元関数\e");
}

/// 1.6: 途中の Call のターゲットが act のメソッド（3 段目の `call`）で、その中で別のグローバルシーンが
/// 実行された後も、次の行は呼び出し元で解決される
#[test]
fn test_mid_call_to_act_method_restores_caller_context() {
    assert_fires("OnActMethodMid", r"\p[0]先元\e");
}

// ---------------------------------------------------------------------------
// 選択肢の探索範囲（tasks 4.2）
// 辞書: dic/choice_scope.pasta（2.6 は dic/characterization.pasta の OnChoiceMenu）。選択肢を出したシーン（A）と
// もう一方のシーン（B）の両方に同名のローカルシーンを置き、ジャンプ先の出力でどちらから探したかを見分ける。
// `\q` の第 3 引数は登録名（元の名前_連番）。
// ---------------------------------------------------------------------------

/// `OnChoiceSelectEx` のリクエスト（Reference0=表示、Reference1=ジャンプ先、Reference2=探索範囲）
fn choice_select(display: &str, target: &str, scope: Option<&str>) -> String {
    let mut req = format!("ID: OnChoiceSelectEx\r\nReference0: {display}\r\nReference1: {target}");
    if let Some(scope) = scope {
        req.push_str(&format!("\r\nReference2: {scope}"));
    }
    req
}

/// 応答の `\q[display,target,scope]` から scope（第 3 引数）を取り出す
fn choice_scope<'a>(value: &'a str, display: &str, target: &str) -> &'a str {
    let head = format!(r"\q[{display},{target},");
    let start = value
        .find(&head)
        .unwrap_or_else(|| panic!("{head} not found in {value}"))
        + head.len();
    let len = value[start..].find(']').expect(r"\q must be closed");
    &value[start..start + len]
}

/// `event` を起こし、続けて `choices` の選択肢（表示、ジャンプ先、`\q` の第 3 引数、選んだときの応答）を
/// その第 3 引数を Reference2 に載せて選ぶ。`event` の応答が `response` で、その中の `\q` の第 3 引数が
/// 送ったものと同じであること（応答に載った探索範囲が SSP から戻る）と、ジャンプ先の出力を確かめる。
/// 選択肢ごとに新しいゴーストで起こし直す
fn assert_choice_jumps(event: &str, response: &str, choices: &[(&str, &str, &str, &str)]) {
    for &(display, target, scope, expected) in choices {
        let (values, log) = run(&[
            &format!("ID: {event}"),
            &choice_select(display, target, Some(scope)),
        ]);
        assert_eq!(values[0], response, "{event}");
        assert_eq!(
            choice_scope(&values[0], display, target),
            scope,
            "{event}: {target}"
        );
        assert_eq!(values[1], expected, "{event}: choose {display}/{target}");
        assert!(lua_warnings(&log).is_empty(), "no warning expected:\n{log}");
    }
}

/// 2.1・2.7: A が途中の Call で B を呼び、戻った後に A が出した選択肢は A の配下から探し、A の配下に
/// 無ければ（B の配下にあっても）グローバルシーンを探す
#[test]
fn test_choice_after_mid_call_routes_to_emitter() {
    assert_choice_jumps(
        "OnCsAfterMid",
        r"\p[0]Ｂ通過選んで\![*]\q[後,後選先,OnCsAfterMid_1]\![*]\q[外,後選外,OnCsAfterMid_1]\e",
        &[
            ("後", "後選先", "OnCsAfterMid_1", r"\p[0]Ａの後選先\e"),
            (
                "外",
                "後選外",
                "OnCsAfterMid_1",
                r"\p[0]グローバルの後選外\e",
            ),
        ],
    );
}

/// 2.2: A が選択肢を出した後に途中の Call で B を呼び、戻って A の続きの行を実行して終わっても、A の配下から探す
#[test]
fn test_choice_before_mid_call_routes_to_emitter() {
    assert_choice_jumps(
        "OnCsBeforeMid",
        r"\p[0]選んで\![*]\q[前,前選先,OnCsBeforeMid_1]Ｂ通過Ａ再開\e",
        &[("前", "前選先", "OnCsBeforeMid_1", r"\p[0]Ａの前選先\e")],
    );
}

/// 2.3・2.7: 呼ばれた側 B が出した選択肢は、A に戻って A が（自分のローカルシーンを呼んでから）終わっても
/// B の配下から探し、B の配下に無ければ（A の配下にあっても）グローバルシーンを探す
#[test]
fn test_choice_from_callee_routes_to_callee() {
    assert_choice_jumps(
        "OnCsFromCallee",
        r"\p[0]選んで\![*]\q[出,出選先,出選Ｂ_1]\![*]\q[外,出選外,出選Ｂ_1]Ａ通過\e",
        &[
            ("出", "出選先", "出選Ｂ_1", r"\p[0]Ｂの出選先\e"),
            ("外", "出選外", "出選Ｂ_1", r"\p[0]グローバルの出選外\e"),
        ],
    );
}

/// 2.4: 1 つの応答に A と、A が途中の Call で呼んだ B が同じジャンプ先名で出した選択肢があるとき、どちらを選んでも出したシーンの
/// 配下から探す
#[test]
fn test_choices_from_both_scenes_in_one_response() {
    assert_choice_jumps(
        "OnCsBoth",
        r"\p[0]Ａから\![*]\q[Ａの,両選先,OnCsBoth_1]Ｂから\![*]\q[Ｂの,両選先,両選Ｂ_1]Ａ再開\e",
        &[
            ("Ａの", "両選先", "OnCsBoth_1", r"\p[0]Ａの両選先\e"),
            ("Ｂの", "両選先", "両選Ｂ_1", r"\p[0]Ｂの両選先\e"),
        ],
    );
}

/// 2.5: A が末尾の Call で B へ移り、B が出した選択肢は B の配下から探す
#[test]
fn test_choice_after_tail_call_routes_to_callee() {
    assert_choice_jumps(
        "OnCsTail",
        r"\p[0]Ａ通過選んで\![*]\q[末,末選先,末選Ｂ_1]\e",
        &[("末", "末選先", "末選Ｂ_1", r"\p[0]Ｂの末選先\e")],
    );
}

/// 2.6: 別のグローバルシーンへの途中の Call を含まないシーンの選択肢は、`\q` に出したシーンが加わり、
/// ジャンプ先は現行どおり（出したシーンの配下、無ければグローバルシーン）
#[test]
fn test_choice_without_mid_call_carries_scope() {
    assert_choice_jumps(
        "OnChoiceMenu",
        r"\p[0]どれ？\_w[450]\![*]\q[甲,選択肢甲,OnChoiceMenu_1]\![*]\q[乙,選択肢乙,OnChoiceMenu_1]\e",
        &[
            ("甲", "選択肢甲", "OnChoiceMenu_1", r"\p[0]メニューの甲\e"),
            ("乙", "選択肢乙", "OnChoiceMenu_1", r"\p[0]グローバルの乙\e"),
        ],
    );
}

/// 2.8: 呼ばれた側 B が選択肢を出した後に `＞チェイントーク` で中断し、中断中に選ばれても B の配下から探す
#[test]
fn test_choice_during_chain_talk_suspension_routes_to_callee() {
    assert_choice_jumps(
        "OnCsSuspend",
        r"\p[0]選んで\![*]\q[止,止選先,止選Ｂ_1]\e",
        &[("止", "止選先", "止選Ｂ_1", r"\p[0]Ｂの止選先\e")],
    );
}

/// 3.2: 式の関数呼び出しが 5 段目で別のグローバルシーン B を実行した後も、B が出した選択肢は B の配下から、
/// A が出した選択肢は A の配下から探す
#[test]
fn test_choices_after_expr_fn_global_scene_route_to_emitters() {
    assert_choice_jumps(
        "OnCsExpr",
        r"\p[0]Ｂから\![*]\q[Ｂの,式選先,式選Ｂ_1]Ａから\![*]\q[Ａの,式選先,OnCsExpr_1]\e",
        &[
            ("Ｂの", "式選先", "式選Ｂ_1", r"\p[0]Ｂの式選先\e"),
            ("Ａの", "式選先", "OnCsExpr_1", r"\p[0]Ａの式選先\e"),
        ],
    );
}

/// 2.9: Reference2 が無い、または既知のグローバルシーン名でない選択は、最後にシーン文脈となったグローバル
/// シーンの配下から探す。OnCsAfterMid では途中の Call の先（後選Ｂ）が最後にシーン文脈となっている
#[test]
fn test_choice_without_known_scope_falls_back_to_last_global_scene() {
    for scope in [None, Some("無いシーン")] {
        let (values, log) = run(&["ID: OnCsAfterMid", &choice_select("後", "後選先", scope)]);
        assert_eq!(values[1], r"\p[0]Ｂの後選先\e", "Reference2 = {scope:?}");
        assert!(lua_warnings(&log).is_empty(), "no warning expected:\n{log}");
    }
}

/// 2.7: 明示的な `＊OnChoiceSelectEx` シーンは Reference2 があっても優先される。見つからない選択 ID は
/// 現行どおり 204（話すことが無い）
#[test]
fn test_choice_routing_order_unchanged() {
    let (values, log) = run_with_extra_dic(
        Some("＊OnChoiceSelectEx\n　さくら：明示シーン\n"),
        &[
            "ID: OnCsAfterMid",
            &choice_select("後", "後選先", Some("OnCsAfterMid_1")),
        ],
    );
    assert_eq!(values[1], r"\p[0]明示シーン\e");
    assert!(lua_warnings(&log).is_empty(), "no warning expected:\n{log}");

    let (values, _) = run(&[
        "ID: OnCsAfterMid",
        &choice_select("無", "どこにも無い先", Some("OnCsAfterMid_1")),
    ]);
    assert_eq!(values[1], "");
}

// ---------------------------------------------------------------------------
// 失敗した Call（tasks 4.3）
// 辞書: dic/failed_call.pasta。失敗表記が Call 行の位置に出て、ログの警告が design.md の
// DynamicCallKey の表（と ActCall の「見つからない」）の文言・件数どおりで、途中の Call の後は
// 呼び出し元の文脈（＠失話題：元）で続く。
// ---------------------------------------------------------------------------

/// Lua 側の警告の本文（`pasta_lua::runtime::log:` の後ろから、呼び出し元の ` lua_source=…` の前まで）
fn lua_warning_messages(log: &str) -> Vec<&str> {
    lua_warnings(log)
        .into_iter()
        .map(|l| {
            let msg = l
                .split_once(" WARN pasta_lua::runtime::log: ")
                .map_or(l, |(_, m)| m);
            msg.split_once(" lua_source=").map_or(msg, |(m, _)| m)
        })
        .collect()
}

/// イベント ID 1 つを送り（応答が 200 であることは `run` が確かめる）、応答が `expected` で、
/// Lua 側の警告が `warnings` と同じ文言・同じ件数・同じ順であることを確かめる
fn assert_failed_call(event_id: &str, expected: &str, warnings: &[&str]) {
    let (value, log) = fire(event_id);
    assert_eq!(value, expected, "{event_id}");
    assert_eq!(lua_warning_messages(&log), warnings, "{event_id}:\n{log}");
}

/// 5.1–5.4・5.2・5.11: `＞＄未代入` は検索せず（`nil` で始まるローカルシーン・関数・グローバルシーンを
/// 呼ばない）、変数のパスを含む警告 1 件と失敗表記を出し、次の行を呼び出し元で解決する
#[test]
fn test_failed_call_undefined_variable() {
    assert_failed_call(
        "OnFcVarMid",
        r"\p[0]前【Call失敗：var.未代入 が nil】後元\e",
        &["act:call - undefined variable: 'var.未代入'"],
    );
}

/// 5.1・5.3・5.5: `＞＠値なし（）` は関数の表記を含む警告 1 件と失敗表記
#[test]
fn test_failed_call_function_returning_nothing() {
    assert_failed_call(
        "OnFcFnMid",
        r"\p[0]前【Call失敗：@値なし() が nil】後元\e",
        &["act:call - key is not a string or number: operand='@値なし()', value=nil"],
    );
}

/// 5.6: `＞＄未代入＆「x」` は連結の警告 1 件だけで、Call の警告を重ねない
#[test]
fn test_failed_call_concat_with_nil_operand() {
    assert_failed_call(
        "OnFcConcatMid",
        r"\p[0]前【Call失敗：値が nil】後元\e",
        &[
            "act:concat - operand is not a string or number: op='&', operand='var.未代入', value=nil",
        ],
    );
}

/// 5.7: `＞「nil」` は文字列 `nil` を検索キーにして通常どおり検索する（警告なし）
#[test]
fn test_call_string_nil_is_searched() {
    assert_failed_call("OnFcNilStrMid", r"\p[0]前nil先後\e", &[]);
}

/// 6.3・6.6: 空文字列（変数と式）は検索せず、それぞれ警告 1 件と失敗表記
#[test]
fn test_failed_call_empty_string() {
    assert_failed_call(
        "OnFcEmptyMid",
        r"\p[0]前【Call失敗：var.空 が空文字列】【Call失敗：値が空文字列】後元\e",
        &[
            "act:call - empty variable: 'var.空'",
            "act:call - key is not a string or number: value='' (string)",
        ],
    );
}

/// 6.3・6.6: 真偽値（変数と関数）は検索せず、それぞれ型名を含む警告 1 件と失敗表記
#[test]
fn test_failed_call_boolean() {
    assert_failed_call(
        "OnFcBoolMid",
        r"\p[0]前【Call失敗：var.真 が boolean】【Call失敗：@真を返す() が boolean】後元\e",
        &[
            "act:call - unsupported value type: 'var.真' (boolean)",
            "act:call - key is not a string or number: operand='@真を返す()', value=true (boolean)",
        ],
    );
}

/// 5.9: 引数リスト付きでも引数の式は書いた順に評価され、Call だけを行わない。失敗表記はキーの評価の
/// 時点（引数の式が積む出力より前）に出る
#[test]
fn test_failed_call_with_argument_list() {
    assert_failed_call(
        "OnFcArgsMid",
        r"\p[0]前【Call失敗：var.未代入 が nil】甲乙後\e",
        &["act:call - undefined variable: 'var.未代入'"],
    );
}

/// 5.8: 値が nil の動的コールが末尾の Call（ローカルシーンの最後の行・グローバルシーンの最後の行）でも
/// エラーにせず、ローカルシーンの後は呼び出し元の続きを実行し、シーンを通常どおり終える
#[test]
fn test_failed_tail_call() {
    assert_failed_call(
        "OnFcTailEnd",
        r"\p[0]前中【Call失敗：var.未代入甲 が nil】後【Call失敗：var.未代入乙 が nil】\e",
        &[
            "act:call - undefined variable: 'var.未代入甲'",
            "act:call - undefined variable: 'var.未代入乙'",
        ],
    );
}

/// 4.6: 存在しないシーンへの静的と動的の Call は、見つからない名前を含む失敗表記と警告 1 件ずつを出し、
/// 次の行を呼び出し元で解決する
#[test]
fn test_failed_call_scene_not_found() {
    assert_failed_call(
        "OnFcNotFound",
        r"\p[0]前【Call失敗：「どこにも無いシーン」が見つからない】【Call失敗：「どこにも無い動的先」が見つからない】後元\e",
        &[
            "act:call - handler not found: key='どこにも無いシーン', mode='scene', via=act",
            "act:call - handler not found: key='どこにも無い動的先', mode='scene', via=act",
        ],
    );
}
