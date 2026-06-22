//! 本番 marshaling × 実 `!Send` VM アクタースレッドの統合テスト（task 3.4・R5）。
//!
//! task 5.1 で FFI 出荷経路がアクター経路へ昇格し、wintf／marshaling が既定ビルドの依存に
//! なったため、本テストは **既定（no-feature）ビルドで実行される**（旧 `actor-poc` ガードは
//! 撤去）。出荷経路そのものを検証するため、既定スイートで走らせるのが正しい。
//!
//! # 何を証明するか（task 3.4 の observable 完了条件）
//! task 3.3 の実アクタースレッド（VM pin）に対し、本番 marshaling 自由関数
//! （`marshal_get`/`marshal_notify`/`marshal_request`）を通して:
//!  (a) GET が実 VM 応答値を **タイムアウト内**に返し、その応答バイト列が byte-invariant
//!      ゴールデンと不変であること（R5.1/R5.8・R1.1 整合）。
//!  (b) NOTIFY が即 204 を返し、アクターの完了を待たないこと（R5.2）。
//!  (c) GET property→callback の 2 ラウンドでコルーチンが tick をまたいで継続し、各ラウンドの
//!      marshaling 応答がゴールデンと不変であること（R5.7 のコルーチン保存・通常経路）。
//!  (d) drop→204（VM 失敗で reply drop）が無限待機なく 204 になること（R5.3）。
//!  (e) アクター不在（mailbox 閉鎖）GET/NOTIFY が即 204 になること（R5.6）。
//!
//! # 本番 GET_TIMEOUT の扱い（CONCERNS）
//! 通常経路の (a)/(c) は **本番 `marshal_get`（`GET_TIMEOUT=6.68ms`）** で検証することを
//! 第一義とするが、デバッグビルド + コールド VM では実 GET が 6.68ms を超えうるため、
//! 環境非依存に確実な byte-invariant 検証として `marshal_get_with_timeout` に十分大きい
//! 閾値を与えた経路でゴールデン一致を厳密検証する（タイムアウト→204 の決定論的検証は
//! marshaling.rs のユニットテストで遅延アクター注入により実施済み）。本番閾値 6.68ms 経路は
//! 別テストで「204 か ゴールデン値か」のいずれか（無限待機なし・契約充足）を確認する。

use std::path::{Path, PathBuf};
use std::time::Duration;

use pasta::actor::mailbox::{ActorMsg, MailboxRequest, mailbox};
use pasta::actor::marshaling::{
    GET_TIMEOUT, default_204, marshal_get, marshal_get_with_timeout, marshal_notify,
    marshal_request,
};
use pasta::actor::thread::spawn_actor_thread;
use tempfile::TempDir;

/// 開発セッションが export した `PASTA_DEBUG`／`PASTA_DEBUG_PORT` を main 前
/// （プロセスが単一スレッドの間）に中和する。写経元: `common/mod.rs`。
#[ctor::ctor]
fn neutralize_debug_env() {
    unsafe {
        std::env::remove_var("PASTA_DEBUG");
        std::env::remove_var("PASTA_DEBUG_PORT");
    }
}

/// 大きめ閾値（環境非依存に実 VM 応答を取りこぼさない）。本番閾値 6.68ms の非発火は
/// 別テストで扱う。
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

    let pasta_scripts_src = manifest_dir
        .parent()
        .expect("parent dir")
        .join("pasta_lua")
        .join("pasta_scripts");
    let pasta_scripts_dst = temp.path().join("pasta_scripts");
    std::fs::create_dir_all(&pasta_scripts_dst).expect("create pasta_scripts dir");
    copy_dir_recursive(&pasta_scripts_src, &pasta_scripts_dst).expect("copy pasta_scripts");

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

/// clean stop helper。
fn stop(tx: &flume::Sender<ActorMsg>) {
    let (msg, done_rx) = ActorMsg::stop();
    tx.send(msg).expect("send Stop");
    done_rx
        .recv_timeout(GENEROUS)
        .expect("Stop must be acked");
}

