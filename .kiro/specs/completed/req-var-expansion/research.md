# Research & Design Decisions: req-var-expansion

## Summary
- **Feature**: `req-var-expansion`
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  - `transfer_date_to_var()` パターンの完全踏襲が可能（ガード句 → 転記 → `return self`）
  - `＄ｒ０`（全角 `ｒ` + 全角数字）は既存PEG文法の `id` ルールで解決され、文法変更不要
  - `act.req.reference` は **0-indexed**（SHIORIプロトコル準拠）、Lua通常テーブルの1-indexedとは異なる

## Research Log

### `transfer_date_to_var()` の実装パターン分析
- **Context**: 新メソッド `transfer_req_to_var()` の設計テンプレートとして既存実装を詳細調査
- **Sources Consulted**: `crates/pasta_lua/scripts/pasta/shiori/act.lua` L87-145
- **Findings**:
  - ガード句: `if not self.req or not self.req.date then return self end`
  - ソースデータをローカル変数にキャッシュ: `local date = self.req.date`
  - 個別フィールドの `if` ガード付き転記（nil フィールドはスキップ）
  - 数値型（`var.year`）と文字列型（`var["年"]`）の二重転記パターン
  - `return self` によるメソッドチェーン対応
  - ヘルパー: `WEEKDAYS_JA/EN`（曜日テーブル）、`to_12hour_format()`（12時間制変換）
- **Implications**: `transfer_req_to_var()` もまったく同じパターンで実装可能。ヘルパー不要（単純転記のみ）

### `act.req.reference` のインデクシング
- **Context**: SHIORIプロトコルのReference番号がLuaテーブルでどのキーに格納されるか確認
- **Sources Consulted**: `crates/pasta_lua/src/runtime/lua_request.rs` L103-104, Rustテスト
- **Findings**:
  - Rust側: `reference.set(nums, value)?;` — `nums` は `"Reference0"` の `"0"` をパースしたi32値
  - **0-indexed**: `reference[0]`, `reference[1]`, ..., `reference[9]`
  - Luaテスト内モックは `[1]=` で始まる場合があるが、実運用では0-based
- **Implications**: ループは `for i = 0, 9 do` で走査する必要がある

### `＄ｒ０` のPEGパース検証
- **Context**: `digit_id` ルールとの衝突を回避する `＄ｒ０` が正しくパースされることの確認
- **Sources Consulted**: `crates/pasta_core/src/parser/pasta.pest`, `crates/pasta_core/src/parser/mod.rs` L826-835
- **Findings**:
  - `ｒ` (U+FF52) は XID_START → `id` ルールの先頭文字として有効
  - `０` (U+FF10) は XID_CONTINUE → `id` ルール内で継続文字として有効
  - `ｒ０` → `id` ルール → `VarScope::Local` → `var.ｒ０`（Luaコード: `var["ｒ０"]`）
  - 実証テスト4件パス済み（`digit_id_var_test.rs`）
- **Implications**: 文法変更不要。DSLの `＄ｒ０` は自動的に `var["ｒ０"]` に解決される

### varキー衝突チェック
- **Context**: `transfer_date_to_var()` のキーと `transfer_req_to_var()` のキーが衝突しないことの確認
- **Sources Consulted**: `crates/pasta_lua/scripts/pasta/shiori/act.lua` L108-145
- **Findings**:
  - date系キー（16個）: `year`, `month`, `day`, `hour`, `min`, `sec`, `wday`, `年`, `月`, `日`, `時`, `分`, `秒`, `曜日`, `week`, `時１２`
  - req系キー（22個）: `ｒ０`〜`ｒ９`, `r0`〜`r9`, `req_id`, `req_base_id`
  - **重複なし** — 名前空間が完全に分離
- **Implications**: 両メソッドの共存に問題なし

### SHIORI_ACT クラス階層
- **Context**: 新メソッドの配置場所の確認
- **Sources Consulted**: `crates/pasta_lua/scripts/pasta/shiori/act.lua` L21-36
- **Findings**:
  - `SHIORI_ACT_IMPL` テーブルに `transfer_date_to_var` が定義されている
  - `SHIORI_ACT.new(actors, req)` で `base.req = req` が設定される
  - メタテーブル連鎖: `act → SHIORI_ACT_IMPL → ACT_IMPL`
- **Implications**: `transfer_req_to_var` も同じ `SHIORI_ACT_IMPL` テーブルにメソッドとして追加する

## Design Decisions

### Decision: 全角キー `ｒ０` の命名規則
- **Context**: `＄０` が `VarScope::Args` に解決される衝突問題の回避
- **Alternatives Considered**:
  1. `＄０`（digit_id） — 既存 args 参照と衝突
  2. `＄r0`（半角ASCII） — IME切替が必要
  3. `＄ｒ０`（全角ｒ + 全角数字） — `id` ルールで解決、IME切替不要
  4. `＄R0` / `＄P0` — 全角入力の一貫性が崩れる
- **Selected Approach**: `＄ｒ０`（全角 `ｒ` + 全角数字）
- **Rationale**: `ｒ` は XID_START で `id` ルールに自然に合致。全角モードのままIME切替なしで入力可能。`ｒ` = reference の意味も明確
- **Trade-offs**: `＄０` のような極めて短い表記は不可能だが、`＄ｒ０` でも十分短い
- **Follow-up**: 実装時にLua側の全角キー文字列をハードコードで定義（ルックアップテーブル）

### Decision: `req_` 接頭辞によるメタデータキー命名
- **Context**: `act.req.id` → `var.???` のキー名選定
- **Alternatives Considered**:
  1. `var.event` — ソース名と無関係で唐突
  2. `var.id` — 汎用すぎて衝突リスク
  3. `var.req_id` — ソース `act.req.id` との対応が明確
- **Selected Approach**: `req_` 接頭辞（`req_id`, `req_base_id`）
- **Rationale**: 転記元 `act.req.*` との対応関係が一目瞭然。将来 `act.req` の他フィールドを追加する場合も一貫性を維持できる

## Risks & Mitigations
- **リスク1**: `act.req.reference` が10個未満の場合 — `nil` チェック付きループで安全にスキップ（Req 1.2）
- **リスク2**: `act.req` 自体が `nil`（非SHIORIコンテキスト） — ガード句で即座に `return self`（Req 3.5）
- **リスク3**: 将来 `transfer_date_to_var` とのキー衝突 — 名前空間が完全に分離されているため衝突リスクは極めて低い

## References
- `crates/pasta_lua/scripts/pasta/shiori/act.lua` — SHIORI_ACT 実装（テンプレートパターン）
- `crates/pasta_lua/src/runtime/lua_request.rs` — SHIORIリクエストパーサー（reference 0-indexed）
- `crates/pasta_core/src/parser/pasta.pest` — PEG文法定義（`var_id`, `digit_id`, `id` ルール）
- `crates/pasta_core/tests/digit_id_var_test.rs` — パーサー実証テスト
- `.kiro/specs/req-var-expansion/gap-analysis.md` — ギャップ分析レポート（`＄０` 衝突発見・解決）
