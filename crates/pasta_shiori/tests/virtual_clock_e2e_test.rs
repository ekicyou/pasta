//! 仮想時刻の時報と日時変数の SHIORI E2E テスト（仕様 `tutorial-ssp-gaps` task 4.1）。
//!
//! フィクスチャ `tests/fixtures/virtual_clock/` は `＊時報その他`（時報）と `＊時刻`（転記してから
//! `＄時` を出す）を持つ。本物のランタイムに `OnSecondChange` と `OnPastaClock`（SSP の仮想時刻を
//! 読んだ返事）を送り、時報と日時変数が仮想の時刻で決まることを確かめる。
//!
//! # 仮想の日時の決め方（実際の正時をまたいでも結果が揺れない）
//! テストの最初に実際の `unix`（T0）を読み、T0 の次の正時を H として、仮想の日時を
//! **H + 1 時間 + 10 秒**にする。
//! - ずれ（仮想 − 返事を受けた時の OS の時刻）は約 1〜2 時間で、猶予（70 秒）やずれを 0 と
//!   みなす幅（2 秒）よりずっと大きい。テストの途中で実際の正時 H をまたいでもずれは 0 にならず、
//!   ずれが変わると次の OnSecondChange で「今」が飛び、時刻の飛び（epoch）による決め直しが必ず起きる。
//! - 決め直しは「仮想の今 − 70 秒」の次の正時（= H + 1 時間）を次の正時にする。仮想の今は
//!   その 10 秒（＋往復の数秒）後なので、返事の次の毎秒の回で必ず時報が出る。
//! - 期待する `＄時` は H + 1 時間の地方時の時。実際の時（T0 の時か H の時）とは必ず違うので、
//!   OS の時計のままかどうかも同じ基準で見分けられる。

mod common;

use common::response::ShioriResponse;
use common::test_env::ShioriTestEnv;
use time::OffsetDateTime;

const FIXTURE: &str = "virtual_clock";
const HOUR_SIGNAL: &str = "時報です。";
const CLOCK_SCENE: &str = "時刻です。";

/// 実際の今（地方時）
fn os_now() -> OffsetDateTime {
    OffsetDateTime::now_local().expect("local offset")
}

/// 仮想の日時（T0 の次の正時 + 1 時間 + 10 秒）を地方時で返す
fn virtual_time(base: OffsetDateTime) -> OffsetDateTime {
    let t0 = base.unix_timestamp();
    let next_hour = t0 - t0.rem_euclid(3600) + 3600;
    OffsetDateTime::from_unix_timestamp(next_hour + 3600 + 10)
        .expect("valid unix")
        .to_offset(base.offset())
}

/// シーンが `＄時`（例: 「14時」）を出す箇所の形（「いまは」を前に付けて 4 時と 14 時を見分ける）
fn says_hour(hour: u8) -> String {
    format!("いまは{hour}時")
}

fn get(env: &mut ShioriTestEnv, headers: &str) -> ShioriResponse {
    env.request(&format!(
        "GET SHIORI/3.0\nCharset: UTF-8\nSender: SSP\nSecurityLevel: local\n{headers}"
    ))
    .expect("request should succeed")
}

/// 毎秒の回（ブロックされない Status）
fn on_second_change(env: &mut ShioriTestEnv, extra: &str) -> ShioriResponse {
    get(
        env,
        &format!("Status: idle\nID: OnSecondChange\nReference0: 1\n{extra}"),
    )
}

/// 仮想の日時を読んだ返事（Reference0〜5 = 年・月・日・時・分・秒。ゼロ埋めなし）
fn on_pasta_clock(env: &mut ShioriTestEnv, v: OffsetDateTime, extra: &str) -> ShioriResponse {
    get(
        env,
        &format!(
            "ID: OnPastaClock\nReference0: {}\nReference1: {}\nReference2: {}\n\
             Reference3: {}\nReference4: {}\nReference5: {}\n{extra}",
            v.year(),
            u8::from(v.month()),
            v.day(),
            v.hour(),
            v.minute(),
            v.second()
        ),
    )
}

fn value(resp: &ShioriResponse) -> String {
    resp.value.clone().unwrap_or_default()
}

/// `＊時刻` を呼んで台詞を返す（転記してから `＄時` を出すシーン）
fn clock_scene(env: &mut ShioriTestEnv) -> String {
    let resp = get(env, "ID: 時刻\n");
    assert_eq!(resp.status_code, 200, "時刻: {:?}", resp.value);
    let text = value(&resp);
    assert!(text.contains(CLOCK_SCENE), "no 時刻 scene: {text}");
    text
}

