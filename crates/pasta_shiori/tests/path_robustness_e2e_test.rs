//! 設置パスの形に依存しない応答バイト一致の E2E 検証（task 3.6）。
//!
//! 設計「Testing Strategy > E2E Tests（pasta_shiori・本番リクエスト経路）」項 2 の
//! 検証である。同一のゴーストを
//!
//!   A: 通常の設置先（短い ASCII の一時ディレクトリ）
//!   B: 300 文字超かつ非 ANSI 名（[`common::NON_ANSI_DIR_NAME`]）を含む設置先
//!
//! の 2 箇所へ設置し、**同一のリクエスト列**を本番のリクエスト経路へ流して、
//! 応答がバイト単位で一致することを確認する（要件 1.2 / 2.2 / 3.5 / 6.1 / 6.2）。
//!
//! # 本番のリクエスト経路
//! `lifecycle::spawn_actor`（FFI `load` / `loadu` が呼ぶ初期化）→
//! `lifecycle::marshal_request`（FFI `request` が呼ぶ marshaling・アクター境界込み）→
//! `lifecycle::teardown_actor`（FFI `unload`）を駆動する。VM はアクタースレッドへ pin され、
//! 応答はホストへ返る文字列そのものである。FFI の `load` 入口は設置パスをシステム ANSI で
//! デコードするため B のパスを表現できない（UTF-8 入口 `loadu` は `ffi_loadu_test.rs` が担当）。
//! 両入口が共通して呼ぶ `spawn_actor` を駆動することで、設置パスの表現の差を持ち込まずに
//! 本番経路そのものを比較できる。
//!
//! # 応答の決定性
//! ランダムトークや時刻を含まない `shiori_lifecycle` フィクスチャを用いる。応答は
//! `entry.lua` が組み立てる固定文字列（シーン `テスト挨拶` の出力、または未検出時の 500）
//! であり、設置パス・実行時刻・乱数のいずれも応答へ混入しない。A 側の応答は既知の
//! ゴールデンとも突き合わせ、「両方とも同じように壊れている」状態での一致を排除する。
//!
//! # 直列実行（プロセス全域 static を共有するため）
//! `actor::lifecycle::MAILBOX` はプロセスグローバルである。A と B は 1 本の `#[test]` の中で
//! 順に駆動し、各設置先の最後に `teardown_actor` してから次を spawn する
//! （既存 `load_failure_visibility_test.rs` / `ffi_actor_lifecycle_test.rs` と同方式）。
//!
//! `mod common;` は `PASTA_DEBUG` / `PASTA_DEBUG_PORT` を main 前に中和する `#[ctor]` を
//! 取り込むためにも必須である（要件 6.4）。`#[ignore]` は付けない（要件 6.6）。

mod common;

use common::{NON_ANSI_DIR_NAME, copy_fixture_into, make_deep_dir};
use pasta::actor::lifecycle::{marshal_request, spawn_actor, teardown_actor};
use pasta::actor::marshaling::default_204;
use std::path::Path;
use tempfile::TempDir;

/// `shiori_lifecycle` フィクスチャの `entry.lua` がシーン `テスト挨拶`
/// （`dic/test/lifecycle.pasta`）を解決して返す 200 応答のゴールデン。
///
/// モジュール解決（`@pasta_search`・トランスパイル済みシーンモジュール）まで完走しないと
/// この文字列は得られないため、一致は「正常動作しての一致」であることの証明になる。
const GOLDEN_SCENE_OK: &str = "SHIORI/3.0 200 OK\r\n\
Charset: UTF-8\r\n\
Value: ライフサイクルテスト成功！\r\n\
\r\n";

/// フィクスチャに存在しないイベント ID。`entry.lua` が決定論的な 500 応答を返す。
const ABSENT_EVENT_ID: &str = "PastaAbsentSceneMarker";

/// 両設置先へ流す同一のリクエスト列（ラベル, 生リクエスト）。
///
/// ロード（`spawn_actor`）に続けて、スクリプトが実際に応答を組み立てる GET・NOTIFY・
/// 未検出 GET・再度の GET を流し、モジュール解決とシーン実行を実際に働かせる。
const REQUEST_SEQUENCE: &[(&str, &str)] = &[
    (
        "1: GET scene",
        "GET SHIORI/3.0\nCharset: UTF-8\nID: テスト挨拶\nSender: SSP\n",
    ),
    (
        "2: NOTIFY unrelated",
        "NOTIFY SHIORI/3.0\nCharset: UTF-8\nID: OnUnrelatedEvent\nSender: SSP\n",
    ),
    (
        "3: GET absent scene",
        "GET SHIORI/3.0\nCharset: UTF-8\nID: PastaAbsentSceneMarker\nSender: SSP\n",
    ),
    (
        "4: GET scene again",
        "GET SHIORI/3.0\nCharset: UTF-8\nID: テスト挨拶\nSender: SSP\n",
    ),
];

/// SHIORI/3.0 リクエストを正規化（改行 → CRLF・終端付与）する。
fn normalize_request(text: &str) -> String {
    let trimmed = text.trim_matches(|c| c == '\r' || c == '\n');
    let mut req = trimmed
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n");
    req.push_str("\r\n\r\n");
    req
}

