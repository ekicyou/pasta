# Gap Analysis: pasta_lua_design_refactor

## 1. 現状調査

### 1.1 ディレクトリ構成

```
crates/pasta_lua/
├── src/                          # Rust側（トランスパイラ）
│   ├── lib.rs
│   ├── transpiler.rs             # メイントランスパイラ
│   ├── code_generator.rs         # Luaコード生成（875行）
│   ├── context.rs
│   ├── config.rs
│   ├── error.rs
│   ├── normalize.rs
│   └── string_literalizer.rs
├── scripts/                      # Lua側（ランタイム）
│   ├── pasta/
│   │   ├── init.lua              # メインモジュール（12行）
│   │   ├── ctx.lua               # 環境コンテキスト（68行）
│   │   ├── act.lua               # アクション（52行）
│   │   ├── actor.lua             # アクター（68行）
│   │   ├── areka/init.lua        # 空ファイル
│   │   └── shiori/init.lua       # 空ファイル
│   ├── ct.lua
│   ├── hello.lua
│   └── README.md
├── scriptlibs/                   # 外部ライブラリ
│   └── lua_test/
└── tests/
    ├── fixtures/sample.expected.lua
    └── transpiler_integration_test.rs
```

### 1.2 既存コンポーネント分析

#### Rust側（code_generator.rs）が生成するAPI呼び出し

| 生成されるコード                       | 用途             | Lua側実装状態 |
| -------------------------------------- | ---------------- | ------------- |
| `PASTA.create_actor("name")`           | アクター作成     | ✅ 実装済      |
| `PASTA.create_scene("module")`         | シーン作成       | ❌ 未実装      |
| `PASTA.create_session(SCENE, ctx)`     | セッション初期化 | ⚠️ 部分実装    |
| `PASTA.clear_spot(ctx)`                | スポットクリア   | ❌ 未実装      |
| `PASTA.set_spot(ctx, "name", n)`       | スポット設定     | ❌ 未実装      |
| `act.アクター:talk(text)`              | 発話             | ⚠️ 構造不一致  |
| `act.アクター:word("name")`            | 単語参照         | ⚠️ 構造不一致  |
| `act:call("module", "label", {}, ...)` | シーン呼び出し   | ❌ 未実装      |
| `act:word("name")`                     | 変数への単語代入 | ❌ 未実装      |
| `var.変数名`, `save.変数名`            | 変数参照         | ⚠️ 部分実装    |

#### 現在のLua実装の問題点

**pasta/init.lua (12行)**
```lua
local MOD = {}
MOD.create_actor = ACTOR.create_actor
function MOD.create_session(global_scene, ctx) end  -- 空実装
return MOD
```
- `create_scene` 未実装
- `create_session` 空実装
- `clear_spot`, `set_spot` 未実装

**pasta/ctx.lua (68行)**
- `CTX.new(save, actors)` 実装済
- `co_action(scene, ...)` 実装済
- `start_action()`, `yield()`, `end_action()` 実装済
- **問題**: `var`, `save`の初期化パターンが要件と一致

**pasta/act.lua (52行)**
```lua
function IMPL.actor(self, actor) ... end
function IMPL.talk(self, actor, text) ... end
function IMPL.sakura_script(self, text) ... end
function IMPL.yield(self) ... end
function IMPL.end_action(self) ... end
```
- **致命的問題**: Rust生成コードは `act.アクター:talk()` パターンだが、現在は `act:talk(actor, text)` パターン
- `act:call()`, `act:word()` 未実装

**pasta/actor.lua (68行)**
- `create_actor(name)` 実装済（キャッシュ付き）
- `spot` プロパティ実装済
- **問題**: `talk()`, `word()` メソッドが未完成

### 1.3 アーキテクチャパターン

現在のパターン:
- **metatableベースOOP**: `setmetatable(obj, { __index = IMPL })`
- **ファクトリ関数**: `MOD.new(...)` でオブジェクト生成
- **コルーチンベース**: `coroutine.create/yield` でyield型実行

---

## 2. 要件実現可能性分析

### 要件からアセットへのマッピング

| 要件                   | 必要なアセット | ギャップ                | 状態         |
| ---------------------- | -------------- | ----------------------- | ------------ |
| **R1: モジュール構造** | 5モジュール    | sceneモジュール不足     | ⚠️ Missing    |
| **R2: CTX設計**        | ctx.lua        | 部分的に実装済          | ⚠️ Constraint |
| **R3: Act設計**        | act.lua        | APIシグネチャ不一致     | ❌ Missing    |
| **R4: Actor設計**      | actor.lua      | talk/wordメソッド不完全 | ⚠️ Missing    |
| **R5: Scene設計**      | scene.lua      | 完全に未実装            | ❌ Missing    |
| **R6: Spot管理**       | init.lua       | 完全に未実装            | ❌ Missing    |
| **R7: トークン仕様**   | act.lua        | 部分実装                | ⚠️ Constraint |
| **R8: Rust互換性**     | 全モジュール   | 多数の不一致            | ❌ Missing    |
| **R9: 拡張ポイント**   | areka/shiori   | 空ファイルのみ          | ✅ OK         |

### 致命的なギャップ

1. **act.アクター:talk() パターンの未実装**
   - Rust生成: `act.さくら:talk("こんにちは")`
   - 現状: `act:talk(actor, "こんにちは")`
   - **解決策**: actオブジェクトに動的にアクタープロキシを生成

