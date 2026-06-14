//! Task 7.1 — `.pasta` ブレークポイント **E2E**（実トランスパイル → 実マップ構築
//! → 実デバッグセッション）。
//!
//! 既存の `.lua` E2E（[`super::tests::full_dap_session_over_tcp_*`] / 4.1）を
//! **本番化**し、`.pasta` 座標経路を end-to-end で駆動する（design 637-639「既存
//! スライス E2E を本番化」）。同じ DAP-over-TCP ハーネス（[`super::tests`] と同型の
//! `DapClient`）で、ローダの本番マップ構築経路（task 4.3
//! [`PastaLoader::build_source_map`]）が産んだ集約 `Arc<SourceMap>` を
//! [`enable`](crate::debug::enable)（task 4.2）へ `SourceMode::Pasta` で渡し、実際の
//! 生成 `.lua` を当該キャッシュ `.lua` パスをチャンク名（`@<path>`）として VM 上で
//! 走らせる。
//!
//! 観測する「done」（task 7.1 完了状態）:
//! 1. **`.pasta` 行 BP ヒット（4.1/4.2）**: `.pasta` のマップ済み行へ BP を張ると
//!    `verified`＝true で登録され、対応する `.lua` 行に到達して停止する。
//! 2. **`.pasta` 提示（5.1/5.2）**: 停止フレームの `stackTrace` が **`.pasta`** の
//!    ファイル・行を提示する（生成 `.lua` 座標**ではない**）。これがテストの「歯」:
//!    resolver/翻訳が無効なら `.lua` 座標が出てこのアサートが落ちる。
//! 3. **停止中 inspect 継続（5.4）**: `.pasta` 提示中も `scopes`/`variables` が
//!    機能する（提示モードは inspect に影響しない）。
//! 4. **最近接調整（4.3）**: マップの無い `.pasta` 行（先頭コメント行）へ BP を張ると
//!    後続最近接のマップ済み `.pasta` 行へ調整され、`verified`＋調整後 `line` が返る。
//!
//! `mlua::Lua`（`!Send`）は VM ホストスレッドにのみ生存し、バウンド `SocketAddr`
//! （`Copy`）と go/done チャネルだけが越境する。全クライアント待機は TEST-ONLY
//! watchdog でバウンドし CI がハングしないようにする（停止コアは無期限）。
use super::*;

use std::io::BufReader;
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use serde_json::{Value, json};

use crate::debug::source_map::{MapBuilderSink, SourceMap};
use crate::debug::transport::{read_frame, write_frame};
use crate::debug::{DebugConfig, SourceMode, enable};
use crate::loader::CacheManager;
use crate::transpiler::LuaTranspiler;

/// TEST-ONLY watchdog so CI cannot hang. The stop core is unbounded.
const WATCHDOG: Duration = Duration::from_secs(15);

/// 最小 `.pasta` フィクスチャ（グローバル単語定義 3 本）。先頭行はコメント（マップ
/// 無し → 4.3 最近接調整の対象）。各単語定義行はマップ済みで、生成 `.lua` の
/// トップレベル文（`PASTA.create_word(...):entry(...)`）として **実行**されるため、
/// その行に張った `.pasta` BP が実フックで発火する。
///
/// `.pasta` 行（1-origin）:
///   1: ＃グローバル単語          <- コメント（マップ無し → 4.3 で行2へ調整）
///   2: ＠あいさつ：こんにちは、やあ   <- マップ済み（→ 生成 `.lua` のある行）
///   3: ＠べつ：A、B、C            <- マップ済み（BP 対象）
///   4: ＠みっつ：x、y             <- マップ済み
const PASTA_FIXTURE: &str = "\
＃グローバル単語
＠あいさつ：こんにちは、やあ
＠べつ：A、B、C
＠みっつ：x、y
";

