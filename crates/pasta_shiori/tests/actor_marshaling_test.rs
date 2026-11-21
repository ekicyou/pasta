//! 本番 marshaling × 実 `!Send` VM アクタースレッドの統合テスト。既定（no-feature）ビルドで走る。
//!
//! 実アクタースレッド（VM pin）に対し、`marshal_get`/`marshal_notify` を通して:
//!  (a) GET が実 VM 応答値をタイムアウト内に返し、応答バイト列が byte-invariant ゴールデンと不変であること。
//!  (b) NOTIFY が即 204 を返し、アクターの完了を待たないこと。
//!  (c) GET property→callback の 2 ラウンドでコルーチンが tick をまたいで継続し、各ラウンドの
//!      応答がゴールデンと不変であること。
//!  (d) drop→204（応答を返せない異常で reply drop）が無限待機なく 204 になること。
//!      リクエスト処理エラーは drop ではなく 500 として返る（(f)）。
//!  (e) アクター不在（mailbox 閉鎖）の GET/NOTIFY が即 204 になること。
//!
//! # GET_TIMEOUT の扱い
//! 通常経路の (a)/(c) の厳密なゴールデン一致は `marshal_get_with_timeout` に十分大きい閾値を
//! 与えて検証する（デバッグビルド + コールド VM でも取りこぼさないため）。本番 `GET_TIMEOUT`
//! 経路は別テストで確認し、タイムアウト→204 の決定論的検証は marshaling.rs のユニットテストにある。

use std::path::{Path, PathBuf};
use std::time::Duration;

use pasta::actor::mailbox::{ActorMsg, MailboxRequest, mailbox};
use pasta::actor::marshaling::{
    GET_TIMEOUT, default_204, marshal_get, marshal_get_with_timeout, marshal_notify,
};
use pasta::actor::thread::spawn_actor_thread_with;
use tempfile::TempDir;

/// 開発セッションが export した `PASTA_DEBUG` を main 前
/// （プロセスが単一スレッドの間）に中和する。写経元: `common/mod.rs`。
#[ctor::ctor(unsafe)]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
    }
}

/// 大きめ閾値（環境非依存に実 VM 応答を取りこぼさない）。
const GENEROUS: Duration = Duration::from_secs(10);

// Round1/Round2 期待バイト列（byte_invariant_test の特性化ゴールデンと同一）。
const EXPECT_ROUND1: &str = "SHIORI/3.0 200 OK\r\n\
Charset: UTF-8\r\n\
Sender: Pasta\r\n\
SecurityLevel: local\r\n\
Value: \\![get,property,OnPastaCallBack1,baseware.version]\\e\r\n\
\r\n";

const EXPECT_ROUND2: &str = "SHIORI/3.0 200 OK\r\n\
Charset: UTF-8\r\n\
Sender: Pasta\r\n\
SecurityLevel: local\r\n\
Value: version=2.6.77\\e\\e\r\n\
\r\n";

