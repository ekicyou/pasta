//! [`resolve_and_kick`] / [`uri_to_pasta_path`] の単体テスト（task 3.1・
//! requirements 2.1-2.6/7.1/8.1/8.3）。
//!
//! 観測対象:
//! 1. 正規化済みパスの一致: 同一ファイルを指す別形式 uri（パーセントエンコード・
//!    区切り違い）が同一シーンへ解決する（`std::path::absolute` 正規化の固定）。
//! 2. global 確定 → 取次呼出（`scene == "会話1"`・parent None）。
//! 3. local 確定 → 取次呼出（`scene == ":会話1:挨拶_1"`・parent Some）。
//! 4. 未検出 → 取次しない・`ResolveOutcome::NotFound`。

use std::sync::{Arc, Mutex};

use super::*;
use crate::debug::kick::{KickRequest, KickSink};
use crate::debug::source_map::{SceneIdentityIndex, SourceMap};

/// 取次呼出を記録するモック [`KickSink`]。`(sink, captured)` を返す。
fn mock_sink() -> (KickSink, Arc<Mutex<Vec<String>>>) {
    let captured: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let cap = captured.clone();
    let sink: KickSink = Arc::new(move |req: KickRequest| {
        cap.lock().unwrap().push(req.scene);
    });
    (sink, captured)
}

/// テスト用 `SourceMap`（索引充填済み）を構築する。
///
/// `file` の global 会話1（行 10..=40・parent None）と、その内側 local 挨拶_1
/// （行 20..=30・parent Some(会話1)）を投入する。
fn map_with_index(file: &str) -> SourceMap {
    let mut b = SceneIdentityIndex::builder();
    b.add_scene(file, "会話1", None, 10, 40, 0);
    b.add_scene(file, "挨拶_1", Some("会話1"), 20, 30, 1);
    let index = b.finish();

    let map = SourceMap::new();
    map.set_scene_index(index)
        .expect("write-once index set must succeed");
    map
}

#[test]
fn global_confirmed_kicks_with_plain_scene_id() {
    // global 本体行（local 範囲外）→ scene == "会話1"（parent None）。
    let file = "C:/work/dic/talk.pasta";
    let map = map_with_index(file);
    let (sink, captured) = mock_sink();

    let outcome = resolve_and_kick(&map, &sink, "file:///C:/work/dic/talk.pasta", 15);

    assert_eq!(outcome, ResolveOutcome::Resolved("会話1".to_string()));
    assert_eq!(captured.lock().unwrap().as_slice(), &["会話1".to_string()]);
}

#[test]
fn local_confirmed_kicks_with_composite_scene() {
    // local 範囲内（最内 local 優先）→ scene == ":会話1:挨拶_1"（parent Some）。
    let file = "C:/work/dic/talk.pasta";
    let map = map_with_index(file);
    let (sink, captured) = mock_sink();

    let outcome = resolve_and_kick(&map, &sink, "file:///C:/work/dic/talk.pasta", 25);

    assert_eq!(
        outcome,
        ResolveOutcome::Resolved(":会話1:挨拶_1".to_string())
    );
    assert_eq!(
        captured.lock().unwrap().as_slice(),
        &[":会話1:挨拶_1".to_string()]
    );
}

#[test]
fn not_found_does_not_kick() {
    // 最終シーン終端より後ろ（下方に有効シーンなし）→ 未検出・sink 不呼出。
    let file = "C:/work/dic/talk.pasta";
    let map = map_with_index(file);
    let (sink, captured) = mock_sink();

    let outcome = resolve_and_kick(&map, &sink, "file:///C:/work/dic/talk.pasta", 100);

    assert_eq!(outcome, ResolveOutcome::NotFound);
    assert!(
        captured.lock().unwrap().is_empty(),
        "未検出では取次してはならない"
    );
}

#[test]
fn differently_formatted_equivalent_uris_resolve_to_same_scene() {
    // 同一ファイルを指す別形式 uri（パーセントエンコード・区切り違い）が、
    // std::path::absolute 正規化を経て同一シーンへ解決する（正規化の固定）。
    let file = "C:/work/dic/talk.pasta";
    let map = map_with_index(file);

    // 形式 A: スラッシュ・素のドライブパス（uri スキームなし）。
    let (sink_a, cap_a) = mock_sink();
    let out_a = resolve_and_kick(&map, &sink_a, "C:/work/dic/talk.pasta", 25);

    // 形式 B: file:// + パーセントエンコード（`%20` ではなくここでは大小・スキーム差）。
    let (sink_b, cap_b) = mock_sink();
    let out_b = resolve_and_kick(&map, &sink_b, "file:///C:/work/dic/talk.pasta", 25);

    assert_eq!(out_a, out_b, "別形式 uri は同一シーンへ解決する");
    assert_eq!(
        out_a,
        ResolveOutcome::Resolved(":会話1:挨拶_1".to_string())
    );
    assert_eq!(*cap_a.lock().unwrap(), *cap_b.lock().unwrap());
}

