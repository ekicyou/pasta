//! Task 1.1 — チャンク名の実機検証とチャンク命名戦略の確定（Validation Hook）。
//!
//! 仕様参照:
//! - requirements.md **4.2**（`.pasta` 行 BP を `.lua` 行群へ登録するための突合キー）/
//!   **5.1**（停止位置を `.pasta` 提示するための `.lua`→`.pasta` 解決キー）
//! - design.md "Source Identity（議題2 確定・構築時一致＋正規化＋実測）" 437-440
//! - design.md ChunkSourceMap/SourceMap "Implementation Notes" → "Validation Hook（最優先タスク）" 475
//! - design.md "Data Contracts & Integration" → チャンク名キー 604
//!   （`@<絶対 .lua パス>`・`loader::cache::source_to_cache_path` 由来）
//! - research.md D-2（本番チャンク名 = `@<キャッシュ .lua の絶対パス>`）
//!
//! # このテストの目的（実証 = spike）
//!
//! 「ラインフックが実機で報告する `lua_Debug.source` 文字列」と「ローダのキャッシュ
//! パス構築（[`CacheManager::source_to_cache_path`]）由来のチャンク名キー」が、
//! **正規化（`@` 除去・パス区切り統一・Windows 大小無視）後に一致する**ことを、
//! **実トランスパイル → 実 `require` ロード → 実ラインフック**で実測検証する。
//!
//! ハードコードした期待値ではなく、ランタイムが実際に報告する source 文字列を
//! フックで捕捉して突合する（design 475 の Validation Hook 要求）。
//!
//! # 確定した命名戦略（このテストが裏付ける判断）
//!
//! **「構築時一致 ＋ 正規化」（design の既定）で十分。`set_name` 明示命名は不要。**
//!
//! 実機（Windows）実測で判明した重要事実:
//! - 本番 `require` 経路のフック source は **`@` 付き ＋ 区切り混在**:
//!   `@<package.path 前置部=`/`>/<モジュール名展開部=`\`>` という形になる。
//!   （`package.path` は forward-slash で設定され、searcher がモジュール名
//!   `pasta.scene.…` の `.` を OS 区切り `\` へ置換して `?` に埋めるため。）
//! - ローダ由来キー（[`CacheManager::source_to_cache_path`] の `PathBuf`）は Windows
//!   では全 backslash。
//! - 両者はバイト一致しないが、[`canonicalize_chunk_name`]（`@` 除去・`\`→`/` 統一・
//!   Windows 小文字化）後は **完全一致**する。よって区切り統一を含む正規化が必須で
//!   あり、それで足りる（`set_name` 改修は不要）。
//!
//! `debug::source_map` の本番化（task 3.1・gate 撤去）により、このテストは default
//! features で常時コンパイル・実行される（7.3）。
//!
//! # lua-require-robustness task 3.5（チャンク識別子とソースマップの無回帰検証）
//!
//! 上記の往復を **本番と同じ searcher**（[`install_module_searcher`]）を設置した VM で
//! 計測する（design.md "Testing Strategy > Integration Tests（pasta_lua）" 5）。
//! あわせて非 ANSI ディレクトリ名配下へ設置したケースを追加する。design.md
//! 「設計フェーズの実測結果」M6 のとおり、本仕様以前は非 ASCII ディレクトリ配下の
//! チャンク識別子が ANSI バイト列となり UTF-8 のソースマップキーと一致しなかった。
//! 本仕様の searcher 置換により、非 ASCII パスでも ASCII パスと同じ構成規則の識別子が
//! 付与され、ソースマップのキーへ解決できる（要件 3.6・3.8）。
//!
//! `mod common` の宣言により `PASTA_DEBUG` 中和ガードが適用される（要件 6.4）。

mod common;

use std::path::Path;
use std::sync::{Arc, Mutex};

use mlua::{Debug, HookTriggers, Lua, LuaOptions, StdLib, VmState};

