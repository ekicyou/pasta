# ギャップ分析レポート: persist-spot-position

**最終更新**: 2026-02-06 - 議題1の結論を反映（STORE.actor_spots設計）

## 要件→既存資産マッピング

### 要件 1: コンフィグでアクターのデフォルトspot値を設定

| 技術要素 | 現状 | ギャップ |
|---------|------|---------|
| `pasta.toml`の`[actor]`セクション読み込み | ✅ **実装済み** - `PastaConfig`の`custom_fields`でTOMLの任意セクションを読み込み可能 | なし |
| `@pasta_config` Luaモジュール経由の配信 | ✅ **実装済み** - `register_config_module()`で`custom_fields`をLuaテーブルとして登録 | なし |
| `STORE.actors`への初期化 | ✅ **実装済み** - `store.lua`末尾で`CONFIG.actor`をSTOREに参照共有 | なし |
| アクターメタテーブル設定 | ✅ **実装済み** - `actor.lua`末尾でSTORE由来アクターにACTOR_IMPL設定 | なし |
| テスト | ✅ **実装済み** - `config_actors_initialization_test.rs`に包括的テスト | なし |
| フィクスチャ | ✅ **実装済み** - `with_actor_config/pasta.toml`にspot=0, spot=1の設定例 | なし |

**結論**: 要件 1 は**既に完全に実装済み**。`[actor."さくら"]`に`spot = 0`を記述すると、ランタイムで`STORE.actors["さくら"].spot == 0`として利用可能。

---

### 要件 2: スポット位置の継続保持とトークン出力制御（統合済み）

**議題1の設計決定**:
- `STORE.actor_spots = {}` フィールドを追加してセッション全体でスポット状態を保持
- `store.lua` 初期化時に `CONFIG.actor[name].spot` から初期値を転送
- `sakura_builder.build()` は純粋関数を維持：入力 `(grouped_tokens, config, actor_spots)`、戻り値 `(script, updated_actor_spots)`
- `SHIORI_ACT:build()` が STORE との入出力を仲介

| 技術要素 | 現状 | ギャップ |
|---------|------|---------|
| **STORE.actor_spots フィールド** | ❌ **未実装** | **要追加** |
| store.lua での CONFIG.actor.spot → STORE.actor_spots 転送 | ❌ **未実装** | **要追加** |
| sakura_builder.build() のシグネチャ変更 | ❌ **未実装** | **要変更** - 現在 `(grouped_tokens, config)` → 変更後 `(grouped_tokens, config, actor_spots)` |
| sakura_builder.build() の戻り値拡張 | ❌ **未実装** | **要変更** - 現在 `script` のみ → 変更後 `(script, updated_actor_spots)` |
| SHIORI_ACT:build() での STORE 入出力仲介 | ❌ **未実装** | **要追加** |
| トランスパイラの clear_spot/set_spot 出力制御 | ✅ **実装済み** | なし - `code_generator.rs` L292-302で `!actors.is_empty()` ガード済み |

**結論**: 要件2の核心機能（STORE.actor_spots による状態保持）は未実装。トークン出力制御は既に実装済み。

**結論**: 既存コードは**既に`!actors.is_empty()`ガードの内側**で`clear_spot()`と`set_spot()`をセットで出力している。つまり、アクター行（`％`）がないシーンでは**既に両方ともスキップされている**。

**テスト確認** (`transpiler_integration_test.rs` test_set_spot_empty_actors):
- `set_spot`が含まれないことをテスト済み ✅
- `PASTA.clear_spot()`が含まれないことをテスト済み ✅（但し旧フォーマット`PASTA.clear_spot()`でチェック → `act:clear_spot()`でもチェックすべき）

⚠️ **テストの微修正が必要**: 既存テスト`test_set_spot_empty_actors`は`"PASTA.clear_spot()"`をチェックしているが、現行コードは`"act:clear_spot()"`を出力する。このテストは実質的に常に成功するが、意味あるアサーションに修正すべき。

------

### 要件 3: サンプルゴーストのコンフィグ設定

| 技術要素 | 現状 | ギャップ |
|---------|------|---------|
| サンプルゴーストpasta.toml | ⚠️ `[actor]`セクション未設定 | **Missing** |
| テンプレート (pasta.toml.template) | ⚠️ `[actor]`セクション未設定 | **Missing** |
| アクター名（女の子、男の子） | ✅ `scripts.rs`で確認済み | なし |

**結論**: サンプルゴーストの`pasta.toml`と`pasta.toml.template`に`[actor]`セクションの追加が必要。

---

## 実装アプローチ

### 採用アプローチ: 既存コンポーネント拡張 ✅

**対象ファイル**:

1. **`store.lua`** — `STORE.actor_spots = {}` フィールド追加 + CONFIG.actor からの初期化ロジック
2. **`sakura_builder.lua`** — `build()` シグネチャ変更：入力に `actor_spots` 追加、戻り値に `updated_actor_spots` 追加
3. **`SHIORI_ACT` (shiori/act.lua)** — `build()` 呼び出し時に STORE.actor_spots の入出力仲介
4. **サンプルゴースト `pasta.toml`** — `[actor]`セクション追加
5. **`pasta.toml.template`** — `[actor]`セクション追加

**トレードオフ**:
- ✅ 純粋関数 `sakura_builder.build()` を維持
- ✅ 状態管理を STORE に集約（既存パターン踏襲）
- ✅ CONFIG → STORE のデータフロー一貫性維持
- ✅ テスト容易性向上（build() への入力を制御可能）

---

## 工数とリスク

| 項目 | 評価 | 根拠 |
|------|------|------|
| **工数** | **M（3〜5日）** | store.lua, sakura_builder.lua, shiori/act.lua の変更 + テスト追加 |
| **リスク** | **Low** | 技術的未知数なし、既存テスト網で回帰防止、純粋関数維持でテスト容易 |

---

## 設計フェーズへの推奨事項

### 主要決定事項（議題1で確定）
1. **STORE.actor_spots による状態管理**: セッション全体でスポット状態を保持
2. **store.lua での初期化**: CONFIG.actor[name].spot → STORE.actor_spots[name]
3. **sakura_builder.build() の純粋関数化**: 入力として actor_spots を受け取り、更新後の値を返す
4. **SHIORI_ACT:build() での仲介**: STORE との入出力を管理

### 重要な発見事項

- **要件1は100%実装済み**: `[actor."さくら"]` → `STORE.actors["さくら"]` のパイプライン完成
- **トークン出力制御は実装済み**: `code_generator.rs`のガード条件で、actorsが空のシーンではclear_spot/set_spotともにスキップ
- **要件2の実装方針確定**: STORE.actor_spots + sakura_builder.build() のシグネチャ変更
- **要件3（旧4）**: サンプルゴーストへのactor設定追加