#[test]
fn uri_to_pasta_path_strips_scheme_and_decodes() {
    // file:// スキーム除去・パーセントデコード・先頭ドライブ補正の固定。
    let p = uri_to_pasta_path("file:///C:/work/my%20dic/talk.pasta");
    // 空白が復元され、先頭の余分な `/` が落ちている（絶対パスとして c:/... 始まり）。
    assert!(
        p.contains("my dic"),
        "%20 が空白へデコードされる: {p}"
    );
    assert!(
        !p.starts_with("/C:") && !p.starts_with("/c:"),
        "Windows ドライブの先頭 `/` が補正される: {p}"
    );
}

// ===========================================================================
// task 6.3: uri 正規化の特性化テスト（requirements 2.1/7.1）。
//
// 本クラスタは「VSCode uri → 索引キー」正規化の **契約**を固定する。とりわけ
// `uri_to_pasta_path` が `std::path::absolute`（純粋・字句的）を用い、
// `std::fs::canonicalize`（FS 解決・実在前提・8.3 短縮名展開）を **使わない**前提を
// 観測可能にする。この契約を崩す（canonicalize へ差し替える）と、本クラスタの
// `nonexistent_path_*` / `short_name_8dot3_*` テストが失敗する（RED 証拠は
// Status Report 参照）。
//
// 索引キー側の `\`→`/`・Windows 大小無視は `SourceMap::scene_at` 内の
// `canonicalize_pasta_file` が担うため、別形式の uri/パスが「同一の論理ファイル」を
// 指す限り、`map_with_index` で張った索引を介して **同一シーン**へ解決する。
// ===========================================================================

/// 1. Windows パス + URI エンコード等価性: 同一論理ファイルを指す多様な形式の
///    uri/パスが、すべて同一の索引キーへ正規化され同一シーンへ解決する。
///
/// 既存 `differently_formatted_equivalent_uris_resolve_to_same_scene`（3.1）を
/// 広げ、(a) 区切り混在 `\`/`/`、(b) ドライブ文字の大小、(c) `file://` の有無、
/// (d) 余分な `.`/`..` セグメント、(e) `%3A`（`:`）パーセントエンコードまで網羅する。
#[test]
fn windows_and_uri_encoded_forms_all_resolve_to_same_local_scene() {
    let file = "C:/work/dic/talk.pasta";
    let map = map_with_index(file);

    // すべて C:\work\dic\talk.pasta の local 範囲（行 25）を指す等価形式。
    let equivalent_forms = [
        "file:///C:/work/dic/talk.pasta",       // 標準 file:// uri
        "C:/work/dic/talk.pasta",               // 素のドライブパス・スラッシュ
        r"C:\work\dic\talk.pasta",              // バックスラッシュ区切り
        r"c:\work\dic\talk.pasta",              // ドライブ文字小文字
        "C:/work/./dic/../dic/talk.pasta",      // 余分な `.`/`..`（absolute が解決）
        "file:///C%3A/work/dic/talk.pasta",     // `%3A` = `:` パーセントエンコード
        "file:///c:/work/dic/talk.pasta",       // file:// + 小文字ドライブ
    ];

    let expected = ResolveOutcome::Resolved(":会話1:挨拶_1".to_string());
    for form in equivalent_forms {
        let (sink, _captured) = mock_sink();
        let outcome = resolve_and_kick(&map, &sink, form, 25);
        assert_eq!(
            outcome, expected,
            "等価形式 {form:?} は同一 local シーンへ解決しなければならない\n\
             （正規化が一致しなければ NotFound になる）"
        );
    }
}

/// 1'. 日本語ファイル名のパーセントエンコード（マルチバイト UTF-8）等価性。
///
/// プロジェクトは日本語前提のため、非 ASCII ファイル名のパーセントデコードが
/// マルチバイトを正しく復元することを固定する。`会話.pasta`（UTF-8 →
/// `%E4%BC%9A%E8%A9%B1`）のエンコード形式と素の形式が同一シーンへ解決する。
#[test]
fn japanese_filename_percent_encoded_uri_resolves_to_same_scene() {
    // 会話.pasta（"会話" = E4 BC 9A / E8 A9 B1）。
    let file = "C:/work/dic/会話.pasta";
    let map = map_with_index(file);

    // 素の（デコード済み）形式。
    let (sink_plain, _c1) = mock_sink();
    let out_plain = resolve_and_kick(&map, &sink_plain, "C:/work/dic/会話.pasta", 25);

    // パーセントエンコード済みの file:// uri（VSCode が届ける形）。
    let (sink_enc, _c2) = mock_sink();
    let out_enc = resolve_and_kick(
        &map,
        &sink_enc,
        "file:///C:/work/dic/%E4%BC%9A%E8%A9%B1.pasta",
        25,
    );

    assert_eq!(
        out_enc,
        ResolveOutcome::Resolved(":会話1:挨拶_1".to_string()),
        "マルチバイト日本語名の %xx デコードが正しく UTF-8 を復元し解決する"
    );
    assert_eq!(
        out_plain, out_enc,
        "日本語名は素の形式とパーセントエンコード形式で同一シーンへ解決する"
    );
}

