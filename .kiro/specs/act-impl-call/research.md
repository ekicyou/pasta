# Research & Design Decisions: act-impl-call

---

## Summary
- **Feature**: `act-impl-call`
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  1. 現在の`ACT_IMPL.call`シグネチャはトランスパイラ出力と不一致（配列 vs 個別引数）
  2. `actor.lua`に多段検索の既存パターンが存在し、流用可能
  3. `SCENE.search`は第3引数追加で既存呼び出しとの互換性を維持可能（Luaの余剰引数無視仕様）

---

## Research Log

### トランスパイラ出力形式の調査

- **Context**: `ACT_IMPL.call`のシグネチャを確定するため、トランスパイラの出力形式を確認
- **Sources Consulted**: 
  - [code_generator.rs](crates/pasta_lua/src/code_generator.rs) L445-448
  - [sample.generated.lua](crates/pasta_lua/tests/fixtures/sample.generated.lua)
- **Findings**:
  - トランスパイラは `act:call(SCENE.__global_name__, "ラベル名", {}, table.unpack(args))` を生成
  - 第1引数: `SCENE.__global_name__` (string) - グローバルシーン名
  - 第2引数: `"ラベル名"` (string) - 検索キー
  - 第3引数: `{}` (table) - 属性（現在は空テーブル）
  - 第4引数以降: 可変長引数
- **Implications**: 新シグネチャ `(self, global_scene_name, key, attrs, ...)` は既存トランスパイラ出力と完全互換

### 既存パターンの調査: actor.lua多段検索

- **Context**: 4段階検索の実装パターンを既存コードから抽出
- **Sources Consulted**:
  - [actor.lua](crates/pasta_lua/scripts/pasta/actor.lua) L180-220 `ACTOR_PROXY_IMPL.word`
- **Findings**:
  - 6レベルの優先順位検索を実装済み
  - Level 1-2: アクター辞書（完全一致→前方一致）
  - Level 3-4: シーン（完全一致→前方一致）
  - Level 5-6: グローバル（完全一致→前方一致）
  - `require("pasta.global")` パターンを使用
- **Implications**: `ACT_IMPL.call`の4段階検索は類似パターンで実装可能

### SCENE.search シグネチャ拡張

- **Context**: 第3引数`attrs`追加の互換性検証
- **Sources Consulted**:
  - [scene.lua](crates/pasta_lua/scripts/pasta/scene.lua) L148
  - Lua 5.4 Reference Manual - Function Arguments
- **Findings**:
  - 現在は2引数シグネチャ: `SCENE.search(name, global_scene_name)`
  - Luaは余剰引数を無視するため、2引数呼び出しは3引数シグネチャと互換
  - `attrs`は将来拡張用に予約、現時点では使用しない
- **Implications**: 単純な第3引数追加で後方互換性を維持可能

---

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| Option A: 直接置き換え | `ACT_IMPL.call`を直接置き換え | 最小変更、既存パターン流用 | なし | **採用** |
| Option B: ヘルパー分離 | `find_handler`を別関数化 | テスト容易性 | オーバーヘッド | 将来検討 |
| Option C: 段階実装 | シグネチャ変更→検索実装 | リスク分散 | 中間状態 | 不採用 |

---

## Design Decisions

### Decision: ACT_IMPL.callシグネチャ

- **Context**: トランスパイラ出力と一致するシグネチャを決定
- **Alternatives Considered**:
  1. 現在の配列形式 `(self, search_result, opts, ...)`
  2. 個別引数形式 `(self, global_scene_name, key, attrs, ...)`
- **Selected Approach**: 個別引数形式
- **Rationale**: トランスパイラ出力と完全一致、現在の実装が誤り
- **Trade-offs**: なし（互換性問題は発生しない）
- **Follow-up**: なし

### Decision: 4段階検索の優先順位

- **Context**: ハンドラー検索の優先順位を確定
- **Alternatives Considered**: 優先順位の入れ替え、レベル追加/削除
- **Selected Approach**: 
  1. `self.current_scene[key]` - シーンローカル
  2. `SCENE.search(key, global_scene_name, attrs)` - スコープ付き検索
  3. `require("pasta.global")[key]` - グローバル関数モジュール
  4. `SCENE.search(key, nil, attrs)` - スコープなし全体検索
- **Rationale**: ローカル優先→スコープ付き→グローバル→フォールバックの自然な階層
- **Trade-offs**: なし
- **Follow-up**: なし

### Decision: SCENE.search第3引数追加

- **Context**: 将来の属性フィルタリング対応
- **Alternatives Considered**:
  1. 単純な第3引数追加
  2. オプショナルテーブル引数方式
- **Selected Approach**: 単純な第3引数追加
- **Rationale**: Lua互換性維持、最小変更、将来拡張可能
- **Trade-offs**: なし
- **Follow-up**: `attrs`の具体的な使用方法は将来仕様で決定

### Decision: pasta_shiori互換実装の同期

- **Context**: テスト用互換実装の更新戦略
- **Alternatives Considered**:
  1. 同時更新（同じPR内）
  2. 別タスク化
- **Selected Approach**: 同時更新
- **Rationale**: CI失敗防止、一貫性保証、効率的
- **Trade-offs**: 作業範囲微増（+1ファイル）
- **Follow-up**: なし

---

## Risks & Mitigations

| リスク | 影響度 | 緩和策 |
|--------|--------|--------|
| シグネチャ変更による予期せぬ互換性問題 | Low | トランスパイラ出力と一致しており、既存コードは動作しない状態 |
| SCENE.searchの第3引数追加による副作用 | Low | Luaの余剰引数無視仕様により既存呼び出しは影響なし |
| pasta_shiori互換実装の同期漏れ | Medium | 同じPR内で両方更新、テスト実行で検証 |

---

## References

- [Lua 5.4 Reference Manual - Functions](https://www.lua.org/manual/5.4/manual.html#3.4.11) - 関数引数の扱い
- [actor.lua](crates/pasta_lua/scripts/pasta/actor.lua) - 多段検索パターンの参考実装
- [scene_search_test.rs](crates/pasta_lua/tests/scene_search_test.rs) - SCENE.search テストケース
