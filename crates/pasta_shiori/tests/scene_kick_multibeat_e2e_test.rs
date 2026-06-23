//! マルチビートシーンキック E2E 結合テスト（仕様 `pasta-scene-kick` task 7.2・
//! requirements 4.1/4.4/5.5・design「Testing Strategy / マルチビート配信（最重要・
//! ユーザー懸念）」）。
//!
//! # 何を貫通実証するか（設計の最重要検証）
//! 設計ディスカッションで「完成さくらを貯める talk FIFO はマルチ yield と二重発火で破綻」と
//! 判明し、**既存 `co_scene` 継続機構の流用**に確定した。本テストはその正しさを E2E で固定する:
//!  - **初回ビートは kick で割り込み配信**（`Status: talking` でも `kick_force` ワンショット
//!    突破・R5.5）。
//!  - **残りビートは既存 `check_talk` 継続（`STORE.co_scene`）で 1 tick 1 ビートずつ順序どおり**
//!    （R4.1）。`is_blocked` に従うビート間ペース（継続 tick は idle で進める）。
//!
//! # 単一テスト・全フェーズ直列（`static MAILBOX` 共有のため）
//! `lifecycle::MAILBOX` はプロセスグローバル。6.1（`scene_kick_e2e_test.rs`）と同方針で、本
//! ファイルは **1 つの `#[test]`** に全フェーズを直列に詰める（別バイナリゆえ MAILBOX は隔離）。
//!
//! # 決定論性の担保（X-Pasta-Time による時刻注入＋固定 talk 間隔）
//! 継続ビート（2 ビート目以降）は `check_talk` が `current_unix >= next_talk_time` のときのみ
//! `STORE.co_scene` を継続 resume する（`next_talk_time` は talk 間隔で前進）。実 wall-clock では
//! tick 間隔が秒オーダーで間に合わないため、本テストは:
//!  1. コピーした `pasta.toml` で `talk_interval_min = talk_interval_max = 10`（固定間隔・
//!     `get_config` の下限クランプ 10 に一致）に上書きし、間隔を決定論化する。
//!  2. 各 OnSecondChange GET に `X-Pasta-Time`（RFC3339・`lua_request.rs` が `req.date` を上書き）
//!     を載せ、`next_talk_time` を確実に跨ぐ固定時刻で tick を前進させる。
//!
//! これにより各ビートが固定文字列で 1 tick 1 ビートずつ順序どおり配信されることを乱数非依存で
//! 検証できる。

use std::path::{Path, PathBuf};

use pasta::actor::lifecycle::{marshal_request, spawn_actor, teardown_actor};
use pasta_lua::debug::KickRequest;
use tempfile::TempDir;

/// 開発セッションが export した `PASTA_DEBUG`／`PASTA_DEBUG_PORT` を main 前に中和する
/// （固定ポート衝突による偽陰性を防ぐ・写経元: `scene_kick_e2e_test.rs`）。
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

/// マルチビートのキック対象シーン名（サンプルに存在しない一意名）。
const KICK_SCENE: &str = "KickMultiBeat";

/// 各ビートの決定論的な固定文字列。`＞チェイントーク` で 3 ビートに分割し、ビート N は
/// `BEAT_TEXTS[N-1]` を talk する。順序逆転・二重出力・stuck を厳密に判定するため互いに
/// 部分文字列にならない一意な文字列にする。
const BEAT_TEXTS: [&str; 3] = [
    "キック初回ビート甲",
    "キック継続ビート乙",
    "キック継続ビート丙",
];

