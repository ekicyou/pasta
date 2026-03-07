# ギャップ分析: dola-cue-pasta-dsl-extension (v2)

> 分析日: 2026-03-02  
> 対象: requirements.md v2（9 要件）  
> 分析手法: gap-analysis.md フレームワーク準拠

---

## 1. 現状の資産調査

### 1.1 pasta_dsl クレート（文法 + パーサー + AST）

| 資産       | パス                                                     | 備考                                                               |
| ---------- | -------------------------------------------------------- | ------------------------------------------------------------------ |
| PEG 文法   | `vendors/pasta/crates/pasta_dsl/src/parser/grammar.pest` | 236 行、pest 2.x                                                   |
| AST 型定義 | `vendors/pasta/crates/pasta_dsl/src/parser/ast/`         | `mod.rs`, `scene.rs`, `action.rs`, `span.rs`                       |
| パーサー   | `vendors/pasta/crates/pasta_dsl/src/parser/`             | `mod.rs`, `parse_scene.rs`, `parse_action.rs`, `parse_elements.rs` |
| 部分パース | `vendors/pasta/crates/pasta_dsl/src/partial.rs`          | 3 段階フォールバック、`infer_rule_from_line()`                     |
| 公開 API   | `vendors/pasta/crates/pasta_dsl/src/lib.rs`              | `parse_str`, `parse_file`, `parse_str_partial`                     |
| 依存       | `Cargo.toml`                                             | pest, pest_derive, thiserror のみ（バックエンド非依存）            |

**既存ラインマーカー**（全角 / 半角両対応）:

| マーカー   | 用途                | 行ルール                                   |
| ---------- | ------------------- | ------------------------------------------ |
| `#` / `＃` | コメント            | `or_comment_eol`                           |
| `&` / `＆` | 属性                | `file_attr_line`, `global_scene_attr_line` |
| `*` / `＊` | グローバルシーン    | `global_scene_line`                        |
| `・` / `-` | ローカルシーン      | `local_scene_line`                         |
| `@` / `＠` | 単語参照 / 関数呼出 | `file_word_line`, `word_ref`, `fn_call`    |
| `$` / `＄` | 変数                | `var_set_line`                             |
| `>` / `＞` | シーン呼出          | `call_scene_line`                          |
| `%` / `％` | アクター定義        | `actor_line`, `scene_actors_line`          |
| `:` / `：` | KV 区切り / 継続行  | `action_line`, `continue_action_line`      |
| `\`        | Sakura Script       | `sakura_script`                            |

**`!` / `！` は未使用** — キューコマンド行の新マーカーとして空きが確認済み。

### 1.2 pasta_dsl AST 拡張ポイント

| 拡張候補                 | 型               | 変更影響                                                                                                                           |
| ------------------------ | ---------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `LocalSceneItem` enum    | 新バリアント追加 | ローカルシーン内の要素。`VarSet`, `CallScene`, `ActionLine`, `ContinueAction` の 4 バリアント → キューコマンド行バリアント追加可能 |
| `Action` enum            | 新バリアント追加 | アクション行内の要素（6 バリアント）。エイリアス展開後の新 Action は不要な可能性あり                                               |
| `FileItem` enum          | 新バリアント追加 | ファイルスコープ要素（4 バリアント）。キューシートモードはシーンスコープなので変更不要                                             |
| `GlobalSceneScope.attrs` | 既存利用         | `&type:cuesheet` の格納先として既に機能する（セマンティクス判定のみ必要）                                                          |
| `Attr { key, value }`    | 既存利用         | `key: "type"`, `value: AttrString("cuesheet")` として自然にパース済み                                                              |

### 1.3 dola CueSheet データモデル

| 型                                   | パス                              | 備考                                                                       |
| ------------------------------------ | --------------------------------- | -------------------------------------------------------------------------- |
| `CueSheet(Vec<Cue>)`                 | `crates/dola/src/cue/sheet.rs`    | Serde 対応、`new()` でソート                                               |
| `Cue { actor, start_time, payload }` | `crates/dola/src/cue/command.rs`  | 全フィールド public                                                        |
| `CuePayload`                         | 同上                              | `Command(CueCommand)` / `Barrier(BarrierKind)` / `Routing(RoutingCommand)` |
| `CueCommand`                         | 同上                              | 6 バリアント: `Text`, `Clear`, `Emote`, `Choice`, `EntityRef`, `Custom`    |
| `BarrierKind`                        | 同上                              | 3 バリアント: `WaitForInput`, `WaitForChoice`, `Timeout`                   |
| `RoutingCommand`                     | 同上                              | 3 バリアント: `RouteAdd`, `RouteSwitch`, `RouteRemove`                     |
| `TimedSchedule<T>`                   | `crates/dola/src/cue/schedule.rs` | tick/ready 2 フェーズ API                                                  |
| `compile_sheet()`                    | `crates/dola/src/cue/sheet.rs`    | `CueSheet → Vec<CompiledCue>`                                              |

### 1.4 wintf ECS 統合層

| 型 / システム                 | パス                                    | 備考                                               |
| ----------------------------- | --------------------------------------- | -------------------------------------------------- |
| `CueQueue`                    | `crates/wintf/src/ecs/cue/queue.rs`     | `TimedSchedule<CueCommand>` ラッパー（459 行）     |
| `CueSheetTracker`             | `crates/wintf/src/ecs/cue/tracker.rs`   | マルチアクター同期                                 |
| `EntityRegistry`              | `crates/wintf/src/ecs/cue/registry.rs`  | `HashMap<EntityKey, Entity>`                       |
| `dispatch_cue_sheet_internal` | `crates/wintf/src/ecs/cue/dispatch.rs`  | Routing 即実行、Command/Barrier をブロードキャスト |
| `PendingCueSheet`             | `crates/wintf/src/ecs/cue/component.rs` | ECS 投入トリガー                                   |

### 1.5 アーキテクチャ上の制約

- **pest 文法はコンパイル時固定**: `#[grammar = "parser/grammar.pest"]` — 文法変更はクレート再コンパイルが必要
- **SakuraScript は生文字列保持**: `Action::SakuraScript { script: String }` — 構造解析なし
- **pasta_dsl は外部リポジトリ**: `vendors/pasta/` に submodule/vendored — 変更の commit 管理が二重
- **partial.rs の `infer_rule_from_line()`**: ラインマーカー → Rule 推論テーブル。`!` 追加時に更新必須

