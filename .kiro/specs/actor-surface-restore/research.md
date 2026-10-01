# ギャップ分析: actor-surface-restore

- 対象要件: `.kiro/specs/actor-surface-restore/requirements.md`（R1〜R5）
- 分析日: 2026-10-01
- 分析範囲: `crates/pasta_lua/pasta_scripts/pasta/`（act.lua / actor.lua / store.lua / shiori/act.lua / shiori/sakura_builder.lua）、トランスパイラのアクション行生成（`crates/pasta_lua/src/code_gen/element_gen.rs`）、テスト資産、SSP 仕様（ukadoc）

## 1. 現状調査（Current State）

### 1.1 出力パイプライン

| 段階 | 場所 | 内容 |
| --- | --- | --- |
| トークン蓄積 | `pasta/act.lua` | `talk`/`sakura_script` はアクター付き。`surface`/`wait`/`newline`/`clear` は**アクター無し**で蓄積（`act:surface(id)` → `{type="surface", id}`） |
| グループ化 | `act.lua` `group_by_actor` / `merge_consecutive_talks` | talk/sakura_script のアクター変化で `{type="actor", actor, tokens}` を切る。アクター無しトークンは直前グループへ入る（グループ前なら**捨てられる**）。`spot`/`clear_spot` は独立トークン |
| 文字列化 | `pasta/shiori/sakura_builder.lua` `BUILDER.build` | アクター切替検出 → `emit_actor_switch` が `\p[spot]` を出力。グループ内は `emit_inner_token` で変換。sakura-script-newline の has-text / pending 状態機械（ビルドローカル）を保持 |
| 呼び出し | `pasta/shiori/act.lua` `SHIORI_ACT_IMPL.build` | `BUILDER.build(token, {spot_newlines}, STORE.actor_spots)` — `STORE.actor_spots` を**直接変更方式**で渡す |

### 1.2 サーフェス変更の実際の発生経路（重要）

- DSL `ぱすた：＠通常　こんにちは` は `act.ぱすた:talk(act.ぱすた:word("通常"))` を生成し（`element_gen.rs:286-289`）、アクター辞書単語の値 `\s[0]` は **talk テキスト文字列の一部**として流れる（後続 talk と連結され `"\s[0]こんにちは"` になる）。
- DSL 中のリテラルさくらスクリプト `\s[3]` は `act.X:sakura_script("\\s[3]")`（`element_gen.rs:346-348`）→ `sakura_script` トークンのテキスト。
- 構造化 `surface` トークン（`act:surface(id)`）は Lua からの明示呼び出し時のみ。
- **結論**: サーフェス状態の大半は「文字列中の `\s[...]`」として出力される。構造化トークンだけを見ても R1.2 を満たせない。**文字列走査によるタグ検出が必須**（Missing）。
- `talk_to_script`（Rust `@pasta_sakura_script`）はさくらスクリプトタグを `TokenKind::SakuraScript` として保持しウェイトを挿入するのみで、`\s[...]` 自体は改変しない（`sakura_script/tokenizer.rs`）。検出は `talk_to_script` 出力後の文字列でも入力テキストでも可能。

### 1.3 アクター／スポット状態

- `STORE.actors`（アクターキャッシュ、`CONFIG.actor` 由来）、`STORE.actor_spots`（アクター名→スポットID、`CONFIG.actor[name].spot` で初期化）。いずれも**ランタイムメモリ常駐**でビルドを跨いで保持、ディスク永続化なし（save は `pasta.save` 別系統）。
- ゴースト再読込は Lua VM 再生成で STORE ごと破棄される（`STORE.reset()` の本番呼び出し元はテスト以外に無い）。→ R4.1/R4.2/R4.3 の「セッション内保持・再読込で破棄・非永続」は `STORE` に新フィールドを置くだけで**既存パターンで満たせる**（Constraint ではなく追い風）。
- アクターごと／スポットごとのサーフェス・着せ替え状態を保持する仕組みは**存在しない**（Missing）。
- `pasta/store.lua` は「他モジュールを require しない」循環回避ポリシーあり。新フィールド追加はこの方針に沿う必要（Constraint）。

