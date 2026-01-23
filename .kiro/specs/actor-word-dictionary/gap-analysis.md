# Gap Analysis: actor-word-dictionary

## 分析サマリー

| 項目 | 評価 |
|------|------|
| スコープ | pasta_core（AST/パーサー）+ pasta_lua（トランスパイラ/ランタイム） |
| 複雑度 | **M（3-7日）** - 既存パターンの拡張 + 新規ランタイム機能 |
| リスク | **Medium** - 既存構造への統合が必要、後方互換性への配慮が必須 |
| 推奨アプローチ | **Option C: ハイブリッド** - 既存拡張 + 新規機能 |

---

## 1. 現状調査

### 1.1 関連ファイル・モジュール

#### pasta_core（言語非依存層）

| ファイル | 現状 | 本仕様との関連 |
|----------|------|----------------|
| [grammar.pest](crates/pasta_core/src/parser/grammar.pest) | ✅ 複数値対応済み（`words` ルール） | 構文レベルは対応済み |
| [ast.rs](crates/pasta_core/src/parser/ast.rs) | ⚠️ `ActorScope` に `code_blocks` フィールドなし | コードブロック対応が必要 |
| [mod.rs（parser）](crates/pasta_core/src/parser/mod.rs) | ⚠️ `parse_actor_scope` で `code_block` 未処理 | パーサー拡張が必要 |
| [word_registry.rs](crates/pasta_core/src/registry/word_registry.rs) | ✅ グローバル/ローカル単語登録可能 | アクター単語は未対応 |

#### pasta_lua（Luaバックエンド層）