---

## 2. 要件実現性分析

### 2.1 要件 → 資産マッピング

| 要件                          | 必要な技術要素                                  | 既存資産                                                                              | ギャップ                                                                                     |
| ----------------------------- | ----------------------------------------------- | ------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| **Req 1: モード識別**         | シーン属性 `&type:cuesheet` の判定              | `GlobalSceneScope.attrs` に `Attr` として自然にパース済み                             | **ギャップなし** — セマンティクスレイヤーでの判定ロジックのみ必要                            |
| **Req 2: 暗黙キーフレーム**   | アクション行終了時点の時刻自動算出              | `ActionLine { actor, actions, span }` — 時刻情報なし                                  | **Missing** — 時刻算出ロジック（「アクション行の長さ」定義）は完全に新規                     |
| **Req 3: `!` コマンド行**     | 新ラインマーカー + 新 AST ノード + パーサー拡張 | `!` / `！` は未使用マーカー。`LocalSceneItem` に新バリアント追加可能                  | **Missing** — PEG ルール、AST 型、パーサーロジックすべて新規                                 |
| **Req 4: エイリアス定義**     | `@alias = Command(args)` 構文 + スコープ管理    | `@name` → `Action::WordRef`, `@name(args)` → `Action::FnCall`                         | **Missing** — 代入形式（`=`）は既存文法に存在しない。新しい行種別が必要                      |
| **Req 5: Say/Emote/Clear**    | アクション行 → CueCommand マッピング            | `ActionLine`, `ContinueAction`, `Action::WordRef`                                     | **部分的** — AST は利用可能だがセマンティクス変換（IR 層）は新規                             |
| **Req 6: Routing 自動生成**   | ActorKey 追跡 + RouteAdd/Switch/Remove 判定     | `SceneActorItem { name, number }`, `RoutingCommand` 型、`EntityRegistry`              | **Missing** — 自動生成ロジックは新規。既存型は再利用可能                                     |
| **Req 7: 後方互換性**         | モード切替によるパーサー分岐                    | `Attr` 値判定は下流処理                                                               | **Constraint** — PEG 文法に `!` ルールを追加しても非 cuesheet シーンでは発火しない設計が必要 |
| **Req 8: エラーハンドリング** | 行番号・カラム番号付きエラー                    | `Span { start_line, start_col, ... }`, `ParseError`, `PartialParseError`              | **部分的** — Span 基盤は再利用可能。キーフレーム検証（名前重複・未宣言）は新規               |
| **Req 9: 成果物**             | `cue.pasta` + `design.md`                       | 旧 `cue.pasta` と旧 `design.md` は **v1 形式（`\cue_*` + `[timestamp]`）で obsolete** | **Must Replace** — v2 形式（`!` コマンド行 + 暗黙キーフレーム + エイリアス）で全面書き換え   |

