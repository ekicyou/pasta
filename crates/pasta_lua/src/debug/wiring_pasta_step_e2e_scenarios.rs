use super::*;
// =======================================================================
// E1（9.1）+ E6（9.4）+ E2（9.1）: step over が、複数 `.lua` 行へ展開された同一
// `.pasta` 行を消化し、未対応行を通過し、次の異なる `.pasta` 行で停止する。さらに
// サブ呼び出しを含む行（`helper(c)`）からの step over はサブ呼び出しに入らない（E2）。
// =======================================================================
/// E1（9.1）/ E6（9.4）/ E2（9.1）を 1 セッションで検証する。
///
/// - **1 回目 step over**（起点 `.pasta` 10・行18 = 単一 `.lua` 行）→ 次の異なる
///   `.pasta` 行 = `.pasta` 11（行19・複数 `.lua` 行展開の 1 本目）で停止。
/// - **E1/E6（2 回目 step over）**: `.pasta` 11（行19）から step over → 行20（**同一
///   `.pasta` 11**・複数 `.lua` 行の 2 本目）を**消化**し、行21（未対応・E6/9.4）を
///   **通過**し、`.pasta` 12（行22）で停止する（**`.pasta` 12**・`.lua` 20/21 ではない）。
/// - **E2（3 回目 step over）**: `.pasta` 12（`helper(c)` を**含む**行22）から step over
///   → サブ呼び出し helper（行2/3/4・`.pasta` 30/31）に入らず、呼出元フレームの次の
///   `.pasta` 行 `.pasta` 13（行23）で停止する。
#[test]
fn e1_e6_e2_step_over_consumes_pasta_line_passes_unmapped_and_skips_sub_call() {
    let map = step_scenario_map();
    let exp = Expected::derive(&map);

    let (host, mut client, thread_id) =
        start_session(Arc::clone(&map), SourceMode::Pasta, STEP_PASTA_FILE, exp.origin_pasta);

    // BP 停止位置は `.pasta` 起点行を提示する（5.1）。
    assert_eq!(
        top_pasta_line(&mut client, thread_id, 10),
        exp.origin_pasta,
        "BP 停止は `.pasta` 起点行（{}）を提示する",
        exp.origin_pasta
    );

    // 1 回目 step over（起点 = 単一 `.lua` 行）→ 複数 `.lua` 行展開の `.pasta` 行（行19）。
    client.send_request(20, "next", json!({ "threadId": thread_id }));
    let _ = client.recv_until(|m| is_response(m, "next"));
    let stopped = client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(stopped["body"]["reason"], "step", "1 回目 step over は reason step");
    assert_eq!(
        top_pasta_line(&mut client, thread_id, 21),
        exp.multi_pasta,
        "1 回目 step over は次の異なる `.pasta` 行 {}（複数 `.lua` 行展開・行{}）で停止する",
        exp.multi_pasta,
        exp.multi_lua_first
    );

    // E1/E6: 2 回目 step over（`.pasta` 11・行19 起点）→ 行20（同一 `.pasta` 11）を消化＋
    // 行21（未対応）を通過 → 次の異なる `.pasta` 行（`.pasta` 12・行22）で停止。
    client.send_request(22, "next", json!({ "threadId": thread_id }));
    let _ = client.recv_until(|m| is_response(m, "next"));
    let stopped = client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(stopped["body"]["reason"], "step", "E1: step over は reason step");
    assert_eq!(
        top_pasta_line(&mut client, thread_id, 23),
        exp.call_helper_pasta,
        "E1/E6/9.1/9.4: step over は同一 `.pasta` 行の 2 本目（.lua {}）を消化し、未対応行 \
         （.lua {}）を通過、次の異なる `.pasta` 行 {} で停止する（`.lua` 行ではない）",
        exp.multi_lua_second,
        exp.unmapped_lua,
        exp.call_helper_pasta
    );

    // E2: 3 回目 step over（`.pasta` 12・helper(c) を含む行22）→ サブ呼び出しに入らず、
    // 呼出元フレームの次の `.pasta` 行（`.pasta` 13・行23）で停止。
    client.send_request(24, "next", json!({ "threadId": thread_id }));
    let _ = client.recv_until(|m| is_response(m, "next"));
    let _ = client.recv_until(|m| is_event(m, "stopped"));
    let after_sub = top_pasta_line(&mut client, thread_id, 25);
    assert_eq!(
        after_sub, exp.next_caller_pasta,
        "E2/9.1: サブ呼び出しを含む行から step over は呼び出し先 helper（`.pasta` \
         30/31）に入らず、次の `.pasta` 行 {}（呼出元フレーム）で停止する",
        exp.next_caller_pasta
    );
    assert_ne!(
        after_sub, exp.callee_first_pasta,
        "E2: helper 内の `.pasta` 行（{}）で停止してはならない",
        exp.callee_first_pasta
    );

    continue_to_end(host, &mut client, thread_id, 30);
}

