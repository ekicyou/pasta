//! debug 無効・未解決・drop の各ゲート検証 E2E 結合テスト（仕様 `pasta-scene-kick` task 7.4・
//! requirements 2.6 / 3.5・design「Testing Strategy（debug 無効ゲート／未解決／drop）」
//! ＋ Error Handling（`kick.unresolved` / `kick.drop`）・`MailboxKickInjector`（D6 no-op））。
//!
//! # 何を観測可能に固定するか（actor/SHIORI レベルの E2E ゲート）
//! 既存ユニットを **跨ぐ全経路（実 sink→MAILBOX→executor→Lua→OnSecondChange）上で観測可能な
//! 応答さくらスクリプト／ライフサイクル不変**として補完する。本ファイルは 3 ゲートのうち
//! actor/SHIORI レベルで最も自然に観測できる 2 つ（未解決破棄・teardown 競合 drop）を 1 本の
//! `#[test]` に直列に詰める（`static MAILBOX` はプロセスグローバルゆえ 6.1/7.2/7.3 と同方針で
//! 別バイナリ・単一テスト）。残る「debug 無効＝非活性（R2.6）」は層が `pasta_lua` debug の
//! enable ゲートなので、そちらの in-crate テスト（`disabled_enable_keeps_wired_kick_sink_inert`）で
//! 観測する（task 7.4 scope の層分担に従う）。
//!
//!  - **未解決シーン（R3.5）**: 存在しないシーン名をキック → アクター/VM で `KICK.try_dispatch`
//!    が nil → `co_scene` を据えず（進行中会話が不変）破棄＋診断ログ。観測: 未解決キック後の
//!    OnSecondChange が **通常応答（キックビート無し・進行中会話の継続が不変）** であること。
//!    進行中のマルチビート会話を先に走らせておき、未解決キックがそれを一切変えない（後続ビートが
//!    通常ペースでそのまま継続する）ことで「co_scene 不変」を観測可能にする。
//!  - **teardown 競合 drop（R3.5 / D6）**: `teardown_actor()`（`MAILBOX.swap(None)`）後に実 sink
//!    クロージャ（`kick_sink()`）を呼ぶ → **panic せず** no-op（MAILBOX 不在で送信されない・
//!    `kick.drop` ベストエフォート）。観測: panic しないこと＋MAILBOX が None のまま（後続の
//!    `marshal_request` がアクター不在の 204 を返すことで MAILBOX 不在を観測）。

use std::path::{Path, PathBuf};

use pasta::actor::lifecycle::{kick_sink, marshal_request, spawn_actor, teardown_actor};
use pasta_lua::debug::KickRequest;
use tempfile::TempDir;

/// 開発セッションが export した `PASTA_DEBUG`／`PASTA_DEBUG_PORT` を main 前に中和する
/// （固定ポート衝突による偽陰性を防ぐ・写経元: `scene_kick_preempt_e2e_test.rs`）。
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

/// 進行中会話（2 ビート）の名前とビート。未解決キックがこの継続を一切変えないことを観測する。
const PREV_SCENE: &str = "GatePrevScene";
const PREV_BEATS: [&str; 2] = ["ゲート前会話初回ビート甲", "ゲート前会話後続ビート乙"];

/// 存在しない（dic に定義しない）一意なシーン名。`KICK.try_dispatch` が nil を返す。
const UNRESOLVED_SCENE: &str = "GateUnresolvedSceneDoesNotExist";

/// teardown drop ゲート用のキックシーン名（実体は問わない・送信されないため）。
const DROP_SCENE: &str = "GateDropSceneAfterTeardown";

/// 固定 talk 間隔（秒）。`get_config` の下限クランプ（10）に一致させ上書きが確実に効く値。
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

