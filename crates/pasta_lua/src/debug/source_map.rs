//! `.pasta`↔生成 `.lua` 行対応の **薄い実証スライス**（consumer 側・R4.4/R4.5）。
//!
//! このモジュール全体は feature `pasta-source-map-slice`（default 無効）の配下に
//! あり、無効時（既定）は一切コンパイル/露出されない（R4.6 ゼロコスト）。本番品質
//! のソースマップ（全 `generate_*` 網羅・本番マップ出力・`.pasta` 座標常時提示）は
//! 本仕様の対象外であり、ダウンストリーム別仕様 `pasta-source-map` の担当である。
//!
//! # 役割（design "SourceMapSeam & CodeGenSourceMapHook"）
//!
//! producer 側のシーム（[`crate::code_gen::source_map`] の [`SourceMapSink`]）は
//! `record(out_line, span)` を呼んで「生成 `.lua` 行 → `.pasta` span」を通知する。
//! 本モジュールはその **consumer 側**で、代表 1 経路について:
//!
//! 1. [`SliceSink`] が `record` コールバックを受けて、各 `span` の開始バイト
//!    オフセットを `.pasta` ソーステキスト上の行番号へ変換し、[`LineMap`] を構築する。
//! 2. [`resolve_lua_to_pasta`] が生成 `.lua` 行 → `.pasta` 位置を引く（R4.4）。
//! 3. [`LineMap::lua_lines_for_pasta`] が `.pasta` 行 → 生成 `.lua` 行（逆引き）を
//!    引き、`.pasta` 行に設定したブレークポイントをフックが実際に止まる `.lua` 行へ
//!    翻訳する（R4.5）。
//!
//! # `normalize_output` 行ズレの扱い（重要）
//!
//! producer の `out_line` は `normalize_output` **適用前**のバッファ行を数えるため、
//! 最終 `.lua` の行番号と一般にはズレ得る。`normalize_output` が削除するのは
//! 「`end` 直前の空行」と「末尾空白」のみであり、これらはトーク行 **より後ろ**に
//! しか現れない。したがって本スライスが選ぶ代表経路（単純 talk 1 行）では、トーク
//! アクションの生成行は normalize 前後で **不変**である（`out_line` == ランタイムが
//! 実際に実行する行）。スライステストはこの不変性を、生成 `.lua` を実フックで走らせ
//! トーク行がその番号で発火することを以て検証する（数合わせの偶然ではない）。完全な
//! 行ズレ補正（`end` より前の経路を含む全網羅）は `pasta-source-map` の残課題。

use std::collections::BTreeMap;

use pasta_dsl::parser::Span;

pub use crate::code_gen::source_map::{PastaPos, SourceMapSink};

/// 生成 `.lua` 行 → `.pasta` 位置の **部分**対応表（design "LineMap"）。
///
/// スライスでは代表経路のみを保持する部分マップである（全網羅は別仕様
/// `pasta-source-map`）。同一 `.lua` 行が複数回 `record` された場合は **最後の**
/// 記録で上書きする（生成器は通常 1 行 1 span を記録するため実害はない。多重記録の
/// 厳密な扱いは本番仕様の残課題）。
#[derive(Debug, Clone, Default)]
pub struct LineMap {
    /// `lua_line`（1 始まり）→ 対応する `.pasta` 位置。
    ///
    /// `BTreeMap` を用いるのは決定的反復順（逆引きの安定性・テスト容易性）のため。
    lua_to_pasta: BTreeMap<u32, PastaPos>,
}

impl LineMap {
    /// 空の対応表を構築する。
    pub fn new() -> Self {
        Self::default()
    }

    /// `lua_line` → `.pasta` 位置の対応を 1 件挿入する（同一行は上書き）。
    pub fn insert(&mut self, lua_line: u32, pos: PastaPos) {
        self.lua_to_pasta.insert(lua_line, pos);
    }

    /// 記録済みの対応件数。
    pub fn len(&self) -> usize {
        self.lua_to_pasta.len()
    }

    /// 対応が 1 件も無いか。
    pub fn is_empty(&self) -> bool {
        self.lua_to_pasta.is_empty()
    }

