# Requirements Document

## Introduction

Pasta DSLのコードジェネレーター（`element_gen.rs`）が `Action::SakuraScript` を `act:sakura_script(literal)` として出力するが、以下の3層すべてに問題がある:

1. **トランスパイラ**（`element_gen.rs`）: `act:sakura_script()` はアクター紐付けがない
2. **actランタイム**（`act.lua` / `actor.lua`）: `sakura_script()` メソッドが未定義
3. **sakura_builder**（`sakura_builder.lua`）: `sakura_script` トークンタイプの処理が未定義

これにより、ランタイムで `attempt to call a nil value (method 'sakura_script')` エラーが発生する。

### 根本原因

`act:sakura_script()` というAPI設計が不適切。さくらスクリプト（`\n`, `\w9` 等）はアクション行の中でアクター発話に埋め込まれるインライン要素であり（doc/spec/07-sakura-script.md §7.1）、常に特定のアクターのコンテキストに属する。3層すべてで `sakura_script` をアクター紐付きメソッド/トークンとして正しく扱う必要がある。

### 他のアクション型との比較

| Action型 | 現行のコード生成出力 | アクター紐付き |
|---|---|---|
| `Talk` | `act.{actor}:talk(literal)` | ✅ |
| `WordRef` | `act.{actor}:talk(act.{actor}:word(name))` | ✅ |
| `VarRef` | `act.{actor}:talk(tostring(var))` | ✅ |
| `FnCall` | `act.{actor}:talk(tostring(SCENE:fn(act)))` | ✅ |
| `Escape` | `act.{actor}:talk(literal)` | ✅ |
| **`SakuraScript`** | **`act:sakura_script(literal)`** | **❌ アクター欠落** |

### データフロー全体像（修正後）

```
[トランスパイラ] act.{actor}:sakura_script(literal)
      ↓
[PROXY_IMPL]   sakura_script(self, text) → self.act:sakura_script(self.actor, text)
      ↓
[ACT_IMPL]     sakura_script(self, actor, text) → token { type="sakura_script", actor=actor, text=text }
      ↓
[group_by_actor] sakura_scriptトークンをtalkと同等にアクターグループ化
      ↓
[sakura_builder] sakura_scriptトークンをtalkと同じくtalk_to_script()で処理
```

### 関連ファイル

- `crates/pasta_lua/src/code_gen/element_gen.rs` — 問題1: コード生成のアクター紐付け
- `crates/pasta_lua/scripts/pasta/act.lua` — 問題2: `ACT_IMPL.sakura_script()` 追加 + `group_by_actor` 対応
- `crates/pasta_lua/scripts/pasta/actor.lua` — 問題2: `PROXY_IMPL.sakura_script()` 追加
- `crates/pasta_lua/scripts/pasta/shiori/sakura_builder.lua` — 問題3: `sakura_script` トークン処理

## Requirements

### Requirement 1: トランスパイラのアクター付きコード生成

**Objective:** Pasta DSLユーザーとして、アクション行内のさくらスクリプト（`\n`, `\w9` 等）が正しくアクターに紐付いてLuaコードに変換されること。

#### Acceptance Criteria

1. When `Action::SakuraScript` をコード生成する場合, the トランスパイラー shall `act.{actor}:sakura_script(literal)` 形式のLuaコードを出力する（`actor` は `generate_action()` の引数から取得）
2. The トランスパイラー shall `act:sakura_script()` 形式（アクター無し）の出力を生成しない

### Requirement 2: actランタイムの `sakura_script` メソッド追加

**Objective:** トランスパイラが出力する `act.{actor}:sakura_script(text)` 呼び出しをランタイムが正しく受け入れ、アクター紐付きトークンとして蓄積すること。

#### Acceptance Criteria

1. The `PROXY_IMPL` shall `sakura_script(self, text)` メソッドを持ち、`self.act:sakura_script(self.actor, text)` を呼び出す（`talk()` と同構造）
2. The `ACT_IMPL` shall `sakura_script(self, actor, text)` メソッドを持ち、`{ type = "sakura_script", actor = actor, text = text }` トークンを `self.token` に蓄積する
3. The `group_by_actor()` shall `sakura_script` トークンを `talk` と同等に扱う: アクター切り替え検出に参加し、`type = "actor"` グループ配下に格納する
4. When さくらスクリプトを含むシーンがランタイムで実行される場合, the pasta_lua shall `attempt to call a nil value` エラーを発生させない

### Requirement 3: sakura_builderの `sakura_script` トークン処理

**Objective:** sakura_builderがアクターグループ内の `sakura_script` トークンを `talk` トークンと同じフローで処理し、さくらスクリプトタグを正しく最終出力に含めること。

#### Acceptance Criteria

1. When sakura_builderがアクターグループ内で `inner_type == "sakura_script"` トークンを検出した場合, the builder shall `talk` トークンと同じく `SAKURA_SCRIPT.talk_to_script(actor, inner.text)` を呼び出して出力バッファに追加する
2. The 既存の `raw_script` トークン処理 shall 変更しない（`raw_script` は生スクリプト挿入として引き続き動作）

### Requirement 4: 既存テストの整合性

**Objective:** 開発者として、3層の変更がスナップショットテストおよび統合テストに正しく反映されること。

#### Acceptance Criteria

1. When `Action::SakuraScript` を含むPasta DSLをトランスパイルする場合, the スナップショットテスト shall 新しい `act.{actor}:sakura_script()` 形式の出力を期待値として保持する
2. The `cargo test -p pasta_lua` shall 全テストがパスする
3. The `ACT_IMPL.raw_script()` メソッド shall 既存の動作を維持する
