//! LuaJIT 標準の Lua ファイル searcher（`package.loaders[2]`）の同型置換。
//!
//! 標準との差は 2 点だけである。
//!
//! 1. ファイルを開く API を narrow `fopen` から Rust std（wide API）へ変える。
//!    std は 248 文字以上の絶対パスへ拡張長プレフィックスを内部で付与するため、
//!    設置パスが 260 文字を超えてもモジュールを解決できる（要件 1.1〜1.4・実測 M2）。
//! 2. `package.path` とモジュール名を ANSI ではなく UTF-8 として解釈する
//!    （要件 2.1・2.3）。
//!
//! 候補パスの生成規則・探索順・チャンク識別子・メッセージ文言は LuaJIT
//! `lib_package.c`（`searchpath` / `loader_Lua`）と同一に保つ。これにより ASCII
//! パスでは変更前とバイト単位で同一の結果になる（要件 3.3・3.4・実測 M3・M4）。
//!
//! パス長の判定・`\?\` の明示付与・短縮名変換・`canonicalize` は一切行わない
//! （要件 1.6・3.7）。候補パス文字列は、ファイルを開くパス・チャンク識別子・
//! エラーメッセージの 3 箇所に**そのまま**使われる。

use mlua::{Function, Lua, Result as LuaResult, Table, Value};
use std::io::Read;

/// モジュール名の `.` を置換するディレクトリ区切り（LuaJIT の `LUA_DIRSEP`）。
#[cfg(windows)]
const DIR_SEP: &str = "\\";
#[cfg(not(windows))]
const DIR_SEP: &str = "/";

/// 検索パス設定文字列の区切り（LuaJIT の `LUA_PATHSEP`）。
const PATH_SEP: char = ';';

/// テンプレート中のモジュール名プレースホルダ（LuaJIT の `LUA_PATH_MARK`）。
const PATH_MARK: char = '?';

/// 想定する `package.loaders` の要素数（preload / Lua / C / C-root・実測 M1）。
const EXPECTED_LOADERS_LEN: usize = 4;

/// 標準の Lua ファイル searcher が置かれている位置。
const LUA_LOADER_INDEX: usize = 2;

/// searcher 本体となる Lua ラッパ。
///
/// Rust 側は値を返すだけにし、ロード失敗の送出はこの Lua 側で行う。Rust の `Err`
/// をそのまま返すと `pcall(require, …)` が受け取る値が文字列から userdata へ
/// 変わるためである（実測 M4）。`error(msg, 0)` は位置情報を付けないので、
/// 標準（C 関数からの `luaL_error`）と文言がバイト単位で一致する。
const WRAPPER_CHUNK: &str = r#"
local find = ...
return function(name)
  local loader, message, is_load_error = find(name)
  if is_load_error then
    error(message, 0)
  end
  if loader ~= nil then
    return loader
  end
  return message
end
"#;

/// 探索の結果。
enum SearchOutcome {
    /// 最初に開けた候補。`candidate` は命名と読み込みに共通の文字列。
    Found { candidate: String, source: Vec<u8> },
    /// すべての候補が開けなかった。`message` は `"\n\tno file '<candidate>'"` の連結。
    NotFound { message: String },
    /// 開けたが読み込めなかった。
    Unreadable {
        candidate: String,
        cause: std::io::Error,
    },
}

/// `package.path` とモジュール名から候補パスを順に生成する（純粋関数・FS 非依存）。
///
/// 空テンプレートはスキップする。戻り値の順序がそのまま探索順である。
/// 長さの判定は行わない（要件 1.6）。
fn candidate_paths(package_path: &str, module_name: &str) -> Vec<String> {
    // 標準は `luaL_gsub(name, ".", LUA_DIRSEP)` をループの外で 1 回だけ行う。
    let name = module_name.replace('.', DIR_SEP);
    package_path
        .split(PATH_SEP)
        .filter(|template| !template.is_empty())
        .map(|template| template.replace(PATH_MARK, &name))
        .collect()
}