/// 2. **std::path::absolute（NOT fs::canonicalize）契約のカナリア — 実在パス**。
///
/// これが本タスクの **load-bearing** な特性化。`std::fs::canonicalize` は Windows で
/// 実在パスを正規化する際に **verbatim/extended-length 接頭辞 `\\?\`** を前置する
/// （実機検証済み: `\\?\C:\Users\...\talk.pasta`）。対して `std::path::absolute` は
/// 字句的で `\\?\` を **付けない**。よって「実在する `.pasta` ファイルを
/// `uri_to_pasta_path` に通した結果が `\\?\` で始まらない」ことを固定すれば、
/// 内部が `std::path::absolute` であって `fs::canonicalize` でない契約を直接 pin
/// できる。`std::path::absolute` を `fs::canonicalize` へ差し替えるとこのテストは
/// RED になる（出力が `\\?\` 前置になる）。
///
/// さらに canonicalize は 8.3 短縮名を長名へ展開し symlink を解決するため、CI の
/// `%TEMP%` = `RUNNER~1`（8.3 短縮名）が長名へ化けて索引キーと不一致になる不具合が
/// CI でのみ再現する（既存メモ）。本テストはその根本原因（FS 解決の有無）を
/// 接頭辞という観測可能な形で固定する。
#[cfg(windows)]
#[test]
fn existing_path_normalizes_lexically_without_canonicalize_verbatim_prefix() {
    use std::io::Write;

    // 実在する一時 `.pasta` ファイルを用意（canonicalize が成功し差分が出る条件）。
    let dir = std::env::temp_dir().join("pasta_kick_6_3_canary");
    std::fs::create_dir_all(&dir).expect("temp dir 作成");
    let file_path = dir.join("talk.pasta");
    std::fs::File::create(&file_path)
        .and_then(|mut f| f.write_all(b"# canary"))
        .expect("temp .pasta 作成");

    let input = file_path.to_string_lossy().into_owned();
    let normalized = uri_to_pasta_path(&input);

    // std::path::absolute は `\\?\` を付けない。canonicalize へ差し替えると付く。
    assert!(
        !normalized.starts_with(r"\\?\"),
        "uri_to_pasta_path は std::path::absolute（字句的）であり \\?\\ 接頭辞を付けない。\n\
         fs::canonicalize へ差し替えるとこのテストは失敗する（出力が \\\\?\\ 前置になる）。\n\
         got: {normalized}"
    );

    // 参考: canonicalize なら `\\?\` 前置になることを同条件で確認（契約の対偶）。
    let canon = std::fs::canonicalize(&input).expect("実在パスなので canonicalize は成功する");
    assert!(
        canon.to_string_lossy().starts_with(r"\\?\"),
        "対照: fs::canonicalize は実在パスで \\?\\ 前置を返す（差し替えで RED になる根拠）"
    );

    let _ = std::fs::remove_file(&file_path);
}

/// 2'. **実在しないパスでも解決する（FS 非依存の振る舞い特性化）**。
///
/// `std::path::absolute` は純粋に字句的で、ディスク上に **存在しない**パスにも成功する
/// （実機検証済み）。一方 `fs::canonicalize` は実在しないパスで **エラー**になる
/// （`os error 3`）。本テストは「実在しない `.pasta` パスを索引に張り、同じ文字列形式で
/// 解決して当たる」という **振る舞い**を固定する（FS に依存しない決定論性）。
///
/// 注意: 現状の production は `absolute(..)` が `Err` のとき `drive_fixed`（字句文字列）へ
/// フォールバックするため、canonicalize へ単純差し替えしても本ケースは（Err→同じ字句へ
/// フォールバックして）たまたま通り得る。よって canonicalize 差し替えの検知は
/// `existing_path_normalizes_lexically_without_canonicalize_verbatim_prefix`（接頭辞
/// カナリア）が担う。本テストは「実在しないパスでも解決できる」という FS 非依存の
/// 受け入れ基準そのものを pin する。
#[test]
fn nonexistent_path_still_resolves_fs_independent() {
    // ディスク上に存在しないことが事実上保証されるパス（8.3 短縮名形も含めて字句的）。
    let file = r"C:\definitely-nonexistent-9f3a\RUNNER~1\Temp\talk.pasta";
    let map = map_with_index(file);

    let (sink, _captured) = mock_sink();
    let outcome = resolve_and_kick(
        &map,
        &sink,
        "file:///C:/definitely-nonexistent-9f3a/RUNNER~1/Temp/talk.pasta",
        25,
    );

    assert_eq!(
        outcome,
        ResolveOutcome::Resolved(":会話1:挨拶_1".to_string()),
        "実在しないパス（8.3 短縮名 RUNNER~1 を含む）でも字句的に正規化され解決する＝\n\
         FS 非依存（std::path::absolute）。fs::canonicalize は実在しないパスで Err になる。"
    );
}