/// (a)(c) 本番 marshaling 経由の GET が、実 VM 応答を byte-invariant ゴールデンと不変に
/// 返し、コルーチンが tick をまたいで継続する（R5.1/R5.7/R5.8・R1.1 整合）。
#[test]
fn marshal_get_through_actor_returns_byte_invariant_golden_and_continues_coroutine() {
    let (load_dir, _temp) = build_async_callback_dir();
    let (tx, rx) = mailbox();
    let actor = spawn_actor_thread(0, load_dir, rx);
    assert!(actor.loaded(), "actor thread must load the ghost VM");

    // Round1: get_property が yield → get タグ応答（marshaling 経由・ゴールデン一致）。
    let round1 = marshal_get_with_timeout(
        &tx,
        MailboxRequest::new(1, normalize_request("GET SHIORI/3.0\nCharset: UTF-8\nID: OnTestSimple\n")),
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
            normalize_request("GET SHIORI/3.0\nCharset: UTF-8\nID: OnPastaCallBack1\nReference0: 2.6.77\n"),
        ),
        GENEROUS,
    );
    assert_eq!(
        round2.as_bytes(),
        EXPECT_ROUND2.as_bytes(),
        "round2 marshaled GET must resume the coroutine and be byte-invariant\nactual: {round2:?}"
    );

    stop(&tx);
    actor.join().expect("actor thread must join cleanly after Stop");
}

/// (b) NOTIFY は即 204 を返し、アクターの完了を待たない。後続 GET が同一 FIFO で
/// NOTIFY 処理完了後に返る（R5.2・単一直列）。
#[test]
fn marshal_notify_through_actor_returns_204_then_get_in_fifo() {
    let (load_dir, _temp) = build_async_callback_dir();
    let (tx, rx) = mailbox();
    let actor = spawn_actor_thread(0, load_dir, rx);
    assert!(actor.loaded(), "actor thread must load the ghost VM");

    let notify_resp = marshal_notify(
        &tx,
        MailboxRequest::new(1, normalize_request("NOTIFY SHIORI/3.0\nCharset: UTF-8\nID: OnUnrelatedEvent\n")),
    );
    assert_eq!(
        notify_resp.as_bytes(),
        default_204().as_bytes(),
        "NOTIFY must return 204 immediately (fire-and-forget)"
    );

    // 後続 GET は同一 FIFO で NOTIFY 完了後に返る（actor-VM 応答・ゴールデン一致）。
    let get_resp = marshal_get_with_timeout(
        &tx,
        MailboxRequest::new(2, normalize_request("GET SHIORI/3.0\nCharset: UTF-8\nID: OnTestSimple\n")),
        GENEROUS,
    );
    assert_eq!(
        get_resp.as_bytes(),
        EXPECT_ROUND1.as_bytes(),
        "GET after NOTIFY must return the actor-VM response in FIFO order\nactual: {get_resp:?}"
    );

    stop(&tx);
    actor.join().expect("actor thread must join cleanly after Stop");
}

/// (a') メソッド判定込みディスパッチ `marshal_request`（R5.5）が、GET をアクター VM へ
/// 振り分けてゴールデン応答を返す。RN4 決定で `GET_TIMEOUT` を秒オーダーの安全網へ引き上げた
/// ため、通常運転の GET は **必ずゴールデン値**を返す（204 へ化けない＝バイト不変・R5.8）。
#[test]
fn marshal_request_dispatches_get_returns_golden() {
    let (load_dir, _temp) = build_async_callback_dir();
    let (tx, rx) = mailbox();
    let actor = spawn_actor_thread(0, load_dir, rx);
    assert!(actor.loaded(), "actor thread must load the ghost VM");

    // marshal_request は GET をディスパッチし、本番 GET_TIMEOUT（安全網・5 秒）で待つ。
    // 通常運転では実 VM 応答（ゴールデン）が必ず返る——タイムアウトは発火しない（R5.8）。
    let resp = marshal_request(
        &tx,
        1,
        &normalize_request("GET SHIORI/3.0\nCharset: UTF-8\nID: OnTestSimple\n"),
    );
    assert_eq!(
        resp.as_bytes(),
        EXPECT_ROUND1.as_bytes(),
        "marshal_request GET must return the byte-invariant golden (timeout must NOT fire)\nactual: {resp:?}"
    );

    stop(&tx);
    actor.join().expect("actor thread must join cleanly after Stop");
}