/// 返事で仮想の日時を送ると、次の毎秒の回で時報のシーンと仮想の `＄時` が出る（2.1・2.2・2.4）
#[test]
fn reply_with_virtual_time_fires_hour_signal_with_virtual_hour() {
    let mut env = ShioriTestEnv::new(FIXTURE);
    let v = virtual_time(os_now());

    // 1 回目の毎秒の回: 次の正時を記録するだけ
    let first = on_second_change(&mut env, "");
    assert_eq!(first.status_code, 204, "first: {:?}", first.value);

    // 次の正時をまたぐ日時を返事で送る → 204
    let reply = on_pasta_clock(&mut env, v, "");
    assert_eq!(reply.status_code, 204, "reply: {:?}", reply.value);

    // 次の毎秒の回: 時報のシーンと仮想の ＄時
    let fired = on_second_change(&mut env, "");
    assert_eq!(fired.status_code, 200, "fired: {:?}", fired.value);
    let text = value(&fired);
    assert!(text.contains(HOUR_SIGNAL), "no hour signal: {text}");
    assert!(
        text.contains(&says_hour(v.hour())),
        "＄時 must be the virtual hour {}: {text}",
        v.hour()
    );

    // 時報でないシーンの日時変数も仮想の時刻（2.2）
    let text = clock_scene(&mut env);
    assert!(
        text.contains(&says_hour(v.hour())),
        "＄時 must be the virtual hour {}: {text}",
        v.hour()
    );
}

/// X-Pasta-Time 付きでは仮想時刻が効かず、時報が出ない（2.8）
#[test]
fn x_pasta_time_ignores_virtual_time() {
    let mut env = ShioriTestEnv::new(FIXTURE);
    let v = virtual_time(os_now());

    let first = on_second_change(&mut env, "X-Pasta-Time: 2025-07-15T12:05:00Z\n");
    assert_eq!(first.status_code, 204, "first: {:?}", first.value);

    let reply = on_pasta_clock(&mut env, v, "X-Pasta-Time: 2025-07-15T12:05:00Z\n");
    assert_eq!(reply.status_code, 204, "reply: {:?}", reply.value);

    let next = on_second_change(&mut env, "X-Pasta-Time: 2025-07-15T12:05:01Z\n");
    assert_eq!(
        next.status_code, 204,
        "no hour signal expected: {:?}",
        next.value
    );

    // X-Pasta-Time 付きの返事はずれを変えない: 続く応答は OS の時計のまま
    assert_os_hour(&mut env, v);
}

/// 読めない値の返事は 204 で、続く応答は OS の時計のまま（2.9）
#[test]
fn unreadable_reply_falls_back_to_os_clock() {
    let mut env = ShioriTestEnv::new(FIXTURE);
    let v = virtual_time(os_now());

    // いったん仮想時刻を効かせる
    on_second_change(&mut env, "");
    let reply = on_pasta_clock(&mut env, v, "");
    assert_eq!(reply.status_code, 204, "reply: {:?}", reply.value);
    let text = clock_scene(&mut env);
    assert!(
        text.contains(&says_hour(v.hour())),
        "virtual hour expected: {text}"
    );

    // 読めない値の返事 → 204、ずれは 0 に戻る
    let bad = get(
        &mut env,
        "ID: OnPastaClock\nReference0: abc\nReference1: 1\nReference2: 1\n\
         Reference3: 1\nReference4: 1\nReference5: 1\n",
    );
    assert_eq!(bad.status_code, 204, "bad reply: {:?}", bad.value);
    assert_os_hour(&mut env, v);
}

/// `＊時刻` の `＄時` が OS の時計の時（要求の前後どちらかの時）で、仮想の時ではない
fn assert_os_hour(env: &mut ShioriTestEnv, v: OffsetDateTime) {
    let before = os_now().hour();
    let text = clock_scene(env);
    let after = os_now().hour();
    assert!(
        text.contains(&says_hour(before)) || text.contains(&says_hour(after)),
        "＄時 must be the OS hour ({before}/{after}): {text}"
    );
    assert!(
        !text.contains(&says_hour(v.hour())),
        "＄時 must not be the virtual hour {}: {text}",
        v.hour()
    );
}