use common::NON_ANSI_DIR_NAME;
use pasta_dsl::parser::parse_str;
use pasta_lua::LuaTranspiler;
use pasta_lua::debug::source_map::canonicalize_chunk_name;
use pasta_lua::loader::CacheManager;
use pasta_lua::runtime::install_module_searcher;

/// 代表 `.pasta`（単純シーン 1 本・トーク 1 行）。トランスパイルして実ロードできる
/// 最小入力。命名検証が目的なので内容は最小でよい。
const PASTA_SRC: &str = "\
＊あいさつ
  さくら：「こんにちは！」
";

/// `.pasta` のキャッシュ相対元（`dic/...` 配下を模す）。
const DIC_REL: &str = "dic/baseware/system.pasta";

/// 期待モジュール名（`source_to_module_name` と一致するはず）。
const EXPECTED_MODULE: &str = "pasta.scene.baseware.system";

/// ALL_SAFE VM（`jit` あり・`debug` 除外）＋ 本番 searcher。hook install は別途行う。
///
/// 本番（`PastaLuaRuntime::setup_package_path`）と同じく `package.loaders[2]` を Rust
/// 実装へ置換し、チャンク識別子の生成経路を本番と一致させる。VM は
/// `Lua::unsafe_new_with` で構築する必要がある（mlua の安全モード `Lua::new` では
/// `package.loaders` が 4 要素にならず設置が拒否される）。
fn build_all_safe_vm() -> Lua {
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, LuaOptions::default()) };
    install_module_searcher(&lua).expect("本番 searcher の設置に失敗");
    lua
}

/// `LoaderContext::generate_package_path` と同形の package.path 文字列を組む
/// （forward-slash 正規化・`?.lua` ＋ `?/init.lua` の 2 パターン）。本番のチャンク名
/// 生成経路を忠実に再現するために、ローダと同じ区切り規約を用いる。
fn package_path_for(cache_lua_root: &Path) -> String {
    let root = cache_lua_root.to_string_lossy().replace('\\', "/");
    format!("{root}/?.lua;{root}/?/init.lua")
}

/// 往復検証の実測値。
struct RoundTrip {
    /// ラインフックが報告した生 source 文字列。
    hook_source_raw: String,
    /// ローダのキャッシュパス構築由来のキー（`@` なし）。
    loader_key_raw: String,
}