    /// 生成 `.lua` 行 → `.pasta` 位置を引く（R4.4 の中核）。
    ///
    /// 対応が無い行には `None` を返す（スライスは部分マップ）。
    pub fn pasta_for_lua(&self, lua_line: u32) -> Option<&PastaPos> {
        self.lua_to_pasta.get(&lua_line)
    }

    /// `.pasta` 行 → 対応する生成 `.lua` 行群（逆引き・R4.5 の中核）。
    ///
    /// `.pasta` 行に設定したブレークポイントを、フックが実際に `(source, line)` で
    /// 突合する **生成 `.lua` 行**へ翻訳するための逆引きである。1 つの `.pasta` 行が
    /// 複数の `.lua` 行へ展開され得る（本番では一般的）ため `Vec` を返す。返り値は
    /// `.lua` 行の昇順（`BTreeMap` 反復順）で決定的。
    pub fn lua_lines_for_pasta(&self, pasta_line: u32) -> Vec<u32> {
        self.lua_to_pasta
            .iter()
            .filter(|(_, pos)| pos.line == pasta_line)
            .map(|(lua_line, _)| *lua_line)
            .collect()
    }
}

/// `.pasta` ソーステキストから [`LineMap`] を構築する [`SourceMapSink`] 実装。
///
/// producer の `record(out_line, span)` ごとに、`span.start_byte`（0 始まり）まで
/// `.pasta` テキスト中の `\n` を数えて `.pasta` 行番号（1 始まり）へ変換し、
/// [`LineMap`] へ蓄積する。`.pasta` ソーステキストとファイルパスを所有する。
///
/// 不変／既定 span（`end_byte == 0`）は producer 側
/// ([`LuaCodeGenerator::record_span`](crate::code_gen::LuaCodeGenerator)) で既に
/// 除外されているため、ここに届くのは有効な span のみである。
pub struct SliceSink {
    /// 元 `.pasta` ファイルパス（[`PastaPos::file`] に載せる）。
    file: String,
    /// 元 `.pasta` ソーステキスト（バイトオフセット→行番号変換に使う）。
    source: String,
    /// 構築中の対応表。
    map: LineMap,
}

impl SliceSink {
    /// `.pasta` の `file`／`source` から空のシンクを構築する。
    pub fn new(file: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            source: source.into(),
            map: LineMap::new(),
        }
    }

    /// 構築済みの [`LineMap`] を取り出す（record 適用後に呼ぶ）。
    pub fn into_line_map(self) -> LineMap {
        self.map
    }

    /// 構築中の [`LineMap`] を借用する（record 適用中の観測用）。
    pub fn line_map(&self) -> &LineMap {
        &self.map
    }

    /// `.pasta` テキスト上で 0 始まりバイトオフセット `byte` が属する行番号
    /// （1 始まり）を返す。`byte` までに現れる `\n` の数 + 1。
    ///
    /// `byte` はソース長で頭打ちし、さらに UTF-8 文字境界へ丸める（パーサ由来の
    /// span は境界に乗るが、任意 span でも slice panic しないための防御）。
    fn pasta_line_at(&self, byte: usize) -> u32 {
        let mut upto = byte.min(self.source.len());
        while upto > 0 && !self.source.is_char_boundary(upto) {
            upto -= 1;
        }
        (self.source[..upto].matches('\n').count() as u32) + 1
    }
}

impl SourceMapSink for SliceSink {
    fn record(&mut self, lua_line: u32, span: Span) {
        let pasta_line = self.pasta_line_at(span.start_byte);
        self.map.insert(
            lua_line,
            PastaPos {
                file: self.file.clone(),
                line: pasta_line,
            },
        );
    }
}