### 2.2 重要な不整合の発見

#### 🔴 Critical: BarrierKind 命名の不一致

**要件 Req 3.3 の記述**:
> Barrier 指定: dola `BarrierKind`（All / Any / Explicit）に対応する進行停止点

**実際の dola コード** (`crates/dola/src/cue/command.rs`):
```rust
pub enum BarrierKind {
    WaitForInput { timeout: Option<f64> },
    WaitForChoice { timeout: Option<f64> },
    Timeout { duration: f64 },
}
```

**`All / Any / Explicit` というバリアントは dola コードに存在しない。** これは要件の誤記または過去の設計案の残存。正しくは `WaitForInput / WaitForChoice / Timeout` であり、要件の修正が必要。

#### ~~🟡 Warning: 暗黙キーフレームの「終了時点」が未定義~~（**解決済み**）

**決定**: パーサーは行の出現順序と構造情報のみを出力する。各コマンドの所要時間は **Duration Resolver トレイト**（外部注入インターフェース）が決定し、CueSheet 構築時に `Cue.start_time` を算出する。dola 内で所要時間は確定しない。→ Req 2 AC 5-6 に反映済み。

#### 🟡 Warning: `@alias = Command(args)` が既存文法と衝突リスク

現行の `@` 系構文:
- `@name` → `word_ref`（ランダムワード辞書参照）
- `@name(args)` → `fn_call`（関数呼出）
- ファイルスコープ `@name：word1, word2` → `file_word_line`（辞書定義）

エイリアス定義 `@alias = Command(args)` は **`=` を使用する新形式** であり、文法レベルでは `=` / `＝` をセパレータとする新しい行種別が必要。`$var = expr` の変数代入行と構造的に類似しており、設計の参考になる。

---

## 3. 実装アプローチ選択肢

### 3.1 アプローチ A: pasta_dsl クレート内で完結

**概要**: grammar.pest に `!` 行ルールを追加し、AST 型を拡張、pasta_dsl 内で完全にパースする。CueSheet への変換は下流（pasta_core / areka）が担当。

**変更対象**:

| ファイル          | 変更内容                                                               |
| ----------------- | ---------------------------------------------------------------------- |
| `grammar.pest`    | `cue_command_line` ルール（`!` マーカー）、`alias_def_line` ルール追加 |
| `ast/scene.rs`    | `LocalSceneItem` に `CueCommandLine`, `AliasDef` バリアント追加        |
| `ast/action.rs`   | `CueCommandKind` enum（Keyframe/KeyframeRef/Barrier/Clear）新規追加    |
| `parse_scene.rs`  | cuesheet モード判定 + 新行種別のパースディスパッチ                     |
| `parse_action.rs` | `!` 行 / エイリアス定義行のパースロジック                              |
| `partial.rs`      | `infer_rule_from_line()` に `!` / `！` → `Rule::cue_command_line` 追加 |

**トレードオフ**:
- ✅ pasta_dsl の構文拡張として自然な配置
- ✅ `Span` 付きエラーが自動的に得られる
- ✅ 部分パース（LSP 対応）にストレートに組込み可能
- ❌ pasta_dsl への変更は外部リポジトリ commit が必要（二重管理）
- ❌ 現行 pasta_dsl 利用者への影響（AST 型の breaking change）
- ❌ CueSheet セマンティクス（時刻算出、ルーティング生成）はどのみち別層

### 3.2 アプローチ B: 新規ブリッジクレート `pasta-cue`

**概要**: pasta_dsl の AST をそのまま受け取り、CueSheet モードのシーンを dola `CueSheet` に変換するブリッジクレートを新設。文法変更は最小限。