/// `base_dir` へゴーストを設置し、本番経路でリクエスト列を駆動して応答を集める。
///
/// 次の設置先が汚れた `MAILBOX` を引き継がないよう、最後に必ず teardown する。
fn drive_install(label: &str, base_dir: &Path) -> Vec<String> {
    copy_fixture_into("shiori_lifecycle", base_dir);

    let loaded = spawn_actor(0, base_dir.to_path_buf());
    assert!(
        loaded,
        "[{label}] 設置パスの形に依存せずロードは成功するべき（要件 1.2 / 2.2）\n  設置先: {}",
        base_dir.display()
    );

    let responses = REQUEST_SEQUENCE
        .iter()
        .map(|(_, text)| marshal_request(&normalize_request(text)))
        .collect();

    let report = teardown_actor();
    assert!(
        report.is_clean() || report.already_done,
        "[{label}] unload はクリーンに終わるべき: {report:?}"
    );

    responses
}

/// A 側（通常の設置先）の応答が既知のゴールデンどおりであることを確認する。
///
/// これが無いと「両方とも同じように壊れている」場合にもバイト一致が成立してしまう。
fn assert_baseline_is_healthy(responses: &[String]) {
    assert_eq!(
        responses[0].as_bytes(),
        GOLDEN_SCENE_OK.as_bytes(),
        "[A] シーン GET は正常応答を返すべき（比較の前提）\nactual: {:?}",
        responses[0]
    );
    assert_eq!(
        responses[1].as_bytes(),
        default_204().as_bytes(),
        "[A] NOTIFY は 204 のままであるべき\nactual: {:?}",
        responses[1]
    );
    assert!(
        responses[2].starts_with("SHIORI/3.0 500 Internal Server Error\r\n")
            && responses[2].contains(ABSENT_EVENT_ID),
        "[A] 未検出シーンの GET はイベント ID を含む 500 を返すべき\nactual: {:?}",
        responses[2]
    );
    assert_eq!(
        responses[3].as_bytes(),
        GOLDEN_SCENE_OK.as_bytes(),
        "[A] 2 回目のシーン GET も同じ正常応答を返すべき\nactual: {:?}",
        responses[3]
    );
}

/// 長パスかつ非 ANSI の設置先と通常の設置先で、同一リクエスト列の応答がバイト一致する。
///
/// 要件 1.2（260 文字超でも同一の起動シーケンス・同一のロード成否）／
/// 要件 2.2（長パス＋非 ANSI）／要件 3.5（通常の設置パスでの応答バイト不変）／
/// 要件 6.1・6.2（長パス・非 ANSI の常時実行検証）。
#[test]
fn responses_are_byte_identical_between_ordinary_and_long_non_ansi_installs() {
    // --- A: 通常の設置先（短い ASCII） ---
    let ordinary_root = TempDir::new().expect("一時ディレクトリの作成に失敗");
    let ordinary_dir = ordinary_root.path().join("ascii_install_path");
    let ordinary = drive_install("A: ordinary", &ordinary_dir);
    assert_baseline_is_healthy(&ordinary);

    // --- B: 300 文字超かつ非 ANSI を含む設置先 ---
    let long_root = TempDir::new().expect("一時ディレクトリの作成に失敗");
    let non_ansi_root = long_root.path().join(NON_ANSI_DIR_NAME);
    std::fs::create_dir_all(&non_ansi_root).expect("非 ANSI ディレクトリの作成に失敗");
    let long_non_ansi_dir = make_deep_dir(&non_ansi_root, 300);

    // 将来の改修で B が短い ASCII パスへ退化しても気付けるようにしておく（要件 6.1 / 6.2）。
    assert!(
        long_non_ansi_dir.display().to_string().chars().count() > 300,
        "[B] 設置先が 300 文字を超えていない: {}",
        long_non_ansi_dir.display()
    );
    assert!(
        long_non_ansi_dir
            .to_string_lossy()
            .contains(NON_ANSI_DIR_NAME),
        "[B] 設置先に非 ANSI セグメントが残っていない: {}",
        long_non_ansi_dir.display()
    );

    let long_non_ansi = drive_install("B: long + non-ANSI", &long_non_ansi_dir);

    // --- 本題: 同一リクエスト列に対する応答のバイト一致 ---
    for (index, (label, _)) in REQUEST_SEQUENCE.iter().enumerate() {
        assert_eq!(
            long_non_ansi[index].as_bytes(),
            ordinary[index].as_bytes(),
            "[{label}] 設置パスの形が違っても応答はバイト単位で一致するべき\
             （要件 1.2 / 2.2 / 3.5）\n  \
             通常の設置先 (len={}): {:?}\n  \
             長パス＋非 ANSI (len={}): {:?}",
            ordinary[index].len(),
            ordinary[index],
            long_non_ansi[index].len(),
            long_non_ansi[index],
        );
    }

    // 後始末: プロセス終了でアクターが漏れないように最終 teardown（冪等）。
    let _ = teardown_actor();
}
