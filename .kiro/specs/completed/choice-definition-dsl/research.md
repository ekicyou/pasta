# Gap Analysis: choice-definition-dsl

## 1. 現状調査

### 1.1 関連アセットマップ

| レイヤー           | ファイル                                              | 既存機能                                                                                                    | ギャップ                                         |
| ------------------ | ----------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- | ------------------------------------------------ |
| **PEG文法**        | `pasta_dsl/src/parser/grammar.pest`                   | `word_marker = _{ at }`, `word_ref = { word_marker ~ id ~ s }`, `cue_cmd_line`, `slfence_ja1`（`「」`括弧） | `？`/`?`修飾子ルールなし。`＠？`の認識不可       |
| **AST**            | `pasta_dsl/src/parser/ast/action.rs`                  | `Action::WordRef { name, span }`                                                                            | 選択肢定義ノード型なし                           |
| **AST**            | `pasta_dsl/src/parser/ast/scene.rs`                   | `LocalSceneItem::CueCommand`, `::ActionLine` 等                                                             | `LocalSceneItem::ChoiceDef`相当なし              |
| **AST**            | `pasta_dsl/src/parser/ast/cue.rs`                     | `CueCommandNode { command, scope, args }`                                                                   | `!select`の特殊扱いなし                          |
| **トランスパイラ** | `pasta_lua/src/code_gen/element_gen.rs`               | `Action::WordRef` → `act.actor:word("name")`, `Action::SakuraScript` → `act.actor:sakura_script(...)`       | 選択肢 → `\![*]\q[...]` 生成なし                 |
| **トランスパイラ** | `pasta_lua/src/code_gen/scope_gen.rs`                 | `LocalSceneItem::CueCommand` → **スキップ**（「dola側で処理」）                                             | `!select`のトランスパイルなし                    |
| **Luaイベント**    | `pasta_lua/pasta_scripts/pasta/shiori/event/init.lua` | `EVENT.fire(req)` → `REG[id]` or `EVENT.no_entry()` → `SCENE.co_exec()`                                     | `OnChoiceSelectEx`ハンドラ未登録                 |
| **Luaシーン検索**  | `pasta_lua/pasta_scripts/pasta/act.lua`               | `find_scene()` → 6段階ハンドラ検索（L1-L5）                                                                 | 検索自体は流用可能。ルーティング呼び出し元が不在 |
| **SHIORI**         | `pasta_shiori/src/lua_request.rs`                     | SHIORI GET/NOTIFY → `req.id`, `req.reference[N]` テーブル生成                                               | Reference解析は実装済み。ギャップなし            |

### 1.2 イベントフロー（現状）

```
SSP → SHIORI GET (OnChoiceSelectEx, Reference0=表示, Reference1=選択ID)
  → lua_request.rs: パース → req テーブル
    → EVENT.fire(req)
      → REG["OnChoiceSelectEx"] → 未登録
        → EVENT.no_entry()
          → SCENE.co_exec("OnChoiceSelectEx")
            → act:find_scene("OnChoiceSelectEx") → 見つからず
              → 204 No Content
```

### 1.3 既存パターン・規約

- **全角/半角等価**: `at = _{ "＠" | "@" }` — 全マーカーで両対応
- **トランスパイルパターン**: AST → `generate_action()` or `generate_local_scene()` → Lua文字列出力
- **キューコマンド規約**: 現在は全てスキップ（dola委譲）。`!select`を特殊扱いする場合、この規約に初めて例外が生まれる
- **シーン検索**: `act:find_scene(name)` は既存の6段階検索を提供。前方一致・シャッフル＆順次消費は`@pasta_search`モジュール経由

---

## 2. 要件実現可能性分析

### Requirement 1: 選択肢マーカー構文

**技術的必要事項**:
- PEG文法: `choice_def`ルール新設（`word_marker ~ question_marker ~ id ~ optional_bracket_text`）
- AST: `ChoiceDef { target: String, display_text: Option<String>, span: Span }` ノード型追加
- シーンモデル: `LocalSceneItem::ChoiceDef(ChoiceDef)` バリアント追加

