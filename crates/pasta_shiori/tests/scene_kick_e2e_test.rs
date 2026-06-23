//! シーンキック E2E 結合テスト（仕様 `pasta-scene-kick` task 6.1・requirements 2.4/3.1/4.2）。
//!
//! # 何を貫通実証するか（DAP→sink→MAILBOX→executor→Lua→OnSecondChange）
//! 本機能の全部品は実装済み（2.x decode/inbound/enable、3.x sink＝`MAILBOX` 投函／executor
//! `Kick` 腕、4.x `SHIORI.kick`→`KICK.install`／`try_dispatch`／dispatch 前段フック）であり、
//! 本テストは**それらを跨ぐ全経路を 1 本貫いて「キック→次 OnSecondChange 応答に指定シーンの
//! 初回ビート（さくらスクリプト）が現れる」**ことを実証する。
//!
//! 駆動する経路（本番ライフサイクル自由関数を使用＝ FFI 入口と同一結線）:
//!  1. `spawn_actor`（FFI `load` 相当）: アクタースレッド上で実ゴースト VM をロードし、
//!     `PastaShiori::load` 内で `RuntimeConfig::with_kick_sink(lifecycle::kick_sink())` が
//!     束縛され、`static MAILBOX` にアクターの `Sender` が store される（requirements 2.4）。
//!  2. **実 sink クロージャ** `lifecycle::kick_sink()` を `KickRequest{ scene }` で呼ぶ。これは
//!     `pasta/playScene` inbound ハンドラ（`try_play_scene_kick`）が注入済み sink を呼ぶのと
//!     同一のクロージャであり、`MAILBOX` を lock-free に読み `ActorMsg::Kick` を `try_send` する。
//!     → DAP inbound から sink を呼ぶのと同じ「sink→MAILBOX」段を本番クロージャで踏む。
//!  3. executor `Kick` 腕（`thread.rs`）が同一 FIFO 上で `PastaShiori::kick(scene)`→Lua
//!     `SHIORI.kick(scene)`→`KICK.install` を呼び、`STORE.kick_pending`/`kick_force` を設置する
//!     （requirements 3.1: 受理処理は resume をブロックしない＝フラグ設置のみ）。
//!  4. 続く OnSecondChange GET（同一 FIFO・`marshal_request` 経由）で `virtual_dispatcher.dispatch`
//!     前段のキック起動フックが保留シーンを起動し、初回ビートを GET 応答へ返す（requirements 4.2）。
//!
//! # 停止ループ非経由（停止非依存）の確認
//! キックは `SessionCommand`/stop_loop を一切経由しない。本テストは debug をブレークさせない通常
//! 実行状態（DAP セッション無し・`PASTA_DEBUG` 中和）で、socket-bridge inbound と同型の「sink→
//! MAILBOX」段を本番 sink で踏むことにより、キックが停止ループに依存せず届くことを示す。さらに
//! キック GET は `Status: talking`（通常はブロック対象）で送り、`kick_force` ワンショット突破で
//! 初回ビートが割り込み配信されることを併せて実証する（requirements 5.1/5.5 の force ゲート）。
//!
//! # 単一テスト・直列（`static MAILBOX` 共有のため）
//! `lifecycle::MAILBOX` はプロセスグローバル。`ffi_actor_lifecycle_test.rs` と同方針で、本ファイルは
//! **1 つの `#[test]`** に全フェーズを直列に詰め、同一バイナリ内に他テストを置かない（順序非依存）。

use std::path::{Path, PathBuf};

use pasta::actor::lifecycle::{marshal_request, spawn_actor, teardown_actor};
use pasta_lua::debug::KickRequest;
use tempfile::TempDir;

/// 開発セッションが export した `PASTA_DEBUG`／`PASTA_DEBUG_PORT` を main 前に中和する
/// （固定ポート衝突による偽陰性を防ぐ・写経元: `ffi_actor_lifecycle_test.rs`）。これにより
/// DAP セッションは張られず、キックは「停止ループ非経由・通常実行中」に届くことになる。
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

/// キック対象シーン名。サンプルゴーストに存在しない一意な名前にし、応答内の固定文字列との
/// 一致でキック由来であることを決定論的に判定する。
const KICK_SCENE: &str = "KickE2EProbe";

/// キックシーンが talk する決定論的な固定文字列（応答さくらスクリプトに現れる初回ビート）。
const KICK_BEAT_TEXT: &str = "キックE2E初回ビート確認";

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

