//! preempt-and-abort・割り込みゲート E2E 結合テスト（仕様 `pasta-scene-kick` task 7.3・
//! requirements 5.1/5.2/5.3/5.5・design「Testing Strategy / 即時 preempt-and-abort」
//! ＋ `KickPreempt`/`KickForceGate` コンポーネント）。
//!
//! # 何を貫通実証するか（actor/SHIORI レベルの観測可能な特性化）
//! 既存の Lua ユニット（`virtual_dispatcher_kick_force_test.lua`＝force ワンショット、
//! `virtual_dispatcher_kick_hook_test.lua`＝preempt、`actor_kick_test.rs`＝last-wins）を、
//! **実 sink→MAILBOX→executor→Lua→OnSecondChange の全経路上で観測可能な応答さくらスクリプト
//! として固定**する。3 つの独立シナリオを 1 本の `#[test]` に直列に詰める（`static MAILBOX`
//! はプロセスグローバルゆえ 6.1/7.2 と同方針で別バイナリ・単一テスト）:
//!
//!  1. **preempt-and-abort（R5.1/5.2/5.3）**: 進行中のマルチビート会話（前シーン）を走らせて
//!     初回ビートを出させ、後続ビートを `co_scene` に残す → 別シーンをキック → キックシーンの
//!     beat が割り込み配信され、**前シーンの後続ビートが以後どの tick でも再出しない**（前
//!     `co_scene` が `set_co_scene` 置換で不到達化・自動復帰なし）。
//!  2. **force ワンショット（R5.5）**: `Status: talking` 中にキック → tick1 で初回ビートが
//!     `kick_force` 突破で割り込み配信。**直後 tick も `talking`** なら kick_force は消費済みで
//!     通常どおりブロック（後続ビートを出さない）→ その後 `idle` にすると継続が進む（対比）。
//!  3. **連続キック最新勝ち（R5.1）**: シーンA をキック → 同 tick 前にシーンB をキック →
//!     次 OnSecondChange で **B の beat が出て A は出ない**（`kick_pending` 上書き・A は preempt）。
//!
//! # 決定論性の担保
//! 継続ビート（2 ビート目以降）は `check_talk` が `current_unix >= next_talk_time` のときのみ
//! `STORE.co_scene` を継続 resume する。`pasta.toml` の talk 間隔を固定（min==max==10）に上書きし、
//! 各 OnSecondChange GET に `X-Pasta-Time` を載せて `next_talk_time` を確実に跨ぐ固定時刻で tick を
//! 前進させる（7.2 と同方針）。各シーンの固定文字列は互いに非部分文字列にし、再出・順序逆転・
//! 二重出力を厳密判定する。

use std::path::{Path, PathBuf};

use pasta::actor::lifecycle::{marshal_request, spawn_actor, teardown_actor};
use pasta_lua::debug::KickRequest;
use tempfile::TempDir;

/// 開発セッションが export した `PASTA_DEBUG`／`PASTA_DEBUG_PORT` を main 前に中和する
/// （固定ポート衝突による偽陰性を防ぐ・写経元: `scene_kick_multibeat_e2e_test.rs`）。
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

/// シナリオ 1 の「前シーン」（進行中会話）名と 2 ビート。前シーンは preempt 対象。
const PREV_SCENE: &str = "KickPreemptPrev";
const PREV_BEATS: [&str; 2] = ["前会話初回ビート甲", "前会話後続ビート乙"];

/// シナリオ 1 で前シーンを preempt するキックシーン（単一ビート）。
const PREEMPT_SCENE: &str = "KickPreemptNew";
const PREEMPT_BEAT: &str = "割り込みキックビート丙";

/// シナリオ 2 の force ワンショット検証用キックシーン（2 ビート）。
const FORCE_SCENE: &str = "KickForceOneShot";
const FORCE_BEATS: [&str; 2] = ["force初回ビート丁", "force継続ビート戊"];

/// シナリオ 3 の連続キック（最新勝ち）用シーン。A は preempt され B のみ起動する。
const LASTWINS_A_SCENE: &str = "KickLastWinsA";
const LASTWINS_A_BEAT: &str = "敗北キックA己";
const LASTWINS_B_SCENE: &str = "KickLastWinsB";
const LASTWINS_B_BEAT: &str = "勝利キックB庚";

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

