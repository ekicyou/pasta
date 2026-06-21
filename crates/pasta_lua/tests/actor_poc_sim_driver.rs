//! SimDriver 忠実シミュレータの単体検証（R5.6・task 5.1）。
//!
//! ファイル全体を `actor-poc` feature でガードする。feature 無効時はこのテスト
//! バイナリは空にコンパイルされ（`#![cfg(...)]`）、既定テストスイートを一切
//! 汚染しない（R7.1/R7.2/R7.4）。
//!
//! 検証する観測可能な完了条件（tasks.md 5.1）:
//! 「`tick(playable)` が GET/NOTIFY tick を発火し `set_talking` が遷移する単体
//!   テストが緑。」
//!
//! # 「実 SSP 相当」忠実性の定義（R5.6・design.md SimDriver）
//!
//! SimDriver は OnSecondChange の周期と GET/NOTIFY（≡Reference3）権威を忠実に
//! 再現する**自前ドライバ（生成器）**である。`tick(playable)` は OnSecondChange
//! 1 周期を発火し、`playable=true→GET(Ref3=1)`（再生可・上書き可）／
//! `playable=false→NOTIFY(Ref3=0)`（再生不能）として**自身が method タグ付け**する。
//! これは Marshal の pest 再パースに依存しない独立した tick 生成器であり、
//! KickHarness(5.2)・Latency(6.1) が消費する gate 権威ソースとなる（Q4）。
#![cfg(feature = "actor-poc")]

use pasta_lua::actor_poc::sim_driver::{SimDriver, SimMethod};

/// 開発セッションが export した `PASTA_DEBUG`／`PASTA_DEBUG_PORT` を main 前
/// （プロセスが単一スレッドの間）に中和する。写経元:
/// `crates/pasta_lua/tests/common/mod.rs:26-32`。
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

/// R5.6（GET tick）: `tick(true)` は GET tick（Reference3=1・再生可/上書き可）を
/// 発火する。method タグと Reference3 値の双方を assert する。
///
/// もし tick が playable=true を NOTIFY/Ref3=0 と誤ラベルすれば、この assert は
/// 失敗する。
#[test]
fn tick_true_fires_get_with_reference3_one() {
    let mut driver = SimDriver::new();
    let tick = driver.tick(true);

    assert_eq!(
        tick.method,
        SimMethod::Get,
        "playable=true must produce a GET tick (再生可/上書き可)"
    );
    assert_eq!(
        tick.reference3, 1,
        "GET tick must carry Reference3=1 (gate 権威: 上書き可)"
    );
    // GET tick は配信可否 gate を通す（上書き可）。
    assert!(
        tick.is_playable(),
        "GET tick (Ref3=1) must be reported as playable"
    );
}

/// R5.6（NOTIFY tick）: `tick(false)` は NOTIFY tick（Reference3=0・再生不能）を
/// 発火する。method タグと Reference3 値の双方を assert する。
///
/// もし tick が playable=false を GET/Ref3=1 と誤ラベルすれば、この assert は
/// 失敗する。
#[test]
fn tick_false_fires_notify_with_reference3_zero() {
    let mut driver = SimDriver::new();
    let tick = driver.tick(false);

    assert_eq!(
        tick.method,
        SimMethod::Notify,
        "playable=false must produce a NOTIFY tick (再生不能)"
    );
    assert_eq!(
        tick.reference3, 0,
        "NOTIFY tick must carry Reference3=0 (gate 権威: 再生不能)"
    );
    // NOTIFY tick は配信可否 gate で無視される（再生不能）。
    assert!(
        !tick.is_playable(),
        "NOTIFY tick (Ref3=0) must be reported as not playable"
    );
}

/// R5.6（method ≡ Reference3 不変条件）: SimMethod と Reference3 が常に一致する
/// （Get⇔1／Notify⇔0）。両方の tick で method タグと Reference3 が連動することを
/// 確認し、片方だけ正しい偽実装を弾く。
#[test]
fn method_and_reference3_are_consistent_across_both_ticks() {
    let mut driver = SimDriver::new();

    let get = driver.tick(true);
    let notify = driver.tick(false);

    assert_eq!(get.method, SimMethod::Get);
    assert_eq!(get.reference3, 1);
    assert_eq!(notify.method, SimMethod::Notify);
    assert_eq!(notify.reference3, 0);

    // method から Reference3 を導出した値が tick の Reference3 と一致する
    // （自己整合・どちらか一方の誤りで失敗する）。
    assert_eq!(get.method.reference3(), get.reference3);
    assert_eq!(notify.method.reference3(), notify.reference3);
}

/// R5.6（talking 遷移）: `set_talking(true)`／`set_talking(false)` が talking
/// 状態を遷移させ、getter で観測できる。状態が実際に変化することを assert する。
///
/// もし set_talking が状態を遷移させなければ（no-op 偽実装）、この assert は
/// 失敗する。
#[test]
fn set_talking_transitions_observable_state() {
    let mut driver = SimDriver::new();

    // 初期状態は非 talking（再生していない）。
    assert!(
        !driver.is_talking(),
        "a fresh SimDriver must start in the non-talking state"
    );

    driver.set_talking(true);
    assert!(
        driver.is_talking(),
        "set_talking(true) must transition into the Status: talking state"
    );

    driver.set_talking(false);
    assert!(
        !driver.is_talking(),
        "set_talking(false) must transition back out of the talking state"
    );
}

/// R5.6（独立生成器・Marshal/pest 非依存）: tick の method 判定は SimDriver 内で
/// 完結し、外部リクエスト文字列のパースに依存しない。引数 `playable`（bool）の
/// みから決定論的に method/Reference3 が決まることを、文字列入力を一切与えずに
/// 確認する。
///
/// これは「自前 method タグ付け（Marshal の再パースには依存しない生成器）」を
/// 観測可能にする: tick は request 文字列を受け取らず bool だけで自己タグ付けする。
#[test]
fn driver_self_tags_method_without_request_string_parsing() {
    let mut driver = SimDriver::new();

    // 同一 bool 入力に対して常に同一の method/Reference3 を返す（決定論）。
    for _ in 0..16 {
        let t = driver.tick(true);
        assert_eq!(t.method, SimMethod::Get);
        assert_eq!(t.reference3, 1);
    }
    for _ in 0..16 {
        let t = driver.tick(false);
        assert_eq!(t.method, SimMethod::Notify);
        assert_eq!(t.reference3, 0);
    }
}

/// R5.6（talking 状態は tick と独立に保持される）: set_talking で設定した状態は
/// tick 発火をまたいで保持され、礼儀 gate（5.2）の参照ソースとして機能する。
#[test]
fn talking_state_persists_across_ticks() {
    let mut driver = SimDriver::new();

    driver.set_talking(true);
    let _ = driver.tick(true);
    let _ = driver.tick(false);
    assert!(
        driver.is_talking(),
        "talking state must persist across tick() calls"
    );

    driver.set_talking(false);
    let _ = driver.tick(true);
    assert!(
        !driver.is_talking(),
        "cleared talking state must persist across tick() calls"
    );
}
