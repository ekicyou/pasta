use super::*;
/// 多対1 Continue: `.pasta` 行（→ 複数 `.lua` 行）に BP を張り、停止後 **1 回だけ**
/// `continue` すると、(1) 同一 `.pasta` 行に対する 2 回目の `stopped` は来ず、(2) 実行は
/// 次の `.pasta` 行（次の停止点）へ進む — を実 DAP-over-TCP で証明する
/// （requirements 1.1 / 1.2 / 3.2 / 6.2(a)）。
#[test]
fn one_continue_escapes_multi_to_one_pasta_line_over_tcp() {
    let fx = build_fixture();

    // 前提の自己点検: 多対1 行が ≥2 の `.lua` 行へ展開される（テストが非空虚である土台）。
    assert!(
        fx.multi_lua_lines.len() >= 2,
        "多対1 前提: `.pasta` 行 {} → `.lua` 行 {:?}",
        fx.multi_pasta_line,
        fx.multi_lua_lines
    );

    let lua_source = fx.lua_source.clone();
    let chunk_name = fx.chunk_name.clone();
    let map = Arc::clone(&fx.map);

    let (addr_tx, addr_rx) = mpsc::channel::<SocketAddr>();
    let (loaded_tx, loaded_rx) = mpsc::channel::<()>();
    let (go_tx, go_rx) = mpsc::channel::<()>();

    // VM HOST スレッド。
    let host = std::thread::spawn(move || -> Result<(), String> {
        let lua = unsafe {
            mlua::Lua::unsafe_new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default())
        };

        let cfg = DebugConfig {
            enabled: true,
            listen: Some("127.0.0.1:0".parse().unwrap()),
            source_mode: SourceMode::Pasta, // 既定だが明示（6.1）。
            ..Default::default()
        };
        let handle = enable(&lua, &cfg, Some(map))
            .map_err(|e| format!("enable failed: {e}"))?
            .ok_or_else(|| "enable returned None for an enabled config".to_string())?;

        let addr = handle
            .local_addr()
            .ok_or_else(|| "enabled handle must expose a bound addr".to_string())?;
        addr_tx.send(addr).map_err(|_| "addr send failed".to_string())?;

        // `require "pasta"` を満たすシムを先に読み込む（BP 設定前・関数定義準備）。
        lua.load(PASTA_SHIM)
            .set_name("@pasta_shim")
            .exec()
            .map_err(|e| format!("shim exec failed: {e}"))?;

        // 生成 `.lua` 本文を **ローダ由来チャンク名**で実行し、`SCENE.__start__` 等を
        // 定義する。多対1 行 BP はまだ張られていないので、定義時に `.lua` 行 7（→ `.pasta`
        // 21）を通過しても停止しない（BP 設定前）。
        lua.load(&lua_source)
            .set_name(format!("@{chunk_name}"))
            .exec()
            .map_err(|e| format!("chunk (definitions) exec failed: {e}"))?;

        // 定義完了をクライアントへ通知し、BP+configurationDone 完了の go を待つ。
        loaded_tx
            .send(())
            .map_err(|_| "loaded signal send failed".to_string())?;
        go_rx
            .recv_timeout(WATCHDOG)
            .map_err(|_| "did not receive go signal before calling __start__".to_string())?;

        // ここで初めて多対1 行が **連続実行**される: 捕捉した SCENE の `__start__(ACT)` を
        // 呼ぶと `.lua` 行 11→12→13（すべて `.pasta` 21）→ 14（`.pasta` 22）が走る。
        // SCENE は chunk の `do ... end` ローカルなので、shim が捕捉した `__last_scene`
        // 経由で取得する。
        lua.load("local s = package.loaded['pasta'].__last_scene; return s.__start__(ACT)")
            .set_name("@invoke_start")
            .exec()
            .map_err(|e| format!("__start__ call failed: {e}"))?;

        lua.remove_global_hook();
        drop(handle);
        Ok(())
    });

    // CLIENT（このスレッド）。
    let addr = addr_rx
        .recv_timeout(WATCHDOG)
        .expect("host must publish the bound addr before the watchdog");
    let mut client = DapClient::connect(addr);

    // initialize ハンドシェイク。
    client.send_request(1, "initialize", json!({ "adapterID": "pasta" }));
    let _ = client.recv_until(|m| is_response(m, "initialize"));
    let _ = client.recv_until(|m| is_event(m, "initialized"));

    // 関数定義（チャンク本文実行）が終わるのを待ってから BP を張る — こうすると BP 有効
    // 下で実行されるのは `__start__` 本体だけになり、多対1 行の連続実行を観測できる。
    loaded_rx
        .recv_timeout(WATCHDOG)
        .expect("host must finish definitions before the watchdog");

    // 多対1 `.pasta` 行（翻訳経路 map+Pasta で複数 `.lua` 行へ展開）と、その **次行**の
    // 両方へ BP を張る。次行 BP は「1 回の continue が次の停止点まで進む」ことを観測する
    // ための停止点であると同時に、coalescing が無ければ continue が **次行へ着く前に**
    // 同一 `.pasta` 行 21 の別 `.lua` 行で再停止することを暴く（テストの歯）。
    client.send_request(
        2,
        "setBreakpoints",
        json!({
            "source": { "path": fx.pasta_path },
            "breakpoints": [
                { "line": fx.multi_pasta_line },
                { "line": fx.next_pasta_line },
            ],
        }),
    );
    let bp_resp = client.recv_until(|m| is_response(m, "setBreakpoints"));
    let bps = bp_resp["body"]["breakpoints"]
        .as_array()
        .expect("breakpoints array");
    assert_eq!(bps.len(), 2, "2 つの BP 応答（多対1 行＋次行）");
    assert!(
        bps.iter().all(|b| b["verified"] == true),
        "両 `.pasta` 行 BP は verified で登録される: {bps:?}"
    );

    client.send_request(3, "configurationDone", json!({}));
    let _ = client.recv_until(|m| is_response(m, "configurationDone"));
    go_tx.send(()).expect("send go signal");

    // (1) 最初の停止: 多対1 `.pasta` 行で stop（reason breakpoint）。
    let stopped1 = client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(
        stopped1["body"]["reason"], "breakpoint",
        "1.2/3.2: 多対1 `.pasta` 行 BP に対応する `.lua` 行で停止する"
    );
    let thread_id = stopped1["body"]["threadId"].as_u64().expect("threadId");
    // 停止位置は多対1 `.pasta` 行（resolver 装着で `.pasta` 座標を提示する）。
    let stop1_line = top_pasta_line(&mut client, thread_id, 10);
    assert_eq!(
        stop1_line, fx.multi_pasta_line,
        "最初の停止は多対1 `.pasta` 行 {} であること",
        fx.multi_pasta_line
    );

    // === 「歯」: ここで **1 回だけ** continue を送る。fix（break-anchor coalescing）が
    // あれば、同じ `.pasta` 行へマップする残りの `.lua` 行は消化され、次の `stopped` は
    // **別の** `.pasta` 行（次の停止点）になる。fix が無ければ、同一 `.pasta` 行 {} の
    // 次の `.lua` 行で再停止し（reason breakpoint・同じ提示行）、下のアサートが落ちる。===
    client.send_request(20, "continue", json!({ "threadId": thread_id }));
    let _ = client.recv_until(|m| is_response(m, "continue"));

    // 1 回の continue 後に来る次の制御フレーム（stopped か terminated）。
    let next = client.recv_until(|m| is_event(m, "stopped") || is_event(m, "terminated"));

    if is_event(&next, "stopped") {
        // 次の停止が **同一 `.pasta` 行** であってはならない（2 回目の同一行 stop の禁止 =
        // requirement 1.1 / 3.2）。停止しているなら次の `.pasta` 停止点であること。
        let next_tid = next["body"]["threadId"].as_u64().unwrap_or(thread_id);
        let stop2_line = top_pasta_line(&mut client, next_tid, 21);
        assert_ne!(
            stop2_line, fx.multi_pasta_line,
            "1.1/3.2: 1 回の continue が同一 `.pasta` 行 {} で **再停止してはならない** \
             （coalescing）。actual={stop2_line}",
            fx.multi_pasta_line
        );
        assert_eq!(
            stop2_line, fx.next_pasta_line,
            "1.2: 1 回の continue は **次の** `.pasta` 行 {} へ進むこと。actual={stop2_line}",
            fx.next_pasta_line
        );
        // 後片付け: 残りを流し切って host を完走させる（CI 無限ループ防止に上限）。
        let mut done = false;
        for seq in 30u64..60u64 {
            client.send_request(seq, "continue", json!({ "threadId": next_tid }));
            let m =
                client.recv_until(|m| is_event(m, "stopped") || is_event(m, "terminated"));
            if is_event(&m, "terminated") {
                done = true;
                break;
            }
        }
        assert!(done, "残りを continue で流し切れること");
    } else {
        // 次行（`.pasta` {next_pasta_line}）には BP を張ってあるので、正常時はここで停止する
        // はず。stopped 無しで terminated したのは、1 回の continue が次の停止点へ到達できて
        // いない（早期完走）ことを意味するので失敗扱いとする。
        panic!(
            "1.2: 1 回の continue は次の `.pasta` 行 {} で停止すべきだが、stopped 無しで \
             terminated した（次の停止点へ到達できていない）",
            fx.next_pasta_line
        );
    }

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

