//! 止まるエラー（Lua のエラーでシーン・ハンドラが止まった）がバルーンの表記になることの SHIORI 経由 E2E テスト
//!
//! 仕様 `runtime-error-balloon` task 6.1（requirements 1.1, 1.6, 1.7, 2.2, 2.4, 2.5, 2.6, 3.3, 3.4, 3.6,
//! 4.1, 4.2, 4.3, 5.6, 6.4）。
//!
//! フィクスチャ `tests/fixtures/runtime_error_balloon/` の pasta.toml の末尾に
//! `[logging] balloon_warning` を書き足して（書かない・false・型違い）、
//! 別名 `＊会話` のローカルシーンの Lua のブロック・作者の `scripts/天気.lua`・REG のハンドラ・
//! `On` で始まらない ID・NOTIFY で止まる場合を起こす。

mod common;

use common::copy_fixture_to_temp;
use common::response::ShioriResponse;
use pasta::actor::mailbox::{ActorMsg, mailbox};
use pasta::actor::marshaling::marshal_request;
use pasta::actor::thread::spawn_actor_thread;

/// `balloon_warning` の 3 通り（書かない・false・型違い）
const UNSET: Option<&str> = None;
const OFF: Option<&str> = Some("false");
const MISTYPED: Option<&str> = Some("\"no\"");

/// 要求の結果: 各要求の応答の全文と、ログの警告（文言だけ）
struct Run {
    raws: Vec<String>,
    warns: Vec<String>,
}

impl Run {
    fn resp(&self, i: usize) -> ShioriResponse {
        ShioriResponse::parse(&self.raws[i]).expect("response should parse")
    }

    /// 設定どうしで照らす形: (ステータス, Value, 一時ディレクトリの名前を伏せた理由の行)。
    /// ヘッダーの順は Lua の表の順で決まらないので、全文ではなくこの形で照らす
    fn shapes(&self) -> Vec<(u16, Option<String>, Option<String>)> {
        (0..self.raws.len())
            .map(|i| {
                let r = self.resp(i);
                let reason = r.headers.get("X-Error-Reason").map(|s| mask_temp(s));
                (r.status_code, r.value, reason)
            })
            .collect()
    }
}

/// 一時ディレクトリの名前（`.tmp` ＋ 6 文字）を伏せる
fn mask_temp(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find(".tmp") {
        out.push_str(&rest[..i + 4]);
        out.push_str("XXXXXX");
        rest = rest[i + 4..].get(6..).unwrap_or("");
    }
    out.push_str(rest);
    out
}

/// `balloon_warning` の行（None なら書かない）を足したゴーストに要求を順に送る。
/// 要求は (メソッド, ID)。`pasta.dll` の `request` と同じ道（アクタースレッドと、GET は応答を待ち
/// NOTIFY は即 204 を返す振り分け）を通す。アクターを止めてからログを読む。
fn run(balloon_warning: Option<&str>, requests: &[(&str, &str)]) -> Run {
    let temp = copy_fixture_to_temp("runtime_error_balloon");
    if let Some(value) = balloon_warning {
        let path = temp.path().join("pasta.toml");
        let mut toml = std::fs::read_to_string(&path).unwrap();
        toml.push_str(&format!("\n[logging]\nballoon_warning = {value}\n"));
        std::fs::write(&path, toml).unwrap();
    }
    let (tx, rx) = mailbox();
    let actor = spawn_actor_thread(0, temp.path().to_path_buf(), rx);
    assert!(actor.loaded(), "actor thread must load the ghost");
    let raws = requests
        .iter()
        .enumerate()
        .map(|(seq, (method, id))| {
            marshal_request(
                &tx,
                seq as u64,
                &format!(
                    "{method} SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nID: {id}\r\n\r\n"
                ),
            )
        })
        .collect();
    let (done, done_rx) = flume::bounded(1);
    tx.send(ActorMsg::Stop { done }).unwrap();
    done_rx.recv().unwrap();
    actor.join().unwrap();

    let log = std::fs::read_to_string(temp.path().join("profile/pasta/logs/pasta.log"))
        .unwrap_or_default();
    assert!(
        log.contains("SHIORI.load called successfully"),
        "log must be captured:\n{log}"
    );
    let warns = log
        .lines()
        .filter_map(|l| l.split_once(" WARN pasta_lua::runtime::log: "))
        // 呼び出し元の位置（一時ディレクトリのパスを含む）は除く
        .map(|(_, rest)| rest.split(" lua_source=").next().unwrap().to_string())
        .collect();
    Run { raws, warns }
}

