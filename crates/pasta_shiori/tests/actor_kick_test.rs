//! シーンキック: Rust↔Lua ブリッジ × executor アーム統合テスト
//! （仕様 `pasta-scene-kick` task 1.2 + 3.2・requirements 3.1）。
//!
//! # 何を証明するか
//! - **ブリッジ**（`PastaShiori::kick`）: VM の Lua グローバル `SHIORI.kick(scene)` を
//!   protected call で呼び、`STORE.kick_pending`（保留シーン名）と `STORE.kick_force`
//!   （割り込み許可）を設置する。連続キックは最後の scene が `kick_pending` を上書きする。
//! - **executor アーム**（`ActorMsg::Kick` → `shiori.kick`）: アクタースレッド上で
//!   fire-and-forget（応答経路なし・NOTIFY 同型）に VM へ届き、複数 Kick が同一 FIFO 上で
//!   投入順に処理され、最後の scene が `kick_pending` に残る。
//! - **ベストエフォート**: `SHIORI.kick` 不在でもアクタースレッドが落ちない（protected call）。

use std::path::{Path, PathBuf};
use std::time::Duration;

use pasta::actor::mailbox::{mailbox, ActorMsg, MailboxRequest, Reply};
use pasta::actor::thread::spawn_actor_thread;
use pasta::{PastaShiori, Shiori};
use tempfile::TempDir;

/// 開発セッションが export した DAP debug env を main 前に中和する（写経元:
/// `actor_thread_vm_test.rs` / `common/mod.rs`）。固定ポート衝突による偽陰性を防ぐ。
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

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

/// 本番 `pasta_scripts`（実 `SHIORI.kick` / `KICK.install` / `STORE` を含む）を temp へ
/// 展開し、`scripts/pasta/shiori/entry.lua` を **ユーザーオーバーライド**で上書きする。
/// オーバーライド entry.lua は実 `pasta.shiori.event.kick` へ委譲する `SHIORI.kick` を
/// 公開しつつ、`SHIORI.request` で `STORE.kick_pending`/`STORE.kick_force` を Value
/// ヘッダにエコーして読み戻せるようにする（kick 設置の観測点）。
fn build_kick_dir() -> (PathBuf, TempDir) {
    let temp = TempDir::new().expect("create temp dir");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // 本番 pasta_scripts（store.lua / event/kick.lua 等の実体）を展開。
    let pasta_scripts_src = manifest_dir
        .parent()
        .expect("parent dir")
        .join("pasta_lua")
        .join("pasta_scripts");
    let pasta_scripts_dst = temp.path().join("pasta_scripts");
    std::fs::create_dir_all(&pasta_scripts_dst).expect("create pasta_scripts dir");
    copy_dir_recursive(&pasta_scripts_src, &pasta_scripts_dst).expect("copy pasta_scripts");

    // ローダが要求する pasta.toml（async_callback フィクスチャに倣う最小 [loader]）。
    std::fs::write(
        temp.path().join("pasta.toml"),
        "[loader]\ndebug_mode = true\n",
    )
    .expect("write pasta.toml");

    // ユーザーオーバーライド entry.lua（scripts/ は pasta_scripts/ より先に検索される）。
    // 実 KICK モジュールへ委譲しつつ、request で STORE 状態をエコーする。
    let entry_dir = temp.path().join("scripts/pasta/shiori");
    std::fs::create_dir_all(&entry_dir).expect("create scripts/pasta/shiori dir");
    std::fs::write(
        entry_dir.join("entry.lua"),
        r#"
local KICK = require("pasta.shiori.event.kick")
local STORE = require("pasta.store")

SHIORI = SHIORI or {}

function SHIORI.load(hinst, load_dir)
    return true
end

-- 実 KICK.install へ委譲（保留フラグ設置のみ）。本番 entry.lua と同一機構。
function SHIORI.kick(scene)
    KICK.install(scene)
end

-- STORE.kick_pending / kick_force を Value ヘッダへエコー（kick 設置の読み戻し点）。
function SHIORI.request(req)
    local pending = STORE.kick_pending
    local force = STORE.kick_force
    local value = string.format(
        "kick_pending=%s,kick_force=%s",
        tostring(pending), tostring(force)
    )
    return "SHIORI/3.0 200 OK\r\n" ..
        "Charset: UTF-8\r\n" ..
        "Value: " .. value .. "\r\n" ..
        "\r\n"
end

function SHIORI.unload()
end

return SHIORI
"#,
    )
    .expect("write override entry.lua");

    (temp.path().to_path_buf(), temp)
}