/// 候補を順に開き、最初に開けたものを採用する。
///
/// 開けなかった理由（不在・権限・不正なパス）は標準と同じく区別せず、次の候補へ進む。
fn search(package_path: &str, module_name: &str) -> SearchOutcome {
    let mut message = String::new();
    for candidate in candidate_paths(package_path, module_name) {
        let Ok(mut file) = std::fs::File::open(&candidate) else {
            message.push_str(&format!("\n\tno file '{candidate}'"));
            continue;
        };
        let mut source = Vec::new();
        return match file.read_to_end(&mut source) {
            Ok(_) => SearchOutcome::Found { candidate, source },
            Err(cause) => SearchOutcome::Unreadable { candidate, cause },
        };
    }
    SearchOutcome::NotFound { message }
}

/// searcher の本体（Lua ラッパから呼ばれる Rust 関数）。
///
/// 戻り値は `(loader, message, is_load_error)`。
/// - 解決成功: `(関数, nil, false)`
/// - 未検出: `(nil, no file 行の連結, false)` — `require` が他の searcher の文字列と
///   連結して `module '<name>' not found:…` を送出する。
/// - ロード失敗: `(nil, メッセージ, true)` — ラッパが `error(message, 0)` する。
fn find_module(
    lua: &Lua,
    module_name: mlua::String,
) -> LuaResult<(Option<Function>, Option<String>, bool)> {
    let module_name = String::from_utf8_lossy(&module_name.as_bytes()).into_owned();

    let package: Table = lua.globals().get("package")?;
    let Value::String(package_path) = package.get::<Value>("path")? else {
        // 標準（`findfile` の `luaL_error`）と同一文言・同一の型（文字列エラー）。
        return Ok((
            None,
            Some("'package.path' must be a string".to_string()),
            true,
        ));
    };
    // `package.path` に UTF-8 として不正なバイト列が含まれる場合は lossy 変換され、
    // その候補は開けず `no file` 行に載る（設計「Risks」）。
    let package_path = String::from_utf8_lossy(&package_path.as_bytes()).into_owned();

    match search(&package_path, &module_name) {
        SearchOutcome::Found { candidate, source } => {
            // チャンク識別子は `@` + 候補パス。標準 `luaL_loadfilex` と同一（実測 M3）。
            match lua
                .load(&source[..])
                .set_name(format!("@{candidate}"))
                .into_function()
            {
                Ok(loader) => Ok((Some(loader), None, false)),
                Err(e) => Ok((None, Some(load_error(&module_name, &candidate, &e)), true)),
            }
        }
        SearchOutcome::NotFound { message } => Ok((None, Some(message), false)),
        SearchOutcome::Unreadable { candidate, cause } => {
            let cause = format!("cannot read {candidate}: {cause}");
            Ok((
                None,
                Some(format!(
                    "error loading module '{module_name}' from file '{candidate}':\n\t{cause}"
                )),
                true,
            ))
        }
    }
}

/// 標準 `loaderror` と同一書式のロード失敗メッセージを組み立てる。
///
/// 構文エラーの本文は `SyntaxError { message }` をそのまま使う。mlua の Display が
/// 付ける `syntax error: ` 接頭辞を載せると標準と文言が食い違うためである（実測 M4）。
fn load_error(module_name: &str, candidate: &str, error: &mlua::Error) -> String {
    let cause = match error {
        mlua::Error::SyntaxError { message, .. } => message.clone(),
        other => other.to_string(),
    };
    format!("error loading module '{module_name}' from file '{candidate}':\n\t{cause}")
}

