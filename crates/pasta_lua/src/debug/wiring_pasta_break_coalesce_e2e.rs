//! Task 3.2 — 多対1 Continue の実 DAP-over-TCP E2E
//! (spec: `pasta-debug-break-coalesce`, requirements **1.1** / **1.2** / **3.2** / **6.2(a)**).
//!
//! # 観測する「done」
//!
//! 1つの `.pasta` 行が複数の `.lua` 行へ展開される（多対1）構成で、その `.pasta` 行へ
//! BP を張って停止させ、**1回だけ** `continue` を送ると、実行は **次の `.pasta` 行**
//! （次の停止点）へ進み、**同一 `.pasta` 行に対する 2 回目の `stopped` は来ない**こと
//! を実ソケットで証明する。これが design "System Flows" / "Testing Strategy →
//! Integration / E2E（multi-to-one Continue）" のシナリオであり、fix（`session.rs`
//! `on_line_impl` の break-anchor coalescing）が無ければ、同じ `.pasta` 行へマップする
//! 次の `.lua` 行で再停止してこのテストが落ちる。
//!
//! # フィクスチャ（Task 3.1 で committed）
//!
//! `tests/fixtures/debug_break_coalesce.pasta` のトーク行
//! `合計は＠加算ループ()　です。`（`.pasta` 行 21）は、生成 `.lua` の
//! `SCENE.__start__` 本体で **3 本の `talk(...)`/`expr_fn(...)` 文**（`.lua` 行 11/12/13）
//! へ展開される（多対1）。次行 `「おはよう」「げんき」`（`.pasta` 行 22）は `.lua` 行 14
//! へ対応し、これが「1 回の continue で到達すべき次の停止点」。実 `.pasta`/`.lua` 行は
//! **ローダの本番マップ構築経路**（[`PastaLoader::build_source_map`]・Task 4.3）から
//! 導出してハードコードを避ける（[`build_fixture`] と同流儀）。
//!
//! # ハーネス（task 7.1 [`super::pasta_bp_e2e`] の踏襲）
//!
//! 実マップを `SourceMode::Pasta` で [`enable`](crate::debug::enable) へ渡し、生成
//! `.lua` を **ローダ由来チャンク名**（`@<キャッシュ .lua パス>`）で VM 上に走らせる。
//! 関数定義は BP 設定前に行い（チャンク本文実行で `SCENE.__start__` 等を定義）、その後
//! クライアントが `.pasta` 行 21 へ BP を張ってから `SCENE.__start__(act)` を呼ぶ。
//! こうすると BP 有効下で実行されるのは `__start__` 本体（`.lua` 行 11→12→13→14）だけ
//! になり、多対1 行の **連続実行**中の coalescing を決定的に観測できる（関数定義行
//! `.lua` 7 もまた `.pasta` 21 へマップするが、それは定義時＝BP 設定前に実行済み）。
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

/// 多対1＋ループ再訪を含む committed フィクスチャ（Task 3.1）。3.2/3.3 が同一ファイルを
/// ロードする。`include_str!` で取り込み、移動時はコンパイルエラーで検知する。
const FIXTURE: &str = include_str!("../../tests/fixtures/debug_break_coalesce.pasta");

/// 多対1 トーク行（`合計は＠加算ループ()　です。`）の `.pasta` 行（1-origin）を
/// FIXTURE から一意に求める（[`debug_break_coalesce_fixture_test`] の同名マーカー）。
const MULTI_TO_ONE_MARKER: &str = "合計は＠加算ループ()";

/// ループ本体行（`SCENE.加算ループ` 内 `total = total + i`）の `.pasta` 行を FIXTURE から
/// 一意に求める（[`debug_break_coalesce_fixture_test`] の同名マーカー / Task 3.3 用）。
/// ループは `GLOBAL.ループ回数 or 3` 回まわるため、この `.pasta` 行は実行時に再訪される。
const LOOP_BODY_MARKER: &str = "total = total + i";

/// フィクスチャのループ反復回数（`GLOBAL.ループ回数` 未設定時の既定 = ループ本体行の再訪回数）。
/// shim は `package.loaded['pasta.global'] = {}`（= `GLOBAL.ループ回数` が nil）を与えるため、
/// `加算ループ` は `for i = 1, 3 do ... end` を回し、ループ本体 `.pasta` 行を N 回再訪する。
const LOOP_VISITS: usize = 3;

/// `require "pasta"` / `require "pasta.global"` を満たし、`SCENE.__start__` を実行可能に
/// する最小シム。`create_scene` は素のテーブルを返し、`act` スタブは `init_scene` /
/// `expr_fn` / `さくら:talk` を no-op で提供する（`.pasta`↔`.lua` 変換とは無関係の純粋な
/// 実行足場であり、BP/coalescing は実セッションが担う）。
const PASTA_SHIM: &str = "\
local PASTA = {}
function PASTA.create_scene(name)
local s = { name = name }
PASTA.__last_scene = s   -- 生成 scene を捕捉（chunk の do...end ローカルを橋渡し）
return s
end
package.loaded['pasta'] = PASTA
package.loaded['pasta.global'] = {}
local actor = {}
actor.__index = actor
function actor:talk(_s) return self end
function actor:expr_fn(_name) return 0 end
ACT = setmetatable({}, {
__index = function(_t, _k)
    return setmetatable({}, actor)
end,
})
function ACT.init_scene(_self, _scene) return {}, {} end
";

