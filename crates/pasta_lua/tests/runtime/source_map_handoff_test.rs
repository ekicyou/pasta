//! Task 4.4 — ランタイムへの集約ソースマップ受け渡しと提示モード供給（統合）。
//!
//! 仕様参照:
//! - requirements.md **3.1**（トランスパイル時にソースマップ生成が有効なとき、構築した
//!   ソースマップをメモリ内に保持し、ブレークポイント解決/停止位置提示の双方向変換に
//!   使用する）。本テストはオーケストレーション側、すなわち「トランスパイル→構築→
//!   enable」の順でマップがランタイムへ到達・保持されることを表明する。
//! - requirements.md **6.3**（作者がデバッグ構成で提示モードを指定すると、その
//!   セッションに指定モードを適用する）。pasta.toml `[debug]` の提示モード/サイドカー
//!   設定がファイル経由で resolve に供給され、解決済み設定へ反映されることを表明する。
//! - design.md File Structure Plan `runtime/mod.rs` 168（集約 `Arc<SourceMap>` を保持し
//!   `with_config`→`enable()` へ受け渡し）/ `loader/config.rs` 170（`[debug]` 提示モード
//!   + サイドカー供給）。
//!
//! # 観測可能な「done」
//!
//! 1. **マップが順序通り enable へ到達（3.1）**: `[debug] enabled = true` の base_dir を
//!    `PastaLoader::load_with_config` で読み込むと、ランタイムは（トランスパイル後に
//!    構築した）集約 `Arc<SourceMap>` を保持し、デバッグハンドルも保持する。
//! 2. **デバッグ無効時はマップ非構築（3.1/7.1）**: 既定（`[debug]` 無し）では
//!    ランタイムはマップを保持しない（`None`）。ハンドルも無い（ゼロコスト）。
//! 3. **ファイル提示モード反映（6.3）**: pasta.toml `[debug] present_as = "lua"` を
//!    与えると、解決済みデバッグ設定の提示モードが `.lua` になる。

use std::path::Path;

use pasta_lua::loader::{CacheManager, PastaLoader};
use pasta_lua::{RuntimeConfig, SourceMode};

/// テスト用に base_dir を作り、`.pasta` と指定の pasta.toml を配置する。
/// pasta_scripts / scriptlibs はクレートルートからコピーしてランタイム初期化を成立させる。
fn make_base_dir(temp: &Path, pasta_toml: &str) {
    std::fs::create_dir_all(temp.join("dic/test")).unwrap();
    std::fs::write(
        temp.join("dic/test/hello.pasta"),
        "\
＊あいさつ
  さくら：「おはよう！」
  さくら：「げんき？」
",
    )
    .unwrap();
    std::fs::write(temp.join("pasta.toml"), pasta_toml).unwrap();

    let crate_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for sub in ["pasta_scripts", "scriptlibs"] {
        let src = crate_root.join(sub);
        let dst = temp.join(sub);
        if src.exists() {
            std::fs::create_dir_all(&dst).unwrap();
            copy_dir(&src, &dst).unwrap();
        }
    }
}

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest = dst.join(entry.file_name());
        if path.is_dir() {
            if entry.file_name() == "profile" {
                continue;
            }
            std::fs::create_dir_all(&dest)?;
            copy_dir(&path, &dest)?;
        } else {
            std::fs::copy(&path, &dest)?;
        }
    }
    Ok(())
}

/// 3.1: デバッグ有効時、トランスパイル後に構築した集約 `Arc<SourceMap>` がランタイムへ
/// 到達・保持され、enable へ受け渡される（ハンドル保持で enable 経路を表明）。
#[test]
fn enabled_debug_runtime_holds_aggregated_source_map_in_order() {
    let temp = tempfile::TempDir::new().unwrap();
    // port 0 は pasta.toml では指定できないが、enabled だけで足りる（bind は
    // 既定ポートだと競合し得るため、ここでは保持の有無のみを確認する）。
    make_base_dir(
        temp.path(),
        "\
[loader]
debug_mode = true

[debug]
enabled = true
port = 0
",
    );

    let runtime = PastaLoader::load_with_config(temp.path(), RuntimeConfig::new())
        .expect("loader must build an enabled-debug runtime");

    // (1) デバッグが有効化されている（enable が呼ばれハンドルを保持）。
    assert!(
        runtime.debug_enabled(),
        "3.1/4.2: enabled [debug] must install the backend (handle held)"
    );

    // (2) 集約ソースマップがランタイムに保持されている（トランスパイル後に構築され
    //     enable へ渡された＝順序通り到達した証拠）。
    let map = runtime
        .debug_source_map()
        .expect("3.1: enabled debug runtime must hold the aggregated Arc<SourceMap>");

    // (3) マップは実際に当該 `.pasta` のチャンクを含む（空マップではない）。
    //     チャンク名キーはローダと同じ `CacheManager::source_to_cache_path` 由来で
    //     再構築する（ハードコードしない）。少なくとも 1 つの `.lua` 行が `.pasta`
    //     位置へ前方解決できる。
    let pasta_file = temp.path().join("dic/test/hello.pasta");
    let cache_manager = CacheManager::new(temp.path().to_path_buf(), "profile/pasta/cache/lua");
    let chunk = cache_manager
        .source_to_cache_path(&pasta_file)
        .to_string_lossy()
        .to_string();
    let resolved_any = (1u32..=200)
        .filter_map(|lua_line| map.resolve_lua_to_pasta(&chunk, lua_line))
        .next();
    assert!(
        resolved_any.is_some(),
        "3.1: held map must contain the transpiled chunk's mappings (chunk={chunk}, file={})",
        pasta_file.display()
    );
}

/// 3.1/7.1: デバッグ無効（[debug] 無し）ではマップを構築・保持しない（None）。
/// ハンドルも無い（ゼロコスト）。
#[test]
fn disabled_debug_runtime_holds_no_source_map() {
    let temp = tempfile::TempDir::new().unwrap();
    make_base_dir(
        temp.path(),
        "\
[loader]
debug_mode = true
",
    );

    let runtime = PastaLoader::load_with_config(temp.path(), RuntimeConfig::new())
        .expect("loader must build a disabled-debug runtime");

    assert!(
        !runtime.debug_enabled(),
        "7.1: no [debug] => debug disabled (zero-cost)"
    );
    assert!(
        runtime.debug_source_map().is_none(),
        "3.1/7.1: disabled debug must NOT build/hold a source map"
    );
}

/// 6.3: pasta.toml `[debug] present_as = "lua"` を与えると、解決済みデバッグ設定の
/// 提示モードが `.lua` になる（env/attach 上書き無し）。
#[test]
fn file_present_as_lua_is_applied_to_resolved_debug_config() {
    let temp = tempfile::TempDir::new().unwrap();
    make_base_dir(
        temp.path(),
        "\
[loader]
debug_mode = true

[debug]
enabled = true
port = 0
present_as = \"lua\"
",
    );

    let runtime = PastaLoader::load_with_config(temp.path(), RuntimeConfig::new())
        .expect("loader must build runtime with present_as=lua");

    assert!(runtime.debug_enabled(), "enabled debug runtime");
    assert_eq!(
        runtime.debug_source_mode(),
        Some(SourceMode::Lua),
        "6.3: pasta.toml [debug] present_as=\"lua\" must be applied to the session"
    );
}
