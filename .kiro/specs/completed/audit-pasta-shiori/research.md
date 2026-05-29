# Research & Design Decisions

## Summary
- **Feature**: `audit-pasta-shiori`
- **Discovery Scope**: Extension（既存コードベースへの品質改善適用）
- **Key Findings**:
  - unsafeブロックは13箇所、うちHGLOBALメモリ操作が最も危険度が高い
  - lua_request.rsのparse_key_value内に5箇所のプロダクションコードunwrap()が存在
  - req.rsのShioriRequestは完全にデッドコード（lua_request.rsが同等機能をLuaテーブル直接構築で提供）
  - res.rsは空ファイル

## Research Log

### unsafeブロック分類
- **Context**: 全unsafeブロックの安全性リスクを評価する
- **Sources Consulted**: pasta_shiori/src/ 全ファイル、Rustonomicon
- **Findings**:
  - `unsafe impl Send/Sync for PastaShiori`（shiori.rs L48-49）: OnceLock + Mutex保護下で使用。SAFETYコメントは既存だが簡素
  - `unsafe impl Send/Sync for ShioriString`（hglobal/mod.rs L23-24）: HGLOBALハンドルのラッパー。所有権セマンティクスは明確だがコメント不足
  - `unsafe { GlobalFree(...) }`（hglobal/mod.rs L31）: Drop実装。has_freeフラグで二重解放を防止
  - `unsafe { GlobalAlloc + from_raw_parts_mut }`（hglobal/mod.rs L52-56）: メモリ確保と書き込み。長さの整合性がcallerに依存
  - `unsafe { from_raw_parts }`（hglobal/mod.rs L89-92）: as_bytes実装。長さの妥当性がcaptureと対になっている
  - `unsafe { MultiByteToWideChar / WideCharToMultiByte }`（windows_api.rs L93, L144）: Windows API呼び出し。入力バリデーション済み
  - `#[unsafe(no_mangle)] extern "C"`（windows.rs 4関数）: DLLエクスポート。NULLチェックが不十分
- **Implications**: NULLポインタチェックの追加が最優先。SAFETYコメントの体系化が必要

### プロダクションコードのunwrap()
- **Context**: パニックリスクのあるunwrap()を特定する
- **Sources Consulted**: lua_request.rs、req.rs（テストコードを除外）
- **Findings**:
  - lua_request.rs L93: `pair.as_str().parse().unwrap()` — SHIORI2バージョン番号パース
  - lua_request.rs L112: `it.next().unwrap()` — key_value後のキー取得
  - lua_request.rs L119-120: `it.next().unwrap()` × 2 — reference番号とvalue取得
  - lua_request.rs L126: `it.next().unwrap()` — value取得
  - lua_request.rs L134: `panic!()` — 未知のキーRuleへのフォールバック
  - req.rs: parse1, parse_key_valueに同等のunwrap()/panic!()パターン
- **Implications**: Pestパーサーの文法ルールにより実行時には到達しないパスだが、防御的プログラミングとしてResult伝搬に変換すべき

### デッドコード分析
- **Context**: `#[allow(dead_code)]`を持つ全アイテムの使用状況を調査
- **Sources Consulted**: cargo警告、grep検索、コード参照分析
- **Findings**:
  - res.rs: 空ファイル → 除去対象
  - MyError::Others: 使用箇所なし → 除去対象
  - MyError::script_error(): `#[allow(dead_code)]`付き → 使用状況を要確認
  - ShioriRequest構造体全体（req.rs）: req.rsのテストでのみ使用。lua_request.rsが実際のプロダクション実装。テスト専用として`#[cfg(test)]`化するか除去を検討
  - ShioriRequestParser（req_parser.rs）: `#[allow(dead_code)]`付きだがPest由来の自動生成コード → Pest派生マクロの制約上必要
  - hglobal/mod.rs 7箇所: clone_from_str, clone_from_slice_nofree, handle, len, is_empty, value, clone_from_str_nofree → テストで使用されるものとFFI境界に必要なものを区別
  - enc.rs: `#![allow(dead_code)]`ファイルレベル → Encodingモジュール全体の使用状況を精査
  - windows_api.rs: `#![allow(dead_code)]`ファイルレベル → Windows APIラッパー、定数の一部は未使用
  - lua_request.rs L34: `lua_date`関数 → X-Pasta-Timeがない場合のデフォルトパスで使用
- **Implications**: req.rsとlua_request.rsの重複が最大の課題。ShioriRequestはテスト専用に制限可能

## Design Decisions

### Decision: ShioriRequest（req.rs）のテスト専用化
- **Context**: req.rsとlua_request.rsに重複するSHIORIリクエストパースロジックが存在
- **Alternatives Considered**:
  1. req.rsを完全除去し、テストもlua_request.rsベースに移行
  2. req.rsを`#[cfg(test)]`化してテスト専用に制限
  3. 現状維持（重複を許容）
- **Selected Approach**: req.rsをテスト専用化（`#[cfg(test)]`）
- **Rationale**: req.rsのテストはSHIORIプロトコルパーサーの文法検証として有用。ただし本番コードパスでは使用されないため`#[cfg(test)]`で明示する
- **Trade-offs**: テストの可読性は維持。パーサーの二重メンテナンスリスクは`#[cfg(test)]`で軽減

### Decision: NULLポインタチェックの追加方式
- **Context**: FFI関数にNULLポインタが渡された場合の防御
- **Selected Approach**: 各extern "C"関数の冒頭でNULL/ゼロ長チェックを追加
- **Rationale**: FFI境界は信頼できない入力として扱うべき。SHIORIプロトコル上、NULLは異常状態であり早期リターンが適切

### Decision: unwrap()のResult伝搬変換方式
- **Context**: Pestパーサー出力のイテレータ操作におけるunwrap()
- **Selected Approach**: `it.next().ok_or(MyError::ParseRequest(...))?` パターンで統一
- **Rationale**: Pestの文法ルールにより理論上は到達しないが、防御的プログラミングとしてResult伝搬が適切。パフォーマンス影響は無視できるレベル
