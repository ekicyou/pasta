use super::*;
// =======================================================================
// 6.2 / 9.5 — `.lua` 提示モード回帰（実 DAP `attach sourcePresentation="lua"`）。
// BP・停止・コールスタックが `.lua` 座標、ステップが `.lua` 行単位。
// 「歯」: 同一 `.lua` 行を `.pasta` モードで提示すると `.pasta` 座標になることを
// 併せて表明し、`.lua` モードのアサートが本物（恒真でない）ことを裏づける。
// =======================================================================
/// requirement **6.2**（`.lua` モードで BP・停止位置・コールスタックを `.lua` 座標で
/// 提示）/ **9.5**（`.lua` モードのステップは `.lua` 行単位）を、実 DAP-over-TCP
/// `attach sourcePresentation="lua"`（VSCode 等価クライアント経路・task 5.5/R6.3）で
/// 提示モードを切替えて end-to-end 検証する。
///
/// サーバ既定は `.pasta`（map present・6.1）だが、`attach` が `.lua` を **強制**する。
/// `.lua` 源（`@...`）への BP を行2（展開 `.pasta` 30 の 1 本目）に張り、停止・
/// コールスタックが `.lua` 行2（チャンク名提示）であること、step over が次の `.lua`
/// 行3（**同一 `.pasta` 30 の 2 本目**）で止まる（`.pasta` 粒度なら同一 `.pasta` を
/// 消化して行5 まで進むがそうならない）ことを表明する。
#[test]
fn mode_switch_lua_presents_lua_coords_and_lua_step_granularity_over_tcp() {
    let map = edge_scenario_map();

    // サーバ既定は Pasta（6.1）。attach で `.lua` を強制（R6.3）。BP は `.lua` 源・
    // 行2（展開 `.pasta` 30 の 1 本目）。`.lua`/`.pasta` 粒度が分岐する行を起点に選ぶ。
    let lua_bp_line = 2u32;
    let (host, mut client, thread_id) = start_session(
        Arc::clone(&map),
        SourceMode::Pasta,            // サーバ既定（6.1）。
        Some(SourceMode::Lua),        // attach で `.lua` へ切替（6.2/6.3）。
        EDGE_SOURCE,                  // `.lua` 源（`@...`・`.pasta` 拡張子ではない）。
        lua_bp_line,
    );

    // 6.2: `.lua` モードの停止・コールスタックは `.lua` 座標を提示する（`.pasta`
    // ではない）。top フレーム source.path はチャンク名（`@...`・`.pasta` で終わらない）、
    // line は `.lua` 行。
    let (lua_src, lua_line) = top_frame(&mut client, thread_id, 10);
    assert!(
        !lua_src.ends_with(".pasta"),
        "6.2: `.lua` モードのコールスタックは `.lua` 座標（チャンク名）を提示すること \
         （`.pasta` ではない）。actual={lua_src:?}"
    );
    assert_eq!(
        crate::debug::source_map::canonicalize_chunk_name(&lua_src),
        crate::debug::source_map::canonicalize_chunk_name(EDGE_SOURCE),
        "6.2: `.lua` モードの提示 source は生成 `.lua` チャンク（{EDGE_SOURCE}）"
    );
    assert_eq!(
        lua_line, lua_bp_line,
        "6.2: `.lua` モードの停止行は `.lua` 行（{lua_bp_line}）。`.pasta` 行（30）ではない"
    );

    // 9.5: `.lua` モードの step over は `.lua` 行単位。行2 → 行3（同一 `.pasta` 30 の
    // 2 本目）で停止する。`.pasta` 粒度なら行3/4 を消化して行5（`.pasta` 40）まで
    // 進むが、`.lua` モードではそうならない。
    client.send_request(20, "next", json!({ "threadId": thread_id }));
    let _ = client.recv_until(|m| is_response(m, "next"));
    let stopped = client.recv_until(|m| is_event(m, "stopped"));
    assert_eq!(stopped["body"]["reason"], "step", "9.5: step over は reason step");
    let (step_src, step_line) = top_frame(&mut client, thread_id, 21);
    assert!(
        !step_src.ends_with(".pasta"),
        "9.5: `.lua` モードのステップ後も `.lua` 座標提示。actual={step_src:?}"
    );
    assert_eq!(
        step_line, 3,
        "9.5: `.lua` モードの step over は次の `.lua` 行（3 = 同一 `.pasta` 30 の 2 本目）で \
         停止する。`.pasta` 粒度のように同一 `.pasta` 行を消化して `.pasta` 40（.lua 5）まで \
         進んではならない"
    );
    assert_ne!(
        step_line, 5,
        "9.5: `.pasta` 粒度の停止先（.lua 5 = `.pasta` 40）に進んではならない（`.lua` 粒度回帰）"
    );

    continue_to_end(host, &mut client, thread_id, 30);
}