// =======================================================================
// E3（9.2）+ E4（9.3）: step into で呼び出し先の最初の対応 `.pasta` 行へ、
// step out で呼出元の次の対応 `.pasta` 行へ。
// =======================================================================
/// E3（9.2）/ E4（9.3）を 1 セッションで検証する。
///
/// - **E3**: 行22（`helper(c)`・`.pasta` 12）で停止 → step into は helper に入り、
///   未対応の行2 を通過して、helper の最初の対応 `.pasta` 行（行3 = `.pasta` 30）で
///   停止する。
/// - **E4**: helper 内（行3）から step out → 呼出元へ戻り、呼出行の `.pasta` 行とは
///   異なる**次の対応 `.pasta` 行**（行23 = `.pasta` 13）で停止する。
#[test]
fn e3_e4_step_into_first_callee_pasta_line_and_step_out_next_caller_pasta_line() {
    let map = step_scenario_map();
    let exp = Expected::derive(&map);

    // 呼び出し行（`.pasta` 12）に BP を張ってそこから step する。
    let (host, mut client, thread_id) = start_session(
        Arc::clone(&map),
        SourceMode::Pasta,
        STEP_PASTA_FILE,
        exp.call_helper_pasta,
    );
    assert_eq!(
        top_pasta_line(&mut client, thread_id, 10),
        exp.call_helper_pasta,
        "BP 停止は呼び出し行の `.pasta`（{}）",
        exp.call_helper_pasta
    );

    // E3: step into（stepIn）→ helper の最初の対応 `.pasta` 行（行3 = `.pasta` 30）。
    client.send_request(20, "stepIn", json!({ "threadId": thread_id }));
    let _ = client.recv_until(|m| is_response(m, "stepIn"));
    let stopped = client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(stopped["body"]["reason"], "step", "E3: step into は reason step");
    assert_eq!(
        top_pasta_line(&mut client, thread_id, 21),
        exp.callee_first_pasta,
        "E3/9.2/9.4: step into は未対応の callee 行（.lua {}）を通過し、helper の最初の \
         対応 `.pasta` 行 {} で停止する",
        exp.callee_unmapped_lua,
        exp.callee_first_pasta
    );

    // E4: step out（stepOut）→ 呼出元へ戻り、呼出行と異なる次の対応 `.pasta` 行
    // （行23 = `.pasta` 13）で停止する。
    client.send_request(30, "stepOut", json!({ "threadId": thread_id }));
    let _ = client.recv_until(|m| is_response(m, "stepOut"));
    let stopped = client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(stopped["body"]["reason"], "step", "E4: step out は reason step");
    let out_line = top_pasta_line(&mut client, thread_id, 31);
    assert_eq!(
        out_line, exp.step_out_pasta,
        "E4/9.3: step out は呼出元へ戻り、次の対応 `.pasta` 行 {} で停止する",
        exp.step_out_pasta
    );
    assert_ne!(
        out_line, exp.call_helper_pasta,
        "E4: 呼出行の `.pasta` 行（{}）で再停止してはならない（次の対応行へ）",
        exp.call_helper_pasta
    );

    continue_to_end(host, &mut client, thread_id, 40);
}

// =======================================================================
// E5: 再帰で別フレームの同一 `.pasta` 行に誤停止しない（depth による frame identity）。
// =======================================================================
/// E5 を検証する。`recur(2)`（行23・`.pasta` 13）から step over すると、recur は
/// 自身を再帰呼び出し（行7/8 = `.pasta` 40/41 を**異なる深さのフレームで複数回**踏む）
/// するが、step over は base フレームより深いフレームを depth で除外するため、それらの
/// 同一 `.pasta` 行に誤停止せず、呼出元フレームの次の `.pasta` 行（行24 = `.pasta` 14）で
/// 停止する。
#[test]
fn e5_recursion_does_not_mis_stop_at_same_pasta_line_in_other_frames() {
    let map = step_scenario_map();
    let exp = Expected::derive(&map);

    // 再帰呼び出し行（`.pasta` 13）に BP を張る。
    let (host, mut client, thread_id) = start_session(
        Arc::clone(&map),
        SourceMode::Pasta,
        STEP_PASTA_FILE,
        exp.recur_call_pasta,
    );
    assert_eq!(
        top_pasta_line(&mut client, thread_id, 10),
        exp.recur_call_pasta,
        "BP 停止は再帰呼び出し行の `.pasta`（{}）",
        exp.recur_call_pasta
    );

    // E5: step over → recur の深いフレーム（`.pasta` 40/41 を複数回踏む）に誤停止せず、
    // 呼出元フレームの次の `.pasta` 行（行24 = `.pasta` 14）で停止する。
    client.send_request(20, "next", json!({ "threadId": thread_id }));
    let _ = client.recv_until(|m| is_response(m, "next"));
    let stopped = client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(stopped["body"]["reason"], "step", "E5: step over は reason step");
    assert_eq!(
        top_pasta_line(&mut client, thread_id, 21),
        exp.after_recur_pasta,
        "E5: 再帰の別フレームで同一 `.pasta` 行（40/41）を踏んでも誤停止せず、呼出元 \
         フレームの次の `.pasta` 行 {} で停止する（depth による frame identity）",
        exp.after_recur_pasta
    );

    continue_to_end(host, &mut client, thread_id, 30);
}

