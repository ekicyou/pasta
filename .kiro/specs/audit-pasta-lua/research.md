# 調査・設計判断記録

## サマリ
- **機能名**: `audit-pasta-lua`
- **ディスカバリ範囲**: Extension（既存システムの内部改善）
- **主要発見事項**:
  - `unsafe` ブロックは4箇所（Lua VM初期化2箇所 + Windows FFI 2箇所）に限定。全て必要最小限の使用。
  - `lua.load().eval()` はハードコードされた `require` 呼び出しが中心で、インジェクションリスクは低い。ユーザースクリプト実行パスは `runtime/mod.rs` の `exec()` に集中。
  - 最大ファイルは transpiler.rs（569行）、element_gen.rs（560行）、loader/mod.rs（535行）、persistence.rs（506行）。

## 調査ログ

### unsafeブロックの分布と安全性

- **コンテキスト**: brief.mdで「2箇所のunsafeブロック」と記載されたが、実際にはWindowsエンコーディングにも存在。
- **調査結果**:
  - `runtime/mod.rs:101` — `Lua::unsafe_new_with(std_lib, ...)`: StdLibパラメータはconfigから取得、デフォルトはALL_SAFE
  - `runtime/enc.rs:146` — `Lua::unsafe_new_with(StdLib::ALL_SAFE, ...)`: テスト用のLua VM初期化
  - `encoding/windows.rs:112, 168` — Windows API (`MultiByteToWideChar`, `WideCharToMultiByte`) 呼び出し
- **含意**: Lua VM初期化の`unsafe`はmlua APIの制約上必須。Windows FFIはバッファ管理が正しいかSAFETYコメント付与が必要。

### lua.load().eval() パスの安全性

- **コンテキスト**: Lua文字列インジェクションリスクの評価
- **調査結果**:
  - `finalize.rs` — ハードコードされた `require('pasta.scene')`, `require('pasta.word')`, `require('pasta')` のみ → リスクなし
  - `runtime/mod.rs:150` — `exec()` メソッドで任意スクリプト実行 → トランスパイラ出力またはローダー経由のファイル内容
  - `runtime/mod.rs:263` — `entry.lua` ファイル読み込み・実行 → ローダーのパス検証に依存
  - テストコード（enc.rs） — テスト内のみ、セキュリティ範囲外
- **含意**: 実行パスは全てトランスパイラ生成コードまたはファイルシステムからの読み込みであり、外部ユーザー入力の直接注入パスは存在しない。ローダーのパス検証の健全性確認が必要。

### コード複雑度分析

- **コンテキスト**: 行数上位ファイルの複雑度ホットスポット
- **調査結果** (上位10ファイル):
  1. transpiler.rs — 569行（マルチフェーズ）
  2. element_gen.rs — 560行（AST要素→Lua変換）
  3. loader/mod.rs — 535行（スクリプト読み込み）
  4. persistence.rs — 506行（データ永続化）
  5. runtime/mod.rs — 443行（VMホスト）
  6. loader/config.rs — 425行（ローダー設定）
  7. tokenizer.rs — 393行（さくらスクリプト）
  8. loader/cache.rs — 387行（キャッシュ管理）
  9. line_breaker.rs — 381行（日本語改行）
  10. wait_inserter.rs — 381行（ウェイト挿入）
- **含意**: 各ファイル400行を目標に削減を試みる。ただし機能密度が高い場合は無理に分割しない。

### Luaスクリプト群の構造

- **コンテキスト**: scripts/, scriptlibs/, pasta_scripts/ の安全性
- **調査結果**:
  - `pasta_scripts/` — 標準ランタイムスクリプト（main.lua, pasta/ 名前空間）
  - `scripts/` — ユーザーカスタムスクリプト（pasta_scripts/より優先）
  - `scriptlibs/` — luacheck, lua_test等の開発ツール
- **含意**: ランタイムスクリプトはグローバル汚染とdangerous function使用を検査。scriptlibsは開発ツールのため検査対象外。

## 設計判断

### 判断: モジュール単位の段階的監査

- **コンテキスト**: 約8,000行を一括監査するか段階的に進めるか
- **選択**: モジュール単位（code_gen → runtime → transpiler → loader → sakura_script → Luaスクリプト）で段階的に監査
- **理由**: 各モジュールは独立性が高く、変更の影響範囲を局所化できる。先にcode_genを安定させることで、transpiler監査時の前提が明確になる。
- **トレードオフ**: 全体最適化の機会を見逃す可能性があるが、リスク管理を優先。

### 判断: 400行ガイドラインの柔軟適用

- **コンテキスト**: brief.mdで「各400行以下」を目標としている
- **選択**: 400行は努力目標とし、機能密度が高い場合は無理に分割しない
- **理由**: 不自然なファイル分割は可読性を損なう。重要なのはファイル分割ではなく、関数レベルの複雑度削減とデッドコード除去。
- **トレードオフ**: 数値目標の達成が不確実になるが、コード品質を優先。

## リスクと緩和策

- リスク1: リファクタリングによる微妙な振る舞い変更 → 既存スナップショットテストで検出、変更前後の出力比較
- リスク2: 性能劣化 → ホットパスの変更は最小限に留め、ベンチマーク実施
- リスク3: unsafe置換によるLuaJIT互換性問題 → mlua APIの制約上、unsafe_new_withは現時点で代替不可。SAFETYコメント付与で対応。

## 参考資料

- [mlua ドキュメント](https://docs.rs/mlua/latest/mlua/) — Lua::unsafe_new_with の安全性要件
- [OWASP Top 10](https://owasp.org/www-project-top-ten/) — Webアプリケーション脆弱性基準（本プロジェクトはデスクトップアプリだが参考として）
- [Rust unsafe ガイドライン](https://doc.rust-lang.org/nomicon/) — SAFETY コメント規約