/// 「歯」（6.2 の teeth）: `.lua` モード回帰の核心アサート（`.lua` 座標・`.lua` 粒度）が
/// **本物**であることを、同一 map・同一 `.lua` 行2 を **`.pasta` モード**で提示すると
/// **`.pasta` 30**（`.lua` 行ではない）が出ることで裏づける。提示モード切替が効いて
/// いなければ、`.lua` モードのテストも `.pasta` を提示して落ちるはず。
#[test]
fn teeth_same_lua_line_in_pasta_mode_presents_pasta_not_lua() {
    let map = edge_scenario_map();

    // `.pasta` モード（既定・attach なし）。`.pasta` 30（展開行）に BP を張り、最初の
    // 対応 `.lua` 行（行2）で停止する。
    let (host, mut client, thread_id) = start_session(
        Arc::clone(&map),
        SourceMode::Pasta,
        None,
        EDGE_PASTA_FILE,
        30,
    );

    // 歯: 同一 `.lua` 行2 が、`.pasta` モードでは `.pasta` 30 を提示する（`.lua` 行
    // ではない）。`.lua` モードのテストはここが `.lua` 座標に変わることを表明している。
    let (pasta_src, pasta_line) = top_frame(&mut client, thread_id, 10);
    assert!(
        pasta_src.ends_with(".pasta"),
        "歯: `.pasta` モードは `.pasta` を提示する（`.lua` モードのテストの差分が観測可能）。\
         actual={pasta_src:?}"
    );
    assert_eq!(
        pasta_line, 30,
        "歯: `.pasta` モードの提示行は `.pasta` 30（`.lua` 行2 ではない）。`.lua` モードの \
         テストの `.lua` 座標アサートは恒真でない"
    );

    continue_to_end(host, &mut client, thread_id, 30);
}

// =======================================================================
// 8.1 — 集約行に確定的単一 `.pasta` 提示。複数 `.pasta` 行が集約された単一 `.lua`
// 行で停止すると、確定的に単一の `.pasta` 位置を提示する（last-write-wins・3.3）。
// =======================================================================
/// requirement **8.1**: 複数 `.pasta` 行が同一 `.lua` 行へ集約された場合、当該 `.lua`
/// 行の停止について **確定的な単一の `.pasta` 位置**を提示する。
///
/// (a) マップ直接: 集約行 `.lua` 行1 の `resolve_lua_to_pasta` は **単一**の `.pasta`
///     位置（20）を返し、反復しても同一（決定的）。`from_forward` の
///     `BTreeMap<lua_line, PastaPos>` が 1 `.lua` 行 → 高々 1 `.pasta` 位置を構造的に
///     担保する（design 286 last-write-wins）。
/// (b) セッション提示: 集約 `.lua` 行で停止すると stackTrace の top フレームが その
///     単一 `.pasta` 20 を提示する。
#[test]
fn edge_8_1_aggregated_lua_line_presents_deterministic_single_pasta() {
    let map = edge_scenario_map();

    // (a) マップ直接: 集約 `.lua` 行1 → 単一 `.pasta` 20。反復一致（確定的）。
    let first = map
        .resolve_lua_to_pasta(EDGE_SOURCE, 1)
        .expect("集約 `.lua` 行1 は対応 `.pasta` を持つ")
        .clone();
    assert_eq!(
        first.line, 20,
        "8.1: 集約 `.lua` 行1 の `.pasta` 位置は確定的単一（20・last-write-wins）"
    );
    // 反復して同一位置を返す（確定性・8.1）。
    for _ in 0..8 {
        let again = map
            .resolve_lua_to_pasta(EDGE_SOURCE, 1)
            .expect("集約行は反復しても対応 `.pasta` を持つ");
        assert_eq!(
            (&again.file, again.line),
            (&first.file, first.line),
            "8.1: 集約行の `.pasta` 位置は反復しても確定的に同一の単一位置"
        );
    }

    // (b) セッション提示: `.pasta` 20 に BP を張り、集約 `.lua` 行1 で停止 → top
    // フレームは単一 `.pasta` 20 を提示する。
    let (host, mut client, thread_id) = start_session(
        Arc::clone(&map),
        SourceMode::Pasta,
        None,
        EDGE_PASTA_FILE,
        20,
    );
    let (src, line) = top_frame(&mut client, thread_id, 10);
    assert!(
        src.ends_with(".pasta"),
        "8.1: 集約行の停止は `.pasta` を提示する。actual={src:?}"
    );
    assert_eq!(
        line, 20,
        "8.1: 集約 `.lua` 行1 の停止は確定的単一の `.pasta` 20 を提示する"
    );

    continue_to_end(host, &mut client, thread_id, 30);
}