/// サンプルゴースト（hello-pasta）を temp へ展開し、決定論的な単一ビートのキック対象シーンを
/// `dic/` に追加する。シーンは固定文字列 1 つだけを talk するため、初回ビートが応答に現れる
/// ことを一致判定できる（ランダムトークの非決定性を避ける）。
fn build_kick_ghost_dir() -> (PathBuf, TempDir) {
    let temp = TempDir::new().expect("create temp dir");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // 実サンプルゴースト（pasta_scripts 実体・実 SCENE/KICK/STORE 機構を含む）を展開。
    let sample_ghost_dir = manifest_dir
        .parent()
        .expect("parent dir")
        .join("pasta_sample_ghost/ghosts/hello-pasta/ghost/master");
    copy_dir_recursive(&sample_ghost_dir, temp.path()).expect("copy hello-pasta ghost");

    // 決定論的なキック対象シーンを追加（dic/*.pasta は loader が自動読み込み）。
    // アクター辞書（女の子＝spot0）は actors.pasta で共通定義済みのため流用する。
    // 単一アクター・単一行＝初回ビートが固定文字列のみで構成される。
    // 行を明示連結する（Rust の `\<改行>` 継続は全角空白 '\u{3000}' を非スキップ警告にするため、
    // 全角空白で始まる .pasta 行はリテラル継続を使わず join で組む）。
    let kick_scene_pasta = [
        "＃ kick_e2e.pasta - task 6.1 E2E 用の決定論的キック対象シーン".to_string(),
        format!("＊{KICK_SCENE}"),
        format!("　女の子：＠通常　{KICK_BEAT_TEXT}"),
        String::new(),
    ]
    .join("\n");
    std::fs::write(temp.path().join("dic").join("kick_e2e.pasta"), kick_scene_pasta)
        .expect("write kick_e2e.pasta");

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

/// OnSecondChange GET を `marshal_request`（FFI `request` 相当・同一 FIFO）経由で送る。
/// `Status` は呼び出し側指定（通常ブロック検証のため `talking` 等を渡せる）。日時は Rust 側で
/// 現在時刻から補完されるため、リクエストには Status と ID のみを載せる。
fn on_second_change(status: &str) -> String {
    marshal_request(&normalize_request(&format!(
        "GET SHIORI/3.0\nCharset: UTF-8\nSender: SSP\nSecurityLevel: local\n\
         Status: {status}\nID: OnSecondChange\nReference0: 1\n",
    )))
}

/// task 6.1: DAP から OnSecondChange までの全経路を 1 本貫通する E2E 結合テスト。
///
/// 単一テスト・全フェーズ直列（`static MAILBOX` 共有のため）。
#[test]
fn kick_to_next_on_second_change_delivers_first_beat_end_to_end() {
    let (load_dir, _temp) = build_kick_ghost_dir();

    // --- フェーズ 1: load（spawn_actor＝FFI load 相当） ---
    // アクタースレッド上で実 VM をロードし、`kick_sink` 束縛＋`static MAILBOX` store を行う。
    let loaded = spawn_actor(0, load_dir.clone());
    assert!(
        loaded,
        "spawn_actor must load the hello-pasta ghost VM on the actor thread"
    );

    // --- フェーズ 2: OnSecondChange を 1 回流して dispatcher の内部タイマを初期化 ---
    // （初回 dispatch は next_hour_unix/next_talk_time を設定し発行スキップ＝キック前段の
    //   通常状態を作る。キックはこの後に注入する。）
    let _init = on_second_change("balloon(0=0)");

    // --- フェーズ 3: 実 sink（lifecycle::kick_sink）でキックを注入 ---
    // これは `pasta/playScene` inbound ハンドラが注入済み sink を呼ぶのと同一クロージャ。
    // sink→MAILBOX.load→ActorMsg::Kick→executor Kick 腕→SHIORI.kick→KICK.install を貫く。
    // 停止ループ（SessionCommand）は一切経由しない（停止非依存・socket-bridge inbound 同型）。
    let sink = pasta::actor::lifecycle::kick_sink();
    sink(KickRequest {
        scene: KICK_SCENE.to_string(),
    });

    // --- フェーズ 4: 次の OnSecondChange GET でキック初回ビートが配信される ---
    // `Status: talking` は通常はブロック対象だが、キックの kick_force ワンショット突破で
    // 割り込み配信される（requirements 5.1/5.5）。GET は同一 FIFO で Kick 処理完了後に返るため、
    // この時点で kick_pending は設置済み＝dispatch 前段フックが保留シーンを起動する。
    let resp = on_second_change("talking");

    // 全経路貫通の証跡: キックシーンの固定初回ビート文字列が応答さくらスクリプトに現れる。
    // （この assert がテストの中核。除去するとキック貫通が検証されなくなる。）
    assert!(
        resp.contains(KICK_BEAT_TEXT),
        "next OnSecondChange after kick must carry the kicked scene's first beat \
         (full path DAP-sink->MAILBOX->executor->Lua->OnSecondChange).\nactual: {resp:?}"
    );
    // 200 OK（さくらスクリプト配信）であることも確認（204 でないこと＝出力が出ている）。
    assert!(
        resp.contains("SHIORI/3.0 200 OK"),
        "kick first beat must be delivered as a 200 OK SakuraScript response.\nactual: {resp:?}"
    );

    // --- 後始末: teardown（プロセス終了でアクターが漏れないように） ---
    let _ = teardown_actor();
}
