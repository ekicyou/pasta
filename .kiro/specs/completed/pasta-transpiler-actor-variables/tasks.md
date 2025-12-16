# Implementation Tasks: pasta-transpiler-actor-variables

## Task Breakdown

### 1. モジュールレベル use 文生成の実装

- [ ] 1.1 (P) グローバルラベルモジュール生成時に3つのuse文を出力する機能を実装
  - `transpile_global_label()` の修正（L276-278付近）
  - モジュールヘッダー直後、`__start__` 関数の前に3つのuse文を順次出力
  - 出力順序: `use pasta::*;`, `use pasta_stdlib::*;`, `use crate::actors::*;`
  - 既存の1つのuse文（`use pasta_stdlib::*;`）を3つに拡張
  - インデント: 4スペース（モジュールレベル）
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

### 2. アクター変数参照の生成

- [ ] 2.1 (P) Statement::Speech処理でアクター代入を変数参照形式に変更
  - `transpile_statement_to_writer()` の Statement::Speech ブランチを修正（L353）
  - 現在: `writeln!(writer, "        ctx.actor = \"{}\";", speaker)`
  - 修正後: `writeln!(writer, "        ctx.actor = {};", speaker)`
  - ダブルクォートを削除し、識別子として出力
  - 日本語識別子をそのまま使用（サニタイズ最小限）
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [ ] 2.2 (P) Actor イベント生成をオブジェクトフィールドアクセスに変更
  - 同じく Statement::Speech ブランチ内を修正（L355）
  - 現在: `writeln!(writer, "        yield Actor(\"{}\");", speaker)`
  - 修正後: `writeln!(writer, "        yield Actor(ctx.actor.name);")`
  - `ctx.actor.name` でアクター名フィールドを参照
  - speaker変数を使用せず、ctx.actorから取得
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

### 3. pasta関数の短縮形呼び出し

- [ ] 3.1 (P) Call文生成でパス修飾を削除
  - `transpile_statement_to_writer()` の Statement::Call ブランチを修正（L375付近）
  - 現在: `crate::pasta::call` をフルパスで出力
  - 修正後: `call` のみを出力（`use pasta::*;` により解決）
  - 引数構造は変更なし: `call(ctx, "ラベル名", #{}, [])`
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [ ] 3.2 (P) Jump文生成でパス修飾を削除
  - `transpile_statement_to_writer()` の Statement::Jump ブランチを修正（L390付近）
  - 現在: `crate::pasta::jump` をフルパスで出力
  - 修正後: `jump` のみを出力（`use pasta::*;` により解決）
  - 引数構造は変更なし: `jump(ctx, "ラベル名", #{})`
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

### 4. テストフィクスチャの更新

- [ ] 4.1 test-project/main.rnのアクター定義をactorsモジュール構造に移行
  - `crates/pasta/tests/fixtures/test-project/main.rn` を編集
  - トップレベルの `pub const さくら/うにゅう/ななこ` を削除
  - `pub mod actors { ... }` ブロックを作成し、その中にアクター定義を移動
  - アクター構造: `pub const さくら = #{ name: "さくら", id: "sakura" };`
  - 全アクター（さくら、うにゅう、ななこ）を同じ構造で定義
  - _Requirements: 3.1_

- [ ] 4.2 comprehensive_control_flow参照実装の更新とトランスパイル出力再生成
  - `crates/pasta/tests/fixtures/comprehensive_control_flow.rn` のアクター定義を確認（必要に応じてactorsモジュール構造に移行）
  - トランスパイラー修正後、`comprehensive_control_flow.transpiled.rn` を再生成
  - 再生成方法: テストまたはトランスパイラー実行で出力ファイルを更新
  - 新しい出力形式（use文3つ、変数参照、短縮形）を反映
  - _Requirements: 5.1, 5.2_

### 5. 単体テストの追加と更新

- [ ] 5.1 (P) アクター変数参照の単体テスト作成
  - テスト名: `test_transpile_actor_variable_reference`
  - Statement::Speech をトランスパイルし、出力に `ctx.actor = さくら;` が含まれることを検証
  - ダブルクォートが含まれないことを確認（正規表現または文字列マッチ）
  - 日本語識別子の正常な出力を確認
  - _Requirements: 1.1, 1.2, 1.4, 5.1, 5.6_

- [ ] 5.2 (P) Actor イベント生成の単体テスト作成
  - テスト名: `test_transpile_actor_event_with_field_access`
  - Statement::Speech をトランスパイルし、出力に `yield Actor(ctx.actor.name);` が含まれることを検証
  - `ctx.actor.name` フィールドアクセスの存在を確認
  - _Requirements: 2.1, 2.2, 5.2_

