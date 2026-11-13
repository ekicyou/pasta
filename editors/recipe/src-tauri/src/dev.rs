//! 開発の門。送る先は 127.0.0.1 の候補のポートの決まったパスだけで、画面は URL を渡せない（9.1・9.2）。
//! 読むのは進んでいる操作の master の決まったファイルだけで、書くコマンドは無い（9.4・9.6・7.9）。

use crate::commands::{AppState, run_blocking};
use crate::paths::{self, Want};
use serde::Serialize;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};
use tauri::State;

/// POST の結果（editors/shared/ghostIO.ts の PostOutcome と同じ形）
#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PostOutcome {
    /// 本文は UTF-8 で読んだもの（壊れたバイトは置き換える）
    Response { status: u16, body: String },
    /// 頼みを書く前に切れた（接続できない）
    Unreachable { detail: String },
    /// 書いた後に切れた・時間切れ
    NoResponse { detail: String },
}

/// ベースウェアの MCP の待ち受けの候補（9801〜9810・9821〜9830）
pub fn check_port(port: u16) -> bool {
    matches!(port, 9801..=9810 | 9821..=9830)
}

/// 宛先は固定
pub fn dev_url(port: u16) -> String {
    format!("http://127.0.0.1:{port}/api/mcp/v1")
}

/// 候補のポートへ 1 回だけ POST する。失敗も PostOutcome で返す（3.1・3.9・9.3）
#[tauri::command]
pub async fn dev_post(port: u16, body: String, timeout_ms: u64) -> PostOutcome {
    if !check_port(port) {
        return PostOutcome::Unreachable {
            detail: format!("port {port} is not a baseware candidate"),
        };
    }
    send(port, body, timeout_ms).await
}

/// 送り手の芯（ポートの検査の前）。試験は空きポートへ向けてこれを呼ぶ
async fn send(port: u16, body: String, timeout_ms: u64) -> PostOutcome {
    // 代理を通さず・HTTP/1 だけ・接続を使い回さず・送り直さない・転送に従わない（宛先は固定）。Origin は付けない
    let client = match reqwest::Client::builder()
        .no_proxy()
        .http1_only()
        .pool_max_idle_per_host(0)
        .retry(reqwest::retry::never())
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_millis(timeout_ms))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return PostOutcome::Unreachable {
                detail: e.to_string(),
            };
        }
    };
    let result = async {
        let res = client
            .post(dev_url(port))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(
                reqwest::header::ACCEPT,
                "application/json, text/event-stream",
            )
            .body(body)
            .send()
            .await?;
        let status = res.status().as_u16();
        let bytes = res.bytes().await?;
        Ok::<_, reqwest::Error>(PostOutcome::Response {
            status,
            body: String::from_utf8_lossy(&bytes).into_owned(),
        })
    }
    .await;
    result.unwrap_or_else(|e| {
        // 接続の失敗は何も書いていない。時間切れ・ほかは書いた後とみなす
        if e.is_connect() {
            PostOutcome::Unreachable {
                detail: e.to_string(),
            }
        } else if e.is_timeout() {
            PostOutcome::NoResponse {
                detail: format!("no response within {timeout_ms} ms"),
            }
        } else {
            PostOutcome::NoResponse {
                detail: e.to_string(),
            }
        }
    })
}

/// ファイルの状態（editors/shared/ghostIO.ts の FileStat と同じ形）
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileStat {
    pub size: u64,
    pub mtime_ms: f64,
    pub is_file: bool,
}

/// 操作の始め。今開いている master を覚える（前の覚えは上書きする。5.2）
#[tauri::command]
pub async fn dev_begin(state: State<'_, AppState>) -> Result<(), String> {
    do_begin(&state)
}

/// 操作の終わり。覚えた master を消す
#[tauri::command]
pub async fn dev_end(state: State<'_, AppState>) -> Result<(), String> {
    state.set_dev_master(None);
    Ok(())
}

#[tauri::command]
pub async fn dev_list(state: State<'_, AppState>, path: String) -> Result<Vec<String>, String> {
    let master = state.dev_master()?;
    run_blocking("一覧", move || do_list(&master, &path)).await?
}