/// 固定 talk 間隔（秒）。`get_config` の下限クランプ（10）に一致させ、上書きが確実に効く
/// 値を使う。`X-Pasta-Time` の前進量はこの倍数で組む。
const TALK_INTERVAL_SECS: i64 = 10;

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !src.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());
        if path.is_dir() {
            if entry.file_name() == "profile" {
                continue;
            }
            std::fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            std::fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

/// サンプルゴースト（hello-pasta）を temp へ展開し、決定論的なマルチビート（3 ビート）の
/// キック対象シーンを `dic/` に追加する。さらに `pasta.toml` の talk 間隔を固定値へ上書きする。
fn build_multibeat_kick_ghost_dir() -> (PathBuf, TempDir) {
    let temp = TempDir::new().expect("create temp dir");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let sample_ghost_dir = manifest_dir
        .parent()
        .expect("parent dir")
        .join("pasta_sample_ghost/ghosts/hello-pasta/ghost/master");
    copy_dir_recursive(&sample_ghost_dir, temp.path()).expect("copy hello-pasta ghost");

    // マルチビート（3 ビート）キックシーンを追加（dic/*.pasta は loader が自動読み込み）。
    // `＞チェイントーク` 各 1 回で 1 yield（=1 ビート境界）。各ビートは固定文字列のみを talk。
    // 全角空白で始まる .pasta 行はリテラル継続を避け join で組む（写経元 6.1 と同方針）。
    let kick_scene_pasta = [
        "＃ kick_multibeat.pasta - task 7.2 E2E 用の決定論的マルチビートキック対象シーン"
            .to_string(),
        format!("＊{KICK_SCENE}"),
        format!("　女の子：＠通常　{}", BEAT_TEXTS[0]),
        "　＞チェイントーク".to_string(),
        format!("　女の子：＠通常　{}", BEAT_TEXTS[1]),
        "　＞チェイントーク".to_string(),
        format!("　女の子：＠通常　{}", BEAT_TEXTS[2]),
        String::new(),
    ]
    .join("\n");
    std::fs::write(
        temp.path().join("dic").join("kick_multibeat.pasta"),
        kick_scene_pasta,
    )
    .expect("write kick_multibeat.pasta");

    // talk 間隔を固定（min == max）にして継続ビートのペースを決定論化する。
    let toml_path = temp.path().join("pasta.toml");
    let toml = std::fs::read_to_string(&toml_path).expect("read pasta.toml");
    let toml = toml
        .replace(
            "talk_interval_min = 180  # 最小トーク間隔（デフォルト: 180）",
            &format!("talk_interval_min = {TALK_INTERVAL_SECS}"),
        )
        .replace(
            "talk_interval_max = 300  # 最大トーク間隔（デフォルト: 300）",
            &format!("talk_interval_max = {TALK_INTERVAL_SECS}"),
        );
    assert!(
        toml.contains(&format!("talk_interval_min = {TALK_INTERVAL_SECS}"))
            && toml.contains(&format!("talk_interval_max = {TALK_INTERVAL_SECS}")),
        "pasta.toml talk_interval override must have applied (sample format changed?)"
    );
    std::fs::write(&toml_path, toml).expect("write pasta.toml");

    (temp.path().to_path_buf(), temp)
}

fn normalize_request(text: &str) -> String {
    let trimmed = text.trim_matches(|c| c == '\r' || c == '\n');
    let mut req = trimmed
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n");
    req.push_str("\r\n\r\n");
    req
}

/// 固定 UTC 基準時刻（毎正時から離し、`check_talk` の hour_margin に掛からないようにする）。
/// 12:05:00Z 起点・最大 +60s でも 12:06:00Z で 13:00 まで余裕がある。
fn pasta_time_at(offset_secs: i64) -> String {
    // 2025-07-15T12:05:00Z の unix（基準）。time クレートを引かず固定文字列を秒だけ前進させる。
    // offset_secs は [0, 55] を使う（分の桁を跨がない範囲に収める）。
    let sec = offset_secs;
    assert!(
        (0..60).contains(&sec),
        "offset must stay within the same minute for deterministic RFC3339 formatting"
    );
    format!("2025-07-15T12:05:{sec:02}Z")
}

/// OnSecondChange GET を `marshal_request`（FFI `request` 相当・同一 FIFO）経由で送る。
/// `X-Pasta-Time` で `req.date` を決定論的に固定し、`Status` を呼び出し側指定にする。
fn on_second_change(status: &str, offset_secs: i64) -> String {
    let when = pasta_time_at(offset_secs);
    marshal_request(&normalize_request(&format!(
        "GET SHIORI/3.0\nCharset: UTF-8\nSender: SSP\nSecurityLevel: local\n\
         Status: {status}\nID: OnSecondChange\nReference0: 1\nX-Pasta-Time: {when}\n",
    )))
}

/// 応答さくらスクリプト内に固定文字列が **ちょうど 1 回**現れることを検証する（二重出力検出）。
fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

/// task 7.2: マルチ yield シーンのキック→継続配信を 1 本貫通する E2E 結合テスト。
///
/// 単一テスト・全フェーズ直列（`static MAILBOX` 共有のため）。
#[test]
fn kick_multibeat_scene_delivers_one_beat_per_tick_in_order_end_to_end() {
    let (load_dir, _temp) = build_multibeat_kick_ghost_dir();

    // --- フェーズ 1: load（spawn_actor＝FFI load 相当・kick_sink 束縛＋MAILBOX store） ---
    let loaded = spawn_actor(0, load_dir.clone());
    assert!(
        loaded,
        "spawn_actor must load the hello-pasta ghost VM on the actor thread"
    );

    // --- フェーズ 2: init OnSecondChange（dispatcher 内部タイマ初期化・next_talk_time 設定） ---
    // status=idle・基準時刻 12:05:00。ここで next_talk_time = base + TALK_INTERVAL が設定される。
    let _init = on_second_change("idle", 0);

    // --- フェーズ 3: 実 sink（lifecycle::kick_sink）でマルチビートシーンをキック注入 ---
    let sink = pasta::actor::lifecycle::kick_sink();
    sink(KickRequest {
        scene: KICK_SCENE.to_string(),
    });

    // --- フェーズ 4: tick 1 — 初回ビートが割り込み配信される（talking でも kick_force 突破） ---
    // X-Pasta-Time は base+5（next_talk_time 未到達だがキックフックは check_talk より前段・force 突破）。
    let resp1 = on_second_change("talking", 5);
    assert!(
        resp1.contains("SHIORI/3.0 200 OK"),
        "tick1 (kick first beat) must be delivered as 200 OK.\nactual: {resp1:?}"
    );
    assert_eq!(
        count_occurrences(&resp1, BEAT_TEXTS[0]),
        1,
        "tick1 must carry beat 1 exactly once (interrupt delivery via kick_force).\nactual: {resp1:?}"
    );
    // 順序逆転・二重出力の検出: 後続ビートは tick1 に現れない。
    assert_eq!(
        count_occurrences(&resp1, BEAT_TEXTS[1]),
        0,
        "tick1 must NOT contain beat 2 (no double-output / order inversion).\nactual: {resp1:?}"
    );
    assert_eq!(
        count_occurrences(&resp1, BEAT_TEXTS[2]),
        0,
        "tick1 must NOT contain beat 3 (no double-output / order inversion).\nactual: {resp1:?}"
    );

    // --- フェーズ 5: tick 2 — 既存 check_talk 継続でビート 2 が配信される（idle で継続を進める） ---
    // base+20 >= next_talk_time(base+10)。継続 resume で 2 ビート目が出る。
    let resp2 = on_second_change("idle", 20);
    assert!(
        resp2.contains("SHIORI/3.0 200 OK"),
        "tick2 (continuation beat 2) must be delivered as 200 OK.\nactual: {resp2:?}"
    );
    assert_eq!(
        count_occurrences(&resp2, BEAT_TEXTS[1]),
        1,
        "tick2 must carry beat 2 exactly once (co_scene continuation).\nactual: {resp2:?}"
    );
    assert_eq!(
        count_occurrences(&resp2, BEAT_TEXTS[0]),
        0,
        "tick2 must NOT re-emit beat 1 (no double-output).\nactual: {resp2:?}"
    );
    assert_eq!(
        count_occurrences(&resp2, BEAT_TEXTS[2]),
        0,
        "tick2 must NOT contain beat 3 (strict ordering).\nactual: {resp2:?}"
    );

    // --- フェーズ 6: tick 3 — 継続でビート 3（最終ビート）が配信される ---
    // base+40 >= 前 tick で更新された next_talk_time(base+20+10=base+30)。
    let resp3 = on_second_change("idle", 40);
    assert!(
        resp3.contains("SHIORI/3.0 200 OK"),
        "tick3 (continuation beat 3) must be delivered as 200 OK.\nactual: {resp3:?}"
    );
    assert_eq!(
        count_occurrences(&resp3, BEAT_TEXTS[2]),
        1,
        "tick3 must carry beat 3 exactly once (final continuation beat).\nactual: {resp3:?}"
    );
    assert_eq!(
        count_occurrences(&resp3, BEAT_TEXTS[0]),
        0,
        "tick3 must NOT re-emit beat 1 (no double-output).\nactual: {resp3:?}"
    );
    assert_eq!(
        count_occurrences(&resp3, BEAT_TEXTS[1]),
        0,
        "tick3 must NOT re-emit beat 2 (no double-output).\nactual: {resp3:?}"
    );

    // --- フェーズ 7: tick 4 — 最終ビート後は通常応答へ戻る（co stuck が無い） ---
    // base+55 で継続 co_scene は消費済み（co dead → STORE.co_scene=nil）。キック由来文字列は
    // 一切現れない（stuck していれば最終ビートを再放出するか継続が残る）。
    let resp4 = on_second_change("idle", 55);
    for beat in BEAT_TEXTS {
        assert_eq!(
            count_occurrences(&resp4, beat),
            0,
            "after the last beat, no kicked-scene text may reappear (no co stuck).\nbeat={beat:?}\nactual: {resp4:?}"
        );
    }

    // --- 後始末: teardown ---
    let _ = teardown_actor();
}