/// `require "pasta"` / `require "pasta.global"` を満たす最小 Lua シム。
/// `PASTA.create_word(name)` は `entry(...)` を持つオブジェクトを返すので、生成
/// `.lua` のトップレベル単語定義文が **副作用付きで実行**でき、各行で実フックが
/// 発火する（BP ヒット観測に必要）。`.pasta`↔`.lua` 変換とは無関係の純粋な実行
/// 足場であり、提示は resolver/翻訳（5.2/5.3）が担う。
const PASTA_SHIM: &str = "\
local word = {}
word.__index = word
function word:entry(...) self.entries = { ... } return self end
local PASTA = {}
function PASTA.create_word(name) return setmetatable({ name = name }, word) end
package.loaded['pasta'] = PASTA
package.loaded['pasta.global'] = {}
";

/// 実 TCP ソケット越しの最小 DAP クライアント（Content-Length フレーミング）。
/// [`super::tests::DapClient`] と同型（モジュール独立のため自前に持つ）。
struct DapClient {
    reader: BufReader<TcpStream>,
    writer: TcpStream,
}

impl DapClient {
    fn connect(addr: SocketAddr) -> Self {
        let stream = TcpStream::connect(addr).expect("client must connect to the bound port");
        stream
            .set_read_timeout(Some(WATCHDOG))
            .expect("TEST-ONLY read timeout");
        let writer = stream.try_clone().expect("clone socket for writing");
        Self {
            reader: BufReader::new(stream),
            writer,
        }
    }

    fn send_request(&mut self, seq: u64, command: &str, arguments: Value) {
        let req = json!({
            "seq": seq,
            "type": "request",
            "command": command,
            "arguments": arguments,
        });
        write_frame(&mut self.writer, &req).expect("client write must succeed");
    }

    fn recv(&mut self) -> Value {
        read_frame(&mut self.reader)
            .expect("client read must succeed (TEST-ONLY timeout)")
            .expect("a frame must be present (peer did not close)")
    }

    fn recv_until(&mut self, mut pred: impl FnMut(&Value) -> bool) -> Value {
        loop {
            let msg = self.recv();
            if pred(&msg) {
                return msg;
            }
        }
    }
}

fn is_event(msg: &Value, name: &str) -> bool {
    msg["type"] == "event" && msg["event"] == name
}

fn is_response(msg: &Value, command: &str) -> bool {
    msg["type"] == "response" && msg["command"] == command
}

/// 実トランスパイルの成果物（E2E が必要とする実座標を **マップから導出**）。
struct Fixture {
    /// ローダの本番経路が産んだ集約マップ（`enable` へ渡す）。
    map: Arc<SourceMap>,
    /// 実行する生成 `.lua` 本文（map が構築されたのと同一バイト）。
    lua_source: String,
    /// VM チャンク名（`@<キャッシュ .lua 絶対パス>`）= ローダ由来チャンク名。
    chunk_name: String,
    /// `.pasta` ファイルパス（`PastaPos.file` / VSCode source.path と一致）。
    pasta_path: String,
    /// BP を張るマップ済み `.pasta` 行（map から導出）。
    mapped_pasta_line: u32,
    /// `mapped_pasta_line` に対応する `.lua` 実行行（停止位置の根拠）。
    mapped_lua_line: u32,
    /// マップの無い `.pasta` 行（4.3 最近接調整の入力）。
    unmapped_pasta_line: u32,
    /// `unmapped_pasta_line` の後続最近接マップ済み `.pasta` 行（調整後の期待値）。
    nearest_adjusted_line: u32,
}