/// 生成 `.lua` 行 → 対応する `.pasta` 位置を引く（R4.4・gated）。
///
/// design "SourceMapSeam" の公開シグネチャ。停止位置の生成 `.lua` 行を `.pasta`
/// 行へ変換し、DAP の `stackTrace`/`stopped` を `.pasta` で提示する起点となる
/// （[`SourceResolver`](crate::debug::dap::SourceResolver) へ接続可能）。対応の無い
/// 行には `None`（スライスは部分マップ）。
pub fn resolve_lua_to_pasta(map: &LineMap, lua_line: u32) -> Option<PastaPos> {
    map.pasta_for_lua(lua_line).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::mpsc;
    use std::thread::JoinHandle;
    use std::time::Duration;

    use mlua::{Debug, Lua, LuaOptions, StdLib};
    use serde_json::json;

    use crate::code_gen::LuaCodeGenerator;
    use crate::config::LineEnding;
    use crate::context::TranspileContext;
    use crate::debug::breakpoints::BreakpointSet;
    use crate::debug::dap::{DapAdapter, default_source_resolver};
    use crate::debug::hook::LineHook;
    use crate::debug::session::DebugSession;
    use crate::debug::types::{FrameInfo, SessionCommand, SessionEvent, SourceRef, StopReason};
    use crate::normalize::normalize_output;
    use pasta_dsl::parser::{FileItem, parse_str};

    /// 代表経路の `.pasta`（単純 talk 1 行）。`.pasta` 行 2 が `さくら：「…」`。
    const SLICE_PASTA: &str = "＊あいさつ\n　さくら：「こんにちは！」\n";
    /// 元 `.pasta` ファイルパス（`PastaPos::file` の確認用）。
    const SLICE_PASTA_FILE: &str = "slice.pasta";
    /// 生成 `.lua` チャンクのソース名（フックが突合する `(source, line)` のソース）。
    const SLICE_LUA_SOURCE: &str = "@slice.lua";
    /// 代表トークの `.pasta` 行（`さくら：「こんにちは！」`）。
    const TALK_PASTA_LINE: u32 = 2;
    /// 代表トークが着地する生成 `.lua` 行（探索で確定。normalize で不変）。
    const TALK_LUA_LINE: u32 = 11;
    /// テスト専用ウォッチドッグ（停止コアは無期限・design "無期限ブロックが正"）。
    const WATCHDOG: Duration = Duration::from_secs(10);

    /// 代表 `.pasta` を `SliceSink` 装着で実トランスパイルし、最終（normalize 後）の
    /// 生成 `.lua` と構築済み [`LineMap`] を返す。
    ///
    /// ランタイムが実際に load するのと同一パイプライン（codegen→normalize）を辿る
    /// ので、`LineMap` の `.lua` 行はランタイム実行行と整合する（normalize 行ズレの
    /// 扱い: 代表経路はトーク行が normalize 前後で不変 — モジュール doc 参照）。
    fn transpile_slice() -> (String, LineMap) {
        let file = parse_str(SLICE_PASTA, SLICE_PASTA_FILE).expect("代表 .pasta はパースできる");
        let mut sink = SliceSink::new(SLICE_PASTA_FILE, SLICE_PASTA);

        let mut buf: Vec<u8> = Vec::new();
        let mut ctx = TranspileContext::new();
        {
            let mut codegen = LuaCodeGenerator::with_line_ending(&mut buf, LineEnding::Lf);
            // producer シームへ consumer シンクを接続（本番は None＝ゼロコスト）。
            codegen.set_source_map(&mut sink);
            codegen.write_header().expect("ヘッダ生成");
            for item in &file.items {
                if let FileItem::GlobalSceneScope(scene) = item {
                    let (_, counter) = ctx.register_global_scene(scene);
                    let merged: HashMap<_, _> = ctx.merge_attrs(&scene.attrs);
                    codegen
                        .generate_global_scene(scene, counter, &ctx, &merged)
                        .expect("シーン生成");
                }
            }
        }
        // ランタイムと同一の後処理（normalize）。
        let lua_code = normalize_output(&String::from_utf8(buf).expect("UTF-8"));
        (lua_code, sink.into_line_map())
    }

    /// 生成 `.lua` を bare VM で走らせるためのスタブ前文を load する。
    ///
    /// `require "pasta"` / `require "pasta.global"` をスタブ化し、`PASTA.create_scene`
    /// が返す SCENE テーブルをグローバル `LAST_SCENE` に捕捉する（do ブロック外から
    /// `__start__` を起動するため）。`ACT` はトーク呼び出しを no-op に潰すスタブ。
    /// 生成 `.lua` 自体は別チャンク（`@slice.lua`）として load するため、その内部
    /// 行番号は 1..N のまま不変（前文の行ズレは影響しない）。
    fn load_stub_preamble(lua: &Lua) {
        const PREAMBLE: &str = r#"
            local stub_mt = {}
            stub_mt.__index = function(_, _) return function(...) return setmetatable({}, stub_mt) end end
            stub_mt.__call = function(_, ...) return setmetatable({}, stub_mt) end
            local function stub() return setmetatable({}, stub_mt) end

            local PASTA = {}
            function PASTA.create_scene(_)
                local SCENE = setmetatable({}, stub_mt)
                LAST_SCENE = SCENE
                return SCENE
            end
            function PASTA.create_actor(_) return stub() end
            function PASTA.create_word(_) return stub() end
            package.preload["pasta"] = function() return PASTA end
            package.preload["pasta.global"] = function() return stub() end

            -- act.<actor> は talk() を持つ no-op オブジェクトを返すスタブ。
            local act_field_mt = { __index = function() return function() end end }
            ACT = setmetatable({}, { __index = function() return setmetatable({}, act_field_mt) end })
            function ACT.init_scene() return {}, {} end
        "#;
        lua.load(PREAMBLE)
            .set_name("@slice_preamble")
            .exec()
            .expect("スタブ前文の load は成功する");
    }

    /// ALL_SAFE VM（`jit` あり・`debug` 除外）。`install` 自身が `jit.off()` を撃つ。
    fn build_all_safe_vm() -> Lua {
        unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, LuaOptions::default()) }
    }

    // =======================================================================
    // (1) マップ構築 + R4.4: 生成 .lua 行 → .pasta 行 変換
    // =======================================================================

    /// `SliceSink` を装着した実トランスパイルが、代表トークの生成 `.lua` 行 →
    /// `.pasta` 行の対応を記録し、`resolve_lua_to_pasta`（R4.4）がその `.lua` 行を
    /// 具体的な `.pasta` 行（2）へ変換できることを実コードで実証する。
    #[test]
    fn slice_resolves_lua_line_to_pasta_line() {
        let (lua_code, map) = transpile_slice();

        // マップは代表経路を捕捉している（部分マップ・非空）。
        assert!(
            !map.is_empty(),
            "SliceSink は少なくとも代表トークの対応を記録する"
        );

        // 生成 .lua の TALK_LUA_LINE 行が実際にトーク呼び出しであること（数合わせ
        // ではなく実生成行であることの裏取り）。
        let nth = lua_code
            .lines()
            .nth((TALK_LUA_LINE - 1) as usize)
            .expect("生成 .lua は TALK_LUA_LINE 行を持つ");
        assert!(
            nth.contains(":talk("),
            "生成 .lua の {TALK_LUA_LINE} 行はトーク呼び出しであるべき: {nth:?}"
        );

        // R4.4: 生成 .lua 行 → .pasta 行 変換。
        let pos = resolve_lua_to_pasta(&map, TALK_LUA_LINE)
            .expect("R4.4: 代表トークの .lua 行は .pasta 行へ変換できる");
        assert_eq!(
            pos.line, TALK_PASTA_LINE,
            "R4.4: 生成 .lua 行 {TALK_LUA_LINE} は .pasta 行 {TALK_PASTA_LINE} へ対応する"
        );
        assert_eq!(
            pos.file, SLICE_PASTA_FILE,
            "PastaPos は元 .pasta ファイルパスを保持する"
        );
    }

    /// 逆引き（`.pasta` 行 → 生成 `.lua` 行）が代表トークについて TALK_LUA_LINE を
    /// 返すこと。R4.5 の BP 翻訳の基盤。
    #[test]
    fn slice_reverse_maps_pasta_line_to_lua_line() {
        let (_lua_code, map) = transpile_slice();
        let lua_lines = map.lua_lines_for_pasta(TALK_PASTA_LINE);
        assert_eq!(
            lua_lines,
            vec![TALK_LUA_LINE],
            "逆引き: .pasta 行 {TALK_PASTA_LINE} は生成 .lua 行 {TALK_LUA_LINE} へ対応する"
        );
        // 対応の無い .pasta 行は空。
        assert!(
            map.lua_lines_for_pasta(999).is_empty(),
            "対応の無い .pasta 行は空ベクタ"
        );
    }

    // =======================================================================
    // (2) R4.5: .pasta 行に設定した BP が、逆引きした生成 .lua 行で実フックヒット
    // =======================================================================

    /// R4.5（中核の end-to-end 実証）: 代表 `.pasta` 行（2）に設定したブレークポイント
    /// を逆引きで生成 `.lua` 行（11）へ翻訳し、その生成 `.lua` を **実デバッグ
    /// セッション**で走らせて、フックが翻訳先の `.lua` 行で **実際に停止**することを
    /// 実コード（モックでない）で実証する。`SliceSink` の記録から導いた行であり、
    /// 番号の偶然一致ではない。
    #[test]
    fn slice_pasta_breakpoint_hits_mapped_lua_line() {
        let (lua_code, map) = transpile_slice();

        // .pasta 行 2 の BP を逆引きで生成 .lua 行へ翻訳（R4.5 の翻訳）。
        let mapped_lua_lines = map.lua_lines_for_pasta(TALK_PASTA_LINE);
        assert_eq!(
            mapped_lua_lines,
            vec![TALK_LUA_LINE],
            "前提: .pasta 行 {TALK_PASTA_LINE} は生成 .lua 行 {TALK_LUA_LINE} へ翻訳される"
        );
        let bp_lua_line = mapped_lua_lines[0];

        // 翻訳先 .lua 行に BP を設定（フックは (source, line) で突合する）。
        let breakpoints = BreakpointSet::new();
        breakpoints.set_breakpoints(&SourceRef::new(SLICE_LUA_SOURCE), &[bp_lua_line]);

        // ホストスレッドで実 DebugSession を駆動し、生成 .lua を走らせる。
        let mut host = start_slice_host(breakpoints, lua_code, bp_lua_line);

        // R4.5: BP がヒットして停止イベントが届く。
        let stopped = host
            .event_rx
            .recv_timeout(WATCHDOG)
            .expect("R4.5: .pasta 由来 BP が生成 .lua 行で停止するはず");
        assert_eq!(
            stopped,
            SessionEvent::Stopped {
                reason: StopReason::Breakpoint,
                thread_id: 1,
            },
            "停止理由は Breakpoint（.pasta 行 {TALK_PASTA_LINE} 由来の BP）"
        );

        // フックが突合した実停止行が、翻訳先 .lua 行であること（裏取り）。
        assert_eq!(
            host.stop_line(),
            bp_lua_line,
            "フックは翻訳先の生成 .lua 行 {bp_lua_line} で停止する（番号偶然ではない）"
        );

        // 続行してホストスレッドを完了させる（ハングしない）。LuaJIT の行フックは
        // 1 ソース行に複数バイトコードがあると同一行で複数回発火し得る（トーク行は
        // index＋メソッド呼び出しで複数命令）。BP ヒットの実証は上の最初の停止で
        // 済んでいるので、ここでは再停止に対しても Continue を送り、ホストスレッドの
        // 完了まで駆動する（ハングしないことの担保）。
        host.cmd_tx
            .send(SessionCommand::Continue)
            .expect("Continue 送信");
        host.drain_to_completion();
    }

    // =======================================================================
    // (E2E) task 8.3: 代表 .pasta 経路を 1 フローで端から端まで実証
    //   transpile（SliceSink 装着）→ 実記録から LineMap 導出 → .pasta 行 BP を
    //   逆引きで生成 .lua 行へ翻訳 → 実 DebugSession で BP ヒット（R4.5）→
    //   その停止フレームを LineMap-backed .pasta SourceResolver 付き DapAdapter で
    //   encode し、応答が .pasta パス＋.pasta 行を報告する（R4.4）。
    //   上の (1)〜(3) が個別に証明した断片を、ひとつの明示的な E2E 証跡へ統合する。
    // =======================================================================

    /// 代表 `.pasta` 経路の **統合 E2E**（task 8.3・R4.4＋R4.5 を 1 フローで）。
    ///
    /// 各ステージのマップは **実 `SliceSink` 記録から導出**（ハードコードの偶然
    /// 一致ではない）であり、`transpile_slice` の実パイプライン（codegen→normalize）
    /// を辿る。フローは:
    ///
    /// 1. **transpile + map**: `SliceSink` を装着して代表 `.pasta` を実トランスパイル
    ///    し、`record(out_line, span)` から `LineMap` を導出する。
    /// 2. **.pasta BP → .lua 行翻訳**: `.pasta` 行 2 の BP を `lua_lines_for_pasta`
    ///    で生成 `.lua` 行へ翻訳する（R4.5 の翻訳・逆引き）。
    /// 3. **実 DebugSession で BP ヒット**: 翻訳先 `.lua` 行に BP を張り、生成 `.lua`
    ///    を実 `DebugSession` で走らせ、フックがその行で **実際に停止**する（R4.5）。
    /// 4. **.pasta 行/source で報告**: 停止フレームを LineMap-backed `.pasta`
    ///    `SourceResolver`（task 5.2 の DAP source 口）付き `DapAdapter` で encode し、
    ///    `stackTrace` 応答が `.pasta` パス＋`.pasta` 行（2）を報告する（R4.4）。
    ///
    /// アサートは具体的（厳密な `.pasta` 行 2・厳密な source パス `slice.pasta`）で
    /// load-bearing。BP ヒット行と DAP 報告行が共に **同一の実記録**（生成 `.lua` 行
    /// `TALK_LUA_LINE`）から導かれることで、`.pasta`↔`.lua` の往復が代表経路で
    /// 解決可能であることを 1 本のテストで証明する。
    ///
    /// NOTE: full DAP-over-TCP `enable()` 経路は既定の `.lua` resolver を使い、スライス
    /// resolver の gated 注入口を持たない（スライスは意図的に「薄い」）。よって E2E は
    /// 5.3 と同じく `DebugSession` ＋ `DapAdapter` レベルで駆動する。これがスライスの
    /// 目的における end-to-end である（`enable()` への gated フック追加は薄いスライスの
    /// 範囲外）。
    #[test]
    fn slice_e2e_pasta_breakpoint_hits_and_reports_pasta_line() {
        // --- ステージ 1: 実トランスパイルで LineMap を導出（ハードコードでない） ---
        let (lua_code, map) = transpile_slice();
        assert!(
            !map.is_empty(),
            "E2E 前提: SliceSink は実記録から代表経路の対応を導出する"
        );

        // --- ステージ 2: .pasta 行 BP を逆引きで生成 .lua 行へ翻訳（R4.5 翻訳） ---
        let mapped_lua_lines = map.lua_lines_for_pasta(TALK_PASTA_LINE);
        assert_eq!(
            mapped_lua_lines,
            vec![TALK_LUA_LINE],
            "E2E: .pasta 行 {TALK_PASTA_LINE} は実記録から生成 .lua 行 {TALK_LUA_LINE} へ翻訳される"
        );
        let bp_lua_line = mapped_lua_lines[0];

        // --- ステージ 3: 実 DebugSession で翻訳先 .lua 行の BP がヒット（R4.5） ---
        let breakpoints = BreakpointSet::new();
        breakpoints.set_breakpoints(&SourceRef::new(SLICE_LUA_SOURCE), &[bp_lua_line]);
        let mut host = start_slice_host(breakpoints, lua_code, bp_lua_line);

        let stopped = host
            .event_rx
            .recv_timeout(WATCHDOG)
            .expect("E2E(R4.5): .pasta 由来 BP が生成 .lua 行で停止するはず");
        assert_eq!(
            stopped,
            SessionEvent::Stopped {
                reason: StopReason::Breakpoint,
                thread_id: 1,
            },
            "E2E: 停止理由は Breakpoint（.pasta 行 {TALK_PASTA_LINE} 由来の BP）"
        );
        assert_eq!(
            host.stop_line(),
            bp_lua_line,
            "E2E: フックは翻訳先の生成 .lua 行 {bp_lua_line} で実際に停止する（番号偶然ではない）"
        );

        // --- ステージ 4: 同じ停止フレームを .pasta source/.pasta 行で報告（R4.4） ---
        // BP ヒット時にフックが突合した実停止行（= bp_lua_line）でフレームを組み、
        // LineMap-backed の .pasta resolver を挿した DapAdapter で encode する。
        // 報告の .pasta 行は「実停止行 → 実記録 LineMap」で導かれ、ステージ 3 と
        // 同一の記録に根ざす（往復の整合）。
        let resolver_map = map.clone();
        let mut dap = DapAdapter::new();
        dap.set_source_resolver(Box::new(move |lua_source: &str, lua_line: u32| {
            match resolve_lua_to_pasta(&resolver_map, lua_line) {
                Some(pos) => crate::debug::dap::ResolvedSource {
                    source: json!({ "path": pos.file }),
                    line: pos.line,
                },
                None => default_source_resolver()(lua_source, lua_line),
            }
        }));
        dap.decode_request(&json!({
            "seq": 1, "type": "request", "command": "stackTrace", "arguments": { "threadId": 1 }
        }));
        let out = dap.encode_event(SessionEvent::Stack(vec![FrameInfo {
            source: SLICE_LUA_SOURCE.to_string(),
            line: host.stop_line(), // 実停止行（= 翻訳先 .lua 行）から報告
            func_name: Some("__start__".to_string()),
        }]));
        let frames = out[0]["body"]["stackFrames"]
            .as_array()
            .expect("stackFrames");
        assert_eq!(
            frames[0]["source"]["path"], SLICE_PASTA_FILE,
            "E2E(R4.4): 停止フレームは .pasta パス {SLICE_PASTA_FILE} で報告される"
        );
        assert_eq!(
            frames[0]["line"], TALK_PASTA_LINE,
            "E2E(R4.4): 停止フレームの行は .pasta 行 {TALK_PASTA_LINE}（実停止 .lua 行 {bp_lua_line} の逆写像）"
        );

        // --- ホストスレッドを完走させる（ハングしない・同一行複数発火を解く） ---
        host.cmd_tx
            .send(SessionCommand::Continue)
            .expect("Continue 送信");
        host.drain_to_completion();
    }

    // =======================================================================
    // (3) R4.3/R4.4 提示: LineMap から組んだ SourceResolver が stackTrace を
    //     .pasta で提示できる（DAP source 取り扱い口への接続実証）。
    // =======================================================================

    /// `LineMap` から組んだ `.pasta` SourceResolver を `DapAdapter` に挿すと、
    /// 生成 `.lua` 行のスタックフレームが `.pasta` パス＋`.pasta` 行で提示される
    /// （R4.4 の提示・5.2 の口へ接続）。既定 resolver は生成 `.lua` のまま（R4.3）。
    #[test]
    fn slice_source_resolver_presents_pasta_in_stack_trace() {
        let (_lua_code, map) = transpile_slice();

        let mut dap = DapAdapter::new();
        // LineMap を参照する .pasta resolver を挿す。対応が有れば .pasta を、
        // 無ければ生成 .lua のまま提示（既定へフォールバック）。
        dap.set_source_resolver(Box::new(move |lua_source: &str, lua_line: u32| {
            match resolve_lua_to_pasta(&map, lua_line) {
                Some(pos) => crate::debug::dap::ResolvedSource {
                    source: json!({ "path": pos.file }),
                    line: pos.line,
                },
                None => default_source_resolver()(lua_source, lua_line),
            }
        }));

        dap.decode_request(&json!({
            "seq": 1, "type": "request", "command": "stackTrace", "arguments": { "threadId": 1 }
        }));
        let out = dap.encode_event(SessionEvent::Stack(vec![FrameInfo {
            source: SLICE_LUA_SOURCE.to_string(),
            line: TALK_LUA_LINE,
            func_name: Some("__start__".to_string()),
        }]));
        let frames = out[0]["body"]["stackFrames"]
            .as_array()
            .expect("stackFrames");
        // 提示は .pasta パス＋.pasta 行（R4.4）。
        assert_eq!(
            frames[0]["source"]["path"], SLICE_PASTA_FILE,
            "R4.4: 停止フレームは .pasta パスで提示される"
        );
        assert_eq!(
            frames[0]["line"], TALK_PASTA_LINE,
            "R4.4: 停止フレームの行は .pasta 行 {TALK_PASTA_LINE}"
        );
    }

    // -----------------------------------------------------------------------
    // ホストスレッド駆動（session.rs のテストハーネス流儀を踏襲）
    // -----------------------------------------------------------------------

    struct SliceHost {
        cmd_tx: mpsc::Sender<SessionCommand>,
        event_rx: mpsc::Receiver<SessionEvent>,
        /// フックが `on_line` 直前に記録した行（停止時の停止行）。
        last_line: Arc<std::sync::atomic::AtomicU32>,
        handle: Option<JoinHandle<Result<(), String>>>,
    }

    impl SliceHost {
        fn stop_line(&self) -> u32 {
            self.last_line.load(std::sync::atomic::Ordering::SeqCst)
        }

        /// 再停止（同一トーク行の複数発火など）に対し Continue を送り続け、ホスト
        /// スレッドの完了まで駆動する。完了の検出は join を別スレッドで待ちつつ、
        /// 並行して `Continue` を再送することで行う（停止コアは無期限ブロックなので
        /// 残った停止を解かないと join がハングする）。
        fn drain_to_completion(&mut self) {
            let handle = self.handle.take().expect("join は 1 回だけ");
            let (done_tx, done_rx) = mpsc::channel();
            std::thread::spawn(move || {
                let _ = done_tx.send(handle.join());
            });

            let deadline = std::time::Instant::now() + WATCHDOG;
            loop {
                match done_rx.recv_timeout(Duration::from_millis(50)) {
                    Ok(joined) => {
                        joined
                            .expect("ホストスレッドは panic しない")
                            .expect("シナリオは完走する");
                        return;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        assert!(
                            std::time::Instant::now() < deadline,
                            "Continue を送り続けてもホストスレッドが完了しない（ハング）"
                        );
                        // まだ停止しているかもしれないので Continue を再送（再停止の解除）。
                        // 受信側が既に終了していても無害（送信失敗は無視）。
                        let _ = self.cmd_tx.send(SessionCommand::Continue);
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        panic!("join チャネルが切断された");
                    }
                }
            }
        }
    }

    /// 生成 `.lua` を走らせる実 `DebugSession` ホストを起動する。
    ///
    /// 別スレッドで ALL_SAFE VM を構築（`mlua::Lua` は `!Send`・境界を越えない）し、
    /// `DebugSession` を `LineHook` としてフック設置、スタブ前文 load 後に生成 `.lua`
    /// を `@slice.lua` として load し、`LAST_SCENE.__start__(ACT)` で代表トーク行を
    /// 実行する。フックラッパは `on_line` 直前の行を `last_line` へ記録する。
    fn start_slice_host(
        breakpoints: BreakpointSet,
        lua_code: String,
        _bp_lua_line: u32,
    ) -> SliceHost {
        let (cmd_tx, cmd_rx) = mpsc::channel::<SessionCommand>();
        let (event_tx, event_rx) = mpsc::channel::<SessionEvent>();

        let last_line = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let hook_last_line = Arc::clone(&last_line);

        let handle = std::thread::spawn(move || -> Result<(), String> {
            let lua = build_all_safe_vm();
            let session = DebugSession::new(breakpoints, cmd_rx, event_tx);

            let handler = move |lua: &Lua, debug: &Debug| {
                // 突合対象（@slice.lua）の行だけ last_line へ記録（前文/起動チャンクは無視）。
                let src = debug
                    .source()
                    .source
                    .as_ref()
                    .map(|c| c.as_ref().to_string())
                    .unwrap_or_default();
                if src == SLICE_LUA_SOURCE {
                    let line = debug.current_line().unwrap_or(0) as u32;
                    hook_last_line.store(line, std::sync::atomic::Ordering::SeqCst);
                }
                session.on_line(lua, debug)
            };

            crate::debug::hook::install(&lua, handler).map_err(|e| e.to_string())?;

            // スタブ前文 → 生成 .lua（@slice.lua・内部行は 1..N 不変）→ __start__ 起動。
            load_stub_preamble(&lua);
            lua.load(&lua_code)
                .set_name(SLICE_LUA_SOURCE)
                .exec()
                .map_err(|e| format!("生成 .lua の load/exec: {e}"))?;
            lua.load("LAST_SCENE.__start__(ACT)")
                .set_name("@slice_invoke")
                .exec()
                .map_err(|e| format!("__start__ 起動: {e}"))?;
            lua.remove_global_hook();
            Ok(())
        });

        SliceHost {
            cmd_tx,
            event_rx,
            last_line,
            handle: Some(handle),
        }
    }
}