/// `base_dir` にゴーストを設置し、トランスパイル → `require` → ラインフック →
/// ソースマップ照合の往復を実測・検証する（設置パスに依存しない検証本体）。
fn run_round_trip(base_dir: &Path) -> RoundTrip {
    // --- 0. `.pasta` ソースを用意 ------------------------------------------------
    let base_dir = base_dir.to_path_buf();
    let source_path = base_dir.join(DIC_REL);
    std::fs::create_dir_all(source_path.parent().unwrap()).expect("mkdir dic");
    std::fs::write(&source_path, PASTA_SRC).expect("write .pasta");

    // --- 1. 実トランスパイル（本番と同一の LuaTranspiler 経路） -------------------
    let pasta_file = parse_str(PASTA_SRC, &source_path.to_string_lossy()).expect("parse .pasta");
    let transpiler = LuaTranspiler::default();
    let mut out: Vec<u8> = Vec::new();
    transpiler
        .transpile(&pasta_file, &mut out)
        .expect("transpile ok");
    let lua_code = String::from_utf8(out).expect("utf-8 lua");

    // --- 2. ローダのキャッシュ構築コードを *再利用* してキャッシュへ保存 ----------
    //     （キーをでっち上げず、本番ローダと同一の path 構築を共有する: design 438）
    let cache_manager = CacheManager::new(base_dir.clone(), "profile/pasta/cache/lua");
    cache_manager.prepare_cache_dir().expect("prepare cache");

    // ローダ由来チャンク名キー（= source_to_cache_path の絶対パス）。
    let loader_cache_path = cache_manager.source_to_cache_path(&source_path);
    let module_name = cache_manager.source_to_module_name(&source_path);
    assert_eq!(
        module_name, EXPECTED_MODULE,
        "前提: モジュール名は require 解決の元になる"
    );

    // save_cache で実ファイルを書く（require が package.path で見つけられるように）。
    let saved_module = cache_manager
        .save_cache(&source_path, &lua_code)
        .expect("save cache");
    assert_eq!(saved_module, EXPECTED_MODULE);
    assert!(
        loader_cache_path.exists(),
        "キャッシュ .lua が source_to_cache_path の位置に保存されている"
    );

    // ローダ由来キー文字列（design 604: `@<絶対 .lua パス>` の `@` なし版を保持し、
    // 突合は正規化で行う）。
    let loader_key_raw = loader_cache_path.to_string_lossy().to_string();

    // --- 3. 本番ロード機構（package.path + require）で実ロードし、フック source を実測 -
    let cache_lua_root = base_dir.join("profile/pasta/cache/lua");
    let pkg_path = package_path_for(&cache_lua_root);

    let lua = build_all_safe_vm();

    // ロード対象チャンクで報告された生 source 文字列を全捕捉する。
    let captured: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&captured);

    // 本仕様のフック（hook.rs install）と同様 jit.off + global line hook を仕掛ける。
    lua.load("jit.off()").exec().expect("jit.off");
    lua.set_global_hook(HookTriggers::EVERY_LINE, move |_lua, debug: &Debug| {
        let src = debug
            .source()
            .source
            .as_ref()
            .map(|c| c.as_ref().to_string())
            .unwrap_or_default();
        if let Ok(mut g) = sink.lock() {
            g.push(src);
        }
        Ok(VmState::Continue)
    })
    .expect("install line hook");

    // PASTA ランタイムをスタブ化（生成 .lua の require("pasta") を満たす最小実装）。
    // 命名検証が目的なので talk 等は no-op に潰す。
    const STUB: &str = r#"
        local stub_mt = {}
        stub_mt.__index = function(_, _) return function(...) return setmetatable({}, stub_mt) end end
        stub_mt.__call = function(_, ...) return setmetatable({}, stub_mt) end
        local function stub() return setmetatable({}, stub_mt) end
        local PASTA = {}
        function PASTA.create_scene(_) return setmetatable({}, stub_mt) end
        function PASTA.create_actor(_) return stub() end
        function PASTA.create_word(_) return stub() end
        function PASTA.finalize_scene() end
        package.preload["pasta"] = function() return PASTA end
        package.preload["pasta.global"] = function() return stub() end
    "#;
    lua.load(STUB)
        .set_name("@chunk_name_stub")
        .exec()
        .expect("stub preamble");

    // package.path をローダと同形にセットしてから require（本番のチャンク名生成経路）。
    lua.load(format!("package.path = [[{pkg_path}]]"))
        .set_name("@chunk_name_setpath")
        .exec()
        .expect("set package.path");
    lua.load(format!("require(\"{EXPECTED_MODULE}\")"))
        .set_name("@chunk_name_require")
        .exec()
        .expect("require transpiled module");

    lua.remove_global_hook();

    // ドライバ/スタブ由来の source を除外し、ロードしたチャンクの source を取り出す。
    // `=pasta_searcher` は本番 searcher の Lua ラッパ（searcher.rs の WRAPPER_CHUNK）。
    let driver_names = [
        "@chunk_name_stub",
        "@chunk_name_setpath",
        "@chunk_name_require",
        "=pasta_searcher",
    ];
    let chunk_sources: Vec<String> = captured
        .lock()
        .unwrap()
        .iter()
        .filter(|s| !driver_names.contains(&s.as_str()) && !s.is_empty())
        .cloned()
        .collect();

    assert!(
        !chunk_sources.is_empty(),
        "ラインフックがロードしたチャンク上で発火し source を報告するはず（実測の前提）"
    );

    // 実測した生 source（全件同一チャンクのはず）。
    let hook_source_raw = chunk_sources[0].clone();
    for s in &chunk_sources {
        assert_eq!(
            s, &hook_source_raw,
            "ロードした単一チャンクの source は一定（{chunk_sources:?}）"
        );
    }

    // --- 4. 実測の記録（CONCERNS/EVIDENCE 用・実機文字列形を残す） ---------------
    eprintln!("=== Task 1.1 chunk-name empirical findings ===");
    eprintln!("HOOK_SOURCE_RAW   = {hook_source_raw:?}");
    eprintln!("LOADER_KEY_RAW    = {loader_key_raw:?}");
    eprintln!(
        "HOOK_CANON        = {:?}",
        canonicalize_chunk_name(&hook_source_raw)
    );
    eprintln!(
        "LOADER_CANON      = {:?}",
        canonicalize_chunk_name(&loader_key_raw)
    );

    // --- 5. 実機 source 文字列の形を検証（design 604: `@<絶対 .lua パス>`） --------
    assert!(
        hook_source_raw.starts_with('@'),
        "フック source は `@` 接頭辞付き（loadfile/require 由来）。got: {hook_source_raw:?}"
    );
    assert!(
        hook_source_raw.ends_with("system.lua"),
        "フック source は対象 .lua で終わる。got: {hook_source_raw:?}"
    );

    // --- 6. 中核アサート: 正規化後に一致（命名戦略 = 構築時一致＋正規化） ----------
    let hook_canon = canonicalize_chunk_name(&hook_source_raw);
    let loader_canon = canonicalize_chunk_name(&loader_key_raw);
    assert_eq!(
        hook_canon, loader_canon,
        "requirements 4.2/5.1・design 437-440: フック source とローダ由来キーは\n\
         正規化（@除去・区切り統一・Windows大小無視）後に一致しなければならない。\n\
         一致しない場合は `set_name` 明示命名が必要（task 4.3 へ申し送り）。\n\
         HOOK_RAW={hook_source_raw:?}\nLOADER_RAW={loader_key_raw:?}"
    );

    // --- 7. set_name 不要の裏取り: *生*文字列はバイト一致しない（区切り差） --------
    //     （正規化が *必要* であることの明示・正規化なしでは突合不能）
    let hook_no_at = hook_source_raw.strip_prefix('@').unwrap();
    assert_ne!(
        hook_no_at, loader_key_raw,
        "実機では生文字列はバイト一致しない（区切り差など）→ 正規化が必須であることの確認"
    );

    RoundTrip {
        hook_source_raw,
        loader_key_raw,
    }
}