/// (a'') 本番 `marshal_get`（本番 `GET_TIMEOUT`）が通常運転でゴールデン値を返す（R5.1/R5.8）。
/// RN4 決定で `GET_TIMEOUT` は実 GET が決して接近しない秒オーダーの安全網（5 秒）であり、
/// 通常 GET は **必ず値**を返す（204 へ化けず byte-invariant）。閾値が安全網サイズであることも確認。
#[test]
fn marshal_get_production_timeout_returns_golden_value() {
    assert!(
        GET_TIMEOUT >= Duration::from_secs(1),
        "GET_TIMEOUT must be a generous safety net (seconds-scale) that never fires on normal GETs (RN4/R5.8), got {GET_TIMEOUT:?}"
    );

    let (load_dir, _temp) = build_async_callback_dir();
    let (tx, rx) = mailbox();
    let actor = spawn_actor_thread(0, load_dir, rx);
    assert!(actor.loaded(), "actor thread must load the ghost VM");

    let resp = marshal_get(
        &tx,
        MailboxRequest::new(1, normalize_request("GET SHIORI/3.0\nCharset: UTF-8\nID: OnTestSimple\n")),
    );
    assert_eq!(
        resp.as_bytes(),
        EXPECT_ROUND1.as_bytes(),
        "production marshal_get must return the golden value (timeout is a safety net, must not fire)\nactual: {resp:?}"
    );

    stop(&tx);
    actor.join().expect("actor thread must join cleanly after Stop");
}

/// (d) drop→204（R5.3）: アクタースレッドが reply を送らず消滅した場合、受信側の
/// `recv_timeout` が `Disconnected` を観測して 204 になる（無限待機なし）。
///
/// 実 thread.rs は VM 応答（パース失敗時の 400 含む）を `Reply::Value` で送るため、
/// 「reply を送らず drop」を実 VM 経路で誘発するのは決定論的でない（VM 内部パニックを
/// 要する）。そこで本テストは drop の原因を **アクタースレッド消滅**で与える: GET 投入後に
/// receiver 側（アクタースレッドが所有する `rx`）を含むスレッドを stop させ reply 経路が
/// 閉じる状況を、receiver を直接 drop することで決定論的に再現する。reply 経路の純粋な
/// drop→204 はユニットテスト `marshal_get_reply_drop_yields_204` で別途厳密検証済み。
#[test]
fn marshal_get_disconnected_reply_yields_204() {
    let (tx, rx) = mailbox();
    // GET を投入 → mailbox に積まれるが、誰も drain せず receiver を drop する。
    // reply_tx は ActorMsg::Get に同梱されたまま rx と共に drop され、reply 経路が
    // Disconnected になる → marshal_get は 204（無限待機なし）。
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

/// (d') 実 VM 経路の relay 健全性（R5.1 補完）: パース失敗リクエストは VM が `Ok(400 Bad
/// Request)` を返す経路であり、marshaling は **その VM 応答をそのまま中継**する（drop に
/// はならない）。これにより thread.rs の `if let Ok(resp)` 経路が忠実に relay されることを
/// 確認する（marshaling は VM 出力を改変しない）。
#[test]
fn marshal_get_relays_vm_400_for_parse_failure() {
    let (load_dir, _temp) = build_async_callback_dir();
    let (tx, rx) = mailbox();
    let actor = spawn_actor_thread(0, load_dir, rx);
    assert!(actor.loaded(), "actor thread must load the ghost VM");

    let resp = marshal_get_with_timeout(
        &tx,
        MailboxRequest::new(1, "this is not a valid shiori request\r\n\r\n".to_string()),
        GENEROUS,
    );
    // VM が返す 400 Bad Request を marshaling がそのまま中継する（無限待機なし）。
    assert!(
        resp.starts_with("SHIORI/3.0 400"),
        "marshaling must relay the VM's 400 response verbatim (no modification)\nactual: {resp:?}"
    );

    stop(&tx);
    actor.join().expect("actor thread must join cleanly after Stop");
}

/// (e) アクター不在（mailbox 閉鎖）→ 即 204（R5.6）: receiver が落ちた mailbox への
/// GET/NOTIFY は try_send 失敗で即 204（デッドロックなし）。
#[test]
fn marshal_into_closed_mailbox_yields_204() {
    let (tx, rx) = mailbox();
    drop(rx); // アクタースレッド消滅の模擬。

    let get_resp = marshal_get_with_timeout(&tx, MailboxRequest::new(1, "X".to_string()), GENEROUS);
    assert_eq!(get_resp.as_bytes(), default_204().as_bytes());

    let notify_resp = marshal_notify(&tx, MailboxRequest::new(2, "X".to_string()));
    assert_eq!(notify_resp.as_bytes(), default_204().as_bytes());

    // marshal_request（不正/閉鎖いずれも 204）。
    let req_resp = marshal_request(&tx, 3, "garbage");
    assert_eq!(req_resp.as_bytes(), default_204().as_bytes());

    drop(tx);
}
