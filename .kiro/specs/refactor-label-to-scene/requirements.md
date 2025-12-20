# Requirements Document

## Project Description (Input)
用語「ラベル」を、「シーン」に変更するためのリファクタリング。現在、ラベルという用語で呼び出し先を管理しているが、より演劇用語に寄せるため「シーン」という用語に変更したい。この仕様による機能変更はないが、文法ドキュメントなどの大規模なリファクタリングが発生する。リファクタリング対象の用語検索・サポート及び実施をお願いしたい。

## Introduction

本仕様は、Pastaプロジェクト全体における用語「ラベル（Label）」を「シーン（Scene）」に統一するためのリファクタリングを定義します。この変更は演劇的なメタファーとの整合性を高め、DSLの直感性を向上させることを目的とします。

**機能変更なし**: 本リファクタリングは純粋な用語変更であり、動作・API・文法のセマンティクスに変更はありません。

---

## Requirements

### Requirement 1: ドキュメント用語統一

**Objective:** 開発者・ユーザーとして、ドキュメント全体で「ラベル」が「シーン」に統一されていることで、演劇的メタファーが一貫して適用され、DSLの理解が容易になる。

#### Acceptance Criteria

1.1. The Pasta Documentation shall 「グローバルラベル」を「グローバルシーン」に置換する（GRAMMAR.md, SPECIFICATION.md, README.md, examples/）

1.2. The Pasta Documentation shall 「ローカルラベル」を「ローカルシーン」に置換する（GRAMMAR.md, SPECIFICATION.md, examples/）

1.3. The Pasta Documentation shall 「ラベル定義」を「シーン定義」に置換する（すべての関連ドキュメント）

1.4. The Pasta Documentation shall 「ラベル呼び出し」を「シーン呼び出し」に置換する（すべての関連ドキュメント）

1.5. The Pasta Documentation shall 「ラベル前方一致」を「シーン前方一致」に置換する（steering/, specs/）

1.6. The Pasta Documentation shall 「重複ラベル」を「重複シーン」に置換する（すべての関連ドキュメント）

---

### Requirement 2: ステアリング用語統一

**Objective:** AI開発支援として、ステアリングドキュメントで用語が統一されていることで、AIが正確なコンテキストを取得できる。

#### Acceptance Criteria

2.1. The Steering Documents shall `.kiro/steering/grammar.md`内の「ラベル」関連用語をすべて「シーン」に置換する

2.2. The Steering Documents shall `.kiro/steering/product.md`内の「ラベル」関連用語をすべて「シーン」に置換する

2.3. The Steering Documents shall `.kiro/steering/structure.md`内の「ラベル」関連用語をすべて「シーン」に置換する

2.4. The Steering Documents shall `.kiro/steering/tech.md`内の「ラベル」関連用語をすべて「シーン」に置換する

---

### Requirement 3: ソースコード識別子リファクタリング（Rust）

**Objective:** 開発者として、Rustソースコード内の識別子が「Scene」に統一されていることで、コードの意図が明確になる。

#### Acceptance Criteria

3.1. The Pasta Transpiler shall `LabelRegistry` を `SceneRegistry` にリネームする

3.2. The Pasta Transpiler shall `LabelInfo` を `SceneInfo` にリネームする

3.3. The Pasta Transpiler shall `label_registry.rs` を `scene_registry.rs` にリネームする

3.4. The Pasta Runtime shall `LabelTable` を `SceneTable` にリネームする

3.5. The Pasta Runtime shall `labels.rs` を `scene.rs` にリネームする

3.6. The Pasta Parser shall `LabelDef` を `SceneDef` にリネームする

3.7. The Pasta Parser shall `LabelScope` を `SceneScope` にリネームする

3.8. The Pasta Stdlib shall `select_label_to_id` を `select_scene_to_id` にリネームする

3.9. The Pasta Transpiler shall `label_selector` を `scene_selector` にリネームする（生成Runeコード内）

3.10. The Pasta Engine shall `execute_label` を `execute_scene` にリネームする（存在する場合）

---

### Requirement 4: ソースコード変数名・コメントリファクタリング

**Objective:** 開発者として、変数名・関数引数・コメント内でも「scene」が使用されていることで、コード全体の一貫性が保たれる。