#[test]
fn hook_source_matches_loader_cache_key_after_normalization() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    run_round_trip(temp.path());
}

/// 非 ANSI ディレクトリ名配下へ設置しても、チャンク識別子がソースマップのキーへ
/// 解決されること（要件 3.6・3.8 / design 実測 M6）。
///
/// 本仕様以前はこの設置でチャンク識別子が ANSI バイト列となり、UTF-8 の
/// ソースマップキーと一致しなかった。
#[test]
fn hook_source_matches_loader_cache_key_under_non_ansi_install_path() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let base_dir = temp.path().join(NON_ANSI_DIR_NAME);
    std::fs::create_dir_all(&base_dir).expect("非 ANSI ディレクトリの作成に失敗");

    let measured = run_round_trip(&base_dir);

    // 非 ASCII 文字が欠落・置換されずにチャンク識別子へ現れる（3.8 / 実測 M6）。
    assert!(
        measured.hook_source_raw.contains(NON_ANSI_DIR_NAME),
        "フック source が非 ANSI ディレクトリ名をそのまま含むはず。got: {:?}",
        measured.hook_source_raw
    );
    assert!(
        measured.loader_key_raw.contains(NON_ANSI_DIR_NAME),
        "ソースマップ側キーも非 ANSI ディレクトリ名を含むはず。got: {:?}",
        measured.loader_key_raw
    );
}
