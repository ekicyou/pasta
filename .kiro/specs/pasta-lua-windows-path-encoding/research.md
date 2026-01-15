# Research & Design Decisions

## Summary
- **Feature**: `pasta-lua-windows-path-encoding`
- **Discovery Scope**: Extension（既存システムへの軽微な修正と新モジュール追加）
- **Key Findings**:
  1. Lua文字列は任意のバイト列を格納可能（`Lua::create_string(&[u8])`）
  2. `package.path`にANSIバイト列を設定すれば標準サーチャーが正しく動作
  3. 既存の`@pasta_search`モジュール登録パターンを`@enc`実装に流用可能

## Research Log

### Lua文字列とエンコーディング
- **Context**: Windows環境でLuaの`require`が日本語パスで失敗する問題の調査
- **Sources Consulted**: 
  - mlua 0.11 API: `Lua::create_string(&[u8])`
  - Lua 5.4 Reference: 文字列は任意のバイト列を格納可能
- **Findings**:
  - Lua文字列は内部的に長さ付きバイト列（UTF-8制約なし）
  - `Lua::create_string`は`&[u8]`を受け入れ、任意のバイト列をLua文字列化
  - `LuaString::as_bytes()`で元のバイト列を取得可能
- **Implications**: 
  - `package.path`にANSIエンコードされたバイト列を直接設定可能
  - カスタムサーチャー実装は不要（当初想定より大幅に簡略化）

### Windows ANSI API
- **Context**: Windows環境でのファイルパス解決方式の確認
- **Sources Consulted**: 
  - Windows API: `fopen`はANSIコードページを使用
  - 既存実装: `crates/pasta_lua/src/encoding/windows.rs`
- **Findings**:
  - `windows-sys` 0.59の`MultiByteToWideChar`/`WideCharToMultiByte`で変換実装済み
  - `Encoding::ANSI.to_bytes()`でUTF-8→ANSI変換
  - `Encoding::ANSI.to_string()`でANSI→UTF-8変換
- **Implications**: 
  - 新規依存追加不要、既存APIで実装可能
  - `to_ansi_bytes`は`Encoding::ANSI.to_bytes`の薄いラッパー

### 既存モジュール登録パターン
- **Context**: `@enc`モジュールの実装方式決定
- **Sources Consulted**: 
  - `crates/pasta_lua/src/search/mod.rs`（`@pasta_search`登録）
  - `crates/pasta_lua/src/runtime/mod.rs`（モジュール登録フロー）
- **Findings**:
  - `package.loaded`に登録することで`require "@name"`で取得可能
  - `lua.create_table()`でモジュールテーブル作成
  - `lua.create_function()`で関数登録
- **Implications**: 
  - `@enc`も同じパターンで実装（`runtime/enc.rs`新規作成）
  - UserDataではなくTableとして公開（関数のみ提供）

### LoaderContext調査
- **Context**: `package.path`設定の既存実装確認
- **Sources Consulted**: 
  - `crates/pasta_lua/src/loader/context.rs`
- **Findings**:
  - `generate_package_path()`は`String`を返す（UTF-8）
  - `setup_package_path()`で`package.set("path", string)`
  - Windowsでは`\\?\`プレフィックス除去済み
- **Implications**: 
  - `generate_package_path_bytes()`を追加し`Vec<u8>`を返す
  - `setup_package_path`で`lua.create_string(&bytes)`使用

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: 最小修正 | 既存構造を維持、4ファイル修正 | 低リスク、既存パターン準拠 | なし | **推奨** |
| B: encoding層リファクタ | encoding全体を再設計 | 整理された構造 | 追加工数、影響範囲拡大 | 過剰設計 |

## Design Decisions

### Decision: `to_ansi_bytes`関数の追加
- **Context**: ANSIバイト列生成関数が必要
- **Alternatives Considered**:
  1. `path_to_lua`を修正して`Vec<u8>`返却
  2. `to_ansi_bytes`新規追加、`path_to_lua`廃止
- **Selected Approach**: Option 2（新規追加、廃止）
- **Rationale**: 
  - `path_to_lua`にバグあり（`String::from_utf8_lossy`）
  - 本番コード未使用（テストのみ）
  - 汎用的な命名（`@enc.to_ansi`と統一）
- **Trade-offs**: テスト削除が必要だが、テスト自体も不完全
- **Follow-up**: 廃止後のテストカバレッジ確認

### Decision: `@enc`モジュールの関数セット
- **Context**: Lua側に公開するエンコーディング関数の範囲
- **Alternatives Considered**:
  1. 必須のみ: `to_ansi`, `to_utf8`
  2. 完全セット: + `to_oem`, `from_oem`
- **Selected Approach**: Option 1（必須のみ）
- **Rationale**: 
  - 要件スコープ最小化
  - OEMはコンソールI/O用（現状不要）
  - 将来追加可能な設計
- **Trade-offs**: OEM必要時に追加作業
- **Follow-up**: OEM需要発生時に拡張

### Decision: runtime/enc.rs配置
- **Context**: `@enc`モジュールのファイル配置
- **Alternatives Considered**:
  1. `src/runtime/enc.rs` - runtimeサブモジュール
  2. `src/enc/mod.rs` - 独立モジュール
- **Selected Approach**: Option 1（runtimeサブモジュール）
- **Rationale**: 
  - `@pasta_search`は独立だがUserData使用
  - `@enc`は純粋な関数テーブルでシンプル
  - runtime初期化で登録するためruntime配下が自然
- **Trade-offs**: なし
- **Follow-up**: なし

## Risks & Mitigations
- **Risk 1**: Windows環境でのテスト不足 → CI設定で明示的にWindowsテスト追加
- **Risk 2**: ANSIコードページ変換エラー → Lua標準エラーパターンで返却（`nil, err`）
- **Risk 3**: 既存テスト回帰 → 既存テストは変更なしで合格を確認

## References
- mlua 0.11 Documentation: `Lua::create_string`, `LuaString::as_bytes`
- Windows API: `MultiByteToWideChar`, `WideCharToMultiByte`
- Lua 5.4 Reference: String型の定義（任意バイト列）
- 既存実装: `crates/pasta_lua/src/search/mod.rs`, `encoding/mod.rs`