#### Acceptance Criteria

4.1. The Pasta Source Code shall ローカル変数名 `label` を `scene` に置換する（文脈に応じて）

4.2. The Pasta Source Code shall 関数引数名 `label` を `scene` に置換する

4.3. The Pasta Source Code shall ドキュメントコメント内の「label」を「scene」に置換する

4.4. The Pasta Source Code shall `label_counters` を `scene_counters` に置換する

4.5. The Pasta Source Code shall `local_label` を `local_scene` に置換する

4.6. The Pasta Source Code shall `global_label` を `global_scene` に置換する

---

### Requirement 5: テストコード用語統一

**Objective:** 開発者として、テストコード内でも用語が統一されていることで、テストの意図が明確になる。

#### Acceptance Criteria

5.1. The Pasta Tests shall テストファイル名に含まれる `label` を `scene` に置換する

5.2. The Pasta Tests shall テスト関数名に含まれる `label` を `scene` に置換する

5.3. The Pasta Tests shall テスト内のアサーションメッセージで「label」を「scene」に置換する

5.4. The Pasta Tests shall フィクスチャファイル内のコメント・ドキュメントで「label」を「scene」に置換する

---

### Requirement 6: エラーメッセージ用語統一

**Objective:** ユーザーとして、エラーメッセージで「シーン」という用語が使用されていることで、ドキュメントとの整合性が保たれる。

#### Acceptance Criteria

6.1. The Pasta Error Handler shall `LabelNotFound` エラーを `SceneNotFound` にリネームする

6.2. The Pasta Error Handler shall エラーメッセージ内の「Label not found」を「Scene not found」に置換する

6.3. The Pasta Error Handler shall エラーメッセージ内の「ラベル」を「シーン」に置換する（日本語メッセージが存在する場合）

---

### Requirement 7: 生成コード用語統一

**Objective:** ランタイムとして、Runeトランスパイル結果内でも「scene」が使用されていることで、デバッグ時の可読性が向上する。

#### Acceptance Criteria

7.1. When Pastaスクリプトをトランスパイルする, the Pasta Transpiler shall 生成されるRune関数名で `label` の代わりに `scene` を使用する

7.2. The Pasta Transpiler shall `__pasta_trans2__::label_selector` を `__pasta_trans2__::scene_selector` に変更する

7.3. The Pasta Transpiler shall `pasta::call(ctx, label, ...)` の引数名を `scene` に変更する

7.4. The Pasta Transpiler shall 生成されるRuneコード内の全ての変数名・引数名で `label` を `scene` に置換する（`label_fn` → `scene_fn`、`label_id` → `scene_id` 等）

7.5. The Pasta Transpiler shall エラーメッセージ内の「ラベルID」を「シーンID」に置換する

---

### Requirement 8: 仕様ドキュメント更新

**Objective:** 開発者として、過去・進行中の仕様ドキュメント内でも用語が統一されていることで、仕様の一貫性が保たれる。

#### Acceptance Criteria

8.1. The Spec Documents shall `.kiro/specs/completed/` 内の完了仕様（全ファイル）で「label/Label」を「scene/Scene」に置換する

8.2. The Spec Documents shall `.kiro/specs/` 内の進行中仕様で「label/Label」を「scene/Scene」に置換する

8.3. The Spec Documents shall 進行中仕様のディレクトリ名で「label」を「scene」に置換する（例: `pasta-label-continuation` → `pasta-scene-continuation`）

---

## 対象ファイルサマリー（IDE リファクタリング活用）

### Rust識別子リファクタリング対象（IDE Rename機能推奨）

| 現在の識別子 | 新しい識別子 | ファイル |
|-------------|-------------|---------|
| `LabelRegistry` | `SceneRegistry` | `src/transpiler/label_registry.rs` |
| `LabelInfo` | `SceneInfo` | `src/transpiler/label_registry.rs` |
| `LabelDef` | `SceneDef` | `src/parser/ast.rs` |
| `LabelScope` | `SceneScope` | `src/parser/ast.rs` |
| `LabelTable` | `SceneTable` | `src/runtime/labels.rs` |
| `select_label_to_id` | `select_scene_to_id` | `src/stdlib/mod.rs` |
| `label_selector` | `scene_selector` | `src/transpiler/mod.rs` (生成コード) |
| `LabelNotFound` | `SceneNotFound` | `src/error.rs` |