/// 実 `.pasta` をディスクへ書き、(a) 本番ローダ経路でマップを構築し
/// （[`PastaLoader::build_source_map`]・task 4.3）、(b) 同一 `.pasta` を同一
/// `transpile_with_source_map` で再トランスパイルして **実行する生成 `.lua` 本文**を
/// 取得する。map のチャンク名キー（ローダ由来 `source_to_cache_path`）を VM の
/// `set_name` に用いるため、停止座標とマップが一致する。
///
/// BP 対象/調整対象の `.pasta` 行は **マップから導出**してハードコードを避ける
/// （map の決定的順序に依存しない・回帰耐性）。
fn build_fixture() -> Fixture {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let base_dir = temp.path().to_path_buf();
    let pasta_file = base_dir.join("dic/baseware/words.pasta");
    std::fs::create_dir_all(pasta_file.parent().unwrap()).expect("mkdir dic");
    std::fs::write(&pasta_file, PASTA_FIXTURE).expect("write .pasta");

    let cache_manager = CacheManager::new(base_dir.clone(), "profile/pasta/cache/lua");

    // (a) 本番ローダ経路の集約マップ（sidecar=false: メモリ既定）。
    let map = crate::loader::PastaLoader::build_source_map(
        std::slice::from_ref(&pasta_file),
        &cache_manager,
        false,
    );

    // チャンク名キー = ローダ由来 `source_to_cache_path`（map のキーと同一）。
    let chunk_name = cache_manager
        .source_to_cache_path(&pasta_file)
        .to_string_lossy()
        .to_string();
    // `.pasta` ファイルキー = `build_source_map_inner` が `parse_str` / `PastaPos.file`
    // に用いる `file_path.to_string_lossy()`（VSCode source.path と一致させる側）。
    let pasta_path = pasta_file.to_string_lossy().to_string();

    // (b) 同一入力で再トランスパイルし、実行する生成 `.lua` 本文を得る（map が
    // 構築されたのと同一の決定的バイト）。
    let content = std::fs::read_to_string(&pasta_file).expect("read .pasta");
    let parsed = pasta_dsl::parse_str(&content, &pasta_path).expect("parse .pasta");
    let transpiler = LuaTranspiler::default();
    let mut sink = MapBuilderSink::new(pasta_path.clone(), chunk_name.clone());
    let mut out = Vec::new();
    transpiler
        .transpile_with_source_map(&parsed, &mut out, Some(&mut sink))
        .expect("transpile .pasta");
    let lua_source = String::from_utf8(out).expect("utf8 .lua");

    // --- 実座標をマップから導出 ---
    // マップ済み `.pasta` 行（と対応 `.lua` 行）を、`.lua`→`.pasta` 前方解決の
    // 昇順走査から収集する。
    let mut mapped: Vec<(u32, u32)> = Vec::new(); // (pasta_line, lua_line)
    for lua_line in 1u32..=200 {
        if let Some(pos) = map.resolve_lua_to_pasta(&chunk_name, lua_line) {
            mapped.push((pos.line, lua_line));
        }
    }
    mapped.sort();
    assert!(
        mapped.len() >= 2,
        "フィクスチャは少なくとも 2 つのマップ済み `.pasta` 行を持つこと: {mapped:?}"
    );

    // BP 対象: 2 番目のマップ済み `.pasta` 行（先頭以外で安定して実行される行）。
    let (mapped_pasta_line, mapped_lua_line) = mapped[1];

    // マップの無い `.pasta` 行: 最小マップ済み行の手前の行（先頭コメント行など）。
    // その後続最近接マップ済み行は最小マップ済み行になる。
    let min_mapped = mapped[0].0;
    assert!(
        min_mapped >= 2,
        "最小マップ済み `.pasta` 行の手前にマップ無し行が必要（4.3 入力）: min={min_mapped}"
    );
    let unmapped_pasta_line = min_mapped - 1; // 先頭コメント行（マップ無し）。
    // 期待される調整先 = `from_line` 以上で最初にマップを持つ `.pasta` 行。
    let nearest_adjusted_line = map
        .nearest_pasta_line_with_mapping(&pasta_path, unmapped_pasta_line)
        .expect("マップ無し行には後続最近接のマップ済み行が存在すること");
    assert_eq!(
        nearest_adjusted_line, min_mapped,
        "未マップ行の後続最近接は最小マップ済み行（4.3）"
    );
    // 未マップ行自身が本当にマップを持たないことを表明（テストの前提）。
    assert!(
        map.resolve_pasta_to_lua(&pasta_path, unmapped_pasta_line)
            .is_empty(),
        "選んだ未マップ `.pasta` 行 {unmapped_pasta_line} は対応 `.lua` を持たないこと"
    );

    // TempDir をリークさせて寿命を伸ばす（map / lua_source は既に取得済みで
    // ディスクは不要だが、念のため明示）。drop で削除されても map は in-memory。
    drop(temp);

    Fixture {
        map,
        lua_source,
        chunk_name,
        pasta_path,
        mapped_pasta_line,
        mapped_lua_line,
        unmapped_pasta_line,
        nearest_adjusted_line,
    }
}