- [ ] 5.3 (P) モジュールレベルuse文生成の単体テスト作成
  - テスト名: `test_transpile_use_statements`
  - グローバルラベルをトランスパイルし、3つのuse文が順序通りに出力されることを検証
  - 順序確認: `use pasta::*;` → `use pasta_stdlib::*;` → `use crate::actors::*;`
  - モジュールヘッダー直後、関数定義前に配置されていることを確認
  - _Requirements: 3.1, 3.2, 5.3_

- [ ] 5.4 (P) Call/Jump短縮形の単体テスト作成
  - テスト名: `test_transpile_call_shorthand`, `test_transpile_jump_shorthand`
  - Statement::Call/Jump をトランスパイルし、`crate::pasta::` プレフィックスが含まれないことを検証
  - 短縮形 `call(ctx, ...)` / `jump(ctx, ...)` の出力を確認
  - _Requirements: 4.1, 4.2, 4.3, 5.4_

### 6. 統合テストの追加

- [ ] 6.1 Rune VMコンパイル検証テストの作成
  - テスト名: `test_rune_vm_compile_actor_variables`
  - トランスパイル済みRuneコード + main.rn（actorsモジュール定義）を統合
  - Rune VMでコンパイルが成功することを確認
  - アクター変数が正しく解決されることを検証
  - コンパイルエラーが発生しないことを確認
  - _Requirements: 5.5_

- [ ] 6.2 actorsモジュールインポート検証テストの作成
  - テスト名: `test_actor_module_import`
  - `use crate::actors::*;` を含むトランスパイル出力でRune VMコンパイル
  - 複数のアクター（さくら、うにゅう、ななこ）がすべて参照可能であることを検証
  - ワイルドカードインポートが正常に機能することを確認
  - _Requirements: 3.2_

- [ ] 6.3 ローカルラベルのuse文継承検証テストの作成
  - テスト名: `test_local_label_inherits_use_statements`
  - グローバルラベル + ローカルラベルを含むスクリプトをトランスパイル
  - ローカルラベル関数内でpasta関数とactorsが使用可能であることを検証
  - ローカルラベル関数にuse文が重複生成されていないことを確認
  - _Requirements: 3.3, 3.4_

### 7. E2Eテストの追加と既存テストの検証

- [ ] 7.1 単純なスクリプトの完全フローテスト作成
  - テスト名: `test_e2e_simple_script`
  - Pastaスクリプトをパース → トランスパイル → Rune VMコンパイル → 実行
  - ScriptEvent::ChangeSpeaker { name: "さくら" } の発生を確認
  - ScriptEvent::Talk の発生を確認
  - アクター名が正しく抽出されることを検証
  - _Requirements: 5.1, 5.2_

- [ ] 7.2 複数アクター会話のE2Eテスト作成
  - テスト名: `test_e2e_multi_actor_conversation`
  - さくら、うにゅう、ななこが登場するスクリプトで完全フロー実行
  - 各アクター変更イベントが正しい順序で発生することを検証
  - ctx.actorが各Speech statementで更新されることを確認
  - _Requirements: 1.1, 2.1_

- [ ] 7.3 Call/Jump短縮形のE2Eテスト作成
  - テスト名: `test_e2e_call_and_jump`
  - Call/Jumpを含むスクリプトで完全フロー実行
  - ラベル遷移が正常に動作することを検証
  - `call()` / `jump()` が `use pasta::*;` により解決されることを確認
  - _Requirements: 4.1, 4.2_

- [ ] 7.4 既存テストスイートの実行と検証
  - 全テスト（268テスト）を実行し、全てパスすることを確認
  - 修正による回帰がないことを検証
  - Pass 2出力の不変性を確認（pasta-transpiler-pass2-output仕様と整合）
  - _Requirements: 5.7_

## Coverage Summary

- **Total Tasks**: 7 major tasks, 18 sub-tasks
- **Requirements Coverage**: All 5 requirements (1.1-5.7) mapped
- **Parallel Execution**: 13 tasks marked with (P)
- **Average Task Size**: 1-2 hours per sub-task

## Quality Validation

- ✅ All requirements mapped to tasks
- ✅ Task dependencies verified (テスト実装はコード修正後に実行可能)
- ✅ Testing tasks included (単体・統合・E2Eテスト網羅)
- ✅ Natural language descriptions focused on capabilities
- ✅ Sequential numbering maintained

## Next Steps

タスクをレビューし、承認後に実装フェーズに進んでください。実装は以下のコマンドで開始できます：

- 特定タスク実行: `/kiro-spec-impl pasta-transpiler-actor-variables 1.1`
- 複数タスク実行: `/kiro-spec-impl pasta-transpiler-actor-variables 1.1,2.1,2.2`
- 全タスク実行: `/kiro-spec-impl pasta-transpiler-actor-variables` (推奨: タスク間でコンテキストをクリア)

**重要**: 実装開始前、またはタスク切り替え時は会話履歴をクリアしてコンテキストを解放することを推奨します。