/// 実 TCP ソケット越しの最小 DAP クライアント（[`super::tests::DapClient`] と同型）。
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

/// FIXTURE 内で `needle` を含む唯一の **非コメント** 行の 1-origin 行番号を返す。
/// コメント行（`＃`/`#`）は除外する（マーカー文字列を解説で含み得るため）。
fn unique_pasta_line(needle: &str) -> u32 {
    let hits: Vec<u32> = FIXTURE
        .lines()
        .enumerate()
        .filter(|(_, l)| {
            let t = l.trim_start();
            !t.starts_with('＃') && !t.starts_with('#') && l.contains(needle)
        })
        .map(|(i, _)| i as u32 + 1)
        .collect();
    assert_eq!(
        hits.len(),
        1,
        "fixture invariant: marker {needle:?} must appear on exactly one line, got {hits:?}"
    );
    hits[0]
}

/// 実トランスパイル成果物（実座標を **マップから導出**）。
struct Fixture {
    /// ローダ本番経路の集約マップ（`enable` へ渡す）。
    map: Arc<SourceMap>,
    /// 実行する生成 `.lua` 本文（map 構築と同一バイト）。
    lua_source: String,
    /// VM チャンク名（`@<キャッシュ .lua 絶対パス>`）= ローダ由来チャンク名。
    chunk_name: String,
    /// `.pasta` ファイルパス（`PastaPos.file` / VSCode source.path と一致）。
    pasta_path: String,
    /// 多対1 トーク行の `.pasta` 行（BP 対象）。
    multi_pasta_line: u32,
    /// `multi_pasta_line` が展開される `.lua` 行群（≥2 で多対1 を裏づける）。
    multi_lua_lines: Vec<u32>,
    /// 多対1 行の **次に到達すべき** `.pasta` 行（continue 後の停止点・歯）。
    next_pasta_line: u32,
}

/// committed `.pasta` をディスクへ書き、(a) 本番ローダ経路でマップを構築し、(b) 同一入力を
/// 同一マップ構築経路で再トランスパイルして実行用 `.lua` 本文を得る（バイト一致）。BP 対象・
/// 次行・`.lua` 展開はすべて map から導出する。
fn build_fixture() -> Fixture {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let base_dir = temp.path().to_path_buf();
    let pasta_file = base_dir.join("dic/test/debug_break_coalesce.pasta");
    std::fs::create_dir_all(pasta_file.parent().unwrap()).expect("mkdir dic");
    std::fs::write(&pasta_file, FIXTURE).expect("write .pasta");

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
    // `.pasta` ファイルキー = `parse_str` / `PastaPos.file`（VSCode source.path 側）。
    let pasta_path = pasta_file.to_string_lossy().to_string();

    // (b) `build_source_map` は生成 `.lua` を破棄するため、同一入力・同一マップ構築経路で
    //     再トランスパイルして **実行する `.lua` 本文**を得る（map と同一の決定的バイト）。
    let content = std::fs::read_to_string(&pasta_file).expect("read .pasta");
    let parsed = pasta_dsl::parse_str(&content, &pasta_path).expect("parse .pasta");
    let transpiler = LuaTranspiler::default();
    let mut sink = MapBuilderSink::new(pasta_path.clone(), chunk_name.clone());
    let mut out = Vec::new();
    transpiler
        .transpile_with_source_map(&parsed, &mut out, Some(&mut sink))
        .expect("transpile .pasta");
    let lua_source = String::from_utf8(out).expect("utf8 .lua");

    // 多対1 行（BP 対象）の `.pasta` 行と、その `.lua` 展開（≥2）。
    let multi_pasta_line = unique_pasta_line(MULTI_TO_ONE_MARKER);
    let mut multi_lua_lines: Vec<u32> = map
        .resolve_pasta_to_lua(&pasta_path, multi_pasta_line)
        .into_iter()
        .map(|(_chunk, lua_line)| lua_line)
        .collect();
    multi_lua_lines.sort_unstable();
    assert!(
        multi_lua_lines.len() >= 2,
        "6.2(a): 多対1 行 {multi_pasta_line} は ≥2 の `.lua` 行へ展開されること（前提）: \
         {multi_lua_lines:?}"
    );

    // continue 後に到達すべき **次の** `.pasta` 行 = 多対1 行の `.lua` 展開の **最大** 行の
    // 直後で初めて現れる、`multi_pasta_line` と異なる `.pasta` 行を map から導出する。
    let last_multi_lua = *multi_lua_lines.last().unwrap();
    let next_pasta_line = (last_multi_lua + 1..=last_multi_lua + 200)
        .find_map(|lua_line| {
            map.resolve_lua_to_pasta(&chunk_name, lua_line)
                .filter(|pos| pos.line != multi_pasta_line)
                .map(|pos| pos.line)
        })
        .expect("多対1 行の後に別の `.pasta` 行が存在すること（next stop point）");
    assert_ne!(
        next_pasta_line, multi_pasta_line,
        "次の停止点は多対1 行とは異なる `.pasta` 行であること（歯の有効性）"
    );

    drop(temp); // map / lua_source は取得済み。ディスクは不要。

    Fixture {
        map,
        lua_source,
        chunk_name,
        pasta_path,
        multi_pasta_line,
        multi_lua_lines,
        next_pasta_line,
    }
}


#[cfg(test)]
#[path = "wiring_pasta_break_coalesce_e2e_scenarios.rs"]
mod scenarios;