/// GET を 1 つ送る
fn get(balloon_warning: Option<&str>, id: &str) -> Run {
    run(balloon_warning, &[("GET", id)])
}

/// 表記を出す設定の応答を、出さない設定の応答と照らす
/// （200・理由の行が出さない設定の 500 と同じ・途中の台詞の後に表記・生成コードとランタイムの場所が無い）。
/// 表記（Value の中の `【…】`）を返す
fn expect_notation(on: &ShioriResponse, off: &ShioriResponse, before: &str) -> String {
    // 出さない設定は今と同じ: 500・理由の行・Value が無い（4.2）
    assert_eq!(off.status_code, 500, "{:?}", off.value);
    assert!(off.value.is_none(), "{:?}", off.value);
    let reason = off.headers.get("X-Error-Reason").expect("reason line");

    // 出す設定は 200 ＋ Value ＋ 出さない設定と同じ理由の行（3.3・3.6）
    assert_eq!(on.status_code, 200, "{} {:?}", on.status_text, on.value);
    assert_eq!(
        on.headers.get("X-Error-Reason").map(|s| mask_temp(s)),
        Some(mask_temp(reason))
    );
    let value = on.value.clone().expect("Value header must exist");

    // 途中の台詞が表記の前にある（3.4）
    let start = value
        .find('【')
        .unwrap_or_else(|| panic!("no notation: {value}"));
    let end = value.rfind('】').unwrap() + '】'.len_utf8();
    let before_pos = value
        .find(before)
        .unwrap_or_else(|| panic!("no {before}: {value}"));
    assert!(before_pos < start, "{value}");
    assert_eq!(value.matches('【').count(), 1, "{value}");

    // 変換した Lua のキャッシュ・標準ランタイムの場所が無い（2.5・2.6）
    let notation = value[start..end].to_string();
    for hidden in ["cache", "pasta_scripts", "init.lua", "entry.lua", "[string"] {
        assert!(!notation.contains(hidden), "{hidden} in {notation}");
    }
    notation
}

/// 警告は 1 行で、変換した Lua のパスを含まない（5.6）
fn expect_one_warning(run: &Run, body: &str) {
    assert_eq!(run.warns.len(), 1, "{:#?}", run.warns);
    assert_eq!(run.warns[0], body, "{:#?}", run.warns);
    assert!(!run.warns[0].contains("cache"), "{:#?}", run.warns);
}

/// 別名 `＊会話` のローカルシーンの Lua のブロックで止まると、書いた名前と `.pasta` の行が出る（2.2・2.4・2.5）
#[test]
fn test_alias_scene_lua_block_error_shows_pasta_location() {
    let on = get(UNSET, "OnTalk");
    let off = get(OFF, "OnTalk");
    let notation = expect_notation(&on.resp(0), &off.resp(0), "途中の台詞");
    // 生のスクリプトでは `\` が `\\` に無効化される（Lua のブロックの `return data.雨` は 11 行目）
    assert_eq!(
        notation,
        "【スクリプトのエラー：＊会話・雨の日（dic\\\\runtime_error_balloon.pasta:11: \
         attempt to index local 'data' (a nil value)）】"
    );
    let body = "スクリプトのエラー：＊会話・雨の日（dic\\runtime_error_balloon.pasta:11: \
                attempt to index local 'data' (a nil value)）";
    expect_one_warning(&on, body);
    // 警告は出さない設定でも同じ（4.2・5.6）
    assert_eq!(off.warns, on.warns);
}