### ファイル名変更対象

| 現在のファイル名 | 新しいファイル名 |
|-----------------|-----------------|
| `src/transpiler/label_registry.rs` | `src/transpiler/scene_registry.rs` |
| `src/runtime/labels.rs` | `src/runtime/scene.rs` |
| `tests/label_id_consistency_test.rs` | `tests/scene_id_consistency_test.rs` |
| `tests/pasta_engine_label_resolution_test.rs` | `tests/pasta_engine_scene_resolution_test.rs` |
| `tests/pasta_transpiler_label_registry_test.rs` | `tests/pasta_transpiler_scene_registry_test.rs` |

### ドキュメント用語置換対象

| ドキュメント | 用語置換数（概算） |
|-------------|------------------|
| `GRAMMAR.md` | 50+ |
| `SPECIFICATION.md` | 30+ |
| `.kiro/steering/*.md` | 30+ |
| `examples/**/*.md` | 10+ |
| `.kiro/specs/**/*.md` | 100+ |

---

## Glossary

### 基本パターン

| 旧用語 | 新用語 | 英語 |
|-------|-------|------|
| ラベル | シーン | Scene |
| グローバルラベル | グローバルシーン | Global Scene |
| ローカルラベル | ローカルシーン | Local Scene |
| ラベル定義 | シーン定義 | Scene Definition |
| ラベル呼び出し | シーン呼び出し | Scene Call |
| ラベル前方一致 | シーン前方一致 | Scene Prefix Match |
| 重複ラベル | 重複シーン | Duplicate Scene |

### 拡張パターン（派生語・複合語）

| 旧用語 | 新用語 | 備考 |
|-------|-------|------|
| ラベルテーブル | シーンテーブル | データ構造 |
| ラベル解決 | シーン解決 | 処理名 |
| ラベル検索 | シーン検索 | 処理名 |
| ラベル名 | シーン名 | 属性 |
| ラベル内 | シーン内 | 位置表現 |
| ラベル関数 | シーン関数 | Rune生成コード |
| ラベル登録 | シーン登録 | 処理名 |
| ラベル管理 | シーン管理 | 処理名 |
| ラベルID | シーンID | 識別子 |
| ラベルマーカー | シーンマーカー | 構文要素 |
| ラベル辞書 | シーン辞書 | データ構造 |
| ラベル配下 | シーン配下 | 位置表現 |
| ラベルブロック | シーンブロック | 構文要素 |
| ラベル行 | シーン行 | 構文要素 |
| ラベルスコープ | シーンスコープ | 概念 |
| ラベル未発見 | シーン未発見 | エラー名 |
| ラベル直後 | シーン直後 | 位置表現 |
| ラベル宣言 | シーン宣言 | 処理名 |
| ラベル実行 | シーン実行 | 処理名 |
| ラベル命名 | シーン命名 | 概念 |
| ラベルジャンプテーブル | シーンジャンプテーブル | データ構造 |
| ラベル継続チェイン | シーン継続チェイン | 機能名 |
| サブラベル | サブシーン | 概念 |
| 親ラベル | 親シーン | 関係性 |
| 同名ラベル | 同名シーン | 概念 |
| 呼び出し先ラベル | 呼び出し先シーン | 関係性 |
| 遷移先ラベル | 遷移先シーン | 関係性 |

### 助詞付きパターン（文脈依存）

文中での助詞結合形（「ラベルから」「ラベルへ」「ラベルに」「ラベルが」「ラベルを」「ラベルの」「ラベルも」「ラベルと」等）も全て「シーン」に置換します。

---

## Out of Scope

- **文法マーカーの変更**: `＊`（グローバルシーン）、`・`（ローカルシーン）のマーカー文字自体は変更しない
- **機能変更**: 動作・API・セマンティクスの変更は行わない
- **外部依存関係**: crates.ioへの公開名やCargo.tomlのパッケージ名は変更しない
- **仕様の選択的除外**: 完了済み・進行中を問わず、全仕様ドキュメントを置換対象とする（歴史的記録はgit履歴で保持）