**構成**:

| クレート           | 責務                                                                                   |
| ------------------ | -------------------------------------------------------------------------------------- |
| `pasta_dsl`        | `!` 行を `Action::SakuraScript` 類似の生文字列として保持（文法のみ追加、AST 変更最小） |
| `pasta-cue` (新規) | `PastaFile` → `CueSheet` 変換。キーフレーム算出、エイリアス解決、ルーティング生成      |
| `areka`            | `pasta-cue` を依存に追加して CueSheet を取得                                           |

**トレードオフ**:
- ✅ pasta_dsl の AST 破壊変更を最小化
- ✅ dola 型との依存が自然（bridge が dola + pasta_dsl を依存）
- ✅ 責務分離が明確
- ❌ 新クレート管理のオーバーヘッド
- ❌ `!` 行の構文エラーを pasta_dsl 側で詳細に検出しにくい
- ❌ LSP 対応（pasta_lsp）との統合が複雑化

### 3.3 アプローチ C: ハイブリッド（推奨検討）

**概要**: pasta_dsl に `!` マーカーの **文法 + 構造化 AST** を追加し（アプローチ A の文法層）、セマンティクス変換（キーフレーム→時刻、ルーティング生成、エイリアス展開）は areka / pasta_core 側の IR 変換層で実装。

**フェーズ分割**:

| フェーズ                | 対象                | 内容                                                                         |
| ----------------------- | ------------------- | ---------------------------------------------------------------------------- |
| **Phase 1: 文法 + AST** | pasta_dsl           | `!` 行ルール、`LocalSceneItem::CueCommand`、エイリアス定義行                 |
| **Phase 2: IR 変換**    | pasta_core or areka | `GlobalSceneScope` → `CueSheet` 変換器（キーフレーム算出、ルーティング生成） |
| **Phase 3: 統合**       | areka               | pasta_core 出力 → dola `CueSheet` → wintf `PendingCueSheet` パイプライン     |

**トレードオフ**:
- ✅ PEG 文法の構文検証とエラー報告が pasta_dsl で完結
- ✅ セマンティクス層は dola 型に直接依存できる
- ✅ LSP 対応（pasta_lsp）で `!` 行の補完・エラー表示が可能
- ✅ 段階的実装に最適
- ❌ pasta_dsl への AST 追加は breaking change（`LocalSceneItem` enum）
- ❌ Phase 2 の配置先決定に追加の設計判断が必要

---

## 4. 設計フェーズ向けリサーチ項目

### 4.1 解決必須（Design Blocker）

| ID      | 項目                                 | 詳細                                                                                                                                                                                                   |
| ------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **R-1** | ~~暗黙キーフレーム所要時間~~（**解決済み**） | パーサーは行の出現順序と構造のみ出力し、所要時間は算出しない。Duration Resolver トレイト（外部注入）が CueSheet 構築時に `Cue.start_time` を算出する。dola 内で所要時間は確定しない。→ Req 2 AC 5-6 に反映済み |
| **R-2** | `@alias = Command(args)` の PEG 文法 | `=` / `＝` セパレータの新行種別。変数代入 `$var = expr` との構造的類似性を活用した設計が有力                                                                                                           |
| **R-3** | ~~BarrierKind 命名修正~~（**解決済み**） | Req 3.3 を `WaitForInput / WaitForChoice / Timeout` に修正済み（commit `d04a934`）                                                                                                       |
| **R-4** | `!` コマンド行の PEG 文法            | キーフレーム宣言・指定・Barrier・Clear の具体的記法（EBNF/PEG）。英語/日本語両対応キーワードの設計                                                                                                     |
| **R-5** | pasta→CueSheet 変換層の配置          | アプローチ A/B/C のいずれか確定。pasta_dsl のリリース戦略（breaking change 許容度）に依存                                                                                                              |

### 4.2 重要だが設計中に解決可能