/// 1 行のマルチビート `.pasta` シーンを `＞チェイントーク` で組む。各ビートは固定文字列のみを
/// talk（女の子＝spot0・actors.pasta 共通定義を流用）。全角空白始まりの行はリテラル継続を避け
/// join で組む（写経元 6.1/7.2 と同方針）。
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

/// サンプルゴースト（hello-pasta）を temp へ展開し、本タスクの決定論的シーン群を `dic/` に
/// 追加し、`pasta.toml` の talk 間隔を固定値へ上書きする。
fn build_preempt_kick_ghost_dir() -> (PathBuf, TempDir) {
    let temp = TempDir::new().expect("create temp dir");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let sample_ghost_dir = manifest_dir
        .parent()
        .expect("parent dir")
        .join("pasta_sample_ghost/ghosts/hello-pasta/ghost/master");
    copy_dir_recursive(&sample_ghost_dir, temp.path()).expect("copy hello-pasta ghost");

    // 本タスクの全シーンを 1 ファイルに定義（dic/*.pasta は loader が自動読み込み）。
    let scenes = [
        "＃ kick_preempt.pasta - task 7.3 E2E 用の決定論的 preempt/force/last-wins 検証シーン群"
            .to_string(),
        scene_block(PREV_SCENE, &PREV_BEATS),
        String::new(),
        scene_block(PREEMPT_SCENE, &[PREEMPT_BEAT]),
        String::new(),
        scene_block(FORCE_SCENE, &FORCE_BEATS),
        String::new(),
        scene_block(LASTWINS_A_SCENE, &[LASTWINS_A_BEAT]),
        String::new(),
        scene_block(LASTWINS_B_SCENE, &[LASTWINS_B_BEAT]),
        String::new(),
    ]
    .join("\n");
    std::fs::write(temp.path().join("dic").join("kick_preempt.pasta"), scenes)
        .expect("write kick_preempt.pasta");

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

/// 基準時刻 2025-07-15T12:10:00Z から `offset_secs` 秒だけ前進した RFC3339 文字列を返す。
/// 時の桁（12）は跨がない範囲（offset は [0, 2940]＝最大 12:59:00）に収め、毎正時の
/// hour_margin（30s）にも掛からないようにする。秒/分は手計算で桁上げする（time クレート非依存）。
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
/// `X-Pasta-Time` で `req.date` を決定論的に固定し、`Status` を呼び出し側指定にする。
fn on_second_change(status: &str, offset_secs: i64) -> String {
    let when = pasta_time_at(offset_secs);
    marshal_request(&normalize_request(&format!(
        "GET SHIORI/3.0\nCharset: UTF-8\nSender: SSP\nSecurityLevel: local\n\
         Status: {status}\nID: OnSecondChange\nReference0: 1\nX-Pasta-Time: {when}\n",
    )))
}

/// キック注入（実 sink＝`pasta/playScene` inbound ハンドラと同一クロージャ）。
fn kick(scene: &str) {
    let sink = pasta::actor::lifecycle::kick_sink();
    sink(KickRequest {
        scene: scene.to_string(),
    });
}

/// 応答さくらスクリプト内に固定文字列が現れる回数（二重出力・再出検出）。
fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

/// task 7.3: preempt-and-abort と割り込みゲートを 1 本貫通で特性化する E2E 結合テスト。
///
/// 単一テスト・全シナリオ直列（`static MAILBOX` 共有のため）。
#[test]
fn kick_preempt_and_force_gate_end_to_end() {
    let (load_dir, _temp) = build_preempt_kick_ghost_dir();

    // --- load（spawn_actor＝FFI load 相当・kick_sink 束縛＋MAILBOX store） ---
    let loaded = spawn_actor(0, load_dir.clone());
    assert!(
        loaded,
        "spawn_actor must load the hello-pasta ghost VM on the actor thread"
    );

    // --- init OnSecondChange（dispatcher 内部タイマ初期化・next_talk_time 設定） ---
    // status=idle・基準時刻 12:10:00。ここで next_talk_time = base + TALK_INTERVAL が設定される。
    let _init = on_second_change("idle", 0);

    // =====================================================================
    // シナリオ 1: preempt-and-abort（R5.1/5.2/5.3）
    //  前シーン（2 ビート）をキックして進行中会話を作り、初回ビートを出させて後続を
    //  co_scene に残す。次に別シーンをキックして preempt → 前シーン後続ビートが以後どの
    //  tick でも再出しないことを assert（自動復帰なし）。
    // =====================================================================

    // 前シーンをキック注入。
    kick(PREV_SCENE);

    // tick: 前シーン初回ビートが配信され、後続ビートは co_scene に残る（idle で継続を許す）。
    // base+10 >= next_talk_time(base+10)。
    let s1_prev1 = on_second_change("idle", 10);
    assert!(
        s1_prev1.contains("SHIORI/3.0 200 OK"),
        "scenario1: prev scene first beat must be delivered as 200 OK.\nactual: {s1_prev1:?}"
    );
    assert_eq!(
        count_occurrences(&s1_prev1, PREV_BEATS[0]),
        1,
        "scenario1: prev scene first beat must appear exactly once.\nactual: {s1_prev1:?}"
    );
    assert_eq!(
        count_occurrences(&s1_prev1, PREV_BEATS[1]),
        0,
        "scenario1: prev scene second beat must NOT appear yet (one beat per tick).\nactual: {s1_prev1:?}"
    );

    // ここで別シーンをキック → 進行中の前 co_scene を preempt（set_co_scene 置換）。
    kick(PREEMPT_SCENE);

    // tick: キックシーンの単一ビートが割り込み配信され、前シーン後続ビートは出ない。
    // base+20 >= next_talk_time（前 tick で更新済み）。
    let s1_new = on_second_change("idle", 20);
    assert!(
        s1_new.contains("SHIORI/3.0 200 OK"),
        "scenario1: preempting kick beat must be delivered as 200 OK.\nactual: {s1_new:?}"
    );
    assert_eq!(
        count_occurrences(&s1_new, PREEMPT_BEAT),
        1,
        "scenario1: preempting kick beat must appear exactly once.\nactual: {s1_new:?}"
    );
    assert_eq!(
        count_occurrences(&s1_new, PREV_BEATS[1]),
        0,
        "scenario1: prev scene second beat must NOT appear on the preempting tick \
         (preempt: prev co_scene replaced).\nactual: {s1_new:?}"
    );

    // preempt-and-abort（自動復帰なし）の中核 assert: 以後どの tick でも前シーンの
    // 後続ビートは再出しない（前 co_scene は不到達・自動復帰されない）。
    for (i, off) in [30_i64, 40, 50].iter().enumerate() {
        let resp = on_second_change("idle", *off);
        assert_eq!(
            count_occurrences(&resp, PREV_BEATS[1]),
            0,
            "scenario1: prev scene must NOT auto-resume after preempt (tick #{}, no auto-restore).\nactual: {resp:?}",
            i + 1
        );
        // 念のため前シーン初回ビートも再放出されないこと（co stuck/再起動なし）。
        assert_eq!(
            count_occurrences(&resp, PREV_BEATS[0]),
            0,
            "scenario1: prev scene first beat must NOT reappear after preempt (tick #{}).\nactual: {resp:?}",
            i + 1
        );
    }

    // =====================================================================
    // シナリオ 2: force ワンショット（R5.5）
    //  Status: talking 中にキック → tick1 で初回ビートが kick_force 突破で割り込み配信。
    //  直後 tick も talking なら kick_force は消費済みで通常ブロック（後続ビートを出さない）。
    //  その後 idle にすると継続が進む（対比）。
    // =====================================================================

    kick(FORCE_SCENE);

    // tick1: talking でも kick_force ワンショットで初回ビート割り込み配信。
    // base+60。
    let s2_t1 = on_second_change("talking", 60);
    assert!(
        s2_t1.contains("SHIORI/3.0 200 OK"),
        "scenario2: force first beat must break through talking as 200 OK.\nactual: {s2_t1:?}"
    );
    assert_eq!(
        count_occurrences(&s2_t1, FORCE_BEATS[0]),
        1,
        "scenario2: force first beat must appear exactly once (kick_force one-shot).\nactual: {s2_t1:?}"
    );
    assert_eq!(
        count_occurrences(&s2_t1, FORCE_BEATS[1]),
        0,
        "scenario2: force second beat must NOT appear on tick1 (one beat per tick).\nactual: {s2_t1:?}"
    );

    // tick2: talking のまま・時刻は next_talk_time を跨ぐ（base+80 >= 前 tick 更新の base+70）。
    // kick_force は消費済みなので talking が通常どおりブロック → 後続ビートは出ない。
    // （force が消費されていなければここで継続ビートが割り込み配信され落ちる＝load-bearing）。
    let s2_t2 = on_second_change("talking", 80);
    assert_eq!(
        count_occurrences(&s2_t2, FORCE_BEATS[1]),
        0,
        "scenario2: force is one-shot — second beat must be blocked while still talking \
         (kick_force already consumed).\nactual: {s2_t2:?}"
    );
    assert_eq!(
        count_occurrences(&s2_t2, FORCE_BEATS[0]),
        0,
        "scenario2: first beat must NOT re-emit on tick2.\nactual: {s2_t2:?}"
    );

    // tick3: idle へ切り替えると通常ペースで継続が進み、2 ビート目が配信される（対比）。
    // base+100 >= next_talk_time（前 tick で base+80+10=base+90 に更新済み）。
    let s2_t3 = on_second_change("idle", 100);
    assert!(
        s2_t3.contains("SHIORI/3.0 200 OK"),
        "scenario2: continuation must resume once idle as 200 OK.\nactual: {s2_t3:?}"
    );
    assert_eq!(
        count_occurrences(&s2_t3, FORCE_BEATS[1]),
        1,
        "scenario2: second beat must be delivered once status is idle (continuation resumes).\nactual: {s2_t3:?}"
    );

    // =====================================================================
    // シナリオ 3: 連続キック最新勝ち（R5.1）
    //  シーンA をキック → 同 tick 前にシーンB をキック（kick_pending 上書き）→
    //  次 OnSecondChange で B の beat が出て A は出ない（A は据えられず preempt）。
    // =====================================================================

    // 直前シナリオの継続 co_scene を消化しておく（次 tick で取り違えないよう idle で空にする）。
    // FORCE_SCENE は 2 ビートで tick3 までに消費済み。base+120 でクリーンな通常応答（キック無し）。
    let s3_drain = on_second_change("idle", 120);
    assert_eq!(
        count_occurrences(&s3_drain, FORCE_BEATS[0]) + count_occurrences(&s3_drain, FORCE_BEATS[1]),
        0,
        "scenario3 setup: prior force scene must be fully drained (no stuck co).\nactual: {s3_drain:?}"
    );

    // A をキック直後に B をキック（同一 OnSecondChange より前・dispatch 未実行）。
    // kick_pending は last-wins で B に上書きされる。
    kick(LASTWINS_A_SCENE);
    kick(LASTWINS_B_SCENE);

    // 次 tick: B の beat が出て A は出ない（A は co_scene に据えられず preempt）。
    // base+130 >= next_talk_time。
    let s3 = on_second_change("idle", 130);
    assert!(
        s3.contains("SHIORI/3.0 200 OK"),
        "scenario3: last kick (B) beat must be delivered as 200 OK.\nactual: {s3:?}"
    );
    assert_eq!(
        count_occurrences(&s3, LASTWINS_B_BEAT),
        1,
        "scenario3: last-wins — B's beat must appear exactly once.\nactual: {s3:?}"
    );
    assert_eq!(
        count_occurrences(&s3, LASTWINS_A_BEAT),
        0,
        "scenario3: last-wins — A's beat must NOT appear (overwritten kick_pending, A preempted).\nactual: {s3:?}"
    );

    // A は以後どの tick でも起動しない（kick_pending 上書きで完全に preempt・自動復帰なし）。
    let s3_after = on_second_change("idle", 140);
    assert_eq!(
        count_occurrences(&s3_after, LASTWINS_A_BEAT),
        0,
        "scenario3: A must never start in a later tick (overwritten, no auto-restore).\nactual: {s3_after:?}"
    );

    // --- 後始末: teardown ---
    let _ = teardown_actor();
}