/// async_callback フィクスチャを temp へ展開し `load_dir` を返す。写経元:
/// `actor_thread_vm_test.rs::build_async_callback_dir`。
fn build_async_callback_dir() -> (PathBuf, TempDir) {
    let temp = TempDir::new().expect("create temp dir");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let fixture_src = manifest_dir.join("tests/fixtures/async_callback");
    copy_dir_recursive(&fixture_src, temp.path()).expect("copy fixture");

    (temp.path().to_path_buf(), temp)
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

/// SHIORI/3.0 リクエストを正規化（改行→CRLF・終端付与）する。写経元:
/// `actor_thread_vm_test.rs::normalize_request`。
fn normalize_request(text: &str) -> String {
    let trimmed = text.trim_matches(|c| c == '\r' || c == '\n');
    let mut req = trimmed
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n");
    req.push_str("\r\n\r\n");
    req
}

fn stop(tx: &flume::Sender<ActorMsg>) {
    let (msg, done_rx) = ActorMsg::stop();
    tx.send(msg).expect("send Stop");
    done_rx.recv_timeout(GENEROUS).expect("Stop must be acked");
}

/// (a)(c) 本番 marshaling 経由の GET が、実 VM 応答をゴールデンと不変に返し、
/// コルーチンが tick をまたいで継続する。
#[test]
fn marshal_get_through_actor_returns_byte_invariant_golden_and_continues_coroutine() {
    let (load_dir, _temp) = build_async_callback_dir();
    let (tx, rx) = mailbox();
    let actor = spawn_actor_thread_with(load_dir, rx, None).expect("actor thread must spawn");
    assert!(actor.loaded(), "actor thread must load the ghost VM");

    let round1 = marshal_get_with_timeout(
        &tx,
        MailboxRequest::new(
            1,
            normalize_request("GET SHIORI/3.0\nCharset: UTF-8\nID: OnTestSimple\n"),
        ),
        GENEROUS,
    );
    assert_eq!(
        round1.as_bytes(),
        EXPECT_ROUND1.as_bytes(),
        "round1 marshaled GET must be byte-invariant with the golden\nactual: {round1:?}"
    );

    // Round2: callback でコルーチン resume・値反映（co_scene が VM 内に persist）。
    let round2 = marshal_get_with_timeout(
        &tx,
        MailboxRequest::new(
            2,
            normalize_request(
                "GET SHIORI/3.0\nCharset: UTF-8\nID: OnPastaCallBack1\nReference0: 2.6.77\n",
            ),
        ),
        GENEROUS,
    );
    assert_eq!(
        round2.as_bytes(),
        EXPECT_ROUND2.as_bytes(),
        "round2 marshaled GET must resume the coroutine and be byte-invariant\nactual: {round2:?}"
    );

    stop(&tx);
    actor
        .join()
        .expect("actor thread must join cleanly after Stop");
}

/// (b) NOTIFY は即 204 を返し、アクターの完了を待たない。後続 GET は同一 FIFO で
/// NOTIFY 処理完了後に返る。
#[test]
fn marshal_notify_through_actor_returns_204_then_get_in_fifo() {
    let (load_dir, _temp) = build_async_callback_dir();
    let (tx, rx) = mailbox();
    let actor = spawn_actor_thread_with(load_dir, rx, None).expect("actor thread must spawn");
    assert!(actor.loaded(), "actor thread must load the ghost VM");

    let notify_resp = marshal_notify(
        &tx,
        MailboxRequest::new(
            1,
            normalize_request("NOTIFY SHIORI/3.0\nCharset: UTF-8\nID: OnUnrelatedEvent\n"),
        ),
    );
    assert_eq!(
        notify_resp.as_bytes(),
        default_204().as_bytes(),
        "NOTIFY must return 204 immediately (fire-and-forget)"
    );

    let get_resp = marshal_get_with_timeout(
        &tx,
        MailboxRequest::new(
            2,
            normalize_request("GET SHIORI/3.0\nCharset: UTF-8\nID: OnTestSimple\n"),
        ),
        GENEROUS,
    );
    assert_eq!(
        get_resp.as_bytes(),
        EXPECT_ROUND1.as_bytes(),
        "GET after NOTIFY must return the actor-VM response in FIFO order\nactual: {get_resp:?}"
    );

    stop(&tx);
    actor
        .join()
        .expect("actor thread must join cleanly after Stop");
}

/// (a'') 本番 `marshal_get`（本番 `GET_TIMEOUT`）が通常運転でゴールデン値を返す。
/// `GET_TIMEOUT` は実 GET が決して接近しない秒オーダーの安全網（5 秒）であり、
/// 通常 GET は必ず値を返す（204 へ化けない）。
#[test]
fn marshal_get_production_timeout_returns_golden_value() {
    assert!(
        GET_TIMEOUT >= Duration::from_secs(1),
        "GET_TIMEOUT must be a generous safety net (seconds-scale) that never fires on normal GETs (RN4/R5.8), got {GET_TIMEOUT:?}"
    );

    let (load_dir, _temp) = build_async_callback_dir();
    let (tx, rx) = mailbox();
    let actor = spawn_actor_thread_with(load_dir, rx, None).expect("actor thread must spawn");
    assert!(actor.loaded(), "actor thread must load the ghost VM");

    let resp = marshal_get(
        &tx,
        MailboxRequest::new(
            1,
            normalize_request("GET SHIORI/3.0\nCharset: UTF-8\nID: OnTestSimple\n"),
        ),
    );
    assert_eq!(
        resp.as_bytes(),
        EXPECT_ROUND1.as_bytes(),
        "production marshal_get must return the golden value (timeout is a safety net, must not fire)\nactual: {resp:?}"
    );

    stop(&tx);
    actor
        .join()
        .expect("actor thread must join cleanly after Stop");
}

/// (d) drop→204: アクタースレッドが reply を送らず消滅した場合、受信側の
/// `recv_timeout` が `Disconnected` を観測して 204 になる（無限待機なし）。
///
/// 実 VM 経路で「reply を送らず drop」を誘発するのは決定論的でないため、receiver を
/// 直接 drop して再現する。純粋な drop→204 はユニットテスト
/// `marshal_get_reply_drop_yields_204` が検証する。
#[test]
fn marshal_get_disconnected_reply_yields_204() {
    let (tx, rx) = mailbox();
    // GET を投入しても誰も drain せず receiver を drop する。reply_tx は rx と共に drop され、
    // reply 経路が Disconnected になる。
    drop(rx);
    let resp = marshal_get_with_timeout(
        &tx,
        MailboxRequest::new(1, "GET SHIORI/3.0\r\nID: X\r\n\r\n".to_string()),
        GENEROUS,
    );
    assert_eq!(
        resp.as_bytes(),
        default_204().as_bytes(),
        "a disconnected reply path (no actor to reply) must surface as 204, no deadlock"
    );
    drop(tx);
}

/// (d') 実 VM 経路の relay 健全性: パース失敗リクエストは VM が `Ok(400 Bad Request)` を返す
/// 経路で、marshaling はその応答をそのまま中継する（drop にはならない）。
#[test]
fn marshal_get_relays_vm_400_for_parse_failure() {
    let (load_dir, _temp) = build_async_callback_dir();
    let (tx, rx) = mailbox();
    let actor = spawn_actor_thread_with(load_dir, rx, None).expect("actor thread must spawn");
    assert!(actor.loaded(), "actor thread must load the ghost VM");

    let resp = marshal_get_with_timeout(
        &tx,
        MailboxRequest::new(1, "this is not a valid shiori request\r\n\r\n".to_string()),
        GENEROUS,
    );
    assert!(
        resp.starts_with("SHIORI/3.0 400"),
        "marshaling must relay the VM's 400 response verbatim (no modification)\nactual: {resp:?}"
    );

    stop(&tx);
    actor
        .join()
        .expect("actor thread must join cleanly after Stop");
}

/// (e) アクター不在（mailbox 閉鎖）→ 即 204: receiver が落ちた mailbox への
/// GET/NOTIFY は try_send 失敗で即 204（デッドロックなし）。
#[test]
fn marshal_into_closed_mailbox_yields_204() {
    let (tx, rx) = mailbox();
    drop(rx); // アクタースレッド消滅の模擬。

    let get_resp = marshal_get_with_timeout(&tx, MailboxRequest::new(1, "X".to_string()), GENEROUS);
    assert_eq!(get_resp.as_bytes(), default_204().as_bytes());

    let notify_resp = marshal_notify(&tx, MailboxRequest::new(2, "X".to_string()));
    assert_eq!(notify_resp.as_bytes(), default_204().as_bytes());

    drop(tx);
}

/// (f) ロード失敗状態の GET は 500 + 単一行 `X-ERROR-REASON` を返す。
///
/// アクタースレッドは生存しているが VM のロードに失敗している状態（`loaded() == false`・
/// `PastaShiori::request` が `MyError::Load` を返す）を、ゴースト資材の無い空ディレクトリで
/// 決定論的に作る。reply を drop して 204 に読み替える無言化はしない。
#[test]
fn marshal_get_on_load_failed_actor_returns_500_with_single_line_reason() {
    let temp = TempDir::new().expect("create temp dir");
    let (tx, rx) = mailbox();
    let actor = spawn_actor_thread_with(temp.path().to_path_buf(), rx, None)
        .expect("actor thread must spawn");
    assert!(
        !actor.loaded(),
        "fixture precondition: the actor VM must fail to load from an empty dir"
    );

    let resp = marshal_get_with_timeout(
        &tx,
        MailboxRequest::new(
            1,
            normalize_request("GET SHIORI/3.0\nCharset: UTF-8\nID: version\nSender: SSP\n"),
        ),
        GENEROUS,
    );

    assert_ne!(
        resp.as_bytes(),
        default_204().as_bytes(),
        "a GET in the load-failed state must NOT be silenced into 204 (4.8)\nactual: {resp:?}"
    );
    assert!(
        resp.starts_with("SHIORI/3.0 500 Internal Server Error\r\n"),
        "a GET in the load-failed state must return 500 (4.3)\nactual: {resp:?}"
    );
    let reason = resp
        .split("\r\n")
        .find_map(|line| line.strip_prefix("X-ERROR-REASON: "))
        .expect("the 500 response must carry an X-ERROR-REASON header (4.3)");
    assert!(
        !reason.is_empty(),
        "X-ERROR-REASON must carry the root cause (4.3)"
    );
    assert!(
        !reason.contains('\r') && !reason.contains('\n'),
        "X-ERROR-REASON must be a single line (4.6)\nactual: {reason:?}"
    );

    stop(&tx);
    actor
        .join()
        .expect("actor thread must join cleanly after Stop");
}