/// 1 行のマルチビート `.pasta` シーンを `＞チェイントーク` で組む（写経元 7.3 と同方針）。
fn scene_block(name: &str, beats: &[&str]) -> String {
    let mut lines = vec![format!("＊{name}")];
    for (i, beat) in beats.iter().enumerate() {
        if i > 0 {
            lines.push("　＞チェイントーク".to_string());
        }
        lines.push(format!("　女の子：＠通常　{beat}"));
    }
    lines.join("\n")
}

/// サンプルゴースト（hello-pasta）を temp へ展開し、進行中会話用シーンを `dic/` に追加し、
/// `pasta.toml` の talk 間隔を固定値へ上書きする（継続ビートのペースを決定論化）。
fn build_gate_ghost_dir() -> (PathBuf, TempDir) {
    let temp = TempDir::new().expect("create temp dir");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let sample_ghost_dir = manifest_dir
        .parent()
        .expect("parent dir")
        .join("pasta_sample_ghost/ghosts/hello-pasta/ghost/master");
    copy_dir_recursive(&sample_ghost_dir, temp.path()).expect("copy hello-pasta ghost");

    // 進行中会話用の 2 ビートシーンを追加（未解決シーンは意図的に定義しない）。
    let scenes = [
        "＃ kick_gate.pasta - task 7.4 ゲート検証用の決定論的進行中会話シーン".to_string(),
        scene_block(PREV_SCENE, &PREV_BEATS),
        String::new(),
    ]
    .join("\n");
    std::fs::write(temp.path().join("dic").join("kick_gate.pasta"), scenes)
        .expect("write kick_gate.pasta");

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

/// 基準時刻 2025-07-15T12:10:00Z から `offset_secs` 秒だけ前進した RFC3339 文字列を返す
/// （写経元 7.3・time クレート非依存の手計算桁上げ・hour 12 内）。
fn pasta_time_at(offset_secs: i64) -> String {
    assert!(
        (0..=2940).contains(&offset_secs),
        "offset must stay within hour 12 for deterministic RFC3339 formatting"
    );
    let total = 600 + offset_secs; // 12:10:00 起点の「時内秒数」（10*60=600）
    let min = total / 60;
    let sec = total % 60;
    assert!(min < 60, "must stay within the same hour");
    format!("2025-07-15T12:{min:02}:{sec:02}Z")
}

/// OnSecondChange GET を `marshal_request`（FFI `request` 相当・同一 FIFO）経由で送る。
fn on_second_change(status: &str, offset_secs: i64) -> String {
    let when = pasta_time_at(offset_secs);
    marshal_request(&normalize_request(&format!(
        "GET SHIORI/3.0\nCharset: UTF-8\nSender: SSP\nSecurityLevel: local\n\
         Status: {status}\nID: OnSecondChange\nReference0: 1\nX-Pasta-Time: {when}\n",
    )))
}

/// キック注入（実 sink＝`pasta/playScene` inbound ハンドラと同一クロージャ）。
fn kick(scene: &str) {
    let sink = kick_sink();
    sink(KickRequest {
        scene: scene.to_string(),
    });
}

/// 応答さくらスクリプト内に固定文字列が現れる回数。
fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

/// task 7.4: 未解決破棄ゲート（R3.5）と teardown 競合 drop ゲート（R3.5 / D6）を 1 本貫通で
/// 観測する E2E 結合テスト。単一テスト・全ゲート直列（`static MAILBOX` 共有のため）。
#[test]
fn kick_unresolved_and_teardown_drop_gates_end_to_end() {
    let (load_dir, _temp) = build_gate_ghost_dir();

    // --- load（spawn_actor＝FFI load 相当・kick_sink 束縛＋MAILBOX store） ---
    let loaded = spawn_actor(0, load_dir.clone());
    assert!(
        loaded,
        "spawn_actor must load the hello-pasta ghost VM on the actor thread"
    );

    // --- init OnSecondChange（dispatcher 内部タイマ初期化・next_talk_time 設定） ---
    let _init = on_second_change("idle", 0);

    // =====================================================================
    // ゲート A: 未解決シーンは進行中シーンを変えず破棄＋ログ（R3.5）
    //  進行中のマルチビート会話を先に走らせ初回ビートを出させる → 後続ビートを co_scene に
    //  残した状態で存在しないシーンをキック → 次 tick が **通常継続応答（前会話の後続ビートが
    //  そのまま出る・キックビート無し・co_scene 不変）** であることを観測。未解決キックで
    //  co_scene が据え替われば前会話後続ビートが消える＝load-bearing。
    // =====================================================================

    // 進行中会話をキック注入（これは実在シーンなので co_scene に据わる）。
    kick(PREV_SCENE);

    // tick: 前会話初回ビートが配信され、後続ビートは co_scene に残る。
    let a_prev1 = on_second_change("idle", 10);
    assert!(
        a_prev1.contains("SHIORI/3.0 200 OK"),
        "gateA: prev conversation first beat must be delivered as 200 OK.\nactual: {a_prev1:?}"
    );
    assert_eq!(
        count_occurrences(&a_prev1, PREV_BEATS[0]),
        1,
        "gateA: prev first beat must appear exactly once.\nactual: {a_prev1:?}"
    );
    assert_eq!(
        count_occurrences(&a_prev1, PREV_BEATS[1]),
        0,
        "gateA: prev second beat must NOT appear yet (one beat per tick).\nactual: {a_prev1:?}"
    );

    // ここで **存在しないシーン** をキック → `KICK.try_dispatch` が nil → co_scene 不変・破棄。
    kick(UNRESOLVED_SCENE);

    // tick: 未解決キックは進行中会話を変えない＝前会話の **後続ビートがそのまま通常継続** で出る。
    let a_after = on_second_change("idle", 20);
    assert!(
        a_after.contains("SHIORI/3.0 200 OK"),
        "gateA: prev conversation must keep continuing as 200 OK after unresolved kick.\nactual: {a_after:?}"
    );
    assert_eq!(
        count_occurrences(&a_after, PREV_BEATS[1]),
        1,
        "gateA: unresolved kick must NOT change the in-progress scene — prev second beat \
         continues normally (co_scene unchanged, unresolved kick discarded).\nactual: {a_after:?}"
    );
    // 未解決シーン名そのものが応答に現れない（キックビートが生成されていない）。
    assert_eq!(
        count_occurrences(&a_after, UNRESOLVED_SCENE),
        0,
        "gateA: unresolved scene must produce no kick beat in the response.\nactual: {a_after:?}"
    );

    // =====================================================================
    // ゲート B: teardown 競合 drop（R3.5 / D6）
    //  teardown_actor()（MAILBOX.swap(None)）後に実 sink クロージャを呼ぶ → panic せず no-op
    //  （MAILBOX 不在で送信されない・kick.drop ベストエフォート）。観測: panic しないこと＋
    //  MAILBOX が None のまま（teardown 後の marshal_request がアクター不在の 204 を返す）。
    // =====================================================================

    // teardown（MAILBOX.swap(None)）= アクター不在状態を作る。
    let _ = teardown_actor();

    // teardown 後に実 sink を呼ぶ → panic せず黙って破棄される（kick.drop・ベストエフォート）。
    // （この行が panic すれば drop ゲートが破れている＝load-bearing。）
    kick(DROP_SCENE);

    // 観測: MAILBOX が None のまま（送信されていない）＝アクター不在 → marshal_request は 204。
    let after_drop = on_second_change("idle", 30);
    assert!(
        after_drop.contains("SHIORI/3.0 204 No Content"),
        "gateB: after teardown the actor is absent — kick must be dropped (no-op) and \
         marshal_request must return 204 (MAILBOX stays None).\nactual: {after_drop:?}"
    );

    // 二重 teardown でもハング/panic しない（冪等・後始末）。
    let _ = teardown_actor();
}
