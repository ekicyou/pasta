//! 画面と Rust の唯一の接点（Tauri のコマンドの表と開いているゴーストの状態）。
//! 相対パスを取るコマンドは、状態の master と paths::resolve_rel（files の各関数の中）を必ず通す（11.2）。

use crate::files::{self, EntryKind, TextFile, TreeListing};
use crate::{check, paths, recent, watch};
use pasta_check::validate::Report;
use pasta_lsp::analysis::{AnalysisEngine, AnalysisResult, TOKEN_MODIFIERS, TOKEN_TYPES};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use tauri::{AppHandle, Emitter, Manager, State};

/// 開いているゴースト。watcher を落とすと見張りが止まる（2.10）
pub struct Ghost {
    pub master: PathBuf,
    pub watcher: watch::Watcher,
}

/// 1 つの窓に 1 つ。ロックはパスを取り出す・置き換える間だけ持つ。
/// 2 つ目は進んでいる開発の操作の master（dev_begin が覚え、dev_end が消す。5.2）
#[derive(Default)]
pub struct AppState(Mutex<Option<Ghost>>, Mutex<Option<PathBuf>>);

impl AppState {
    fn lock(&self) -> MutexGuard<'_, Option<Ghost>> {
        // 毒されても中身（パスと見張り）は壊れていない
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub(crate) fn master(&self) -> Result<PathBuf, String> {
        self.lock()
            .as_ref()
            .map(|g| g.master.clone())
            .ok_or_else(|| "ゴーストを開いていません".into())
    }

    /// 見張りの知らせが今のゴーストのものか（落とした直後の古い見張りの知らせを捨てる）
    fn is_current(&self, master: &Path) -> bool {
        self.lock().as_ref().is_some_and(|g| g.master == master)
    }

    /// 進んでいる開発の操作の master を置き換える（None で消す）
    pub(crate) fn set_dev_master(&self, master: Option<PathBuf>) {
        *self.1.lock().unwrap_or_else(|e| e.into_inner()) = master;
    }

    pub(crate) fn dev_master(&self) -> Result<PathBuf, String> {
        self.1
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
            .ok_or_else(|| "開発の操作が進んでいません".into())
    }

    /// 置き換える。古い見張りはロックを放してから落とす
    pub(crate) fn replace(&self, ghost: Ghost) {
        let old = self.lock().replace(ghost);
        drop(old);
    }
}

/// 画面に登録するコマンドの表（設計の API Contract）
pub fn handler() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        initial_path,
        resolve_ghost,
        open_ghost,
        list_tree,
        read_text,
        write_text,
        create_entry,
        rename_entry,
        analyze,
        legend,
        check_ghost,
        recent_list,
        recent_remove,
    ]
}

/// 画面の入力と別のスレッドで走らせ、パニックを `Err` に写す（7.7・6.5）
pub(crate) async fn run_blocking<T: Send + 'static>(
    what: &str,
    work: impl FnOnce() -> T + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|e| format!("{what}に失敗しました: {e}"))
}

/// 起動の引数の最初の 1 つ（2.1）
#[tauri::command]
pub async fn initial_path() -> Option<String> {
    let arg = std::env::args_os().nth(1)?;
    Some(abs_string(Path::new(&arg)))
}

/// 2 つ目の起動の argv[1] を、その起動の cwd と結んだ絶対パスにする（2.11）
pub fn second_launch_path(argv: &[String], cwd: &str) -> Option<String> {
    argv.get(1).map(|a| abs_string(&Path::new(cwd).join(a)))
}