### 1.4 SSP 側の挙動（ukadoc）

- `\s[ID]`: 「現スコープ側のサーフェスを変更」。サーフェスはスコープ単位で保持され、スクリプト終了後も残る。`\sN`（0〜9 の短縮形）、エイリアス名、`-1`（非表示）あり。
- `\![bind,カテゴリ名,パーツ名,数値]`: 1=着衣/0=脱衣、パーツ名空欄=カテゴリ単位、数値省略=トグル。実行後 `OnDressupChanged` → `OnNotifyDressupInfo` が通知される。
- `\![bind-noevent,...]`: 同じ操作でイベントを発生させない（**SSP 2.8.23 以降**）。R3.7 の仮定の実現手段候補。
- 補足: brief.md の「`\b` 系」は誤記の可能性が高い（`\b` はバルーン切替。着せ替えは `\![bind]`）。

### 1.5 テスト資産・規約

- `crates/pasta_lua/tests/lua_specs/sakura_builder_test.lua`（26 ケース、`surface` トークンのケースあり）+ `lua_unittest_runner.rs`。
- `crates/pasta_shiori/tests/byte_invariant_test.rs`（バイト不変ゴールデン）— R5.1/R5.3 の回帰ゲートとして流用可能。
- `crates/pasta_shiori/tests/support/scripts/pasta/` に act.lua / store.lua 等の**テスト用コピー（現行と乖離あり）**が存在。sakura_builder.lua のコピーは無い。store.lua に新フィールドを追加した場合の影響有無は設計で確認（Unknown）。
- `BUILDER.build` は既に `luacheck: ignore 561`（循環的複雑度 22 > 15）で `ponytail:` コメント付き。さらに分岐を足すと悪化（Constraint）。プロジェクト方針は「特性化テスト先行・1抽出=1検証=1コミット」。

## 2. 要件→資産マップ

| 要件 | 既存資産 | ギャップ |
| --- | --- | --- |
| R1.1 構造化 surface 記録 | `emit_inner_token` の `surface` 分岐 | **Missing**: 記録処理 |
| R1.2 テキスト中 `\s` 記録 | `talk`/`sakura_script` 分岐（文字列出力のみ） | **Missing**: 出力文字列からの `\s[...]`/`\sN` 抽出。エスケープ `\\s` の誤検出回避が必要（**Research Needed**） |
| R1.3 最後の値 | — | Missing（抽出時に最後の一致を採用するだけ） |
| R1.4 スポット表示中更新 | `last_spot`（ビルドローカル） | **Missing**: スポット単位の表示状態（ビルド跨ぎ） |
| R1.5 ID 非解釈 | `tostring(inner.id)` | 既存方針と整合（文字列で保持） |
| R2.1-2.3 切替時復旧 | `emit_actor_switch`（`\p` 出力の単一箇所） | **Missing**: 比較と `\s` 再出力。挿入点は `\p` 出力直後で明確 |
| R2.4 冗長許容 | — | 仮定どおりなら追加処理不要 |
| R2.5 改行非干渉 | 改行判定は「非空 talk」のみ参照 | 復旧タグを buffer へ直接 put すれば has-text に影響しない（**既存構造で充足**） |
| R2.7 ビルド跨ぎ | `STORE.actor_spots` 直接変更方式 | 同方式で STORE に状態を置けば充足 |
| R3.1-3.3 bind 記録/復旧 | 無し | **Missing**: `\![bind,...]` 引数パース（`"..."` クォート・`\,` エスケープ考慮が必要、**Research Needed**） |
| R3.4 トグル/カテゴリ単位 | — | Missing（不明化ロジック） |
| R3.7 noevent | — | **Constraint/Unknown**: SSP 2.8.23 以上要求。非 SSP ベースウェアでの扱い |
| R4.1-4.3 ライフサイクル | STORE 常駐・VM 再生成で破棄 | 既存パターンで充足 |
| R4.5 clear_spot で保持 | `clear_spots`（actor_spots のみ nil 化） | 新状態を clear_spot で触らなければ充足 |
| R5.1-5.4 回帰防止 | byte_invariant_test / sakura_builder_test | 「専用スポットでは常に一致→無出力」の不変条件を特性化テストで固定する必要 |
| R5.5 スポット移動追従 | — | 規則適用のみ。ただし spot 0 フォールバック（未設定アクター）が意図せず共有スポット化して復旧を誘発しうる（**Constraint**） |