/// シーンから呼んだ作者の `scripts/天気.lua` で止まると、`scripts\天気.lua:N:` が出る（2.5）
#[test]
fn test_author_lua_error_shows_script_location() {
    let on = get(UNSET, "OnWeather");
    let off = get(OFF, "OnWeather");
    let notation = expect_notation(&on.resp(0), &off.resp(0), "天気の途中");
    // 投げ直し・シーン関数（変換した Lua）の場所は出ず、作者の .lua の場所だけが残る
    assert_eq!(
        notation,
        "【スクリプトのエラー：＊OnWeather（scripts\\\\天気.lua:6: \
         attempt to index local 'forecast' (a nil value)）】"
    );
    expect_one_warning(
        &on,
        "スクリプトのエラー：＊OnWeather（scripts\\天気.lua:6: \
         attempt to index local 'forecast' (a nil value)）",
    );
}

/// REG のハンドラで止まると、要求の ID を対象にし、ハンドラの途中の台詞の後に表記が出る（2.2・3.4）
#[test]
fn test_handler_error_shows_request_id() {
    let on = get(UNSET, "OnHandlerFail");
    let off = get(OFF, "OnHandlerFail");
    let notation = expect_notation(&on.resp(0), &off.resp(0), "ハンドラの途中");
    assert_eq!(
        notation,
        "【スクリプトのエラー：OnHandlerFail（scripts\\\\main.lua:7: handler boom）】"
    );
    expect_one_warning(
        &on,
        "スクリプトのエラー：OnHandlerFail（scripts\\main.lua:7: handler boom）",
    );
}

/// 型違いの値は書かれていないときと同じ（表記を出す）（4.3）
#[test]
fn test_mistyped_value_shows_notation() {
    for id in ["OnTalk", "OnWeather", "OnHandlerFail"] {
        let on = get(UNSET, id);
        let mistyped = get(MISTYPED, id);
        assert_eq!(mistyped.shapes(), on.shapes(), "{id}");
        assert_eq!(mistyped.warns, on.warns, "{id}");
    }
}

/// `On` で始まらない ID は、どの設定でも今と同じ 500（理由の行・Value が無い）。警告は 1 行出る（1.6・5.6）
#[test]
fn test_non_on_id_stays_500() {
    let off = get(OFF, "plain_fail");
    let resp = off.resp(0);
    assert_eq!(resp.status_code, 500, "{:?}", resp.value);
    assert!(resp.value.is_none());
    assert!(resp.headers.contains_key("X-Error-Reason"));
    for setting in [UNSET, MISTYPED] {
        let other = get(setting, "plain_fail");
        assert_eq!(other.shapes(), off.shapes(), "{setting:?}");
        assert_eq!(other.warns, off.warns, "{setting:?}");
    }
    expect_one_warning(
        &off,
        "スクリプトのエラー：plain_fail（scripts\\main.lua:13: plain boom）",
    );
}

/// NOTIFY で止まっても応答は今と同じ（即 204）。警告は 1 行出る（1.7・5.6）
#[test]
fn test_notify_response_unchanged() {
    let off = run(OFF, &[("NOTIFY", "OnHandlerFail")]);
    assert_eq!(off.resp(0).status_code, 204, "{}", off.raws[0]);
    for setting in [UNSET, MISTYPED] {
        let other = run(setting, &[("NOTIFY", "OnHandlerFail")]);
        assert_eq!(other.shapes(), off.shapes(), "{setting:?}");
        assert_eq!(other.warns, off.warns, "{setting:?}");
    }
    expect_one_warning(
        &off,
        "スクリプトのエラー：OnHandlerFail（scripts\\main.lua:7: handler boom）",
    );
}

/// 表記を出した次の要求は普通の応答になる（1.1）
#[test]
fn test_next_request_after_notation_is_normal() {
    let on = run(UNSET, &[("GET", "OnTalk"), ("GET", "OnFine")]);
    assert!(on.resp(0).value.unwrap().contains('【'));
    let next = on.resp(1);
    assert_eq!(next.status_code, 200, "{:?}", next.value);
    assert!(!next.headers.contains_key("X-Error-Reason"));
    let value = next.value.unwrap();
    assert!(value.contains("普通のトーク"), "{value}");
    assert!(!value.contains('【'), "{value}");
}