/// `package.loaders[2]` を Rust 実装の Lua ファイル searcher へ置換する。
///
/// VM 構築時（他のどの Lua コードよりも前）に 1 回だけ呼ぶ。
///
/// `package.loaders` と `package.searchers` は LuaJIT では同一テーブルのため、
/// インプレース代入が両名称から見える（実測 M1）。preload（1 番目）と C searcher
/// （3・4 番目）は触らない。
///
/// # Errors
///
/// `package.loaders` が想定レイアウト（4 要素のテーブル）でない場合に `Err` を返す。
/// mlua / LuaJIT の更新でレイアウトが変わったときに、黙って別の searcher を上書き
/// しないための検証である。この `Err` は起動失敗として伝搬し、500 とログで可視化
/// される。
pub fn install_module_searcher(lua: &Lua) -> LuaResult<()> {
    let package: Table = lua.globals().get("package")?;
    let loaders: Table = package.get("loaders")?;

    let len = loaders.raw_len();
    if len != EXPECTED_LOADERS_LEN {
        return Err(mlua::Error::runtime(format!(
            "unexpected 'package.loaders' layout: expected {EXPECTED_LOADERS_LEN} entries, found {len}"
        )));
    }

    let find = lua.create_function(find_module)?;
    let wrapper: Function = lua
        .load(WRAPPER_CHUNK)
        .set_name("=pasta_searcher")
        .into_function()?
        .call(find)?;

    loaders.raw_set(LUA_LOADER_INDEX, wrapper)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 検索パス設定文字列は `;` で分割され、空要素はスキップされる（3.1 / 3.2）。
    #[test]
    fn candidate_paths_splits_on_path_separator_and_skips_empty() {
        let got = candidate_paths(";a/?.lua;;b/?/init.lua;", "m");
        // テンプレート側の区切りはそのまま残る（置換対象はモジュール名の `.` だけ）。
        assert_eq!(got, vec!["a/m.lua".to_string(), "b/m/init.lua".to_string()]);
    }

    /// テンプレート中の `?` は最初の 1 個ではなく全て置換される。
    #[test]
    fn candidate_paths_replaces_every_mark() {
        let got = candidate_paths("x/?/?.lua", "m");
        assert_eq!(got, vec!["x/m/m.lua".to_string()]);
    }

    /// モジュール名の `.` は OS のディレクトリ区切りへ置換される（M3）。
    #[test]
    fn candidate_paths_replaces_dots_with_dir_separator() {
        let got = candidate_paths("r/?.lua", "pasta.shiori.entry");
        assert_eq!(
            got,
            vec![format!("r/pasta{DIR_SEP}shiori{DIR_SEP}entry.lua")]
        );
    }

    /// 非 ASCII のモジュール名（`dic/会話.pasta` 由来）も UTF-8 のまま扱う（2.1 / 2.3）。
    #[test]
    fn candidate_paths_keeps_non_ascii_module_name() {
        let got = candidate_paths("r/?.lua", "pasta.scene.会話");
        assert_eq!(got, vec![format!("r/pasta{DIR_SEP}scene{DIR_SEP}会話.lua")]);
    }

    /// `?` を含まないテンプレートはそのまま候補になる（標準と同じ）。
    #[test]
    fn candidate_paths_keeps_template_without_mark() {
        let got = candidate_paths("fixed.lua", "m");
        assert_eq!(got, vec!["fixed.lua".to_string()]);
    }

    /// 独自のパス長上限を持たない（1.6）。
    #[test]
    fn candidate_paths_has_no_length_limit() {
        let deep = format!("C:/{}/?.lua", "d".repeat(400));
        let got = candidate_paths(&deep, "m");
        assert_eq!(got.len(), 1);
        assert!(got[0].len() > 400);
        assert!(!got[0].contains("\\?\\"));
    }

    /// 未検出メッセージは候補順の `no file` 行の連結である（1.5 / M4）。
    #[test]
    fn search_not_found_message_lists_every_candidate_in_order() {
        let outcome = search("no/such/?.lua;other/?/init.lua", "m");
        match outcome {
            SearchOutcome::NotFound { message } => assert_eq!(
                message,
                "\n\tno file 'no/such/m.lua'\n\tno file 'other/m/init.lua'"
            ),
            _ => panic!("見つからないはずの候補で NotFound 以外になった"),
        }
    }

    /// 想定レイアウト（4 要素）なら 2 番目だけが置換される（M1）。
    #[test]
    fn install_module_searcher_replaces_only_the_lua_loader() {
        let lua = production_like_vm();
        let before: Vec<mlua::Function> = loaders_of(&lua);
        install_module_searcher(&lua).expect("標準レイアウトの VM で設置に失敗した");
        let after: Vec<mlua::Function> = loaders_of(&lua);

        assert_eq!(after.len(), 4);
        assert_eq!(before[0], after[0]);
        assert_ne!(before[1], after[1]);
        assert_eq!(before[2], after[2]);
        assert_eq!(before[3], after[3]);
    }

    /// レイアウトが想定と異なる VM では設置せずに Err を返す。
    #[test]
    fn install_module_searcher_rejects_unexpected_layout() {
        let lua = production_like_vm();
        lua.load("table.remove(package.loaders)")
            .exec()
            .expect("searcher テーブルの縮小に失敗した");
        let err = install_module_searcher(&lua).expect_err("レイアウト違反なのに設置が成功した");
        assert!(err.to_string().contains("package.loaders"), "{err}");
    }

    /// ラッパ経由の往復: 解決したチャンクの識別子が `@` + 候補パスであり、未検出と
    /// 構文エラーがいずれも Lua **文字列**エラーとして届く（3.4 / 1.5 / 4.9 / M3 / M4）。
    #[test]
    fn installed_searcher_resolves_and_raises_string_errors() {
        let dir = tempfile::tempdir().expect("一時ディレクトリの作成に失敗した");
        let root = dir.path().to_string_lossy().replace('\\', "/");
        std::fs::write(
            dir.path().join("ok.lua"),
            "return debug.getinfo(1, 'S').source",
        )
        .unwrap();
        std::fs::write(dir.path().join("bad.lua"), "return return").unwrap();

        let lua = production_like_vm();
        install_module_searcher(&lua).unwrap();
        let package: mlua::Table = lua.globals().get("package").unwrap();
        package.set("path", format!("{root}/?.lua")).unwrap();

        let source: String = lua.load("return require('ok')").eval().unwrap();
        assert_eq!(source, format!("@{root}/ok.lua"));

        let (missing_type, missing): (String, String) = lua
            .load("local ok, e = pcall(require, 'missing') return type(e), tostring(e)")
            .eval()
            .unwrap();
        assert_eq!(missing_type, "string");
        assert!(
            missing.contains(&format!("no file '{root}/missing.lua'")),
            "{missing}"
        );

        let (bad_type, bad): (String, String) = lua
            .load("local ok, e = pcall(require, 'bad') return type(e), tostring(e)")
            .eval()
            .unwrap();
        assert_eq!(bad_type, "string");
        assert!(
            bad.starts_with(&format!(
                "error loading module 'bad' from file '{root}/bad.lua':
	"
            )),
            "{bad}"
        );
        assert!(!bad.contains("syntax error:"), "{bad}");
    }

    /// 本番と同じ構築方法（`unsafe_new_with`）の VM。安全モードの `Lua::new` は
    /// C searcher を落とすため、`package.loaders` のレイアウトが本番と一致しない。
    fn production_like_vm() -> Lua {
        unsafe { Lua::unsafe_new_with(mlua::StdLib::ALL, mlua::LuaOptions::default()) }
    }

    fn loaders_of(lua: &mlua::Lua) -> Vec<mlua::Function> {
        let package: mlua::Table = lua.globals().get("package").unwrap();
        let loaders: mlua::Table = package.get("loaders").unwrap();
        (1..=loaders.raw_len())
            .map(|i| loaders.get(i).unwrap())
            .collect()
    }
}
