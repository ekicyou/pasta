//! ghost/master の見張り（200 ms まとめて 1 回知らせる・profile を除く）。

use notify::{EventKind, RecursiveMode, Watcher as _};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// 見張りの本体。落とすと止まる（送り口が落ちて、まとめのスレッドも終わる）
pub type Watcher = notify::RecommendedWatcher;

/// 見張りを始める。知らせは別のスレッドで on_batch（相対パスの集合・重なりなし）を呼ぶ。返り値を落とすと止まる
pub fn start(
    master: PathBuf,
    on_batch: impl Fn(Vec<String>) + Send + 'static,
) -> Result<Watcher, String> {
    let (tx, rx) = mpsc::channel::<String>();
    let root = master.clone();
    // 失った知らせ・見張りの誤りは捨てる（画面がディスクと比べ直して埋める）
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        let Ok(event) = res else { return };
        // 読んだだけの知らせで作り直しを繰り返さない
        if matches!(event.kind, EventKind::Access(_)) {
            return;
        }
        for path in event.paths {
            let Ok(rel) = path.strip_prefix(&root) else {
                continue;
            };
            let parts: Vec<String> = rel
                .iter()
                .map(|s| s.to_string_lossy().into_owned())
                .collect();
            // master そのもの・最上位の profile の下（11.4）は知らせない
            if parts
                .first()
                .is_some_and(|p| !p.eq_ignore_ascii_case("profile"))
            {
                let _ = tx.send(parts.join("/"));
            }
        }
    })
    .map_err(|e| format!("{} を見張れません: {e}", master.display()))?;
    watcher
        .watch(&master, RecursiveMode::Recursive)
        .map_err(|e| format!("{} を見張れません: {e}", master.display()))?;
    std::thread::spawn(move || {
        // 最初の知らせから 200 ms ためて 1 回知らせる
        while let Ok(first) = rx.recv() {
            let mut batch = BTreeSet::from([first]);
            let deadline = Instant::now() + Duration::from_millis(200);
            while let Some(left) = deadline.checked_duration_since(Instant::now()) {
                match rx.recv_timeout(left) {
                    Ok(p) => {
                        batch.insert(p);
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => break,
                    Err(mpsc::RecvTimeoutError::Disconnected) => return,
                }
            }
            on_batch(batch.into_iter().collect());
        }
    });
    Ok(watcher)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::mpsc;
    use std::time::Duration;

    fn watch(master: &std::path::Path) -> (Watcher, mpsc::Receiver<Vec<String>>) {
        let (tx, rx) = mpsc::channel();
        let w = start(master.to_path_buf(), move |paths| {
            let _ = tx.send(paths);
        })
        .unwrap();
        (w, rx)
    }

    #[test]
    fn add_change_remove_arrive_as_one_batch() {
        let dir = tempfile::tempdir().unwrap();
        let master = dir.path();
        fs::create_dir(master.join("dic")).unwrap();
        fs::write(master.join("dic").join("b.pasta"), "").unwrap();
        fs::write(master.join("c.lua"), "").unwrap();
        let (_w, rx) = watch(master);

        fs::write(master.join("dic").join("a.pasta"), "x").unwrap();
        fs::write(master.join("dic").join("b.pasta"), "y").unwrap();
        fs::remove_file(master.join("c.lua")).unwrap();

        let mut batch = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        // 中身が変わったフォルダそのものも知らせる OS がある（Windows）
        batch.retain(|p| p != "dic");
        assert_eq!(batch, ["c.lua", "dic/a.pasta", "dic/b.pasta"]);
        // まとめて 1 回だけ（2 回目は来ない）
        assert!(rx.recv_timeout(Duration::from_secs(1)).is_err());
    }

    #[test]
    fn changes_under_top_level_profile_are_dropped() {
        let dir = tempfile::tempdir().unwrap();
        let master = dir.path();
        let (_w, rx) = watch(master);

        fs::create_dir_all(master.join("Profile").join("sub")).unwrap();
        fs::write(master.join("Profile").join("sub").join("save.txt"), "x").unwrap();
        fs::write(master.join("Profile").join("p.txt"), "x").unwrap();
        std::thread::sleep(Duration::from_millis(800));
        // 見張りが生きている証しに profile の外を変える。profile の知らせが漏れていればここで混ざる
        fs::write(master.join("a.pasta"), "x").unwrap();

        let batch = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(batch, ["a.pasta"]);
        assert!(rx.recv_timeout(Duration::from_secs(1)).is_err());
    }
}