| ファイル | 現状 | 本仕様との関連 |
|----------|------|----------------|
| [code_generator.rs](crates/pasta_lua/src/code_generator.rs#L106) | ⚠️ 単一値のみ出力（`first_word` のみ使用） | 配列形式出力が必要 |
| [scripts/pasta/word.lua](crates/pasta_lua/scripts/pasta/word.lua) | ✅ グローバル/ローカル単語ビルダーあり | アクター単語未対応 |
| [scripts/pasta/actor.lua](crates/pasta_shiori/tests/support/scripts/pasta/actor.lua) | ⚠️ `word` メソッドは4レベル検索だが不完全 | ランダム選択未実装 |

### 1.2 既存パターン・規約

#### 単語辞書の既存構造

```lua
-- グローバル単語（word.lua）
global_words[key] = { {値1, 値2}, {値3} }  -- entry()呼び出しごとに配列追加

-- シーン単語（scene.lua）
SCENE:create_word("key"):entry("値1", "値2")
```

#### アクター属性の現行出力（code_generator.rs）

```lua
do
    local ACTOR = PASTA.create_actor("さくら")
    ACTOR.通常 = [=[\s[0]]=]  -- 単一値のみ
end
```

#### 目標出力（requirements.md）

```lua
do
    local ACTOR = PASTA.create_actor("さくら")
    ACTOR.通常 = { [=[\s[0]]=], [=[\s[100]]=] }  -- 配列形式
end
```

### 1.3 統合ポイント

1. **AST変換（pasta_core）**: `ActorScope` 構造体に `code_blocks: Vec<CodeBlock>` 追加
2. **パーサー（pasta_core）**: `parse_actor_scope` で `Rule::code_block` を処理
3. **コード生成（pasta_lua）**: `generate_actor` で全 `words` を配列形式で出力
4. **ランタイム（pasta_lua）**: `actor.lua` の `PROXY:word` でランダム選択実装

---

## 2. 要件実現可能性分析

### 要件→アセットマップ

| 要件 | 既存アセット | ギャップ | 種別 |
|------|--------------|----------|------|
| **R1: DSL構文** | `grammar.pest` の `words` ルール | なし（対応済み） | ✅ |
| **R2: Lua配列出力** | `code_generator.rs` の `generate_actor` | 単一値→配列形式変更 | ⚠️ 要修正 |
| **R3: ランダム選択** | `actor.lua` の `PROXY:word` | ランダム選択ロジック未実装 | ❌ Missing |
| **R4: フォールバック検索** | `PROXY:word` の4レベル検索 | 前方一致検索未実装 | ⚠️ 要拡張 |
| **R5: Lua関数定義** | `CodeBlock` AST型 | `ActorScope` に `code_blocks` なし | ❌ Missing |
| **R6: グローバル単語** | `word.lua` の `create_global` | 対応済み | ✅ |
| **R7: 後方互換性** | 既存テスト群 | 配列形式でも単一値動作保証必要 | ⚠️ 要確認 |

### 複雑度シグナル

- **単純なCRUD**: ❌
- **アルゴリズムロジック**: ランダム選択（`math.random`）
- **ワークフロー**: 検索優先順位（6レベル）
- **外部統合**: なし

---

## 3. 実装アプローチオプション

### Option A: 既存コンポーネント拡張

#### 対象ファイル

| ファイル | 変更内容 |
|----------|----------|
| `ast.rs` | `ActorScope` に `code_blocks: Vec<CodeBlock>` 追加 |
| `mod.rs` | `parse_actor_scope` で `Rule::code_block` 処理追加 |
| `code_generator.rs` | `generate_actor` で配列形式出力 |
| `actor.lua` | `PROXY:word` にランダム選択追加 |

#### トレードオフ

- ✅ 既存構造を維持、影響範囲が明確
- ✅ 既存テストをベースに拡張可能
- ❌ `ActorScope` 構造体変更は互換性リスク
- ❌ actor.lua の4レベル検索ロジックが複雑化

### Option B: 新規コンポーネント作成

#### 新規ファイル

| ファイル | 責務 |
|----------|------|
| `actor_word_registry.rs` | アクター単語辞書専用レジストリ |
| `actor_word.lua` | アクター単語検索専用モジュール |

#### トレードオフ

- ✅ 責務分離が明確
- ✅ 既存コードへの影響を最小化
- ❌ ファイル数増加
- ❌ 既存 `WordDefRegistry` との整合性維持が必要

### Option C: ハイブリッドアプローチ（推奨）

#### フェーズ分割

**Phase 1: AST/パーサー拡張（pasta_core）**
- `ActorScope` に `code_blocks` フィールド追加
- `parse_actor_scope` で `code_block` 処理追加

**Phase 2: トランスパイラ修正（pasta_lua）**
- `generate_actor` で配列形式出力
- コードブロックの展開処理追加

**Phase 3: ランタイム拡張（pasta_lua）**
- `actor.lua` の `word` メソッド拡張（**Lua実装**）
- フォールバック検索ロジック（アクター→シーン→グローバル）を**Lua関数として実装**
- 個別検索アルゴリズム（単語辞書の前方一致解決等）は**Rustヘルパー関数**を利用

#### トレードオフ

- ✅ 段階的な実装・テストが可能
- ✅ 既存パターンを活用しつつ必要な拡張を実施
- ✅ 各フェーズで動作確認可能
- ❌ 3フェーズの整合性管理が必要

---

## 4. 工数・リスク評価

### 工数: **M（3-7日）**

| タスク | 工数 |
|--------|------|
| AST/パーサー拡張 | 1日 |
| トランスパイラ修正 | 1-2日 |
| ランタイム実装 | 2-3日 |
| テスト作成 | 1日 |

**根拠**: 既存パターン（シーン単語）を参考に実装可能。新規アーキテクチャ変更なし。

### リスク: **Medium**

| リスク項目 | 影響 | 対策 |
|------------|------|------|
| 後方互換性 | 既存スクリプトの動作変更 | 単一値も配列として処理、ランダム選択で同一動作 |
| AST構造変更 | 下流コンポーネントへの影響 | 段階的テスト、リグレッション確認 |
| ランタイム性能 | ランダム選択のオーバーヘッド | O(1)アクセス（Luaテーブル）で問題なし |

---

## 5. Research Needed（設計フェーズへ持ち越し）

1. **前方一致検索の実装方式**: 現在の `word.lua` には前方一致検索がない。**Lua側でフォールバック検索ロジックを実装し、個別検索（前方一致解決等）はRustヘルパー関数を利用**
2. **コードブロック言語判定**: `actor_scope` 内の `code_block` は常に Lua と仮定するか、言語識別子を見るか？

---

## 6. 推奨事項

### 設計フェーズへの推奨

1. **Option C（ハイブリッド）を採用**: 段階的実装で確実に進行
2. **Phase 1 を最優先**: AST/パーサー変更は他すべての基盤
3. **既存テストを活用**: `transpiler_integration_test.rs` を拡張
4. **Research Needed 項目を設計で解決**: 特に前方一致検索の実装方式

### 設計フェーズでの決定事項

- [ ] 前方一致検索用のRustヘルパー関数設計（Lua→Rust FFI）
- [ ] Lua側フォールバック検索ロジックの詳細設計
- [ ] コードブロック言語: 固定 Lua or 識別子参照?

---

## 改訂履歴

| 日付 | 内容 |
|------|------|
| 2026-01-23 | ギャップ分析初版作成 |