## 3. 実装アプローチの選択肢

### Option A: 既存 `sakura_builder.lua` を拡張

- `emit_actor_switch` の `\p` 出力直後に復旧処理、`emit_inner_token` の surface/talk/sakura_script 出力時に記録処理を追加。状態は `STORE` に新フィールド（例: アクター別・スポット別の外見状態）を追加し、`shiori/act.lua` から `actor_spots` と同様に渡す。
- ✅ 変更ファイル最小（sakura_builder.lua / shiori/act.lua / store.lua）、既存の「STORE 直接変更方式」に整合。
- ❌ `BUILDER.build` の複雑度がさらに上がる（既に 22）。`emit_inner_token` に記録副作用が混入し責務が濁る。`BUILDER.build` のシグネチャ（第3引数 actor_spots）拡張が必要。

### Option B: 外見状態トラッカーを新モジュール化

- 新モジュール（例: `pasta/shiori/appearance.lua`）に「出力文字列からの `\s`/bind 抽出」「アクター/スポット状態の記録」「切替時の復旧タグ生成」を純関数＋状態テーブルとして集約。ビルダーは `\p` 出力直後と各トークン出力直後に 2 箇所でフックするだけ。
- ✅ タグ抽出・差分計算を単体テストしやすい（純関数）。ビルダーの複雑度増加を最小化。sakura-script-newline の状態機械と責務分離（brief の「責務の縫い目」と整合）。
- ❌ ファイル追加。ビルダーとの接続インターフェース設計が必要。

### Option C: ハイブリッド（推奨候補）

- 抽出・差分計算は新モジュール（B）、状態保持は `STORE` 新フィールド、ビルダー側は最小フック（A）。段階: ①特性化テストで現行出力を固定（専用スポット・同一スポット交代・ビルド跨ぎ）→ ②サーフェス復旧のみ導入 → ③着せ替え復旧を別ステップで導入（R3 の範囲確定後）。
- ✅ R3（着せ替え）の範囲がディスカッションで縮小・除外されても②までで独立に完結できる。小ステップ・可逆の方針に合致。
- ❌ 計画がやや複雑。

## 4. 工数・リスク

- **工数: M（3〜7日）** — サーフェス復旧のみなら S 相当だが、文字列からのタグ抽出（エスケープ・クォート考慮）、bind の状態モデル、ビルド跨ぎ・回帰の特性化テスト整備を含むため M。
- **リスク: Medium** — 既存パターン（STORE 常駐状態・単一の `\p` 出力点）は明確だが、(1) テキスト内タグ抽出の誤検出、(2) bind の状態不確定（トグル・カテゴリ単位・SSP 側メニュー操作）、(3) `bind-noevent` のベースウェア依存、(4) spot 0 フォールバックによる意図しない共有スポット化、が不確実要素。

## 5. 設計フェーズへの申し送り

### 推奨方針
- Option C。サーフェス復旧（R1/R2/R4/R5）を先行し、着せ替え復旧（R3）は範囲確定後に同一仕様内の後続ステップとして実装。
- 復旧タグの挿入点は `emit_actor_switch` の `\p[spot]` 出力直後に一本化（sakura-script-newline の pending フラッシュは「次の一般文字列直前」なので、順序 `\p[0]\s[N]\n[150]text` が自然に成立する）。
- 状態は `STORE` 常駐（`actor_spots` と同ライフサイクル）。永続化しない。