| ID   | 項目                               | 詳細                                                                           |
| ---- | ---------------------------------- | ------------------------------------------------------------------------------ |
| R-6  | 複数 `@command` の処理順           | 1 アクション行に `@happy@sad` のようにある場合（最初のみ？最後のみ？エラー？） |
| R-7  | `%` 行なし時のデフォルトスロット   | 出現順 0,1,2... or エラー                                                      |
| R-8  | `RouteRemove` 発行条件             | シーン終了時自動？明示 `!` コマンドのみ？                                      |
| R-9  | `CueCommand::Clear` 生成ポリシー   | シーン遷移時自動 or `!clear` 明示のみ                                          |
| R-10 | 継続行の CueCommand::Text 追加挙動 | `:content` → 直前 Cue の Text に結合？別 Cue？                                 |

### 4.3 将来拡張（設計考慮のみ）

| ID  | 項目                                    |
| --- | --------------------------------------- |
| F-1 | グローバルスコープのエイリアス定義      |
| F-2 | Storyboard 統合（キーフレーム相互参照） |
| F-3 | `!timeout`, `!wait` などの追加コマンド  |
| F-4 | `CueCommand::Custom` のパラメータ記法   |

---

## 5. 複雑度・リスク評価

### 工数見積もり: **M（3〜7 日）**

**根拠**: 
- PEG 文法追加は既存パターンの延長で manageable
- AST 型追加は breaking change だが影響範囲が限定的（`LocalSceneItem` enum）
- キーフレーム→時刻変換ロジックは新規だが、アルゴリズム自体は単純（累積加算）
- 「設計のみ」スコープのため、コード実装はない
- 成果物は `cue.pasta` + `design.md` の 2 ファイル

### リスク: **Medium**

| リスク要因                                     | 影響度 | 緩和策                                                            |
| ---------------------------------------------- | ------ | ----------------------------------------------------------------- |
| pasta_dsl AST breaking change                  | 中     | Feature flag `cuesheet` で有効化。非 cuesheet 利用者には影響なし  |
| ~~暗黙キーフレーム所要時間の未定義~~（解決済み） | ~~高~~ | Duration Resolver トレイト外部注入で解決。Req 2 AC 5-6 反映済み |
| 外部リポジトリ（vendors/pasta）commit 二重管理 | 低     | 文法・AST 変更は PR 単位で管理。areka 側では submodule update     |
| ~~BarrierKind 要件不整合~~（解決済み）            | ~~中~~ | Req 3.3 修正済み（commit `d04a934`）                               |

---

## 6. 要件修正提案

設計フェーズ開始前に以下の要件修正を推奨:

### 6.1 ~~Req 3.3: BarrierKind 名称修正~~（**適用済み**）

Req 3.3 を `WaitForInput / WaitForChoice / Timeout` に修正済み（commit `d04a934`）。

### 6.2 ~~Req 2: 暗黙キーフレーム所要時間の明確化~~（**適用済み**）

**決定**: 外部注入 Duration Resolver トレイトによる所要時間算出。Req 2 AC 5-6 + 設計注記として反映済み。

### 6.3 Req 3: コマンド種別の網羅性

`Clear` は Req 3.3 で追加済みだが、`WaitForInput` / `WaitForChoice` / `Timeout` の個別記法について Req 3.3 の Barrier 指定とは別に表形式で述べる余地がある。これは design.md のコマンド表で詳細化するのが適切。

---

## 7. 推奨事項と次のステップ

### 設計フェーズへの推奨

1. **アプローチ C（ハイブリッド）を推奨検討** — pasta_dsl で文法 + 構造化 AST、セマンティクス変換は下流層
2. ~~**R-1（暗黙キーフレーム所要時間）を最優先リサーチ**~~ → **解決済み**: Duration Resolver トレイト外部注入
3. ~~**R-3（BarrierKind 名称）を設計開始前に要件修正**~~ → **解決済み**: commit `d04a934`
4. **旧 design.md / cue.pasta の全面書き換え** — v1（`\cue_*` + `[timestamp]`）は完全に obsolete
5. **`!` コマンド行の PEG 文法を design.md の中核成果物として** — 実装者が直接利用可能な精度で

### 次のアクション

```
/kiro-spec-design dola-cue-pasta-dsl-extension
```

design.md リビルド時には以下を含めること:
- `!` コマンド行の PEG/EBNF 文法（R-4）
- エイリアス定義行の PEG/EBNF 文法（R-2）
- キーフレーム→CueSheet 時刻変換アルゴリズム（R-1）
- CueCommand 記法対応表（英語/日本語）
- 実装フェーズ計画（MVP → Full）
- `cue.pasta` の v2 全面書き換え