// =======================================================================
// 8.2 — 展開行は同一 `.pasta` 提示。単一 `.pasta` 行が複数 `.lua` 行へ展開された
// 場合、それら `.lua` 行のいずれで停止しても同一の `.pasta` 行を提示する。
// =======================================================================
/// requirement **8.2**: 単一 `.pasta` 行（30）が複数 `.lua` 行（2/3/4）へ展開された
/// とき、**いずれの `.lua` 行で停止しても**同一の `.pasta` 30 を提示する。
///
/// (a) マップ直接: `resolve_lua_to_pasta` が 行2/3/4 すべてで同一 `.pasta` 30 を返す。
/// (b) セッション提示: 各展開 `.lua` 行に **`.lua` 直接 BP**（`.lua` 源）を張り、それぞれ
///     `.pasta` モードで停止させて、top フレームが毎回同一 `.pasta` 30 を提示すること
///     を end-to-end で確認する（`.lua` 源 BP でも `.pasta` 提示は resolver が担う）。
#[test]
fn edge_8_2_expanded_pasta_line_same_pasta_at_every_lua_line() {
    let map = edge_scenario_map();

    // (a) マップ直接: 展開 `.lua` 行2/3/4 はすべて同一 `.pasta` 30。
    for lua_line in [2u32, 3, 4] {
        let pos = map
            .resolve_lua_to_pasta(EDGE_SOURCE, lua_line)
            .unwrap_or_else(|| panic!("展開 `.lua` 行{lua_line} は対応 `.pasta` を持つ"));
        assert_eq!(
            pos.line, 30,
            "8.2: 展開 `.lua` 行{lua_line} は単一 `.pasta` 30 へ写像する"
        );
    }

    // (b) セッション提示: 各展開 `.lua` 行で停止 → 毎回同一 `.pasta` 30 を提示する。
    // `.lua` 直接 BP（`.lua` 源）を使い、停止を当該 `.lua` 行に確定させつつ、提示は
    // `.pasta` モードの resolver（task 5.2）で `.pasta` 30 になることを確認する。
    for lua_line in [2u32, 3, 4] {
        let (host, mut client, thread_id) = start_session(
            Arc::clone(&map),
            SourceMode::Pasta, // 提示は `.pasta`（resolver 装着）。
            None,
            EDGE_SOURCE,       // `.lua` 直接 BP（停止行を当該 `.lua` 行に確定）。
            lua_line,
        );
        let (src, presented) = top_frame(&mut client, thread_id, 10);
        assert!(
            src.ends_with(".pasta"),
            "8.2: 展開 `.lua` 行{lua_line} の停止は `.pasta` を提示する。actual={src:?}"
        );
        assert_eq!(
            presented, 30,
            "8.2: 展開 `.lua` 行{lua_line} のいずれで停止しても同一 `.pasta` 30 を提示する"
        );
        continue_to_end(host, &mut client, thread_id, 30);
    }
}

// =======================================================================
// 8.3 — 提示順安定（決定的）。同一 `.pasta` 位置に対するマッピングの提示順が
// 反復・複数回構築をまたいで安定（決定的）。
// =======================================================================
/// requirement **8.3**: 同一 `.pasta` 位置に対するマッピングの提示順序を安定
/// （決定的）に保つ。
///
/// (a) `ChunkSourceMap::lua_lines_for_pasta` が展開 `.pasta` 30 に対し `.lua` 行の
///     **昇順**（[2, 3, 4]）を返し、反復しても同一順（決定的）。
/// (b) 集約 `SourceMap::resolve_pasta_to_lua` が `.pasta` 30 に対し
///     `(chunk, lua_line)` の決定的順序（チャンク名昇順 → `.lua` 行昇順）を返し、
///     反復しても同一。
/// (c) マップを **複数回構築**（`edge_scenario_map` を再呼び出し）しても同一順序
///     （ビルド非依存の決定論）。
#[test]
fn edge_8_3_presentation_order_is_stable_deterministic() {
    let map = edge_scenario_map();

    // (a) チャンクレベルの逆引きは `.lua` 行昇順で決定的。反復一致。
    // チャンクへ直接アクセスはできない（private）ため、集約 `SourceMap` 経由の
    // 逆引きで順序の安定を確認する（resolve_pasta_to_lua は同一決定的順序を返す）。
    let baseline: Vec<u32> = map
        .resolve_pasta_to_lua(EDGE_PASTA_FILE, 30)
        .into_iter()
        .map(|(_chunk, lua_line)| lua_line)
        .collect();
    assert_eq!(
        baseline,
        vec![2, 3, 4],
        "8.3: 展開 `.pasta` 30 の逆引きは `.lua` 行昇順 [2, 3, 4]（決定的）"
    );

    // (b) 反復しても完全一致（`(chunk, lua_line)` 列ごと）。
    let baseline_full = map.resolve_pasta_to_lua(EDGE_PASTA_FILE, 30);
    for _ in 0..16 {
        let again = map.resolve_pasta_to_lua(EDGE_PASTA_FILE, 30);
        assert_eq!(
            again, baseline_full,
            "8.3: 同一 `.pasta` 位置の提示順は反復しても安定（決定的）"
        );
    }

    // (c) マップを複数回構築しても同一順序（ビルド非依存の決定論）。
    for _ in 0..4 {
        let rebuilt = edge_scenario_map();
        let order: Vec<u32> = rebuilt
            .resolve_pasta_to_lua(EDGE_PASTA_FILE, 30)
            .into_iter()
            .map(|(_chunk, lua_line)| lua_line)
            .collect();
        assert_eq!(
            order, baseline,
            "8.3: 複数回構築しても提示順は安定（決定的・BTreeMap 由来）"
        );
    }
}