// =======================================================================
// E7: コルーチン跨ぎ（yield/resume）の `.pasta` ステップ。
// =======================================================================
/// E7 を検証する。コルーチン本体内（行13・`.pasta` 50）で停止し、step over は yield 行
/// （行14・`.pasta` 51）に到達。さらに step over で `coroutine.yield()` をまたぐと、
/// コルーチンは中断し駆動ループ（別スレッド・行25/26）が再 resume するが、それらは
/// thread 不一致で skip され、同一コルーチンの resume 後の `.pasta` 行（行15・
/// `.pasta` 52）で停止する（step 鍵が yield/resume をまたいで生存・採択B）。
#[test]
fn e7_pasta_step_over_crosses_coroutine_yield_resume() {
    let map = step_scenario_map();
    let exp = Expected::derive(&map);

    // コルーチン本体の起点行（`.pasta` 50）に BP を張る。
    let (host, mut client, thread_id) = start_session(
        Arc::clone(&map),
        SourceMode::Pasta,
        STEP_PASTA_FILE,
        exp.co_origin_pasta,
    );
    assert_eq!(
        top_pasta_line(&mut client, thread_id, 10),
        exp.co_origin_pasta,
        "BP 停止はコルーチン本体の `.pasta` 起点行（{}）",
        exp.co_origin_pasta
    );

    // 1 回目 step over: 同一フレーム次行 = yield 行（`.pasta` 51）。
    client.send_request(20, "next", json!({ "threadId": thread_id }));
    let _ = client.recv_until(|m| is_response(m, "next"));
    let _ = client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(
        top_pasta_line(&mut client, thread_id, 21),
        exp.co_yield_pasta,
        "E7: 1 回目 step over は yield 行の `.pasta`（{}）に停止する",
        exp.co_yield_pasta
    );

    // 2 回目 step over: `coroutine.yield()` をまたぐ。駆動ループ（別スレッド）を
    // skip し、resume 後の `.pasta` 行（`.pasta` 52）で停止する（採択B 生存）。
    client.send_request(30, "next", json!({ "threadId": thread_id }));
    let _ = client.recv_until(|m| is_response(m, "next"));
    let stopped = client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(stopped["body"]["reason"], "step", "E7: step over は reason step");
    assert_eq!(
        top_pasta_line(&mut client, thread_id, 31),
        exp.co_post_yield_pasta,
        "E7: yield をまたぐ step over は駆動ループ（別スレッド）を skip し、resume 後の \
         `.pasta` 行 {} で停止する（step 鍵が yield/resume をまたいで生存）",
        exp.co_post_yield_pasta
    );

    continue_to_end(host, &mut client, thread_id, 40);
}