fn abs_string(path: &Path) -> String {
    std::path::absolute(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .into_owned()
}

/// 事件 `open-request` の中身
#[derive(Clone, Serialize)]
pub struct OpenRequest {
    pub path: Option<String>,
}

/// 事件 `ghost-changed` の中身（master からの相対・'/' 区切り）
#[derive(Clone, Serialize)]
struct GhostChanged {
    paths: Vec<String>,
}

#[derive(Serialize)]
pub struct Resolved {
    master: String,
    file: Option<String>,
}

/// master の外のパスの有無を見る唯一のコマンド（11.1 の例外）
#[tauri::command]
pub async fn resolve_ghost(path: String) -> Result<Resolved, String> {
    let found = paths::find_master(Path::new(&path))?;
    Ok(Resolved {
        master: found.master.to_string_lossy().into_owned(),
        file: found.file,
    })
}

#[derive(Serialize)]
pub struct GhostInfo {
    master: String,
    name: String,
    version: String,
}

/// ゴーストのフォルダの名前（master の 2 つ上）
fn ghost_name(master: &Path) -> String {
    master
        .parent()
        .and_then(Path::parent)
        .and_then(Path::file_name)
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// 画面の言う master を信じず find_master で確かめ、見張りを始めて状態を置き換える。
/// ゴーストでない・見張れないときは状態を変えずに Err（11.1・11.2：任意のフォルダを範囲の根にさせない）
fn open_in(
    state: &AppState,
    master: &str,
    on_batch: impl Fn(&Path, Vec<String>) + Send + 'static,
) -> Result<PathBuf, String> {
    let found = paths::find_master(Path::new(master))?.master;
    let watched = found.clone();
    let watcher = watch::start(found.clone(), move |paths| on_batch(&watched, paths))?;
    state.replace(Ghost {
        master: found.clone(),
        watcher,
    });
    Ok(found)
}

/// 状態を置き換え、題名を変え、最近開いたものに記録する（1.6・2.9・2.10・3.1）
#[tauri::command]
pub async fn open_ghost(
    app: AppHandle,
    state: State<'_, AppState>,
    master: String,
) -> Result<GhostInfo, String> {
    let notify_app = app.clone();
    let path = open_in(&state, &master, move |watched, paths| {
        if notify_app.state::<AppState>().is_current(watched) {
            let _ = notify_app.emit("ghost-changed", GhostChanged { paths });
        }
    })?;
    let info = GhostInfo {
        name: ghost_name(&path),
        version: app.package_info().version.to_string(),
        master: path.to_string_lossy().into_owned(),
    };
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_title(&format!("{} - pasta-recipe {}", info.name, info.version));
    }
    // 記録だけの失敗では開く操作を止めない（Error Handling）
    if let Ok(dir) = app.path().app_data_dir() {
        let _ = recent::save(&dir, &recent::push(recent::load(&dir), &info.master));
    }
    Ok(info)
}

fn do_list_tree(state: &AppState) -> Result<TreeListing, String> {
    files::list_tree(&state.master()?)
}

fn do_read_text(state: &AppState, rel: &str) -> Result<TextFile, String> {
    files::read_text(&state.master()?, rel)
}

fn do_write_text(state: &AppState, rel: &str, text: &str, bom: bool) -> Result<(), String> {
    files::write_text(&state.master()?, rel, text, bom)
}

fn do_create_entry(
    state: &AppState,
    parent_rel: &str,
    name: &str,
    kind: EntryKind,
) -> Result<String, String> {
    files::create_entry(&state.master()?, parent_rel, name, kind)
}

fn do_rename_entry(state: &AppState, rel: &str, new_name: &str) -> Result<String, String> {
    files::rename_entry(&state.master()?, rel, new_name)
}

#[tauri::command]
pub async fn list_tree(state: State<'_, AppState>) -> Result<TreeListing, String> {
    do_list_tree(&state)
}

#[tauri::command]
pub async fn read_text(state: State<'_, AppState>, rel: String) -> Result<TextFile, String> {
    do_read_text(&state, &rel)
}

#[tauri::command]
pub async fn write_text(
    state: State<'_, AppState>,
    rel: String,
    text: String,
    bom: bool,
) -> Result<(), String> {
    do_write_text(&state, &rel, &text, bom)
}

#[tauri::command]
pub async fn create_entry(
    state: State<'_, AppState>,
    parent_rel: String,
    name: String,
    kind: EntryKind,
) -> Result<String, String> {
    do_create_entry(&state, &parent_rel, &name, kind)
}

#[tauri::command]
pub async fn rename_entry(
    state: State<'_, AppState>,
    rel: String,
    new_name: String,
) -> Result<String, String> {
    do_rename_entry(&state, &rel, &new_name)
}

/// 重いので画面の入力と別のスレッドで。検査できない理由はそのまま Err（9.10）
#[tauri::command]
pub async fn check_ghost(state: State<'_, AppState>) -> Result<Report, String> {
    let master = state.master()?;
    run_blocking("検査", move || check::check_ghost(&master)).await?
}

fn recent_entries(list: Vec<String>) -> Vec<recent::RecentEntry> {
    list.into_iter()
        .map(|master| recent::RecentEntry {
            name: ghost_name(Path::new(&master)),
            master,
        })
        .collect()
}

/// 記録は %APPDATA%\<識別子>\recent.toml（Tauri の app_data_dir）
#[tauri::command]
pub async fn recent_list(app: AppHandle) -> Vec<recent::RecentEntry> {
    app.path()
        .app_data_dir()
        .map(|dir| recent_entries(recent::load(&dir)))
        .unwrap_or_default()
}

#[tauri::command]
pub async fn recent_remove(app: AppHandle, master: String) {
    if let Ok(dir) = app.path().app_data_dir() {
        let _ = recent::save(&dir, &recent::remove(recent::load(&dir), &master));
    }
}

/// 文書 1 つを解析する。値は拡張の WASM と同じ形のまま返す（13.4）
#[tauri::command]
pub async fn analyze(source: String) -> Result<AnalysisResult, String> {
    run_blocking("解析", move || AnalysisEngine::analyze(&source)).await
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Legend {
    token_types: &'static [&'static str],
    token_modifiers: &'static [&'static str],
}

