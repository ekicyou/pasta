# Implementation Plan

- [x] 1. デッドコード除去と空ファイル削除
- [x] 1.1 空ファイル res.rs の除去
  - `crates/pasta_shiori/src/util/res.rs` を削除する
  - `crates/pasta_shiori/src/util/mod.rs` から `pub mod res;` 行を除去する
  - `cargo check -p pasta_shiori` がエラーなしで通ること
  - _Requirements: 4.3_
  - _Boundary: util/mod.rs, util/res.rs_

- [x] 1.2 (P) MyError::Others バリアントの除去
  - error.rs から `Others` バリアントと `#[allow(dead_code)]` を除去する
  - `MyError::script_error()` メソッドの使用状況を確認し、未使用なら `#[allow(dead_code)]` を除去するか、使用箇所を特定してコメント付与する
  - `cargo check -p pasta_shiori` がエラーなしで通ること
  - _Requirements: 4.4_
  - _Boundary: error.rs_

- [x] 1.3 (P) parsers/req.rs の `#[cfg(test)]` 化
  - `ShioriRequest` 構造体および関連メソッド群を `#[cfg(test)]` で囲む
  - テストモジュール（`mod tests`）はそのまま維持し、`#[allow(dead_code)]` を除去する
  - `cargo test -p pasta_shiori` で既存テスト（req_1, req_2, req_3）が全パスすること
  - _Requirements: 4.1, 4.2_
  - _Boundary: parsers/req.rs_

- [x] 1.4 (P) hglobal モジュールの `#[allow(dead_code)]` 精査
  - hglobal/mod.rs の7箇所の `#[allow(dead_code)]` を精査し、テスト専用メソッドは `#[cfg(test)]` に置換、FFI境界で必要なものは理由コメントを付与する
  - hglobal/enc.rs のファイルレベル `#![allow(dead_code)]` を精査し、必要なアイテムのみ個別に `#[allow(dead_code)]` を残すか除去する
  - hglobal/windows_api.rs のファイルレベル `#![allow(dead_code)]` を精査し、未使用の定数（MB_COMPOSITE, MB_USEGLYPHCHARS, WC_DISCARDNS, WC_SEPCHARS, WC_DEFAULTCHAR, WC_NO_BEST_FIT_CHARS等）を除去する
  - `cargo check -p pasta_shiori` および `cargo test -p pasta_shiori` がパスすること
  - _Requirements: 4.1, 4.2_
  - _Boundary: hglobal/mod.rs, hglobal/enc.rs, hglobal/windows_api.rs_

- [x] 1.5 (P) lua_request.rs の `#[allow(dead_code)]` 精査
  - `lua_date` 関数の `#[allow(dead_code)]` を検証し、parse_request内での使用を確認してコメントを更新する
  - req_parser.rs の `#[allow(dead_code)]` について Pest 派生マクロの制約としてコメントを付与する
  - `cargo check -p pasta_shiori` がパスすること
  - _Requirements: 4.1, 4.2_
  - _Boundary: lua_request.rs, parsers/req_parser.rs_

- [x] 2. unsafeブロックのSAFETYコメント付与
- [x] 2.1 shiori.rs の unsafe impl Send/Sync にSAFETYコメント追加
  - `unsafe impl Send for PastaShiori` と `unsafe impl Sync for PastaShiori` に、OnceLock + Mutex による保護、シングルスレッドDLLコンテキストでの使用を明記するSAFETYコメントを追加する
  - 既存コメントが簡素な場合は充実化する
  - SAFETYコメントがRustdocスタイル（`// SAFETY:` プレフィックス）で記述されていること
  - _Requirements: 1.1_
  - _Boundary: shiori.rs_

- [x] 2.2 (P) hglobal/mod.rs の unsafe ブロックにSAFETYコメント追加
  - `unsafe impl Send/Sync for ShioriString` に所有権セマンティクス（has_freeフラグ）の根拠を明記する
  - `Drop::drop` 内の `GlobalFree` 呼び出しに二重解放防止メカニズムを明記する
  - `clone_from_slice_impl` 内の `GlobalAlloc + from_raw_parts_mut` に長さの整合性保証を明記する
  - `as_bytes` 内の `from_raw_parts` にcaptureとの対応関係を明記する
  - 各SAFETYコメントが前提条件（precondition）と不変条件（invariant）を記述していること
  - _Requirements: 1.1, 1.2_
  - _Boundary: hglobal/mod.rs_

- [x] 2.3 (P) hglobal/windows_api.rs の unsafe ブロックにSAFETYコメント追加
  - `multi_byte_to_wide_char` 内のunsafeブロックにWindows API呼び出しの前提条件を明記する
  - `wide_char_to_multi_byte` 内のunsafeブロックにWindows API呼び出しの前提条件を明記する
  - 入力バリデーション（空文字列チェック等）が実施済みであることをSAFETYコメント内で参照していること
  - _Requirements: 1.1_
  - _Boundary: hglobal/windows_api.rs_