#[tauri::command]
pub async fn dev_stat(state: State<'_, AppState>, path: String) -> Result<FileStat, String> {
    let master = state.dev_master()?;
    run_blocking("状態の取得", move || do_stat(&master, &path)).await?
}

/// start からのバイト列を JSON にせずに返す
#[tauri::command]
pub async fn dev_read(
    state: State<'_, AppState>,
    path: String,
    start: u64,
) -> Result<tauri::ipc::Response, String> {
    let master = state.dev_master()?;
    let bytes = run_blocking("読み取り", move || do_read(&master, &path, start)).await??;
    Ok(tauri::ipc::Response::new(bytes))
}

fn do_begin(state: &AppState) -> Result<(), String> {
    state.set_dev_master(Some(state.master()?));
    Ok(())
}

fn do_list(master: &Path, path: &str) -> Result<Vec<String>, String> {
    let dir = paths::dev_path(master, Path::new(path), Want::List)?;
    let entries = fs::read_dir(&dir).map_err(|e| format!("{path} を読めません: {e}"))?;
    Ok(entries
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect())
}

fn do_stat(master: &Path, path: &str) -> Result<FileStat, String> {
    let file = paths::dev_path(master, Path::new(path), Want::Read)?;
    let meta = fs::metadata(&file).map_err(|e| format!("{path} を読めません: {e}"))?;
    let mtime_ms = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0.0, |d| d.as_secs_f64() * 1000.0);
    Ok(FileStat {
        size: meta.len(),
        mtime_ms,
        is_file: meta.is_file(),
    })
}