2. **create_scene の未実装**
   - シーンテーブルの登録・管理機構が存在しない
   - **解決策**: scene.lua を新規作成

3. **act:call() の未実装**
   - シーン間呼び出しのルーティング機構がない
   - **解決策**: シーンレジストリとラベル解決を実装

---

## 3. 実装アプローチオプション

### Option A: 既存コンポーネントの拡張

**対象ファイル**:
- `pasta/init.lua`: 不足APIを追加
- `pasta/act.lua`: アクタープロキシパターンを追加
- `pasta/actor.lua`: talk/wordメソッドを完成

**変更内容**:
```lua
-- act.lua: 動的アクタープロキシ
function IMPL:__index(actor_name)
    return ActorProxy.new(self, actor_name)
end
```

**トレードオフ**:
- ✅ 既存構造を維持、変更量が最小
- ✅ 既存テストへの影響が限定的
- ❌ act.luaが肥大化するリスク
- ❌ メタテーブル多重継承が複雑化

### Option B: 新規コンポーネント作成

**新規ファイル**:
- `pasta/scene.lua`: シーン管理モジュール（新規）
- `pasta/actor_proxy.lua`: アクタープロキシ（新規）
- `pasta/word_table.lua`: 単語テーブル（新規）

**責務分離**:
```
pasta/
├── init.lua          # 公開API集約（薄いファサード）
├── ctx.lua           # 環境コンテキスト（既存維持）
├── act.lua           # アクション基底（簡素化）
├── actor.lua         # アクター定義（既存維持）
├── actor_proxy.lua   # act.アクター パターン実装（新規）
├── scene.lua         # シーン管理・ルーティング（新規）
└── word_table.lua    # 単語解決（新規）
```

**トレードオフ**:
- ✅ 責務が明確に分離
- ✅ 各モジュールが単純でテストしやすい
- ❌ ファイル数増加
- ❌ インターフェース設計が必要

### Option C: ハイブリッドアプローチ（推奨）

**フェーズ1: 最小限の互換性確保**
- `pasta/init.lua` に不足API追加（create_scene, clear_spot, set_spot）
- `pasta/act.lua` にメタテーブル拡張（動的アクタープロキシ）

**フェーズ2: 構造整理**
- `pasta/scene.lua` 新規作成（シーンレジストリ）
- 単語解決ロジックの整理

**フェーズ3: リファクタリング**
- 責務分離の最適化
- テストカバレッジ拡充

**トレードオフ**:
- ✅ 段階的に互換性を確保しながら改善
- ✅ 各段階で動作確認可能
- ❌ 計画的な調整が必要
- ❌ 一時的に不整合が発生する可能性

---

## 4. 実装複雑度とリスク

### 工数見積もり: **M (3-7日)**

**根拠**:
- 既存Luaコードベースは小規模（約200行）
- Rust側code_generator.rsが期待するAPIは明確
- fixtures/sample.expected.lua が参照実装として機能
- 新規パターン（動的アクタープロキシ）の導入が必要

### リスク評価: **Medium**

**リスク要因**:
1. **動的アクタープロキシの複雑性**: メタテーブル `__index` での動的生成が正しく動作するか
2. **シーン呼び出しのルーティング**: `act:call("module", "label", ...)` の解決ロジック
3. **コルーチン連携**: 既存のyieldパターンとの整合性

**緩和策**:
- fixtures/sample.expected.lua を統合テストのゴールドスタンダードに
- 単体テスト（lua_test）で各モジュールを個別検証
- transpiler_integration_test.rs との組み合わせテスト

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ: **Option C（ハイブリッド）**

**理由**:
1. Rust側code_generator.rsは既に安定しており、Lua側のみ修正が必要
2. 段階的アプローチにより、各段階で動作確認が可能
3. 最終的な構造は明確だが、一度にすべてを変更するリスクを回避

### 設計フェーズで決定すべき事項

1. **動的アクタープロキシの実装パターン**
   - `__index` メタメソッドのみ vs 明示的プロキシオブジェクト
   - Research Needed: Luaメタテーブルのベストプラクティス

2. **シーンレジストリの設計**
   - グローバルテーブル vs モジュールローカル
   - シーン関数の解決アルゴリズム（`__name_N__` パターン）

3. **単語テーブルの統合**
   - グローバル単語 vs シーンローカル単語の解決順序
   - Research Needed: 現在のRust側WordDefRegistryとの対応

4. **テスト戦略**
   - lua_testフレームワークの活用方法
   - Rust統合テストとの連携

### Research Needed（設計フェーズ持ち越し）

- [ ] Lua 5.3/5.4 メタテーブルの `__index` 動的生成パターン
- [ ] コルーチンとメタテーブルの相互作用
- [ ] mlua（Rust-Lua連携）でのLuaファイル読み込みパス設定

---

## 6. サマリー

| 項目               | 結果                                       |
| ------------------ | ------------------------------------------ |
| **ギャップ規模**   | 中程度（Lua側のみ、約200行の修正/追加）    |
| **推奨アプローチ** | Option C: ハイブリッド（段階的実装）       |
| **工数**           | M (3-7日)                                  |
| **リスク**         | Medium（動的プロキシ、シーンルーティング） |
| **ブロッカー**     | なし（設計フェーズで解決可能）             |