/// Lua eval で `STORE.kick_pending` / `STORE.kick_force` を読み戻す（ブリッジ直呼び検証）。
fn read_kick_state(shiori: &PastaShiori) -> String {
    let runtime = shiori.runtime().expect("runtime should exist");
    let lua = runtime.lua();
    lua.load(
        r#"
        local STORE = require("pasta.store")
        return string.format(
            "kick_pending=%s,kick_force=%s",
            tostring(STORE.kick_pending), tostring(STORE.kick_force)
        )
    "#,
    )
    .eval::<String>()
    .expect("eval STORE kick state")
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

fn get_via_actor(tx: &flume::Sender<ActorMsg>, seq: u64, raw_request: &str) -> String {
    let (msg, reply_rx) = ActorMsg::get(MailboxRequest::new(seq, normalize_request(raw_request)));
    tx.send(msg).expect("send GET into mailbox");
    match reply_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("GET reply must arrive within timeout")
    {
        Reply::Value(v) => v,
    }
}

/// ブリッジ直呼び（actor 不要）: `PastaShiori::kick(scene)` が VM の `SHIORI.kick` を
/// 呼び、`STORE.kick_pending`/`STORE.kick_force` を設置する。連続キックは last-wins。
#[test]
fn pasta_shiori_kick_installs_pending_flags_last_wins() {
    let (load_dir, _temp) = build_kick_dir();
    let mut shiori = PastaShiori::default();
    assert!(
        shiori.load(0, load_dir.as_os_str()).unwrap(),
        "ghost VM must load"
    );

    // 初期状態: kick_pending は nil（未設置）。
    assert_eq!(
        read_kick_state(&shiori),
        "kick_pending=nil,kick_force=false",
        "before any kick, pending must be unset"
    );

    // 単発キック: pending=intro, force=true が設置される。
    shiori.kick("intro");
    assert_eq!(
        read_kick_state(&shiori),
        "kick_pending=intro,kick_force=true",
        "kick must install pending scene and force flag"
    );

    // 連続キック: 最後の scene が kick_pending を上書きする（last-wins）。
    shiori.kick("middle");
    shiori.kick("outro");
    assert_eq!(
        read_kick_state(&shiori),
        "kick_pending=outro,kick_force=true",
        "consecutive kicks must overwrite pending with the last scene"
    );
}

/// ブリッジのベストエフォート: `SHIORI.kick` が存在しない VM へ `kick` を呼んでも
/// パニックせず握りつぶす（protected call・アクタースレッドを落とさない根拠）。
#[test]
fn pasta_shiori_kick_swallows_missing_function() {
    let (load_dir, _temp) = build_kick_dir();

    // SHIORI.kick を持たない entry.lua で上書き（request のみ提供）。
    let entry_path = load_dir.join("scripts/pasta/shiori/entry.lua");
    std::fs::write(
        &entry_path,
        r#"
SHIORI = SHIORI or {}
function SHIORI.load(hinst, load_dir) return true end
function SHIORI.request(req)
    return "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\n\r\n"
end
function SHIORI.unload() end
return SHIORI
"#,
    )
    .unwrap();

    let mut shiori = PastaShiori::default();
    assert!(shiori.load(0, load_dir.as_os_str()).unwrap());

    // SHIORI.kick 不在でも panic せず戻る（best-effort・swallow）。
    shiori.kick("intro");
}

/// executor アーム: アクタースレッドへ `ActorMsg::Kick` を fire-and-forget で投入すると
/// VM の `SHIORI.kick` が呼ばれ `STORE.kick_pending` が設置される。連続 Kick は同一
/// FIFO 上で投入順に処理され、最後の scene が kick_pending に残る（後続 GET で観測）。
#[test]
fn actor_kick_arm_installs_pending_last_wins() {
    let (load_dir, _temp) = build_kick_dir();

    let (tx, rx) = mailbox();
    let actor = spawn_actor_thread(0, load_dir, rx);
    assert!(actor.loaded(), "actor thread must load the ghost VM");

    // 連続 Kick（応答経路なし・即投入）。同一 FIFO 上で投入順に処理される。
    tx.send(ActorMsg::kick("intro")).expect("send Kick(intro)");
    tx.send(ActorMsg::kick("outro")).expect("send Kick(outro)");

    // 後続 GET は同一 FIFO で Kick 処理完了後に返り、STORE 状態をエコーする。
    let resp = get_via_actor(&tx, 1, "GET SHIORI/3.0\nCharset: UTF-8\nID: OnObserve\n");
    assert!(
        resp.contains("kick_pending=outro"),
        "last Kick scene must remain in kick_pending (FIFO last-wins): {resp:?}"
    );
    assert!(
        resp.contains("kick_force=true"),
        "Kick must set kick_force: {resp:?}"
    );

    // clean stop。
    let (stop, done_rx) = ActorMsg::stop();
    tx.send(stop).expect("send Stop");
    done_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("Stop must be acked");
    actor.join().expect("actor thread must join cleanly");
}

/// executor アームのベストエフォート: `SHIORI.kick` 不在 VM へ `ActorMsg::Kick` を
/// 送ってもアクタースレッドは落ちず、後続 GET が同一 FIFO で正常に返る。
#[test]
fn actor_kick_arm_survives_missing_kick_function() {
    let (load_dir, _temp) = build_kick_dir();
    let entry_path = load_dir.join("scripts/pasta/shiori/entry.lua");
    std::fs::write(
        &entry_path,
        r#"
SHIORI = SHIORI or {}
function SHIORI.load(hinst, load_dir) return true end
function SHIORI.request(req)
    return "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nValue: alive\r\n\r\n"
end
function SHIORI.unload() end
return SHIORI
"#,
    )
    .unwrap();

    let (tx, rx) = mailbox();
    let actor = spawn_actor_thread(0, load_dir, rx);
    assert!(actor.loaded());

    tx.send(ActorMsg::kick("intro")).expect("send Kick");

    // アクタースレッドは生存し、後続 GET が同一 FIFO で返る。
    let resp = get_via_actor(&tx, 1, "GET SHIORI/3.0\nCharset: UTF-8\nID: OnObserve\n");
    assert!(
        resp.contains("alive"),
        "actor thread must survive Kick with missing SHIORI.kick: {resp:?}"
    );

    let (stop, done_rx) = ActorMsg::stop();
    tx.send(stop).expect("send Stop");
    done_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("Stop must be acked");
    actor.join().expect("actor thread must join cleanly");
}
