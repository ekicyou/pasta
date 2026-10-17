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
//! パス長の判定・`\\?\` の明示付与・短縮名変換・`canonicalize` は一切行わない
//! （要件 1.6・3.7）。候補パス文字列は、ファイルを開くパス・チャンク識別子・
//! エラーメッセージの 3 箇所に**そのまま**使われる。

use mlua::{Function, Lua, Result as LuaResult, Table, Value};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

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
                Ok(loader) => {
                    record_chunk(lua, &candidate)?;
                    Ok((Some(loader), None, false))
                }
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

/// 読み込んだファイルの表を Lua から読む名前（`require("@pasta_chunks")`）。
pub const CHUNK_TABLE_MODULE: &str = "@pasta_chunks";

/// 標準ランタイムの置き場所（ゴーストのフォルダーから見た形）。この下は表に控えない。
const RUNTIME_REL: &str = "profile/pasta/pasta_scripts/";

/// LuaJIT がエラー文のチャンク名を縮める長さ（`LUA_IDSIZE`）。
const LUA_IDSIZE: usize = 60;

/// 表を作るための場所と、キーごとに最初に読み込んだ候補パス（重複の見分け用）。
struct ChunkTable {
    /// ゴーストのフォルダー（`\` を `/` にそろえ、末尾の `/` を落とした形）。
    ghost_dir: String,
    /// 変換した Lua のキャッシュ（ゴーストから見た形・`/` 区切り・末尾に `/`）。
    cache_rel: String,
    /// キー → そのキーを最初に作った候補パス。標準ランタイム・ゴーストの外も含む。
    seen: HashMap<Vec<u8>, String>,
}

/// 候補パスを、エラー文の位置の接頭辞に出る形（`@` を除いたチャンク名）にする。
///
/// LuaJIT `lj_debug_shortname` と同じく、60 バイト以上なら `...` ＋ 末尾 56 バイト。
/// バイトで切るので、先頭で UTF-8 の文字が割れることがある。
fn error_form(candidate: &str) -> Vec<u8> {
    let bytes = candidate.as_bytes();
    if bytes.len() < LUA_IDSIZE {
        return bytes.to_vec();
    }
    let mut key = b"...".to_vec();
    key.extend_from_slice(&bytes[bytes.len() - (LUA_IDSIZE - 4)..]);
    key
}

/// 読み込んだファイルの表（`@pasta_chunks`）を作り、探索関数が控えられるようにする。
///
/// キーは [`error_form`] の形、値は次のどれか。
/// - 作者の `.lua`: ゴーストのフォルダーから見たパス（区切りは `\`。例 `scripts\天気.lua`）
/// - 変換した Lua のキャッシュ: `true`（辞書の印）
/// - 2 つのファイルが同じキーになった: `false`（見分けられない。無いのと同じに扱う）
///
/// 標準ランタイムとゴーストの外のファイルは控えない（重複の見分けには数える）。
pub(crate) fn register_chunk_table(lua: &Lua, ghost_dir: &Path, cache_dir: &str) -> LuaResult<()> {
    let ghost_dir = ghost_dir.to_string_lossy().replace('\\', "/");
    let cache_rel = format!("{}/", cache_dir.replace('\\', "/").trim_matches('/'));
    lua.set_app_data(ChunkTable {
        ghost_dir: ghost_dir.trim_end_matches('/').to_string(),
        cache_rel,
        seen: HashMap::new(),
    });
    let package: Table = lua.globals().get("package")?;
    let loaded: Table = package.get("loaded")?;
    loaded.set(CHUNK_TABLE_MODULE, lua.create_table()?)
}