### Research Needed
1. テキスト中サーフェスタグの抽出規則: `\s[...]` / `\sN` の検出、`\\s`（エスケープされたバックスラッシュ）の除外、`\s` 以外の `\s` 始まりタグ（存在有無）の確認。抽出対象を `talk_to_script` 出力後とするか入力テキストとするか。
2. `\![bind,...]` 引数パース: ダブルクォート・`\,` エスケープ、`bind-noevent` 自体が作者により出力された場合の記録扱い。
3. `bind-noevent` の最低 SSP バージョン（2.8.23）要件をどこに明記するか／非対応ベースウェア時のフォールバック。
4. 発話テキスト内スコープ切替タグ（`\0` `\1` `\h` `\u` `\p[N]`）以降の `\s`/bind の帰属をどう扱うか（無視／記録停止／帰属先スポット追跡）。
5. `crates/pasta_shiori/tests/support/scripts/pasta/store.lua`（テスト用コピー）への影響有無。
6. 起動直後に SSP が表示する既定サーフェス（descript / surfaces.txt 由来）を pasta が知り得るか（既定サーフェス設定を導入する場合の前提）。
7. カテゴリ単位 bind（パーツ名空欄）の結果状態: `,,0`（カテゴリ全脱衣）を確定状態として扱えるか、`,,1` の意味を SSP 仕様で確認。確定できれば R3.4 の不明化対象から外す（要件ディスカッション #7 で設計調査へ先送り）。

---

# 設計フェーズ調査（Research & Design Decisions）

- 追記日: 2026-10-01（上記ギャップ分析は要件フェーズの記録としてそのまま保持）

## Summary
- **Feature**: `actor-surface-restore`
- **Discovery Scope**: Extension（既存 `sakura_builder.lua` への統合中心・light discovery）
- **Key Findings**:
  - `talk_to_script` はタグを改変せずウェイト／改行タグを挿入するだけなので、出力後文字列の走査で 1.2 を満たせる。構造化 `surface` トークンも出力文字列 `\s[ID]` として同じ経路で観測できる（専用分岐不要）。
  - `STORE.actors[name]` は `CONFIG.actor[name]` そのもの。既定サーフェス／既定着せ替えはアクターテーブル直下の任意キーとして Rust 変更なしで読める。
  - `crates/pasta_shiori/tests/support/scripts/pasta/store.lua` は検索パス上位の旧コピーで本番 `store.lua` を覆う。`STORE.appearance` が nil になる環境が実在するため、ビルダーの nil 許容が必須。

## Research Log（§5 Research Needed の解決）