- [x] 2.4 (P) windows.rs の extern "C" 関数にSAFETYドキュメント追加
  - DllMain, load, unload, request の各関数の `# Safety` ドキュメントセクションにCaller側の前提条件を明記する
  - `#[unsafe(no_mangle)]` の使用理由をコメントに追加する
  - 全4関数にSafetyセクション付きドキュメントコメントが存在すること
  - _Requirements: 1.1, 2.4_
  - _Boundary: windows.rs_

- [x] 3. FFI境界の入力検証強化
- [x] 3.1 windows.rs の load 関数にNULLチェック追加
  - `load` 関数の冒頭で `hdir` がNULLまたは `len` がゼロの場合に `false` を返すガード処理を追加する
  - ガード条件に該当した場合に `tracing::warn!` でログ出力する
  - 既存のSHIORIホストからの正常呼び出しが影響を受けないこと
  - _Requirements: 2.1, 2.4_
  - _Boundary: windows.rs_

- [x] 3.2 (P) windows.rs の request 関数にNULLチェック追加
  - `request` 関数の冒頭で `req` がNULLの場合に `*len = 0` を設定し `ptr::null_mut()` を返すガード処理を追加する
  - ガード条件に該当した場合に `tracing::warn!` でログ出力する
  - 既存のSHIORIホストからの正常呼び出しが影響を受けないこと
  - _Requirements: 2.2, 2.4_
  - _Boundary: windows.rs_

- [x] 3.3 (P) ShioriString の入力検証確認
  - `ShioriString::capture` がNULLハンドルとゼロ長の組み合わせで安全に動作するか確認する
  - `to_ansi_str` と `to_utf8_str` が不正データ（非UTF-8等）に対してパニックしないことを確認する
  - 必要に応じてガード処理を追加する
  - 不正入力でMyError::EncodeAnsi / EncodeUtf8が適切に返されること
  - _Requirements: 2.3_
  - _Boundary: hglobal/mod.rs_

- [x] 4. プロダクションコードのパニック除去
- [x] 4.1 lua_request.rs の parse_key_value 内 unwrap()/panic!() 変換
  - `it.next().unwrap()` を `it.next().ok_or_else(|| ...)?` に変換する（4箇所）
  - `pair.as_str().parse().unwrap()` を `pair.as_str().parse().map_err(|_| ...)?` に変換する（1箇所）
  - `panic!()` を `return Err(MyError::ParseRequest(...))` に変換する（1箇所）
  - 必要に応じて MyError に新しいバリアントを追加するか、既存の ParseRequest を使用する
  - 既存テスト（parse_request呼び出し経路）が全パスすること
  - _Requirements: 3.1, 3.2, 3.3_
  - _Depends: 1.2_
  - _Boundary: lua_request.rs, error.rs_

- [x] 4.2 (P) parsers/req.rs の unwrap()/panic!() 変換
  - ShioriRequest::parse1 内の `pair.as_str().parse().unwrap()` を Result 伝搬に変換する
  - ShioriRequest::parse_key_value 内の `it.next().unwrap()` を Result 伝搬に変換する（4箇所）
  - `panic!()` を `return Err(...)` に変換する
  - `#[cfg(test)]` 内でのみ使用されるため、テスト内のエラー処理は簡潔な形式で可
  - 既存テスト（req_1, req_2, req_3）が全パスすること
  - _Requirements: 3.4_
  - _Depends: 1.3_
  - _Boundary: parsers/req.rs_

- [x] 5. 冗長表現の削減
- [x] 5.1 error.rs の From 実装の簡潔化
  - 冗長な `From` 実装パターンを確認し、可能であれば `thiserror` の `#[from]` 属性で置き換える
  - MyError バリアントの命名・構造が一貫していることを確認する
  - 外部振る舞い（エラーメッセージ、to_shiori_response出力）が変わらないこと
  - _Requirements: 5.2, 5.3_
  - _Boundary: error.rs_

- [x] 5.2 (P) lua_request.rs と req.rs のパースロジック重複確認
  - 両ファイルのparse_key_valueロジックの重複範囲を特定する
  - req.rsが`#[cfg(test)]`化された前提で、共通化可能な部分があれば統合する（ただしテスト専用コードの過度な共通化は避ける）
  - 共通化が不要と判断した場合はその理由をコメントとして残す
  - _Requirements: 5.1, 5.3_
  - _Depends: 1.3, 4.1, 4.2_
  - _Boundary: lua_request.rs, parsers/req.rs_

- [x] 6. 最終検証
- [x] 6.1 全テストパスの確認
  - `cargo test -p pasta_shiori` の全テストがパスすることを確認する
  - `cargo test` （ワークスペース全体）でリグレッションがないことを確認する
  - `cargo clippy -p pasta_shiori` で新しい警告がないことを確認する
  - 全テスト結果がグリーンであること
  - _Requirements: 6.1, 6.2, 6.3_

- [x] 6.2 unsafeブロック最終確認
  - `grep -n "unsafe" crates/pasta_shiori/src/` で全unsafeブロックにSAFETYコメントが付与されていることを確認する
  - `#[allow(dead_code)]` の残存数を確認し、各残存箇所に理由が明記されていることを確認する
  - unsafe使用数が監査前と同数以下であること
  - _Requirements: 1.1, 1.2, 1.3, 4.1, 4.2_