**ギャップ**: Missing — 新規PEGルール + ASTノード。ただし既存の`word_ref`・`slfence_ja1`パターンに沿って実装可能。

**複雑度**: 低 — 既存パターンの自然な拡張。

**設計判断ポイント**:
- `？`の半角対応: `?` を使うか。PEG文法で `question = _{ "？" | "?" }` 追加
- `＠？` を `word_ref` の拡張として実装するか、独立ルールとして実装するか

### Requirement 2: さくらスクリプト自動生成

**技術的必要事項**:
- トランスパイラ: `ChoiceDef` → `act.{actor}:sakura_script("\\![*]\\q[display,target]")` 生成

**ギャップ**: Missing — 新規コード生成パス。

**設計判断ポイント**:
- **アクター問題**: `＠？target` は独立行でアクター指定がない。どのアクターの`sakura_script()`を呼ぶか？
  - 案1: 直前のアクション行のアクターを継承
  - 案2: デフォルトアクター（`sakura`）を固定使用
  - 案3: さくらスクリプトタグを直接出力（アクター非依存）
- **エスケープ**: `target`や`display_text`に特殊文字（`\`, `[`, `]`, `,`）が含まれる場合の処理

### Requirement 3: 選択肢コールバック自動ルーティング

**技術的必要事項**:
- Lua: `OnChoiceSelectEx`イベントハンドラの登録
- ルーティングロジック: Reference1 → `act:find_scene(choice_id)` → 実行

**ギャップ**: Missing — ルーティングハンドラ自体が不在。ただしインフラ（`EVENT.fire`, `REG`, `act:find_scene`, `SCENE.co_exec`）は全て既存。

**実装方式案**:

| 方式                            | 概要                                                                                       | 明示的ハンドラ優先                                             |
| ------------------------------- | ------------------------------------------------------------------------------------------ | -------------------------------------------------------------- |
| **A: REG登録**                  | `REG["OnChoiceSelectEx"]`にデフォルトハンドラ登録。内部でユーザー定義シーン検索→auto-route | ハンドラ内で`act:find_scene("OnChoiceSelectEx")`を先行チェック |
| **B: no_entry拡張**             | `EVENT.no_entry()`にフォールバック分岐追加                                                 | 自然に`＊OnChoiceSelectEx`シーンが優先される                   |
| **C: 専用ルーティングレイヤー** | `EVENT.fire()`内にchoice専用分岐追加                                                       | 明示的な優先順位制御                                           |

**推奨: 方式A** — `REG`登録は既存パターン（他のデフォルトハンドラと同様）に沿う。ユーザー定義`＊OnChoiceSelectEx`シーンの優先はハンドラ内で`act:find_scene("OnChoiceSelectEx")`を先行チェックすることで実現。

**スコープ解決（ディスカッション解決済み）**: 自動ルーティング時のローカルシーン検索を実現するため、`STORE.last_global_scene`に最後に実行されたグローバルシーン名を記録する。ルーティングハンドラは`SCENE.search(choice_id, STORE.last_global_scene)`を呼び、ローカル→グローバルの順で前方一致検索する。既存の`SCENE.search(name, global_scene_name)`が第2引数でこのフォールバックを既にサポートしている。

**記録タイミング（ディスカッション補足）**: `STORE.last_global_scene`の更新は`ACT_IMPL.init_scene(self, scene)`内で行う。`co_exec`のコルーチンや`find_scene`の検索結果は再生終了後にnilになるため、これらに依存してはならない。`init_scene`はトランスパイラ出力の各シーン関数冒頭で呼ばれ、`scene.__global_name__`からグローバルシーン名を取得できる。ローカルシーン実行時も親グローバルシーンのSCENEテーブルが渡されるため、`__global_name__`は常に正しいグローバルシーン名を返す。

**複雑度**: 中 — ロジック自体は単純だが、コルーチン実行パス・エラーハンドリング・スコープ解決の正確性が必要。

### Requirement 4: 選択肢タイムアウト

**技術的必要事項**:
- `!select(秒数)` → さくらスクリプトタグ生成

**ギャップ**: Missing + Constraint

**制約**: 現在キューコマンドは**全てトランスパイル対象外**（dola委譲）。`!select`を特殊トランスパイルする場合、この規約に初めて例外が生まれる。

**実装方式案**:

| 方式                            | 概要                                                                                  | 影響                                               |
| ------------------------------- | ------------------------------------------------------------------------------------- | -------------------------------------------------- |
| **A: トランスパイラ特殊ケース** | `!select`のみ`scope_gen.rs`で特殊処理→`sakura_script("\\![set,choicetimeout,N]")`生成 | キュー規約に例外。将来他のキューも特殊化する前例に |
| **B: 選択肢ブロック属性**       | `＠？`行群の属性として扱い、ChoiceDef側でタイムアウト情報を保持                       | PEG/AST変更が大きくなる                            |
| **C: dola委譲維持**             | `!select`はdola側で処理。Pastaはパススルーのみ                                        | 追加実装なし。ただしdola側の対応が前提             |

**Research Needed**: SSPの`choicetimeout`さくらスクリプトタグの正確な仕様。`\![set,choicetimeout,ミリ秒]`形式か要確認。

### Requirement 5: コールバックシーンの互換性

**ギャップ**: なし — 既存アーキテクチャで完全にサポート済み。`act:find_scene()`は通常シーン（`＊target`・`・target`）を6段階検索する。追加実装不要。

---

## 3. 実装アプローチ

### Option A: 既存コンポーネント拡張（推奨）

既存の`＠`マーカーパイプライン（PEG→AST→transpiler）に`＠？`を新バリアントとして追加し、Luaイベントシステムに`OnChoiceSelectEx`デフォルトハンドラを登録する。

**変更対象ファイル**:
| ファイル                      | 変更内容                                       |
| ----------------------------- | ---------------------------------------------- |
| `grammar.pest`                | `choice_def`ルール追加、`question`マーカー定義 |
| `ast/action.rs` or 新ファイル | `ChoiceDef`ノード型追加                        |
| `ast/scene.rs`                | `LocalSceneItem::ChoiceDef`バリアント追加      |
| パーサー構築コード            | `choice_def` → `ChoiceDef` AST変換             |
| `scope_gen.rs`                | `ChoiceDef` → Luaコード生成                    |
| `event/init.lua`等            | `OnChoiceSelectEx`デフォルトハンドラ登録       |
| サンプルゴースト辞書          | 選択肢デモシーン追加                           |

**Trade-offs**:
- ✅ 既存パターンに沿った自然な拡張
- ✅ 新規ファイル最小限
- ✅ テスト既存インフラ活用可能
- ❌ `!select`特殊ケースがキュー規約に例外を作る

### Option B: 新コンポーネント作成

選択肢関連のPEG/AST/トランスパイラを独立モジュールとして分離する。

**Trade-offs**:
- ✅ 関心分離が明確
- ❌ 過剰設計 — 規模に対してファイル分散が大きい
- ❌ 既存パターンとの乖離

### Option C: ハイブリッド

PEG/AST/トランスパイラはOption Aで拡張し、Luaルーティングのみ独立スクリプトファイルとして追加する。

**Trade-offs**:
- ✅ Rust側は最小変更、Lua側は分離
- ✅ Luaハンドラは独立テスト可能
- ❌ わずかに複雑度が上がる

---

## 4. 複雑度・リスク評価

**工数**: **M（3-7日）** — PEG/AST/トランスパイラは既存パターンの拡張だが、Lua自動ルーティングとテスト網羅に時間を要する

**リスク**: **低〜中**
- **低**: PEG/AST/トランスパイラ — 確立されたパターンに沿う拡張
- **中**: Lua自動ルーティング — コルーチン実行パスの正確性、ユーザー定義ハンドラとの優先順位制御
- **中**: `!select`キューコマンド — dola委譲規約への例外（将来の前例）

---

## 5. 設計フェーズへの推奨事項

### 優先アプローチ: Option A（既存コンポーネント拡張）+ Luaハンドラ分離

### 設計フェーズで解決すべき判断事項
1. **アクター問題**: `＠？`独立行のさくらスクリプト出力時、どのアクターコンテキストを使用するか
2. **`!select`の扱い**: トランスパイラ特殊ケースとするか、dola委譲を維持するか
3. **自動ルーティング方式**: REG登録（方式A）の詳細設計、ユーザー定義ハンドラ優先ロジック
4. **特殊文字エスケープ**: target/display_textのさくらスクリプトエスケープ規則

### Research Needed（設計フェーズで調査）
- SSPの`choicetimeout`さくらスクリプトタグの正確な仕様
- `\![*]`メニューマークの挙動確認（SSP実装依存）
- `OnChoiceSelectEx` vs `OnChoiceSelect`のReference構造差異

---

## 6. 設計判断（design.md で確定）

設計フェーズで以下を確定した。詳細は `design.md` 参照。

### B1: アクター問題 → Luaハイブリッド＋DSLはアクター紐づき
選択肢はLuaランタイムレベルではアクター非依存の構造化トークン `{ type = "choice", target, display }` として蓄積する。`group_by_actor` ではハイブリッド分類（アクターグループ内 or トップレベル）。DSLレベルではトーク行内に記述するため自然にアクターに紐づく。SHIORI層（sakura_builder）ではアクター紐づき前提で処理してよい。アクター非依存のDSL構文は現行スコープ外（Luaランタイムは対応済み）。`％`アクター指定・直前アクター継承は不要。

### B2: `!select` 委譲規約 → トランスパイラ特殊ケース（案A採用）
`choicetimeout` さくらスクリプトタグは出力に含める必要があるため、`scope_gen.rs` の `LocalSceneItem::CueCommand` 分岐で cue 名 `select` を特例処理し `act:choice_timeout(秒)` を生成する。他の cue コマンドの dola 委譲ポリシーは不変。

### B3: 特殊文字エスケープ → SHIORI層（sakura_builder）で一律エスケープ
target は文法 `id` 規則によりデリミタ（`,` `]` `\`）を含まない。display（`「」`内）は `\q[...]` デリミタとの衝突を避けるため、`sakura_builder`（SHIORI層）でエスケープする。`@pasta_sakura_script` のエスケープヘルパを利用（無ければ最小エスケープを実装）。コアランタイム層（act.lua）はエスケープを行わない。

### ルーティング方式 → 明示ハンドラ優先 + スコープ付き検索
`choice_select.lua`（`boot.lua` 参考）で `OnChoiceSelectEx` 既定ハンドラを実装。明示 `＊OnChoiceSelectEx` シーン優先 → `SCENE.search(choice_id, STORE.last_global_scene)` 前方一致検索（ローカル→グローバル）→ シャッフル＆順次消費 → 非マッチ時 204 委譲。

### グローバルシーン記録 → init_scene で記録
`ACT_IMPL.init_scene(self, scene)` がシーン実行開始時に `STORE.last_global_scene = scene.__global_name__` を記録（`co_exec` 戻り値はプレイバック後 nil 化するため開始時記録が必須）。`SCENE.search(name, global_scene_name, attrs)`・`scene.__global_name__`・`init_scene` の実在を確認済み。

### Open Question（実装時に確定）
- `OnChoiceSelectEx` の Reference インデックス（Reference0 vs Reference1 が選択ID）は実機/ukadoc で確定し、`choice_select.lua` 内で定数化して吸収する。要件意図「選択IDでシーン検索」は不変。