/// Task 3.3 — ループ再訪の実 DAP-over-TCP E2E
/// （spec: `pasta-debug-break-coalesce`, requirements **2.2** / **6.2(b)**）。
///
/// # 観測する「done」
///
/// 同一 `.pasta` 行（`SCENE.加算ループ` 本体の `total = total + i`）をループで N 回
/// 再訪する構成で、その `.pasta` 行へ BP を張る。停止のたびに `continue` を送ると、ループ
/// 反復ごとに **同一 `.pasta` 行で再び停止** し、合計 **ちょうど N 回**（= `LOOP_VISITS`）の
/// `stopped`（reason breakpoint・同一提示行）を観測できることを実ソケットで証明する。
///
/// アンカー coalescing（fix）は **同一 `.pasta` 行に連続して留まる間**だけ再停止を抑止する。
/// ループは反復ごとに `for ... do`（条件/増分 = 本体とは別の `.pasta` 行）を経由するため、
/// 本体行へ再入するたびにアンカーがクリアされ、再び停止する。したがって観測される停止数は
///   - over-suppression（coalescing がループをまたいで効いてしまう）なら **< N**、
///   - 壊れた coalescing（同一行内で複数停止）なら **> N**、
///
/// となり、**ちょうど N** を要求することで両誤りを弁別する（テストの「歯」）。
///
/// # ハーネス
///
/// [`build_fixture`] と同一の本番ローダ経路でマップ＋生成 `.lua` を得るが、本テストは
/// **ループ本体 `.pasta` 行**（[`LOOP_BODY_MARKER`]）へ BP を張り、`SCENE.加算ループ(ACT)`
/// を呼んでループ本体を N 回実行させる（多対1 行ではなくループ行が観測対象）。
#[test]
fn loop_revisit_yields_one_stop_per_iteration_over_tcp() {
    // --- フィクスチャ（本番ローダ経路）を build_fixture と同流儀で構築し、ループ本体行を導出 ---
    let temp = tempfile::TempDir::new().expect("temp dir");
    let base_dir = temp.path().to_path_buf();
    let pasta_file = base_dir.join("dic/test/debug_break_coalesce.pasta");
    std::fs::create_dir_all(pasta_file.parent().unwrap()).expect("mkdir dic");
    std::fs::write(&pasta_file, FIXTURE).expect("write .pasta");

    let cache_manager = CacheManager::new(base_dir.clone(), "profile/pasta/cache/lua");
    let map = crate::loader::PastaLoader::build_source_map(
        std::slice::from_ref(&pasta_file),
        &cache_manager,
        false,
    );
    let chunk_name = cache_manager
        .source_to_cache_path(&pasta_file)
        .to_string_lossy()
        .to_string();
    let pasta_path = pasta_file.to_string_lossy().to_string();

    // 生成 `.lua` 本文（map 構築とバイト一致）。
    let content = std::fs::read_to_string(&pasta_file).expect("read .pasta");
    let parsed = pasta_dsl::parse_str(&content, &pasta_path).expect("parse .pasta");
    let transpiler = LuaTranspiler::default();
    let mut sink = MapBuilderSink::new(pasta_path.clone(), chunk_name.clone());
    let mut out = Vec::new();
    transpiler
        .transpile_with_source_map(&parsed, &mut out, Some(&mut sink))
        .expect("transpile .pasta");
    let lua_source = String::from_utf8(out).expect("utf8 .lua");

    // ループ本体 `.pasta` 行は単一の `.lua` 実行座標へ対応する（前提＝再訪は同一行）。
    let loop_pasta_line = unique_pasta_line(LOOP_BODY_MARKER);
    let loop_lua_lines: Vec<u32> = map
        .resolve_pasta_to_lua(&pasta_path, loop_pasta_line)
        .into_iter()
        .map(|(_chunk, lua_line)| lua_line)
        .collect();
    assert_eq!(
        loop_lua_lines.len(),
        1,
        "6.2(b) 前提: ループ本体 `.pasta` 行 {loop_pasta_line} は単一 `.lua` 座標を持つ: \
         {loop_lua_lines:?}"
    );

    drop(temp);

    let (addr_tx, addr_rx) = mpsc::channel::<SocketAddr>();
    let (loaded_tx, loaded_rx) = mpsc::channel::<()>();
    let (go_tx, go_rx) = mpsc::channel::<()>();

    let map_host = Arc::clone(&map);

    // VM HOST スレッド。
    let host = std::thread::spawn(move || -> Result<(), String> {
        let lua = unsafe {
            mlua::Lua::unsafe_new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default())
        };

        let cfg = DebugConfig {
            enabled: true,
            listen: Some("127.0.0.1:0".parse().unwrap()),
            source_mode: SourceMode::Pasta,
            ..Default::default()
        };
        let handle = enable(&lua, &cfg, Some(map_host))
            .map_err(|e| format!("enable failed: {e}"))?
            .ok_or_else(|| "enable returned None for an enabled config".to_string())?;

        let addr = handle
            .local_addr()
            .ok_or_else(|| "enabled handle must expose a bound addr".to_string())?;
        addr_tx.send(addr).map_err(|_| "addr send failed".to_string())?;

        // require シム（BP 設定前）。
        lua.load(PASTA_SHIM)
            .set_name("@pasta_shim")
            .exec()
            .map_err(|e| format!("shim exec failed: {e}"))?;

        // 生成 `.lua` 本文をローダ由来チャンク名で実行し、`SCENE.加算ループ` 等を定義する。
        // BP 未設定なので定義時の通過では停止しない。
        lua.load(&lua_source)
            .set_name(format!("@{chunk_name}"))
            .exec()
            .map_err(|e| format!("chunk (definitions) exec failed: {e}"))?;

        loaded_tx
            .send(())
            .map_err(|_| "loaded signal send failed".to_string())?;
        go_rx
            .recv_timeout(WATCHDOG)
            .map_err(|_| "did not receive go signal before calling 加算ループ".to_string())?;

        // ここで初めてループ本体行が **反復実行**される: 捕捉した SCENE の `加算ループ(ACT)`
        // を呼ぶと `for i = 1, 3 do total = total + i end` がループ本体行を N 回再訪する。
        lua.load("local s = package.loaded['pasta'].__last_scene; return s['加算ループ'](ACT)")
            .set_name("@invoke_loop")
            .exec()
            .map_err(|e| format!("加算ループ call failed: {e}"))?;

        lua.remove_global_hook();
        drop(handle);
        Ok(())
    });

    // CLIENT。
    let addr = addr_rx
        .recv_timeout(WATCHDOG)
        .expect("host must publish the bound addr before the watchdog");
    let mut client = DapClient::connect(addr);

    client.send_request(1, "initialize", json!({ "adapterID": "pasta" }));
    let _ = client.recv_until(|m| is_response(m, "initialize"));
    let _ = client.recv_until(|m| is_event(m, "initialized"));

    loaded_rx
        .recv_timeout(WATCHDOG)
        .expect("host must finish definitions before the watchdog");

    // ループ本体 `.pasta` 行のみへ BP を張る。
    client.send_request(
        2,
        "setBreakpoints",
        json!({
            "source": { "path": pasta_path },
            "breakpoints": [ { "line": loop_pasta_line } ],
        }),
    );
    let bp_resp = client.recv_until(|m| is_response(m, "setBreakpoints"));
    let bps = bp_resp["body"]["breakpoints"]
        .as_array()
        .expect("breakpoints array");
    assert_eq!(bps.len(), 1, "1 つの BP 応答（ループ本体行）");
    assert!(
        bps.iter().all(|b| b["verified"] == true),
        "ループ本体 `.pasta` 行 BP は verified で登録される: {bps:?}"
    );

    client.send_request(3, "configurationDone", json!({}));
    let _ = client.recv_until(|m| is_response(m, "configurationDone"));
    go_tx.send(()).expect("send go signal");

    // === 「歯」: 停止のたびに continue。ループは N 回まわるので、同一 `.pasta` 行で
    // ちょうど N 回 stop する。< N なら coalescing がループをまたいで効きすぎ、> N なら
    // 同一行内で過剰停止（coalescing 不全）。両者を「ちょうど N」で弁別する。===
    let mut stop_count = 0usize;
    let mut seq = 10u64;
    loop {
        let ev = client.recv_until(|m| is_event(m, "stopped") || is_event(m, "terminated"));
        if is_event(&ev, "terminated") {
            break;
        }
        // stopped: ループ本体 `.pasta` 行であることを検証して数える。
        assert_eq!(
            ev["body"]["reason"], "breakpoint",
            "2.2: ループ本体行 BP に対応する `.lua` 行で停止する"
        );
        let tid = ev["body"]["threadId"].as_u64().expect("threadId");
        let stop_line = top_pasta_line(&mut client, tid, seq);
        seq += 1;
        assert_eq!(
            stop_line, loop_pasta_line,
            "2.2/6.2(b): 各停止はループ本体 `.pasta` 行 {loop_pasta_line} であること。actual={stop_line}"
        );
        stop_count += 1;
        assert!(
            stop_count <= LOOP_VISITS,
            "2.2: 停止数がループ反復回数 {LOOP_VISITS} を超えた（coalescing 不全 / 同一行で過剰停止）: \
             既に {stop_count} 回停止"
        );
        // 次の反復へ。
        client.send_request(seq, "continue", json!({ "threadId": tid }));
        let _ = client.recv_until(|m| is_response(m, "continue"));
        seq += 1;
    }

    // ちょうど N 回停止（再訪ごとに 1 回、再訪ごとにアンカーがクリアされ再停止する）。
    assert_eq!(
        stop_count, LOOP_VISITS,
        "2.2/6.2(b): ループ本体 `.pasta` 行はループ反復回数 {LOOP_VISITS} と同数だけ停止すること \
         （N 訪問 → N 停止）。actual={stop_count}"
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
                .expect("loop scenario must run to completion");
        }
        Err(RecvTimeoutError::Timeout) => {
            panic!("host VM thread did not finish within the watchdog (hang?)");
        }
        Err(RecvTimeoutError::Disconnected) => panic!("join watcher disconnected"),
    }
}

/// stackTrace の top フレーム `line`（`.pasta` 提示中は `.pasta` 行）を返し、`.pasta`
/// 提示の「歯」を効かせる（[`super::pasta_step_e2e::top_pasta_line`] と同型）。
fn top_pasta_line(client: &mut DapClient, thread_id: u64, seq: u64) -> u32 {
    client.send_request(seq, "stackTrace", json!({ "threadId": thread_id }));
    let stack = client.recv_until(|m| is_response(m, "stackTrace"));
    let frames = stack["body"]["stackFrames"]
        .as_array()
        .expect("stackFrames array");
    assert!(!frames.is_empty(), "stack must have the stopped frame");
    let top_src = frames[0]["source"]["path"]
        .as_str()
        .expect("top frame source path");
    assert!(
        top_src.ends_with(".pasta"),
        "`.pasta` 提示中は top フレームが `.pasta` を提示すること: {top_src:?}"
    );
    frames[0]["line"].as_u64().expect("top frame line") as u32
}
