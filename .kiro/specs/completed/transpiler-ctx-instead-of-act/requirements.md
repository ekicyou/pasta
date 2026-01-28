# Requirements Document

## イントロダクション

pasta_luaトランスパイラにおいて、ユーザー定義シーン関数呼び出し時の第1引数として`ctx`を生成している箇所があるが、これは`act`オブジェクトとすべきである。この問題により生成されたLuaコードが`ctx`変数未定義エラーでランタイム失敗する。

本仕様では、`code_generator.rs`内のすべての関数呼び出し生成ロジックを修正し、`SCENE.関数(ctx, ...)`ではなく`SCENE.関数(act, ...)`形式で出力されるようにする。

### 影響範囲

| ファイル                                                                                | 問題箇所         | 影響                           |
| --------------------------------------------------------------------------------------- | ---------------- | ------------------------------ |
| [`code_generator.rs`](../../../crates/pasta_lua/src/code_generator.rs)                  | L533, L595, L663 | 関数呼び出し生成               |
| [`sample.generated.lua`](../../../crates/pasta_lua/tests/fixtures/sample.generated.lua) | L87              | テストフィクスチャ（要再生成） |

---

## Requirements

### Requirement 1: トークアクション内の関数呼び出し引数修正

**Objective:** As a **トランスパイラ開発者**, I want **アクター発言内の関数呼び出しが正しい第1引数`act`で生成される**, so that **Luaランタイムで正常に実行できる**

#### Acceptance Criteria

1. When `Action::FnCall`がトークアクション内で検出される, the LuaCodeGenerator shall 第1引数として`act`を生成する
2. When 生成されたLuaコードが実行される, the 関数呼び出しは `act`オブジェクトにアクセスできる
3. The LuaCodeGenerator shall 生成パターン`SCENE.関数名(act, 引数...)`を出力する

**検証方法:**
- [x] [`code_generator.rs`](../../../crates/pasta_lua/src/code_generator.rs#L533)の`Action::FnCall`ブロックで`ctx`を`act`に置換
- [x] トランスパイラテストが`SCENE.関数(act, ...)`形式を出力することを確認
- [x] `sample.generated.lua`再生成後、ランタイムエラーが解消されることを確認

---

### Requirement 2: 式評価内の関数呼び出し引数修正（直接出力）

**Objective:** As a **トランスパイラ開発者**, I want **変数代入式や演算式内の関数呼び出しが正しい第1引数`act`で生成される**, so that **複雑な式でも正常に評価できる**

#### Acceptance Criteria

1. When `Expr::FnCall`が`generate_expr()`で処理される, the LuaCodeGenerator shall 第1引数として`act`を生成する
2. When 生成された式が評価される, the 関数呼び出しは`act`オブジェクトにアクセスできる
3. The LuaCodeGenerator shall 生成パターン`SCENE.関数名(act, 引数...)`を出力する

**検証方法:**
- [x] [`code_generator.rs`](../../../crates/pasta_lua/src/code_generator.rs#L595)の`Expr::FnCall`ブロックで`ctx`を`act`に置換
- [x] `save.変数 = SCENE.関数(act, value)`形式が生成されることを確認
- [x] 二項演算内の関数呼び出しでもランタイムエラーが発生しないことを確認

---

### Requirement 3: 式評価内の関数呼び出し引数修正（バッファ出力）

**Objective:** As a **トランスパイラ開発者**, I want **バッファ経由で式を生成する際も正しい第1引数`act`が使用される**, so that **すべての式評価パスで一貫性が保たれる**

#### Acceptance Criteria

1. When `Expr::FnCall`が`generate_expr_to_buffer()`で処理される, the LuaCodeGenerator shall 第1引数として`act`を生成する
2. The LuaCodeGenerator shall バッファ出力でも直接出力と同じ`SCENE.関数名(act, 引数...)`パターンを使用する
3. While 複数の式評価パスが存在する, the LuaCodeGenerator shall すべてのパスで同一の引数規則を適用する

**検証方法:**
- [x] [`code_generator.rs`](../../../crates/pasta_lua/src/code_generator.rs#L663)の`generate_expr_to_buffer`内`Expr::FnCall`ブロックで`ctx`を`act`に置換
- [x] バッファ経由と直接出力で生成コードが一致することを確認
- [x] 既存のトランスパイラ統合テストが全て通過することを確認

---

### Requirement 4: テストフィクスチャの更新

**Objective:** As a **品質保証担当**, I want **修正後のトランスパイラで生成されたフィクスチャが期待値として使用される**, so that **リグレッション防止が機能する**

#### Acceptance Criteria

1. When トランスパイラの修正が完了する, the 開発者 shall [`sample.generated.lua`](../../../crates/pasta_lua/tests/fixtures/sample.generated.lua)を再生成する
2. When テストが実行される, the テストランナー shall 新しいフィクスチャを期待値として使用する
3. The 新しいフィクスチャ shall `SCENE.関数(ctx, ...)`ではなく`SCENE.関数(act, ...)`を含む

**検証方法:**
- [x] `sample.generated.lua`の87行目が`SCENE.関数(act, 2 + 1)`に変更されていることを確認
- [x] 関連する`.expected.lua`ファイルも同様に更新されていることを確認
- [x] `cargo test --package pasta_lua`が全て成功することを確認

---

### Requirement 5: ドキュメント整合性の確認と更新

**Objective:** As a **プロジェクト保守担当**, I want **関連ドキュメントがコード変更を反映している**, so that **将来の開発者が混乱しない**

#### Acceptance Criteria

1. The 実装者 shall [`SOUL.md`](../../../SOUL.md)のコアバリュー・設計原則との整合性を確認する
2. The 実装者 shall [`SPECIFICATION.md`](../../../SPECIFICATION.md)の言語仕様更新が必要かを検証する（該当する場合）
3. The 実装者 shall [`GRAMMAR.md`](../../../GRAMMAR.md)の文法リファレンス同期を確認する（該当する場合）
4. The 実装者 shall [`TEST_COVERAGE.md`](../../../TEST_COVERAGE.md)に新規テストのマッピングを追加する
5. The 実装者 shall [`crates/pasta_lua/README.md`](../../../crates/pasta_lua/README.md)のAPI変更を反映する（該当する場合）
6. Where ステアリング関連領域が影響を受ける, the 実装者 shall [`steering/*`](../../../.kiro/steering/)の該当ファイルを更新する

**検証方法:**
- [x] ドキュメント更新チェックリストを完了する（SOUL.md, SPECIFICATION.md, GRAMMAR.md, TEST_COVERAGE.md - すべて変更不要と確認）
- [x] コミットメッセージに`fix:`プレフィックスを含める
- [x] レビュー時にドキュメント変更が確認される

---

## 関連仕様

- **lua-api-implementation-investigation**: Lua API実装状況調査（本仕様の問題発見元）
- **Phase 0完了基準**: トランスパイラの品質向上はPhase 0 DoD達成に貢献