// =======================================================================
// E8（9.5）: `.lua` モード回帰 — ステップは `.lua` 行単位（`.pasta` 消化しない）。
// =======================================================================
/// E8（9.5）を検証する。`SourceMode::Lua`（map は存在するが提示モードが `.lua`）の
/// とき、step over は**`.lua` 行単位**で進む。複数 `.lua` 行へ展開された `.pasta` 行の
/// 1 本目（行19）で停止 → step over は次の `.lua` 行20（**同一 `.pasta` 11 の 2 本目**）で
/// 停止する。`.pasta` 粒度なら行20 を消化して行22（`.pasta` 12）まで進むが、`.lua`
/// モードではそうならない。停止位置も `.lua` 座標で提示される。
#[test]
fn e8_lua_mode_steps_at_lua_granularity_regression() {
    let map = step_scenario_map();
    let exp = Expected::derive(&map);

    // Lua モード: 複数 `.lua` 展開行の 1 本目（行19）へ `.lua` 直接 BP を張る（`.pasta`
    // 翻訳経路は通さない）。`.lua`/`.pasta` 粒度が分岐する行を起点に選ぶ。
    let lua_source_path = STEP_SOURCE; // `@...`（`.pasta` 拡張子ではない）。
    let (host, mut client, thread_id) = start_session(
        Arc::clone(&map),
        SourceMode::Lua,
        lua_source_path,
        exp.multi_lua_first,
    );

    // `.lua` モードでは stackTrace は `.lua` 座標（`.pasta` ではない）を提示する。
    assert_eq!(
        top_frame_line(&mut client, thread_id, 10),
        exp.multi_lua_first,
        "E8/9.5: `.lua` モードの停止は `.lua` 行（{}）を提示する",
        exp.multi_lua_first
    );

    // E8: step over → 次の `.lua` 行（行20 = 同一 `.pasta` 11 の 2 本目）。`.pasta` 粒度の
    // ように行20 を消化して行22 まで進んではならない。
    client.send_request(20, "next", json!({ "threadId": thread_id }));
    let _ = client.recv_until(|m| is_response(m, "next"));
    let stopped = client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(stopped["body"]["reason"], "step", "E8: step over は reason step");
    let lua_step_line = top_frame_line(&mut client, thread_id, 21);
    assert_eq!(
        lua_step_line, exp.multi_lua_second,
        "E8/9.5: `.lua` モードは `.lua` 行単位（次行 {} で停止）。`.pasta` 粒度のように \
         同一 `.pasta` 行（{}）を消化して次の `.pasta` 行まで進んではならない",
        exp.multi_lua_second, exp.multi_pasta
    );
    assert_ne!(
        lua_step_line, exp.call_helper_lua,
        "E8: `.pasta` 粒度の停止先（.lua {}）に進んではならない（`.lua` 粒度回帰）",
        exp.call_helper_lua
    );

    continue_to_end(host, &mut client, thread_id, 30);
}

// =======================================================================
// 「歯」（teeth）: 中核シナリオ（E1）で `.pasta` 粒度を無効化（`SourceMode::Lua`）
// すると、停止が `.lua` 行になり `.pasta` 行**ではない**ことを示す。E1 の核心アサート
// （`.pasta` 11 の複数 `.lua` 行を消化して `.pasta` 12 で停止）が `.pasta` 粒度に
// **真に依存**していることの証左。
// =======================================================================
/// teeth: E1 の核心と同一の起点（複数 `.lua` 行展開の `.pasta` 11・行19）・操作でも、
/// `SourceMode::Lua` では step over の停止が `.lua` 行20（同一 `.pasta` 11 の 2 本目）に
/// なり、E1 が期待する `.pasta` 停止位置（`.pasta` 12 = `.lua` 22）には**到達しない**。
/// `.pasta` ステップが無効なら E1 のアサートが落ちることの実証。
#[test]
fn teeth_lua_mode_stops_at_lua_line_not_pasta() {
    let map = step_scenario_map();
    let exp = Expected::derive(&map);

    let (host, mut client, thread_id) =
        start_session(Arc::clone(&map), SourceMode::Lua, STEP_SOURCE, exp.multi_lua_first);

    // E1 と同じ起点（`.pasta` 11・行19）で同じ step over を `.lua` モードで行う。
    client.send_request(20, "next", json!({ "threadId": thread_id }));
    let _ = client.recv_until(|m| is_response(m, "next"));
    let _ = client.recv_until(|m| is_event(m, "stopped"));
    let lua_step_line = top_frame_line(&mut client, thread_id, 21);

    // teeth: `.lua` 粒度では同一 `.pasta` 行の 2 本目（行20）で止まる（E1 の `.pasta`
    // 停止 = `.lua` 22 ではない）。つまり E1 の核心アサート（`.pasta` 12）は `.pasta`
    // 粒度が ON のときだけ通る。
    assert_eq!(
        lua_step_line, exp.multi_lua_second,
        "teeth: `.pasta` 粒度を無効化（Lua モード）すると step over は `.lua` 行 {} で \
         停止する（E1 の `.pasta` 消化後の停止位置 .lua {} ではない）",
        exp.multi_lua_second, exp.call_helper_lua
    );
    assert_ne!(
        lua_step_line, exp.call_helper_lua,
        "teeth: `.pasta` 粒度が OFF なら E1 の停止位置（.lua {}）には到達しない \
         → E1 のアサートは `.pasta` 粒度に真に依存（恒真ではない）",
        exp.call_helper_lua
    );

    continue_to_end(host, &mut client, thread_id, 30);
}
