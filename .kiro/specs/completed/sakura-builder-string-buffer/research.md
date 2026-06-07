# Gap Analysis: sakura-builder-string-buffer

調査日: 2026-06-07 / 対象: 既存コードベース（pasta_lua）への統合ギャップ分析

## 1. 現状調査（Current State）

### 対象アセット
| アセット | 場所 | 役割 |
| --- | --- | --- |
| `BUILDER.build` | [sakura_builder.lua:57-144](crates/pasta_lua/pasta_scripts/pasta/shiori/sakura_builder.lua#L57) | `buffer={}` + `table.insert` ×多数 + `table.concat(buffer).."\\e"`。**本タスクの改修対象** |
| 唯一の呼び出し元 | [act.lua:68-72](crates/pasta_lua/pasta_scripts/pasta/shiori/act.lua#L68) | `BUILDER.build(token, {...}, STORE.actor_spots)` → 戻り値（文字列）をそのまま返す |
| 既存テスト | [sakura_builder_test.lua](crates/pasta_lua/tests/lua_specs/sakura_builder_test.lua) | describe/test/expect による振る舞い検証。回帰の主防壁 |
| フォールバック前例 | [config.lua:13-16](crates/pasta_lua/pasta_scripts/pasta/config.lua#L13) | `local ok, m = pcall(require, "@..."); if not ok then m = {} end` |
| 条件付き require 前例 | actor.lua:141 | `pcall(require, "@pasta_search")` |

### 規約（Conventions）
- **モジュール構造**: `--- @module pasta.xxx` ヘッダ → `local M = {}` → 関数定義 → `return M`
- **require 書式**: スクリプトツリー内は `require("pasta.xxx")`（ドット区切り）、Rust 登録モジュールは `require("@pasta_xxx")`
- **package.path**: Rust 側（`lua_unittest_runner.rs:24-31`）が `pasta_scripts/?.lua` 等を `;` 連結で設定。**新規 `pasta/buf.lua` は追加設定なしで自動 resolve**される
- **luacheck**: [.luacheckrc](crates/pasta_lua/.luacheckrc) は Lua 5.1 モード。`require`/`pcall`/`string`/`table` は許可リスト済み。未使用変数は `_` プレフィックスで抑止。`scriptlibs/**` は除外

### テストハーネス（統合面）
- 入口: `crates/pasta_lua/tests/lua_unittest_runner.rs` の `run_lua_unit_tests()`
- ランタイム: `Lua::unsafe_new_with(StdLib::ALL_SAFE, LuaOptions::default())`
- `package.loaded` に `@pasta_log` / `@pasta_sakura_script` / `@pasta_search` / `@pasta_persistence` を登録
- テスト登録: [lua_specs/init.lua](crates/pasta_lua/tests/lua_specs/init.lua) の `specs` テーブルにファイル名を追記すると実行される
- フレームワーク: `scriptlibs/lua_test/`（`test.lua` = describe/test、`expect.lua` = アサーション）

## 2. 要件実現性（Requirement-to-Asset Map）

| 要件 | 必要アセット | 状態 | 備考 |
| --- | --- | --- | --- |
| R1: `pasta.buf` の `new()` | 新規ファイル `pasta/buf.lua` | **Missing** | module 規約＋pcall フォールバック前例あり。低リスク |
| R2: 最小実装フォールバック | put/結合取り出しの最小バッファ + メタテーブル | **Missing** | config.lua/actor.lua の pcall パターンを踏襲 |
| R3: `build` のバッファ化・出力不変 | `BUILDER.build` 改修 | **Constraint** | 出力契約（`\e`終端・バイト一致）は act.lua:68 が依存。変更不可 |
| R4: 回帰検証・フォールバック経路試験 | 既存 test + 新規フォールバック試験 | **Missing(部分)** | 既存 test は流用可。フォールバック経路の決定論的検証に工夫が要る（§5 参照） |

## 3. 実装アプローチ・オプション

### Option A: sakura_builder にバッファをインライン（buf.lua を作らない）
- **概要**: `sakura_builder.lua` 内で直接 `pcall(require,"string.buffer")` を行いバッファ生成
- ✅ ファイル増えない / ❌ **R1（再利用可能な `pasta.buf` 提供）に反する**・他箇所で再利用不可
- → **却下**: 要件が独立モジュールを明示

### Option B（推奨）: 新規 `pasta.buf` モジュール + `sakura_builder` 改修
- **概要**: `pasta/buf.lua` を新設し `new()` を公開（LuaJIT 採用 or 最小実装）。`sakura_builder.build` を `local buffer = buf.new()` → `buffer:put(x)` → `buffer:put("\\e"); return buffer:tostring()` へ置換
- **統合点**: `buf.new()` の戻り値は `:put` と結合取り出しメソッドを持つ。`act.lua` は無改修（出力契約不変）
- ✅ 関心の分離が明確・単体テスト容易・既存パターン（pcall フォールバック）に整合
- ✅ 影響範囲が `buf.lua`(新規) + `sakura_builder.lua`(1関数) + テストに限定
- ❌ ファイルが1つ増える（軽微）

### Option C: Option B + 他ホットパスも同時バッファ化
- **概要**: B に加え act.lua/word.lua 等も buf 化
- ❌ 調査の結果、**他に `concat` ホットパスは存在しない**（act/word は配列蓄積のみで concat せず）。価値が薄くスコープ拡大のみ
- → **却下**: Out of scope（将来 buf.lua 再利用先として記録するに留める）

## 4. 複雑度・リスク

- **Effort**: **S（数時間〜1日）** — 新規は小モジュール1つ + 1関数のリファクタ。既存パターン流用、統合は package.path 自動解決
- **Risk**: **Low** — 確立パターン（pcall フォールバック）の延長、技術は既知、スコープ明確、既存テストが回帰を捕捉。correctness はフォールバックにより環境非依存で保証される

## 5. Research Needed（設計フェーズへ持ち越す論点）

1. **`require("string.buffer")` の実在性（最重要）**: 本 mlua 0.11 `luajit52`/`vendored` ビルドで `string.buffer` が実際にロード可能かは **empirical 未確認**（cargo registry キャッシュ不在のため静的確認不可）。LuaJIT 2.1 同梱のため可能性は高いが、**実装時に最小の Lua 評価 or Rust テストで実機確認**すべき。
   - 影響: 利用可能なら高速化の主目的が達成。**不可なら常にフォールバック**となり、最小実装（table+concat）は現状と同等で高速化効果が出ない（ただし correctness は不変で合格）。この場合の価値の扱いを設計で判断。
   - **決定（要件ディスカッション #1）**: 選択肢「correctness + 可用性検証」を採用。Requirement 5 として、対象ランタイムで String Buffer 高速パスが採用されることを実機検証することを要件化した。利用不可と判明した場合は finding として記録しエスカレーションし、黙ってフォールバックを合格としない。速度の数値ベンチは引き続き Out-of-scope。設計フェーズでは「実機確認の手段（テスト or 起動時診断）」を確定する。
2. **結合取り出しメソッドの選定**: LuaJIT buffer は `:tostring()`（非破壊）と `:get()`（破壊的消費）の両方を持つ。`build` は一度しか取り出さないため `:tostring()` が単純で最小実装と揃えやすい。設計で1つに確定し、フォールバックは**同一メソッド名・同一シグネチャ**で提供する（R2-3）。
3. **フォールバック経路の決定論的テスト**: 通常環境では実 `string.buffer` が優先されるため、最小実装経路を試験するには seam が要る。候補: (a) `buf.lua` が最小実装コンストラクタを内部公開し test から直接検証、(b) test 側で最小実装バッファを直接生成して put/tostring の連結結果を実 buffer と比較。設計で testable seam を決める（R4-2）。
4. **`put` の引数形**: 現状 `table.insert(buffer, x)` は単一値。`buffer:put(x)` へ 1:1 写像で十分（複数引数 put は不要）。最小実装も単一引数 put を満たせば足りる。

## 6. 設計フェーズへの推奨

- **推奨アプローチ**: Option B（新規 `pasta.buf` + `sakura_builder.build` 改修）
- **主要決定事項**: (1) string.buffer 実在性の実機確認、(2) 取り出しメソッドを `tostring` に確定、(3) フォールバック経路の testable seam 設計
- **不変条件**: さくらスクリプト出力のバイト一致（`act.lua:68` 出力契約）・`\e` 終端・空入力時 `\e` のみ

---

# 設計フェーズ調査・判断ログ（design）

## Summary
- **Discovery Scope**: Extension（既存 `sakura_builder` 改修 + 新規葉ユーティリティ `pasta.buf`）
- **Key Findings**:
  - vendored LuaJIT に `lib_buffer.c` が**コンパイル済み**を実物確認 → `string.buffer` 同梱の物証
  - 採用パターンは「アダプタ無しの選択」：ネイティブ object を wrap せず直返しで性能維持
  - フォールバックの testable seam は `pasta.buf.new_fallback()` の明示公開で確保

## Research Log

### string.buffer の実在性（最重要・Research Needed #1 の決着）
- **Context**: 高速パスが本当に採用されるか（要件 5）の物証確認
- **Sources Consulted**:
  - `./target/debug/build/mlua-sys-475be6bbfbcb493d/out/luajit-build/src/lib_buffer.c` — **実在**（vendored LuaJIT に lib_buffer.c がコンパイル対象として展開済み）
  - probe テスト（使い捨て `zz_buffer_probe.rs`）で `Lua::new()` / `ALL_SAFE` / `ALL` の3構成での `pcall(require,"string.buffer")` を実行確認しようとした
- **Findings**:
  - lib_buffer.c の存在は確定。LuaJIT 2.1 に String Buffer Library が同梱されている物証。
  - probe の実行は**ビルド環境起因で失敗**（mlua-sys 0.10 の別フィンガープリント再ビルドが `luajit.h` コピー失敗：`os error 2`）。これは string.buffer の可否とは無関係なインクリメンタルビルドの不具合。git-bash/PowerShell 双方で同症状。
  - 既存の正規ビルド（`475...`）は無傷でツールチェイン自体は機能する。
- **Implications**:
  - 設計は string.buffer を主パスとして commit。最終的な「ロード可否（特に mlua StdLib が preload を公開するか）」は **R5 の `string_buffer_availability_test.rs`（正規ビルド環境で実行）** が決定的に確認する。
  - 万一不可なら finding としてエスカレーション。StdLib/preload の Rust 側対応は本スペック外（境界外）。

## Design Decisions

### Decision: バックエンド選択は「アダプタ無し」で object 直返し
- **Context**: ネイティブと最小実装を切り替えつつ、高速化目的を殺さない（要件 1, 2）
- **Alternatives Considered**:
  1. ラッパークラスで両者を包む統一 object — 間接呼び出しが毎 put に乗り、ネイティブの利点を削ぐ
  2. ロード時にバックエンドの生成関数を束縛し、生成 object をそのまま返す
- **Selected Approach**: 2。`M.new = string.buffer.new`（or `FallbackBuffer.new`）をロード時に1度だけ確定
- **Rationale**: ネイティブ object に余計な間接層を挟まない。フォールバックはメソッド名（`put`/`tostring`）のみ一致させれば利用側は無差別
- **Trade-offs**: 利点＝性能維持・実装最小。妥協＝両 object が `put`/`tostring` の同名契約を厳守する必要（テストで担保）

### Decision: 取り出しメソッドは `:tostring()`（Research Needed #2 の決着）
- **Selected Approach**: 非破壊の `:tostring()` を採用。`build` は1度しか取り出さないが、フォールバック（`table.concat`）と意味論が一致し最小実装が単純
- **Follow-up**: フォールバックも同名 `:tostring()` を実装

### Decision: フォールバック testable seam は `new_fallback()` 公開（Research Needed #3 の決着）
- **Selected Approach**: `FallbackBuffer` を常時定義し `pasta.buf.new_fallback()` で明示生成可能に。通常環境（ネイティブ優先）でも最小実装経路を決定論的にテストできる
- **Rationale**: string.buffer を不在化させる困難な細工が不要。要件 4.2 を素直に満たす

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| アダプタ無し選択（採用） | 生成関数をロード時束縛・object 直返し | 性能維持・最小実装 | 両 object が同名契約を厳守 | テストで契約担保 |
| 統一ラッパー | 共通 object で両者を包む | 利用側が完全均一 | put 毎の間接層で高速化を削ぐ | 却下 |
| build へインライン | buf.lua を作らず直書き | ファイル増えない | 要件1（再利用モジュール）違反・再利用不可 | 却下 |

## Synthesis Outcomes
- **一般化**: `pasta.buf` 自体が一般化（再利用可能バッファ）。インターフェースのみ汎用化し実装は現要件（put/tostring）に限定。将来の act.lua 等は seam 変更なしに再利用可能。
- **Build vs Adopt**: ADOPT＝LuaJIT `string.buffer`（実績・プラットフォームネイティブ）。BUILD＝最小フォールバックのみ（不在環境用）。
- **簡素化**: ラッパー層・診断モジュールを排し、`backend` フィールド1つで採用検証 seam を提供。get/set/encode 等の未要求 API は実装しない。

## Risks & Mitigations
- R5 がフォールバックを示す可能性 — `lib_buffer.c` 物証から低リスク。availability_test で実機確認・不可なら finding 化
- ネイティブ `:tostring()` 非提供 — LuaJIT 2.1 で提供確認。フォールバック同名実装で利用側を保護
- `put` 非文字列入力 — `build` は全片を文字列化済み。契約 Precondition で string/number 限定を明示

## References
- [LuaJIT String Buffer Library](https://luajit.org/ext_buffer.html) — `put`/`tostring`/`get` 等の API 仕様
- [config.lua:13](crates/pasta_lua/pasta_scripts/pasta/config.lua#L13) — `pcall(require)` フォールバックの既存前例

---

# 設計バリデーション（validate-design）結果

判定: **GO（条件付き）**。アーキ上の根本不整合なし。出力バイト一致の核心不変条件は堅持。
以下2件は **未解決（後日議論）** として記録。design.md への反映は保留中。

## Open Review Items（未解決・後日議論）

### CI-1: R5 検証の構築経路ミスマッチ（Traceability 5.1, 5.2）
- **問題**: 本番ランタイムは [runtime/mod.rs:108](crates/pasta_lua/src/runtime/mod.rs#L108) の `unsafe_new_with(RuntimeConfig.to_stdlib())`（既定 `std_all → StdLib::ALL_SAFE`、[runtime_config.rs:152](crates/pasta_lua/src/runtime/runtime_config.rs#L152)）。一方 Lua テストハーネスは `Lua::new()`（[lua_unittest_runner.rs:8](crates/pasta_lua/tests/lua_unittest_runner.rs#L8)）で**別構築**。
- **懸念**: `string_buffer_availability_test.rs` を素の `ALL_SAFE` ハードコードにすると本番経路とドリフトしうる。`buf_test.lua` の `backend=="luajit"` 表明は `Lua::new()` 上で走るため本番の証明にならない。
- **推奨**: 可用性テストは `RuntimeConfig::default().to_stdlib()` ＋ `unsafe_new_with`（理想は実 `Runtime` 構築）で本番同一経路を検証。`buf_test.lua` の backend 表明は補助と明示し、R5 authoritative は Rust テストに固定。
- **状態**: ✅ 解決（設計ディスカッション #2）。選択肢1（`RuntimeConfig::default().to_stdlib()` ＋ `unsafe_new_with` の本番同一経路）を採用し design.md へ反映。`buf_test` の backend 表明は補助、R5 authoritative は Rust テストに固定。

### CI-2: build() をフォールバックで実走させる seam の欠落（Traceability 2.4, 3.5, 4.2）
- **問題**: `build()` は内部 `buf.new()` のみ。native 優先環境では build がフォールバック経路を一度も通らず、3.5/4.2 は transitive 論証に留まる。
- **選択肢**:
  - (a) 後方互換注入: 既存任意引数 `config` に `config.buffer_factory`（既定 `buf.new`）を追加。テストが `buf.new_fallback` を渡し build をフォールバックで実走→native とバイト比較。3.5/4.2 を実行パスで担保。
  - (b) transitive 許容: seam を足さず、buf 単体の native↔fallback 同一性＋build が buf を単一利用する事実で間接担保。R4.2 を buf 単体レベルに scope。
- **状態**: ✅ 解決（設計ディスカッション #3）。選択肢(a)（後方互換な `config.buffer_factory` 注入・既定 `buf.new`）を採用。テストが `buf.new_fallback` を注入して build をフォールバックで実走し native とバイト比較。3.5/4.2 を実行パスで担保。

> どちらも**テスト戦略の精緻化**であり、アーキ再設計は不要。タスク生成前に解決するか、実装フェーズで織り込むかも後日判断。

---

# 設計ディスカッション解決ログ

## 議題 #1（解決済み）: Lua バージョン取得関数の追加
- **発端**: 「Lua の正確なバージョンを返す関数」を本スペックに追加したいとの要望。`jit.version` 調査（research 上掲）が直接の根拠。
- **決定**:
  - **場所**: `pasta/lua_version.lua`（単一責務の focused module。config.lua/buf.lua と同流儀）
  - **返り値**: **単一整数**。`1xy=標準Lua x.y`／`2xy=LuaJIT x.y`（例: Lua 5.4→154、5.5→155、LuaJIT 2.0→220、2.1→221）。`>=200` で LuaJIT 判定。Luau 等（3xx 相当）は本プロジェクト未使用のため対象外。
  - **判定方式**: `rawget(_G,"jit")` の有無で LuaJIT を判定（`_VERSION` は LuaJIT を "Lua 5.1" と誤報するため不可）。LuaJIT は `jit.version_num`（例 20100）から major/minor 抽出。標準 Lua は `_VERSION` を解析。
  - **要件反映**: Requirement 6（5 AC）を新設。R5-2 と連携し R5-3「finding に lua_version 数値を併記」を追加。
- **設計反映**: design.md に `pasta.lua_version` コンポーネント／File Structure（`lua_version.lua` + `lua_version_test.lua`）／Traceability 6.1-6.5・5.3／Testing／Error Handling 連携を追記。
- **根拠（jit.version 調査の要点）**: `_VERSION` は実装非依存の言語版のみ（LuaJIT でも "Lua 5.1"）。LuaJIT 検出は `jit` テーブル（`rawget(_G,"jit")`）が定石。リポジトリの luacheck も [builtin_standards/init.lua:291](crates/pasta_lua/scriptlibs/luacheck/builtin_standards/init.lua#L291) で同手法。`jit.version_num` は数値比較向き。