/// 読み込めた候補パスを表に控える（[`register_chunk_table`] の前なら何もしない）。
fn record_chunk(lua: &Lua, candidate: &str) -> LuaResult<()> {
    let key = error_form(candidate);
    let value = {
        let Some(mut table) = lua.app_data_mut::<ChunkTable>() else {
            return Ok(());
        };
        match table.seen.get(&key) {
            Some(first) if first == candidate => return Ok(()),
            Some(_) => Value::Boolean(false),
            None => {
                table.seen.insert(key.clone(), candidate.to_string());
                let path = candidate.replace('\\', "/");
                let Some(rel) = path
                    .strip_prefix(&table.ghost_dir)
                    .and_then(|p| p.strip_prefix('/'))
                else {
                    return Ok(()); // ゴーストの外
                };
                if rel.starts_with(&table.cache_rel) {
                    Value::Boolean(true)
                } else if rel.starts_with(RUNTIME_REL) {
                    return Ok(());
                } else {
                    Value::String(lua.create_string(rel.replace('/', "\\"))?)
                }
            }
        }
    };
    let package: Table = lua.globals().get("package")?;
    let loaded: Table = package.get("loaded")?;
    let Value::Table(chunks) = loaded.get::<Value>(CHUNK_TABLE_MODULE)? else {
        return Ok(());
    };
    chunks.raw_set(lua.create_string(&key)?, value)
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

    /// キーの形は LuaJIT が実際にエラー文に出すチャンク名と一致する（59・60・61 バイトと
    /// 日本語で割れる場合を、`@` 付きの名前で読み込んだチャンクのエラーで確かめる）。
    #[test]
    fn error_form_matches_luajit_error_prefix() {
        let lua = production_like_vm();
        let cases = [
            "a".repeat(59),
            "b".repeat(60),
            "c".repeat(61),
            format!("C:/ghost/{}/scripts/天気.lua", "d".repeat(40)),
            // 61 バイト。末尾 56 バイトは「気」の 3 バイト目から始まる（文字が割れる）。
            format!("天気{}", "e".repeat(55)),
        ];
        for name in cases {
            let message: mlua::String = lua
                .load("local function f() error('x') end local ok, e = pcall(f) return e")
                .set_name(format!("@{name}"))
                .eval()
                .unwrap();
            let mut expected = error_form(&name);
            expected.extend_from_slice(b":1: x");
            assert_eq!(message.as_bytes().to_vec(), expected, "{name}");
        }
        assert_eq!(error_form(&"a".repeat(59)), "a".repeat(59).into_bytes());
        assert_eq!(
            error_form(&"c".repeat(61)),
            format!("...{}", "c".repeat(56)).into_bytes()
        );
    }

    /// 探索関数が読み込んだファイルを表に控える（2.5 / 2.6）: 作者の `.lua` はゴーストから
    /// 見たパス、変換した Lua のキャッシュは `true`、標準ランタイムは控えない。キーは
    /// エラー文の位置の接頭辞と同じ形で、チャンク名は `@` ＋ 候補パスのまま。
    #[test]
    fn chunk_table_records_author_and_cache_but_not_runtime() {
        let dir = tempfile::tempdir().unwrap();
        let ghost = dir.path().join("ghost");
        let write = |rel: &str| {
            let path = ghost.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, "error('boom')").unwrap();
        };
        write("scripts/天気.lua");
        write("profile/pasta/cache/lua/pasta/scene/天気.lua");
        write("profile/pasta/pasta_scripts/rt.lua");
        std::fs::write(
            ghost.join("scripts/src.lua"),
            "return debug.getinfo(1, 'S').source",
        )
        .unwrap();

        let lua = ghost_vm(&ghost);
        let root = ghost.to_string_lossy().replace('\\', "/");

        let source: String = lua.load("return require('src')").eval().unwrap();
        assert_eq!(source, format!("@{root}/scripts/src.lua"));

        let chunks = chunk_table(&lua);
        for (module, expected) in [
            (
                "天気",
                Some(Value::String(
                    lua.create_string("scripts\\天気.lua").unwrap(),
                )),
            ),
            ("pasta.scene.天気", Some(Value::Boolean(true))),
            ("rt", None),
        ] {
            let err: mlua::String = lua
                .load(format!("local ok, e = pcall(require, '{module}') return e"))
                .eval()
                .unwrap();
            let err = err.as_bytes().to_vec();
            let key = &err[..err.windows(4).position(|w| w == b":1: ").unwrap()];
            let got: Value = chunks.raw_get(lua.create_string(key).unwrap()).unwrap();
            match expected {
                Some(v) => assert_eq!(got, v, "{module}"),
                None => assert!(got.is_nil(), "{module}: {got:?}"),
            }
        }
        // 作者の .lua の値は、縮めた形のキーで引ける（一時フォルダーは 60 バイトを超える）。
        let author_key = error_form(&format!("{root}/scripts/src.lua"));
        assert!(author_key.starts_with(b"..."), "一時フォルダーが短すぎる");
        let got: Value = chunks
            .raw_get(lua.create_string(&author_key).unwrap())
            .unwrap();
        assert_eq!(
            got,
            Value::String(lua.create_string("scripts\\src.lua").unwrap())
        );
    }

    /// 2 つのファイルが同じキーになったら、そのキーは見分けられない（`false`）。同じファイルを
    /// 読み直しても重複にはならない（2.6）。
    #[test]
    fn chunk_table_drops_duplicate_keys() {
        let dir = tempfile::tempdir().unwrap();
        let ghost = dir.path().join("ghost");
        let tail = "t".repeat(50);
        for rel in [
            format!("scripts/x/{tail}/m.lua"),
            format!("scripts/y/{tail}/m.lua"),
            "scripts/once.lua".to_string(),
        ] {
            let path = ghost.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, "return 1").unwrap();
        }
        let lua = ghost_vm(&ghost);
        let root = ghost.to_string_lossy().replace('\\', "/");
        let chunks = chunk_table(&lua);

        lua.load(format!("require('x.{tail}.m')")).exec().unwrap();
        let key = error_form(&format!("{root}/scripts/x/{tail}{DIR_SEP}m.lua"));
        let key_y = error_form(&format!("{root}/scripts/y/{tail}{DIR_SEP}m.lua"));
        assert_eq!(key, key_y, "末尾 56 バイトがそろっていない");
        let first: Value = chunks.raw_get(lua.create_string(&key).unwrap()).unwrap();
        assert!(first.is_string(), "{first:?}");

        lua.load(format!("require('y.{tail}.m')")).exec().unwrap();
        let got: Value = chunks.raw_get(lua.create_string(&key).unwrap()).unwrap();
        assert_eq!(got, Value::Boolean(false));
        // もう一度最初のファイルを読んでも、見分けられないまま。
        lua.load(format!(
            "package.loaded['x.{tail}.m'] = nil require('x.{tail}.m')"
        ))
        .exec()
        .unwrap();
        let got: Value = chunks.raw_get(lua.create_string(&key).unwrap()).unwrap();
        assert_eq!(got, Value::Boolean(false));

        // 同じファイルの読み直しは重複ではない。
        lua.load("require('once') package.loaded.once = nil require('once')")
            .exec()
            .unwrap();
        let key = error_form(&format!("{root}/scripts/once.lua"));
        let got: Value = chunks.raw_get(lua.create_string(&key).unwrap()).unwrap();
        assert_eq!(
            got,
            Value::String(lua.create_string("scripts\\once.lua").unwrap())
        );
    }

    /// 本番と同じ検索パス（`LoaderContext::generate_package_path` の形）と表を持つ VM。
    fn ghost_vm(ghost: &std::path::Path) -> Lua {
        let lua = production_like_vm();
        install_module_searcher(&lua).unwrap();
        register_chunk_table(&lua, ghost, "profile/pasta/cache/lua").unwrap();
        let root = ghost.to_string_lossy().replace('\\', "/");
        let path = [
            "scripts",
            "profile/pasta/pasta_scripts",
            "profile/pasta/cache/lua",
        ]
        .iter()
        .map(|p| format!("{root}/{p}/?.lua;{root}/{p}/?/init.lua"))
        .collect::<Vec<_>>()
        .join(";");
        let package: Table = lua.globals().get("package").unwrap();
        package.set("path", path).unwrap();
        lua
    }

    fn chunk_table(lua: &Lua) -> Table {
        let package: Table = lua.globals().get("package").unwrap();
        let loaded: Table = package.get("loaded").unwrap();
        loaded.get(CHUNK_TABLE_MODULE).unwrap()
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