/// task 7.1 の本体: `.pasta` 行 BP ヒット・`.pasta` 提示・停止中 inspect・最近接調整を
/// 1 本の実 DAP-over-TCP セッションで end-to-end 検証する。
///
/// requirements: **4.1**（`.pasta` 行 BP → 対応 `.lua` 行群登録）/ **4.2**（対応
/// `.lua` 行到達で停止）/ **4.3**（対応なし → 後続最近接へ調整・有効位置提示）/
/// **5.1**（停止位置 `.pasta` 提示）/ **5.2**（コールスタック各フレーム `.pasta`
/// 提示）/ **5.4**（`.pasta` 提示中も変数/コルーチン inspect 利用可能）。
#[test]
fn pasta_breakpoint_hits_presents_pasta_inspects_and_nearest_adjusts_over_tcp() {
    let fx = build_fixture();

    // host へ渡す値（`!Send` を越境させない）。
    let lua_source = fx.lua_source.clone();
    let chunk_name = fx.chunk_name.clone();
    let map = Arc::clone(&fx.map);

    let (addr_tx, addr_rx) = mpsc::channel::<SocketAddr>();
    let (go_tx, go_rx) = mpsc::channel::<()>();

    // VM HOST スレッド: `mlua::Lua`（!Send）を専有。実マップを Pasta モードで
    // `enable` へ渡し、実生成 `.lua` をローダ由来チャンク名で走らせる。
    let host = std::thread::spawn(move || -> Result<(), String> {
        let lua = unsafe {
            mlua::Lua::unsafe_new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default())
        };
        // `require "pasta"` を満たすシムを先に読み込む（フック未装着の準備実行）。
        lua.load(PASTA_SHIM)
            .set_name("@pasta_shim")
            .exec()
            .map_err(|e| format!("shim exec failed: {e}"))?;

        let cfg = DebugConfig {
            enabled: true,
            listen: Some("127.0.0.1:0".parse().unwrap()),
            source_mode: SourceMode::Pasta, // 既定だが明示（6.1）。
            ..Default::default()
        };
        // 実マップを `Some` で渡す（task 4.2・design 582: map+Pasta で `.pasta`
        // resolver/BP 翻訳/stepper が装着される）。
        let handle = enable(&lua, &cfg, Some(map))
            .map_err(|e| format!("enable failed: {e}"))?
            .ok_or_else(|| "enable returned None for an enabled config".to_string())?;

        let addr = handle
            .local_addr()
            .ok_or_else(|| "enabled handle must expose a bound addr".to_string())?;
        addr_tx.send(addr).map_err(|_| "addr send failed".to_string())?;

        // クライアントが setBreakpoints/configurationDone を終えるまで待つ。
        go_rx
            .recv_timeout(WATCHDOG)
            .map_err(|_| "did not receive go signal before running the VM".to_string())?;

        // 実生成 `.lua` を **ローダ由来チャンク名**（`@<キャッシュ .lua パス>`）で
        // 走らせる。フックはこの source を報告し、`should_pause` が正規化突合する
        // ので `.pasta` BP（→ `.lua` 行登録）が発火する（4.2）。
        lua.load(&lua_source)
            .set_name(format!("@{chunk_name}"))
            .exec()
            .map_err(|e| format!("scenario exec failed: {e}"))?;
        lua.remove_global_hook();
        drop(handle);
        Ok(())
    });

    // CLIENT（このスレッド）。
    let addr = addr_rx
        .recv_timeout(WATCHDOG)
        .expect("host must publish the bound addr before the watchdog");
    let mut client = DapClient::connect(addr);

    // --- initialize → capabilities + `initialized` ---
    client.send_request(1, "initialize", json!({ "adapterID": "pasta" }));
    let init_resp = client.recv_until(|m| is_response(m, "initialize"));
    assert_eq!(init_resp["success"], true, "initialize must succeed");
    let _initialized = client.recv_until(|m| is_event(m, "initialized"));

    // --- (4.3) 最近接調整: マップ無し `.pasta` 行へ BP → verified＋調整後 line ---
    // 先に最近接調整を表明する（設定リクエストの応答だけで判定でき、VM 実行に
    // 依存しない）。同一 `.pasta` source なので後段のヒット用 BP で置換される。
    client.send_request(
        2,
        "setBreakpoints",
        json!({
            "source": { "path": fx.pasta_path },
            "breakpoints": [{ "line": fx.unmapped_pasta_line }],
        }),
    );
    let adj_resp = client.recv_until(|m| is_response(m, "setBreakpoints"));
    let adj_bps = adj_resp["body"]["breakpoints"]
        .as_array()
        .expect("breakpoints array");
    assert_eq!(adj_bps.len(), 1, "1 つの BP 応答");
    assert_eq!(
        adj_bps[0]["verified"], true,
        "4.3: マップ無し行 BP は後続最近接へ調整され verified になる"
    );
    assert_eq!(
        adj_bps[0]["line"].as_u64().expect("adjusted line") as u32,
        fx.nearest_adjusted_line,
        "4.3: 調整後の有効位置（後続最近接のマップ済み `.pasta` 行）が提示される"
    );

    // --- (4.1) `.pasta` のマップ済み行へ BP → verified（元の行で確定）---
    client.send_request(
        3,
        "setBreakpoints",
        json!({
            "source": { "path": fx.pasta_path },
            "breakpoints": [{ "line": fx.mapped_pasta_line }],
        }),
    );
    let bp_resp = client.recv_until(|m| is_response(m, "setBreakpoints"));
    let bps = bp_resp["body"]["breakpoints"]
        .as_array()
        .expect("breakpoints array");
    assert_eq!(bps.len(), 1);
    assert_eq!(
        bps[0]["verified"], true,
        "4.1: マップ済み `.pasta` 行 BP は verified で登録される"
    );
    assert_eq!(
        bps[0]["line"].as_u64().expect("bp line") as u32,
        fx.mapped_pasta_line,
        "4.1: マップ済み行は調整されず元の `.pasta` 行のまま"
    );

    // --- configurationDone → VM 実行開始 ---
    client.send_request(4, "configurationDone", json!({}));
    let cfg_resp = client.recv_until(|m| is_response(m, "configurationDone"));
    assert_eq!(cfg_resp["success"], true);
    go_tx.send(()).expect("send go signal");

    // --- (4.2) `.pasta` 行に対応する `.lua` 行へ到達 → stopped(breakpoint) ---
    let stopped = client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(
        stopped["body"]["reason"], "breakpoint",
        "4.2: `.pasta` 行 BP に対応する `.lua` 行で停止する"
    );
    let thread_id = stopped["body"]["threadId"].as_u64().expect("threadId");

    // --- (5.1/5.2) stackTrace は **`.pasta`** 座標を提示する（`.lua` ではない）---
    client.send_request(10, "stackTrace", json!({ "threadId": thread_id }));
    let stack = client.recv_until(|m| is_response(m, "stackTrace"));
    let frames = stack["body"]["stackFrames"]
        .as_array()
        .expect("stackFrames array");
    assert!(!frames.is_empty(), "停止フレームが存在する（5.2）");

    // === テストの「歯」: top フレームの source.path が `.pasta`・line が `.pasta`
    // 行であること。resolver/翻訳（5.2/5.3）が無効なら、ここに生成 `.lua` の
    // チャンク名（`@<...lua>`）と `.lua` 行が出てこのアサートが落ちる。===
    let top_src = frames[0]["source"]["path"]
        .as_str()
        .expect("top frame source path");
    assert!(
        top_src.ends_with(".pasta"),
        "5.1/5.2: 停止フレームは `.pasta` を提示すること（`.lua` ではない）。actual={top_src:?}"
    );
    // 正規化突合（区切り/大小）で `.pasta` パスと一致する。
    assert_eq!(
        crate::debug::source_map::canonicalize_chunk_name(top_src),
        crate::debug::source_map::canonicalize_chunk_name(&fx.pasta_path),
        "5.1: 提示 `.pasta` パスは元 `.pasta` ファイルと一致する"
    );
    assert_eq!(
        frames[0]["line"].as_u64().expect("top frame line") as u32,
        fx.mapped_pasta_line,
        "5.1: 提示行は **`.pasta` 行**（{}）。`.lua` 行（{}）であってはならない",
        fx.mapped_pasta_line,
        fx.mapped_lua_line
    );
    // `.pasta` 行と `.lua` 行が異なることを前提として確認（歯の有効性の裏付け）:
    // もし提示が `.lua` のままなら line==mapped_lua_line になり、上の assert が
    // 落ちる。両者が異なるフィクスチャを選んでいる。
    assert_ne!(
        fx.mapped_pasta_line, fx.mapped_lua_line,
        "フィクスチャは `.pasta` 行 ≠ `.lua` 行（提示差が観測可能）"
    );

    // --- (5.4) `.pasta` 提示中でも inspect（scopes/variables）が機能する ---
    let frame_id = frames[0]["id"].as_u64().expect("frame id");
    client.send_request(11, "scopes", json!({ "frameId": frame_id }));
    let scopes = client.recv_until(|m| is_response(m, "scopes"));
    assert_eq!(scopes["success"], true, "5.4: `.pasta` 提示中も scopes が成功する");
    let scope_arr = scopes["body"]["scopes"]
        .as_array()
        .expect("scopes array");
    assert!(
        !scope_arr.is_empty(),
        "5.4: 停止フレームの scope が利用可能（提示モードは inspect に影響しない）"
    );
    let var_ref = scope_arr[0]["variablesReference"]
        .as_u64()
        .expect("variablesReference");
    // variables 要求も成功する（中身の有無に依らず、提示モードで壊れないこと）。
    client.send_request(12, "variables", json!({ "variablesReference": var_ref }));
    let vars = client.recv_until(|m| is_response(m, "variables"));
    assert_eq!(
        vars["success"], true,
        "5.4: `.pasta` 提示中でも variables 要求が成功する（inspect 継続）"
    );
    assert!(
        vars["body"]["variables"].is_array(),
        "5.4: variables 応答は配列を返す（inspect 経路が機能している）"
    );

    // --- continue → 実行完走 → host スレッド終了（terminated 観測）---
    //
    // 注意: マップ済み行（単語定義）は生成 `.lua` 上で `create_word():entry()` の
    // **複数呼び出しを含む 1 行**であり、Lua のラインフックは同一ソース行で複数回
    // 発火し得る（呼び出しから戻る際に同じ行へ再入する）。よって 1 つの `.pasta`
    // 行 BP がその行で複数回停止し得る。これは行 BP の正当な挙動なので、`continue`
    // を繰り返して流し切り、chunk 完走 → `drop(handle)` が出す `terminated`（host が
    // 完了した決定的シグナル）を観測するまでループする。各 `continue` 後に来る
    // フレームは `stopped`（再停止 → もう一度 continue）か `terminated`（完了 →
    // 終了）のいずれか。ループ回数は BP 行の再入数で有限（CI 無限ループ防止に上限）。
    let mut terminated = false;
    for continue_seq in 30u64..60u64 {
        client.send_request(continue_seq, "continue", json!({ "threadId": thread_id }));
        // continue ack の後に来る次の制御フレーム（stopped/terminated）を待つ。
        let next = client.recv_until(|m| {
            is_event(m, "stopped") || is_event(m, "terminated")
        });
        if is_event(&next, "terminated") {
            terminated = true;
            break;
        }
        // それ以外は同一 BP 行への再停止 → もう一度 continue（reason は breakpoint）。
        assert_eq!(
            next["body"]["reason"], "breakpoint",
            "再停止は同一 `.pasta` 行 BP のはず（多重呼び出し行の再入）"
        );
    }
    assert!(
        terminated,
        "chunk 完走 → `terminated` が来るまで continue で流し切れること"
    );

    // host VM スレッドが watchdog 内で完了することを確認（ハング無し）。
    let (done_tx, done_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = done_tx.send(host.join());
    });
    match done_rx.recv_timeout(WATCHDOG) {
        Ok(joined) => {
            joined
                .expect("host VM thread must not panic")
                .expect("scenario must run to completion after continue");
        }
        Err(RecvTimeoutError::Timeout) => {
            panic!("host VM thread did not finish within the watchdog (hang?)");
        }
        Err(RecvTimeoutError::Disconnected) => panic!("join watcher disconnected"),
    }
}