fn do_read(master: &Path, path: &str, start: u64) -> Result<Vec<u8>, String> {
    let file = paths::dev_path(master, Path::new(path), Want::Read)?;
    let cant = |e: std::io::Error| format!("{path} を読めません: {e}");
    if !fs::metadata(&file).map_err(cant)?.is_file() {
        return Err(format!("{path} はファイルではありません"));
    }
    let mut f = fs::File::open(&file).map_err(cant)?;
    f.seek(SeekFrom::Start(start)).map_err(cant)?;
    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes).map_err(cant)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::Ghost;
    use crate::watch;
    use std::io::{ErrorKind, Write};
    use std::net::{TcpListener, TcpStream};
    use std::path::PathBuf;
    use std::thread::{self, JoinHandle};

    #[test]
    fn check_port_takes_only_candidates() {
        for p in [9800, 9811, 9820, 9831, 80, 0, 65535] {
            assert!(!check_port(p), "{p}");
        }
        let ok: Vec<u16> = (0..=u16::MAX).filter(|&p| check_port(p)).collect();
        let want: Vec<u16> = (9801..=9810).chain(9821..=9830).collect();
        assert_eq!(ok, want);
    }

    #[test]
    fn dev_url_is_fixed() {
        assert_eq!(dev_url(9801), "http://127.0.0.1:9801/api/mcp/v1");
    }

    #[test]
    fn non_candidate_port_is_unreachable_without_sending() {
        // 待ち受けていても候補の外へは送らない
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let out = tauri::async_runtime::block_on(dev_post(port, "{}".into(), 500));
        assert!(matches!(out, PostOutcome::Unreachable { .. }), "{out:?}");
        listener.set_nonblocking(true).unwrap();
        assert_eq!(listener.accept().unwrap_err().kind(), ErrorKind::WouldBlock);
    }

    /// 頭と本文を読み切る（Content-Length の分だけ）
    fn read_request(s: &mut TcpStream) -> String {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 1024];
        loop {
            let n = s.read(&mut chunk).unwrap();
            assert!(n > 0, "closed before the request ended");
            buf.extend_from_slice(&chunk[..n]);
            let text = String::from_utf8_lossy(&buf).into_owned();
            if let Some(end) = text.find("\r\n\r\n") {
                let len = text[..end]
                    .lines()
                    .find_map(|l| {
                        let (k, v) = l.split_once(':')?;
                        k.eq_ignore_ascii_case("content-length")
                            .then(|| v.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                if buf.len() >= end + 4 + len {
                    return text;
                }
            }
        }
    }

    /// 1 つだけ受けて読み、then で応える。生の頼みと待ち受けを返す
    fn serve(then: fn(&mut TcpStream)) -> (u16, JoinHandle<(String, TcpListener)>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let h = thread::spawn(move || {
            let (mut s, _) = listener.accept().unwrap();
            let raw = read_request(&mut s);
            then(&mut s);
            (raw, listener)
        });
        (port, h)
    }

    /// 送り直しが無い（2 つ目の接続が来ていない）
    fn assert_no_second_connection(listener: TcpListener) {
        listener.set_nonblocking(true).unwrap();
        assert_eq!(listener.accept().unwrap_err().kind(), ErrorKind::WouldBlock);
    }

    #[test]
    fn sends_fixed_request_without_origin_once() {
        let (port, h) = serve(|s| {
            s.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\no\xffk")
                .unwrap();
        });
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#;
        let out = tauri::async_runtime::block_on(send(port, body.into(), 2000));
        assert_eq!(
            out,
            PostOutcome::Response {
                status: 200,
                body: "o\u{fffd}k".into()
            }
        );
        let (raw, listener) = h.join().unwrap();
        let (head, sent) = raw.split_once("\r\n\r\n").unwrap();
        let mut lines = head.lines();
        assert_eq!(lines.next(), Some("POST /api/mcp/v1 HTTP/1.1"));
        let headers: Vec<String> = lines.map(|l| l.to_ascii_lowercase()).collect();
        assert!(!headers.iter().any(|l| l.starts_with("origin:")), "{head}");
        assert!(
            headers.contains(&"content-type: application/json".into()),
            "{head}"
        );
        assert!(
            headers.contains(&"accept: application/json, text/event-stream".into()),
            "{head}"
        );
        assert!(
            headers.contains(&format!("host: 127.0.0.1:{port}")),
            "{head}"
        );
        assert_eq!(sent, body);
        assert_no_second_connection(listener);
    }

    #[test]
    fn redirect_is_not_followed() {
        // 転送の指図が来ても従わず、そのまま応答として返す
        let (port, h) = serve(|s| {
            s.write_all(b"HTTP/1.1 307 Temporary Redirect\r\nLocation: /elsewhere\r\nContent-Length: 0\r\n\r\n")
                .unwrap();
        });
        let out = tauri::async_runtime::block_on(send(port, "{}".into(), 2000));
        assert_eq!(
            out,
            PostOutcome::Response {
                status: 307,
                body: String::new()
            }
        );
        let (_, listener) = h.join().unwrap();
        assert_no_second_connection(listener);
    }

    #[test]
    fn closed_after_read_is_no_response_and_not_resent() {
        // 読んで応えずに切る。送り直さない
        let (port, h) = serve(|_| {});
        let out = tauri::async_runtime::block_on(send(port, "{}".into(), 2000));
        assert!(matches!(out, PostOutcome::NoResponse { .. }), "{out:?}");
        let (_, listener) = h.join().unwrap();
        assert_no_second_connection(listener);
    }

    #[test]
    fn silent_after_read_is_no_response() {
        // 読んで黙る。相手が切るまで（長くても 5 秒）持つ
        let (port, h) = serve(|s| {
            s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let _ = s.read(&mut [0u8; 1]);
        });
        let out = tauri::async_runtime::block_on(send(port, "{}".into(), 300));
        assert_eq!(
            out,
            PostOutcome::NoResponse {
                detail: "no response within 300 ms".into()
            }
        );
        let (_, listener) = h.join().unwrap();
        assert_no_second_connection(listener);
    }

    #[test]
    fn no_listener_is_unreachable() {
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        // Windows は断られた接続を約 2 秒送り直してから失敗にするので、時間はそれより長く取る
        let out = tauri::async_runtime::block_on(send(port, "{}".into(), 10_000));
        assert!(matches!(out, PostOutcome::Unreachable { .. }), "{out:?}");
    }

    /// root/<name>/ghost/master に descript.txt と pasta.log を置き、AppState でそれを開く
    fn open(state: &AppState, root: &Path, name: &str) -> PathBuf {
        let master = std::path::absolute(root.join(name).join("ghost/master")).unwrap();
        fs::create_dir_all(master.join("profile/pasta/logs")).unwrap();
        fs::write(master.join("descript.txt"), name).unwrap();
        fs::write(master.join("profile/pasta/logs/pasta.log"), "0123456789").unwrap();
        state.replace(Ghost {
            watcher: watch::start(master.clone(), |_| {}).unwrap(),
            master: master.clone(),
        });
        master
    }

    // コマンドの本体と同じく、覚えた master を通して読む
    fn read(state: &AppState, abs: &Path, start: u64) -> Result<Vec<u8>, String> {
        do_read(&state.dev_master()?, &abs.to_string_lossy(), start)
    }
    fn stat(state: &AppState, abs: &Path) -> Result<FileStat, String> {
        do_stat(&state.dev_master()?, &abs.to_string_lossy())
    }
    fn list(state: &AppState, abs: &Path) -> Result<Vec<String>, String> {
        do_list(&state.dev_master()?, &abs.to_string_lossy())
    }

    #[test]
    fn reads_only_inside_the_running_operation() {
        let tmp = tempfile::tempdir().unwrap();
        let state = AppState::default();
        assert!(do_begin(&state).unwrap_err().contains("開いていません"));
        let hello = open(&state, tmp.path(), "hello");
        let log = hello.join("profile/pasta/logs/pasta.log");
        // 始めの前は断る
        assert!(
            read(&state, &log, 0)
                .unwrap_err()
                .contains("進んでいません")
        );
        assert!(stat(&state, &log).is_err());
        assert!(list(&state, &hello).is_err());

        do_begin(&state).unwrap();
        assert_eq!(read(&state, &log, 0).unwrap(), b"0123456789");
        assert_eq!(read(&state, &log, 7).unwrap(), b"789");
        assert_eq!(
            read(&state, &hello.join("DESCRIPT.TXT"), 0).unwrap(),
            b"hello"
        );
        let st = stat(&state, &log).unwrap();
        assert_eq!((st.size, st.is_file), (10, true));
        assert!(st.mtime_ms > 0.0);
        let mut names = list(&state, &hello).unwrap();
        names.sort();
        assert_eq!(names, ["descript.txt", "profile"]);
        // 門の外・ファイルでないもの
        assert!(read(&state, &hello.join("profile/pasta"), 0).is_err());
        assert!(read(&state, &hello.join("dic/a.pasta"), 0).is_err());

        // 途中で別のゴーストを開いても、前の master は終わりまで読め、新しい master は断る
        let emo2 = open(&state, tmp.path(), "emo2");
        assert_eq!(read(&state, &log, 0).unwrap(), b"0123456789");
        assert!(read(&state, &emo2.join("descript.txt"), 0).is_err());
        assert!(list(&state, &emo2).is_err());

        // dev_end の本体
        state.set_dev_master(None);
        // 終わりの後は前も新しいも断る
        assert!(read(&state, &log, 0).is_err());
        assert!(read(&state, &emo2.join("descript.txt"), 0).is_err());
        // 次の始めは今の master を覚える
        do_begin(&state).unwrap();
        assert_eq!(
            read(&state, &emo2.join("descript.txt"), 0).unwrap(),
            b"emo2"
        );
        assert!(read(&state, &log, 0).is_err());
    }

    #[test]
    fn file_stat_serializes_like_ghost_io() {
        let j = serde_json::to_value(FileStat {
            size: 3,
            mtime_ms: 1.5,
            is_file: true,
        })
        .unwrap();
        assert_eq!(
            j,
            serde_json::json!({"size": 3, "mtimeMs": 1.5, "isFile": true})
        );
    }

    #[test]
    fn serializes_like_ghost_io() {
        let j = |o: PostOutcome| serde_json::to_value(o).unwrap();
        assert_eq!(
            j(PostOutcome::Response {
                status: 200,
                body: "b".into()
            }),
            serde_json::json!({"kind": "response", "status": 200, "body": "b"})
        );
        assert_eq!(
            j(PostOutcome::Unreachable { detail: "d".into() }),
            serde_json::json!({"kind": "unreachable", "detail": "d"})
        );
        assert_eq!(
            j(PostOutcome::NoResponse { detail: "d".into() }),
            serde_json::json!({"kind": "noResponse", "detail": "d"})
        );
    }
}