/// セマンティックトークンの凡例（正本は pasta_lsp の 1 か所・6.3）
#[tauri::command]
pub async fn legend() -> Legend {
    Legend {
        token_types: TOKEN_TYPES,
        token_modifiers: TOKEN_MODIFIERS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pasta_lsp::analysis::AnalysisEngine;
    use tauri::async_runtime::block_on;

    /// 全角文字・サロゲートペア・柵の中の Lua・構文の誤り（最上位の地の文）を含む文書
    const SOURCE: &str = "＊挨拶\n```lua\nlocal x = \"😀\"\n```\n　女の子：こんにちは😀＠単語　です。\n　・朝\n　　女の子：おはよう\n\n＊別れ\n壊れた行\n";

    #[test]
    fn analyze_json_is_the_native_serialization() {
        let native = serde_json::to_string(&AnalysisEngine::analyze(SOURCE)).unwrap();
        let app = serde_json::to_string(&block_on(analyze(SOURCE.into())).unwrap()).unwrap();
        assert_eq!(app, native);
        // 誤りの診断・サロゲートペアを含む行のトークンが実際に出ていて、欄の名前と順が拡張と同じ
        assert!(
            app.starts_with(r#"{"tokens":[{"deltaLine":0,"deltaStartCharacter":"#),
            "{app}"
        );
        assert!(
            app.contains(r#"],"diagnostics":[{"range":{"start":{"line":"#),
            "{app}"
        );
        assert!(
            app.contains(r#"],"scenes":[{"name":"挨拶","kind":"global","#),
            "{app}"
        );
    }

    #[test]
    fn panic_in_blocking_work_becomes_err() {
        let err = block_on(run_blocking("解析", || -> u32 {
            panic!("試しのパニック")
        }))
        .unwrap_err();
        assert!(err.starts_with("解析に失敗しました"), "{err}");
        let err = block_on(run_blocking("検査", || -> u32 {
            panic!("試しのパニック")
        }))
        .unwrap_err();
        assert!(err.starts_with("検査に失敗しました"), "{err}");
        assert_eq!(block_on(run_blocking("解析", || 7)), Ok(7));
    }

    /// tmp/hello/ghost/master を開いた状態と、master の外に置いた秘密のファイル
    fn opened() -> (tempfile::TempDir, AppState) {
        let tmp = tempfile::tempdir().unwrap();
        let master = tmp.path().join("hello/ghost/master");
        std::fs::create_dir_all(master.join("dic")).unwrap();
        std::fs::write(master.join("dic/a.pasta"), "＊挨拶\n").unwrap();
        std::fs::write(tmp.path().join("hello/ghost/secret.txt"), "秘密").unwrap();
        let state = AppState::default();
        state.replace(Ghost {
            watcher: watch::start(master.clone(), |_| {}).unwrap(),
            master,
        });
        (tmp, state)
    }

    #[test]
    fn rel_outside_the_ghost_is_refused_without_touching_disk() {
        let (tmp, state) = opened();
        let outside = tmp.path().join("hello/ghost");
        let refused = |r: Result<String, String>| {
            let err = r.unwrap_err();
            assert!(err.contains("外"), "{err}");
        };
        refused(do_read_text(&state, "../secret.txt").map(|t| t.text));
        refused(do_write_text(&state, "../secret.txt", "上書き", false).map(|_| String::new()));
        refused(do_write_text(&state, "../new.txt", "x", false).map(|_| String::new()));
        refused(do_create_entry(&state, "..", "made.txt", EntryKind::File));
        // 名前の門が先に「使えない名前」として拒む
        assert!(do_create_entry(&state, "", "../made.txt", EntryKind::File).is_err());
        refused(do_rename_entry(&state, "../secret.txt", "moved.txt"));
        refused(do_read_text(&state, "C:/Windows/win.ini").map(|t| t.text));
        // 読み書き・作成・名前の変更のどれも起きていない
        assert_eq!(
            std::fs::read_to_string(outside.join("secret.txt")).unwrap(),
            "秘密"
        );
        for name in ["new.txt", "made.txt", "moved.txt"] {
            assert!(!outside.join(name).exists(), "{name}");
            assert!(!tmp.path().join("hello").join(name).exists(), "{name}");
        }
        // 中のものは通る
        assert_eq!(
            do_read_text(&state, "dic/a.pasta").unwrap().text,
            "＊挨拶\n"
        );
        assert_eq!(do_list_tree(&state).unwrap().files, ["dic/a.pasta"]);
    }

    #[test]
    fn rel_commands_without_an_open_ghost_return_a_reason() {
        let state = AppState::default();
        let err = do_read_text(&state, "dic/a.pasta").unwrap_err();
        assert!(err.contains("開いていません"), "{err}");
        assert!(do_list_tree(&state).is_err());
    }

    #[test]
    fn open_only_accepts_a_ghost_and_keeps_state_on_failure() {
        let (tmp, state) = opened();
        let master = tmp.path().join("hello/ghost/master");
        // ゴーストでないフォルダ（C:\ のような根）は Err・開いているゴーストは変わらない
        let err = open_in(&state, &tmp.path().to_string_lossy(), |_, _| {}).unwrap_err();
        assert!(err.contains("ghost/master"), "{err}");
        assert!(state.is_current(&master));
        assert!(do_read_text(&state, "../secret.txt").is_err());
        // ghost/master もゴーストのフォルダも同じ master に着く
        let other = tmp.path().join("emo2/ghost/master");
        std::fs::create_dir_all(&other).unwrap();
        let opened = open_in(
            &state,
            &tmp.path().join("emo2").to_string_lossy(),
            |_, _| {},
        );
        assert_eq!(opened.unwrap(), other);
        assert!(state.is_current(&other));
        assert_eq!(
            open_in(&state, &master.to_string_lossy(), |_, _| {}).unwrap(),
            master
        );
        assert!(state.is_current(&master));
    }

    #[test]
    fn replacing_the_ghost_ignores_the_old_watcher() {
        let (tmp, state) = opened();
        let old = tmp.path().join("hello/ghost/master");
        assert!(state.is_current(&old));
        let other = tmp.path().join("emo2/ghost/master");
        std::fs::create_dir_all(&other).unwrap();
        state.replace(Ghost {
            watcher: watch::start(other.clone(), |_| {}).unwrap(),
            master: other.clone(),
        });
        assert!(!state.is_current(&old));
        assert!(state.is_current(&other));
        assert_eq!(ghost_name(&other), "emo2");
    }

    #[test]
    fn second_launch_path_joins_cwd() {
        let argv = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(
            second_launch_path(&argv(&["pasta-recipe.exe"]), r"C:\w"),
            None
        );
        assert_eq!(
            second_launch_path(&argv(&["pasta-recipe.exe", r"hello\ghost"]), r"C:\w").as_deref(),
            Some(r"C:\w\hello\ghost")
        );
        assert_eq!(
            second_launch_path(&argv(&["x", r"D:\g\emo2", "more"]), r"C:\w").as_deref(),
            Some(r"D:\g\emo2")
        );
    }

    #[test]
    fn recent_entries_carry_the_ghost_folder_name() {
        let list = recent_entries(vec![r"C:\ghosts\hello\ghost\master".into()]);
        assert_eq!(list[0].name, "hello");
        assert_eq!(list[0].master, r"C:\ghosts\hello\ghost\master");
    }

    #[test]
    fn legend_is_the_rust_one() {
        let json = serde_json::to_value(block_on(legend())).unwrap();
        assert_eq!(json["tokenTypes"][8], "actorName");
        assert_eq!(json["tokenTypes"].as_array().unwrap().len(), 17);
        assert_eq!(
            json["tokenModifiers"],
            serde_json::json!(["declaration", "definition", "global"])
        );
    }
}