### 1. テキスト中サーフェスタグの抽出規則
- **Sources**: `crates/pasta_lua/src/sakura_script/tokenizer.rs`（`SAKURA_TAG_PATTERN = \[0-9a-zA-Z_!+*?&-]+(?:\[[^\]]*\])?`）、`sakura_script/mod.rs` `talk_to_script_impl`、ukadoc `\s[ID番号]` / `\sID番号`。
- **Findings**: `\s` 始まりの表示系タグは `\s[ID]` と `\s0`〜`\s9` のみ（`\_s` は名前が `_` 始まりで別タグ）。トークナイザは `\` を特別扱いしないが、SSP 上 `\` はバックスラッシュ文字であり `\s[5]` はサーフェス変更ではない。`talk_to_script` は `\s[...]` を改変しない。
- **Implications**: 走査対象は**出力後文字列**（SSP が実際に受け取るもの）。`\` は 2 文字読み飛ばし。タグ境界はトークナイザと同じ規則、分類は名前の先頭で行う。先頭タグ列の先読み（2.4・3.10）は未変換のトークンテキストで行う（タグは不変なので結果は一致）。

### 2. `\![bind,...]` 引数パース
- **Sources**: ukadoc `\![bind,カテゴリ名,パーツ名,数値]`、`shiori/act.lua` `escape_tag_arg`（SSP の引数クォート規約）。
- **Findings**: 引数は `,` 区切り、`,` を含む値は `"..."` で囲む規約。カテゴリ名・パーツ名に `,` `"` `\` を含めるシェルは稀。
- **Implications**: 単純 `,` 分割で解釈。`"`・`\` を含む bind は解釈不能として全スポットの着せ替えを不明化（仮定・OQ-5）。作者が書いた `bind-noevent` も `bind` と同じく記録する。

### 3. `bind-noevent` の最低バージョン
- **Sources**: ukadoc `\![bind-noevent,...]`（2.8.23）。
- **Implications**: 要件 3.7 どおりバージョン判定なし。design.md Technology Stack とマニュアル（`first-ghost.md` の `dressup` 説明）に明記する。

### 4. スコープ切替タグ以降の帰属
- **Sources**: ukadoc `\p[ID番号]` / `\pID番号`（0〜9）、`\0` `\1` `\h` `\u`。
- **Implications**: 要件 1.7 で確定済み（記録停止＋全スポット不明化）。認識対象は `\0` `\1` `\h` `\u` `\p[N]` `\p0`〜`\p9`。

### 5. pasta_shiori テスト用 store.lua コピー
- **Sources**: `crates/pasta_shiori/tests/common/mod.rs` `copy_support_dirs`、tech.md「scripts/ 優先順位」。
- **Findings**: `tests/support/scripts/` はテスト用ゴーストの `scripts/` へコピーされ、`profile/pasta/pasta_scripts/` より優先される。support には `shiori/act.lua`・`sakura_builder.lua` が無いため、それらは内蔵（新）版、`store.lua` は旧コピーという組み合わせになる。
- **Implications**: コピーは変更しない。`BUILDER.build` の第4引数 nil → `APPEARANCE.new()` で吸収。

### 6. 起動直後の既定サーフェス
- **Findings**: pasta は descript.txt / surfaces.txt を読まず、SSP 起動時の表示サーフェスを知る手段を持たない（SSP イベント経由の取得は要件で対象外）。
- **Implications**: 作者宣言の `surface` キーのみを既定とし、スポット表示中状態は起動直後「不明」（4.4）。

### 7. カテゴリ単位 bind の結果状態
- **Sources**: ukadoc bind の記述例「`\1\![bind,arm,,0]` … armカテゴリのパーツを全て解除」。
- **Findings**: `,,0` は「カテゴリ全解除」と明記。`,,1` の意味は記載なし。
- **Implications**: `,,0` を確定状態にするには「カテゴリ既定 0」という別表現が状態モデルに必要になる。得られる効果は冗長な `bind-noevent` 数個の削減のみのため採用しない（3.4 どおり不明化。OQ-4）。
- **改訂（設計ディスカッション #4・#5）**: 不明化の範囲を発話アクター本人に狭めた結果、`,,0` を不明扱いにすると全脱衣したアクターの見た目が復旧されない取りこぼしが生じると判明。パーツ名 `""` を「残り全パーツ」として扱う表現で `,,0` を確定状態化する（要件 3.11）。ukadoc では `bind-noevent` は `bind` と同書式のため `\![bind-noevent,cat,,0]` で復旧できる。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A | sakura_builder.lua 内に全実装 | ファイル追加なし | `BUILDER.build` / `emit_inner_token` の複雑度増、単体テスト困難 | 不採用 |
| B | 新モジュールが状態も保持 | 凝集 | ライフサイクルが STORE と二重化、reset 経路が別になる | 不採用 |
| C | 純ロジック新モジュール＋STORE フィールド＋最小フック | 複雑度増なし、状態直渡しで単体テスト可、既存 `actor_spots` 方式と同形 | 引数が増える | **採用** |

## Design Decisions

### Decision: 観測は「出力文字列の走査」に一本化
- **Alternatives**: (1) トークン種別ごとに専用分岐（surface トークンは id を直接記録）、(2) 出力文字列の走査のみ。
- **Selected**: (2)。`emit_inner_token` が生成した文字列を `observe` に渡す。
- **Rationale**: surface トークンは `\s[ID]` 文字列になるので同じ走査で拾える。分岐を増やさない。
- **Trade-offs**: 文字列を二度見るコスト。バックスラッシュ無しの早期リターンで通常テキストは無負荷。

### Decision: 状態は 1 フィールド `STORE.appearance = { actors, spots }`
- **Rationale**: `BUILDER.build` への引数が 1 つで済み、全スポット不明化は `state.spots = {}` の 1 行。nil を「不明」とし番兵値を作らない。
- **Trade-offs**: `bind` は 2 段テーブル（カテゴリ→パーツ）。カテゴリ不明化（3.4）が `binds[cat] = nil` で済む。

### Decision: 冗長復旧の抑止は「先頭タグ列」の先読み
- **Alternatives**: (1) 復旧タグを遅延出力（最初の一般文字直前）、(2) 切替時にグループ先頭を先読み。
- **Selected**: (2)。要件 2.1・2.6 が復旧位置を `\p[spot]` 直後（保留改行より前）と定めるため遅延は不可。
- **Follow-up**: 先頭タグ列の終端条件（一般文字・スコープ切替・raw_script）をテストで固定。

### Decision: アクター設定キーは `surface` / `dressup`
- **Rationale**: 既存キー `spot`・`budoux` と同じ直下フラット配置・短い名詞。`dressup` は SSP イベント名（OnDressupChanged）と語を揃える。入れ子テーブルは状態モデル（カテゴリ→パーツ→値）と同形で変換不要。
- **Trade-offs**: アクター直下キーはアクター単語検索（A1 完全一致）でも見えるため、`＠surface` が数値を返す（`spot` と同じ既存の性質）。

### Decision: 2 ステップ導入
- ステップ A（サーフェス）: 走査（サーフェス・スコープ切替・bind の存在検出）、`surface` の記録／復旧、`surface` 既定キー、STORE フィールド、フック。
- ステップ B（着せ替え）: `binds` の記録／不明化／復旧、`dressup` 既定キー。A の関数へ追記するのみでフックは変更しない。

### Synthesis
- **Generalization**: サーフェスと着せ替えは「アクター既知 vs スポット表示中の差分を再出力」という同一問題。`AppearanceEntry` を共通形にし、`observe` / `restore` の 2 関数に集約。
- **Build vs Adopt**: Rust トークナイザを Lua へ公開して再利用する案は、API 追加と Rust 変更を伴うため不採用。Lua パターン数行で足りる。
- **Simplification**: スコープ別帰属追跡・`,,1` 確定化・引数クォート完全解釈・バージョン判定・新規ログ・CONFIG からの初期転送はいずれも不採用。

## Risks & Mitigations
- 専用スポット構成でも不明化後に `\s[既知]` が再出力されバイト等価でなくなる（OQ-1）— 視覚的には同一サーフェスの再指定。ディスカッションで許容可否を確定。
- 既存テストの期待値変化（スポット0フォールバック共有。OQ-7）— 特性化を先行し、変化するケースを列挙して意図を確認。
- タグ誤検出 — `\` 読み飛ばし・外側タグの `[...]` 読み飛ばしを単体テストで固定。
- SSP 2.8.23 未満で着せ替え復旧が無効 — 要件で許容済み。マニュアルに明記。

## References
- ukadoc さくらスクリプト一覧: `\s[ID番号]`、`\sID番号`、`\p[ID番号]`、`\pID番号`、`\![bind,...]`、`\![bind-noevent,...]`（https://ssp.shillest.net/ukadoc/manual/list_sakura_script.html）
- `crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua`、`pasta/store.lua`、`pasta/shiori/act.lua`、`pasta/act.lua`、`pasta/actor.lua`
- `crates/pasta_lua/src/sakura_script/{mod,tokenizer}.rs`、`crates/pasta_lua/src/code_gen/element_gen.rs`
- `.kiro/specs/completed/sakura-script-newline/`、`.kiro/specs/completed/persist-spot-position/`
