# ギャップ分析: act-sakura-script-method

## 分析サマリー

- **スコープ**: 3層（トランスパイラ / actランタイム / sakura_builder）にまたがるが、各層の変更は小規模かつ局所的
- **既存パターンの活用**: `talk()` メソッドの実装パターンが3層すべてで完全なテンプレートとなる。`sakura_script()` は `talk()` のほぼコピーで実装可能
- **リスク**: 低。既存アーキテクチャに自然に乗る拡張であり、新パターンの導入や外部依存の追加はない
- **スナップショットへの影響**: 現行テストフィクスチャにはさくらスクリプト使用例がないため、既存スナップショットの破壊はない。新規テスト追加が必要
- **`merge_consecutive_talks` の扱い**: `sakura_script` トークンは `talk` とは結合すべきでない（wait/surface等と同じ分離トークン）。現在の `else` ブロックの動作がそのまま正しい

## 要件–資産マッピング

### Requirement 1: トランスパイラのアクター付きコード生成

| 技術要件 | 既存資産 | ギャップ |
|---|---|---|
| `act.{actor}:sakura_script(literal)` 出力 | `element_gen.rs` L188-191: `Action::SakuraScript` ハンドラー | **修正**: `act:sakura_script()` → `act.{actor}:sakura_script()` に1行変更。`actor` 引数は既に利用可能 |
| スナップショットテスト更新 | `snapshot_test.rs`: 8テスト | **影響なし**: 現行フィクスチャにさくらスクリプト使用例がない |
| さくらスクリプト付きスナップショット追加 | なし | **Missing**: 新規テストケース追加が必要 |

### Requirement 2: actランタイムの `sakura_script` メソッド追加

| 技術要件 | 既存資産 | ギャップ |
|---|---|---|
| `PROXY_IMPL.sakura_script()` | `actor.lua` L108-110: `PROXY_IMPL.talk()` | **Missing**: `talk()` と同構造で追加（3行） |
| `ACT_IMPL.sakura_script()` | `act.lua` L179-183: `ACT_IMPL.talk()` | **Missing**: `talk()` と同構造で追加。トークンtype=`"sakura_script"` |
| `group_by_actor()` でのアクター検出 | `act.lua` L33-47: `talk` 分岐でアクター変更検出 | **修正**: `sakura_script` を `talk` と同等にアクター変更検出に参加させる |
| `merge_consecutive_talks()` での扱い | `act.lua` L62-110: `talk` 結合ロジック | **影響なし**: `sakura_script` は非talkトークンとして `else` 分岐で正しく分離動作する |
| Luaテスト追加 | `act_grouping_test.lua`: 14テスト | **Missing**: `sakura_script` トークンのグループ化テストを追加 |

### Requirement 3: sakura_builderの `sakura_script` トークン処理

| 技術要件 | 既存資産 | ギャップ |
|---|---|---|
| `sakura_script` トークン → `talk_to_script()` | `sakura_builder.lua` L101-103: `talk` トークン処理 | **Missing**: `sakura_script` 用の `elseif` 分岐を1行追加 |
| `raw_script` 既存動作維持 | `sakura_builder.lua` L114: `raw_script` 処理 | **影響なし**: 変更不要 |

### Requirement 4: テスト整合性

| 技術要件 | 既存資産 | ギャップ |
|---|---|---|
| スナップショット更新 | `transpiler/snapshots/` 8個の .snap | **影響なし**(後述) |
| `cargo test -p pasta_lua` パス | 既存テスト全般 | **確認必要**: 変更後にテスト実行 |
| 新規テスト | なし | **Missing**: さくらスクリプト含むトランスパイル＋実行テスト |

## 実装アプローチ評価

### Option A: 既存コンポーネント拡張（推奨）

`sakura_script` を `talk` と同パターンで3層に追加する拡張アプローチ。

**変更対象ファイル**:

| ファイル | 変更種別 | 変更規模 |
|---|---|---|
| `element_gen.rs` L190 | 修正（1行） | format文字列の変更のみ |
| `act.lua` L33付近 | 修正（6行追加） | `group_by_actor()` に `sakura_script` 分岐追加 |
| `act.lua` L188付近 | 追加（8行） | `ACT_IMPL.sakura_script()` メソッド |
| `actor.lua` L110付近 | 追加（5行） | `PROXY_IMPL.sakura_script()` メソッド |
| `sakura_builder.lua` L103付近 | 追加（2行） | `sakura_script` トークンの `elseif` 分岐 |
| `act_grouping_test.lua` | 追加（30行程度） | `sakura_script` テストケース |
| `snapshot_test.rs` | 追加（15行程度） | さくらスクリプト含むスナップショット |

**トレードオフ**:
- ✅ 既存 `talk()` パターンの完全な踏襲 — 認知負荷ゼロ
- ✅ 新規ファイルなし、最小限の差分
- ✅ 各層の変更が独立しており、段階的にテスト可能
- ⚠️ `talk` と `sakura_script` の類似コードが増える（ただし各3-8行程度で許容範囲）

### Option B: `talk()` に統合（却下済み）

前回の要件書で検討されたが、ユーザーにより明確に却下。`sakura_script` は意味的に `talk` とは異なる概念であり、独立メソッド/トークンとして保持すべき。

### Option C: ジェネリックトークンメソッド（過度な抽象化）

`act.{actor}:emit(type, text)` のような汎用メソッドを導入する案。不要な抽象化であり、既存パターンとの乖離が大きい。却下。

## `group_by_actor()` における設計判断

### sakura_script のアクター切り替え検出への参加

**決定: 解釈A — アクター切り替え検出に参加する**

さくらスクリプトは `\s[ID]` 等のサーフェス切り替えなど、アクターに影響するコマンドを発行できる。Pasta DSLでもアクション行（= アクター紐付き行）に記述される要素であるため、`talk` と同等にアクター検出対象とする。

```lua
elseif t == "talk" or t == "sakura_script" then
    local talk_actor = token.actor
    if current_actor_token == nil or talk_actor ~= current_actor then
        -- 新しいグループ開始
    end
    table.insert(current_actor_token.tokens, token)
```
- `sakura_script` トークンがアクター切り替えを引き起こせる
- `talk` より前に `sakura_script` が来ても正しくグループ化される

### merge_consecutive_talks() の扱い

**決定: 結合しない（現行動作を維持）**
- `sakura_script` はさくらスクリプトタグ（`\n`, `\w9` 等）を含む制御文字でありテキストではない
- `surface`, `wait`, `newline` と同じく、連続talkの分離点として機能すべき
- 現在の `else` ブロックがそのまま正しい動作

## 実装の複雑度とリスク

- **工数**: **S（1日以内）** — 既存パターンの踏襲のみ、新規パターンなし
- **リスク**: **低** — 変更は局所的、既存テストに破壊的影響なし、rollbackは各ファイル1箇所のrevertで完了

## 設計フェーズへの推奨事項

1. **アプローチ**: Option A（既存コンポーネント拡張）
2. **`group_by_actor()` 設計（決定済み）**: 解釈A — アクター切り替え検出に参加。理由: さくらスクリプトはアクターに影響するコマンドを発行でき、Pasta DSLのアクション行に記述される要素であるため
3. **`merge_consecutive_talks()` 設計（決定済み）**: 変更不要（sakura_scriptは分離トークン）
4. **テスト戦略**: 
   - Rustスナップショット: さくらスクリプト含む .pasta のスナップショットテスト追加
   - Lua単体テスト: `act_grouping_test.lua` に `sakura_script` テストケース追加
5. **Research Needed**: なし — 全ての変更箇所が特定済みで、既存パターンの踏襲のみ
