# ギャップ分析: pasta-manual-debugging

> 対象要件: `.kiro/specs/completed/pasta-manual-debugging/requirements.md`（R1〜R8）
> 分析日: 2026-06-08
> 性質: 既存コードベース上の **ドキュメント追加** 仕様（実装変更なし）

## 分析サマリ

- **本仕様は文書のみ**。デバッグ機能（Rust/DAP バックエンド）は完成・出荷済みで、本仕様はその使い方を mdBook マニュアルに章として追加する。実装リスクは低い。
- **重要事実の確定**: `.pasta` ソースレベルデバッグは **本番出荷済み**（`pasta-source-map` 完了 = 2026-06-08）。提示モード切替・サイドカー出力まで実装完了。マニュアルは「実験的・将来」ではなく **本番機能** として記述する（R6.2 の根拠が確定）。
- **既存 `DEBUGGING.md` は陳腐化**（「`.pasta` ソースレベル = 実験的・将来」と記述）。移植ではなく **最新事実への全面改稿 + マニュアルへの統合** が必要。
- **検証基盤との親和性が高い**: デバッグ章は `doc/spec` 由来を持たないため `drift-check` の未マップ警告対象外。`verify-content.mjs` の D-voice（ボイス検査）は `book/src` 配下を動的走査するため自動適用される。ただし A〜F の**網羅アサーション**はデバッグ章を対象に含まないため、R8.4（章の存在・本文・ボイスの機械確認）を満たすには検証スクリプトへの軽微な追記が論点。
- **推奨アプローチ = Option B（新規 `book/src/debug/` セクション）+ 既存検証への最小結線**。

---

## 1. 現状資産の調査（Current State）

### 1.1 既存ドキュメント資産

| 資産 | 役割 | 本仕様での扱い |
| ---- | ---- | -------------- |
| `DEBUGGING.md`（ルート） | VSCode/LuaJIT デバッグ運用ガイド。全体像・有効化・構造的制約・緩和策を網羅 | **主要シード**。ただし陳腐化（`.pasta` ソースレベル=実験的/将来）。最新化して章へ統合し、ルートはリダイレクト化（R6.1/6.3） |
| `book/src/`（mdBook） | 公開マニュアル（introduction/getting-started/grammar/lua/reference） | **デバッグ章を新設**。現状デバッグ章は皆無 |
| `book/AUTHORING.md` | 執筆規約（Claudia 令嬢ボイス・Do/Don't・流用/リンク方針） | デバッグ章も準拠（R7） |
| `book/CONTENT-REVIEW.md` | 編集レビュー記録（人手確認 + 機械検証の二段構え） | デバッグ章の確認記録を追補する形が自然 |

### 1.2 マニュアル検証基盤（`book/tools/`）

| スクリプト | 検査内容 | デバッグ章への影響 |
| ---------- | -------- | ------------------ |
| `verify-content.mjs` | A:文法網羅 / B:Lua網羅 / C:チュートリアル / **D:ボイス** / E:外部参照 / F:バージョン。`book/src` を動的走査 | **D-voice は自動適用**（散文部にボイスマーカー必須、コードフェンス内にナレーション混入禁止）。A〜F の網羅アサートは grammar/lua 等に限定でデバッグ章を含まない → R8.4 を満たすには **デバッグ章用の存在・本文アサート追記**が論点 |
| `drift-check.mjs` | ①ドリフト（manual-sources.toml の sha256 比較）②未マップ doc/spec 検出 ③リンク切れ | デバッグ章は doc/spec 由来なし → **②の対象外（警告にならない）**。章から外部リンクを張れば ③ の検査対象 |
| `verify-static.mjs` | 静的出力のみ・file:// オフライン解決・SUMMARY 全章の HTML/リンク健全 | SUMMARY に追加すれば**自動検証対象** |
| `verify-search.mjs` | 日本語 bigram 検索の動作・クライアント完結・索引/ランタイム/head.hbs 一致 | 章本文が検索索引に入るか**自動実検証**（R1.4） |
| `verify-drift-gate.mjs` | drift-check と完了ゲートの結線 | doc/spec 未触のため影響小 |
| `tutorial-check.mjs` | first-ghost.md と hello-pasta dic の逐語一致 | デバッグ章は非対象 |

- **実行方法**: `book/package.json` に scripts は無く、`node book/tools/<script>.mjs` を直接実行。`mdbook build book` で HTML 生成。
- **日本語検索**: `theme/head.hbs` にインライン bigram tokenizer。`mdbook build` 時に `book/src` 全ファイルが索引化されるため、SUMMARY 記載済みのデバッグ章は自動で検索対象。

### 1.3 デバッグ実装の確定事実（マニュアル記載値・ソース検証済み）

> いずれも `crates/pasta_lua/src/loader/config.rs:425-486` および `crates/pasta_lua/src/debug/mod.rs` でソース確認済み。

**有効化（`pasta.toml [debug]`）**:
| キー | 既定 | 意味 |
| ---- | ---- | ---- |
| `enabled` | `false` | デバッグバックエンド有効化 |
| `port` | `9276` | DAP リスナーの TCP ポート |
| `present_as` | 省略=`.pasta` | 提示モード（`"pasta"`/`"lua"`、case-insensitive、不正値は `.pasta` フォールバック） |
| `source_map_sidecar` | `false` | `.lua.map` サイドカーをディスク出力（メモリ内マップが常に主） |

**環境変数（設定ファイルより優先）**:
- `PASTA_DEBUG` — truthy = `1`/`true`/`yes`/`on`（case-insensitive・trim 後）。falsy = `0`/`false`/`no`/`off`/空。
- `PASTA_DEBUG_PORT` — u16。
- `PASTA_DEBUG_SOURCE_MODE` — `pasta`/`lua`、不正値は `.pasta` フォールバック + 警告。
- `PASTA_DEBUG_SOURCE_MAP_SIDECAR` — bool（同 truthy 規約）。

**優先順位**:
- `enabled`/`port`/`source_map_sidecar`: `env > pasta.toml [debug] > 既定`
- 提示モード: `DAP attach 引数 sourcePresentation > env PASTA_DEBUG_SOURCE_MODE > pasta.toml present_as > 既定 .pasta`

**接続**:
- 既定 `127.0.0.1:9276`（loopback 限定・TCP）。外部接続不可。
- **attach のみ**（VM がデバッグ有効で先に待機 → VSCode が接続）。launch 非対応。

**無効時（本番ゼロコスト・R5/R6 整合）**:
- フック未設置（`enable()` が hook を張らない）／TCP リスナー未 bind（ポート未開放）／Lua へ `debug`・`std_debug` 非露出（サンドボックス維持）。
- transpile 時のソースマップ記録は sink=None で no-op → 出力バイト不変。

**`.pasta` ソースレベル機能（pasta-source-map = 完了・本番出荷）**:
- `.pasta` 行ブレークポイント／`.pasta` 座標での停止・コールスタック常時提示／`.pasta` 粒度ステップ（over/into/out・コルーチン跨ぎ）／変数 inspect／提示モード切替（`.pasta`既定 ⇄ `.lua`）／任意 `.lua.map` サイドカー出力。実 DAP-over-TCP E2E で実証済み。

**構造的制約**:
- `pasta_shiori` の request 処理が `Arc<Mutex>` で直列・blocking。ブレーク中は VM スレッドが復帰せず、現リクエスト＋後続 SHIORI リクエスト全てが continue まで待機 → SSP タイムアウトの恐れ。**既知・意図的**。根本解決（ホスト非同期化）はスコープ外。

---

## 2. 要件 → 資産マップ（gap tags）

| 要件 | 既存資産 | ギャップ | タグ |
| ---- | -------- | -------- | ---- |
| R1 マニュアル統合・公開導線 | mdBook 基盤・SUMMARY・検索・静的検証 | デバッグ章ファイル新設＋SUMMARY 追記が必要 | Missing(章) / 基盤=Reuse |
| R2 有効化方法 | config.rs の確定値・DEBUGGING.md §1 | 最新値で記述（present_as/sidecar も追加） | Reuse + 加筆 |
| R3 VSCode 接続 | editors/vscode `contributes.debuggers`・DEBUGGING.md §1 | launch.json 具体例の正確な field 確定 | Constraint(要 design 確認) |
| R4 `.pasta` ソースレベル操作 | pasta-source-map 完了・debug/source_map.rs | 出荷済み事実を本番機能として記述 | Reuse(事実確定済) |
| R5 構造的制約・緩和策 | DEBUGGING.md §2-4 | ほぼ流用可（最新化のみ） | Reuse |
| R6 情報源一本化 | DEBUGGING.md（陳腐化）| 全面改稿 + リダイレクト化 + 二重管理回避 | Constraint(陳腐化是正) |
| R7 執筆ボイス準拠 | AUTHORING.md・基準ボイスサンプル | 型に倣って執筆 | Reuse(規約) |
| R8 実装整合・非回帰 | verify-content/drift/static/search | D-voice 自動適用。**デバッグ章の網羅アサート追記**が論点（R8.4） | Constraint(検証結線) |

ギャップの中心は2点のみ: **(a) 章コンテンツの新規執筆**（事実は確定済みなので執筆作業）、**(b) R8.4 の機械確認をどう満たすか**（verify-content への最小追記の要否）。

---

## 3. 実装アプローチ選択肢

### Option A: 既存章への相乗り（例: lua/ 配下にデバッグ節を追加）
- ✅ ファイル数最小
- ❌ デバッグは文法/Lua とは読者文脈が異なり、章の責務が肥大化。SUMMARY の見通しも悪化。R1「独立セクション」と整合しにくい。

### Option B: 新規 `book/src/debug/` セクション（推奨）
- 章構成案: `debug/index.md`（概要・全体像・有効化）＋必要に応じ `debug/vscode-attach.md`（接続手順）＋`debug/pasta-source-level.md`（`.pasta` 操作）＋`debug/constraints.md`（構造的制約・緩和策）。最小は単一 `index.md` でも R1〜R5 を満たせる。
- 検証結線: SUMMARY 追記で verify-static/search が自動適用。D-voice 自動適用。R8.4 のため verify-content にデバッグ章の存在・本文アサートを最小追記。
- ルート `DEBUGGING.md` → 章へのリダイレクト化（R6.3）。
- ✅ 責務分離が明確・読者導線が自然・R1〜R8 と素直に整合
- ✅ 既存検証基盤をほぼそのまま活用
- ❌ ファイルが増える（が、他セクションと同パターンで一貫）

### Option C: ハイブリッド（章新設 + DEBUGGING.md を当面薄く残す）
- ✅ 段階移行に安全
- ❌ R6.4「二重管理しない」と緊張。中途半端だと陳腐化リスクが残る。最終的に B へ収束させる必要。

**推奨 = Option B**。理由: 本仕様の要件（独立セクション・情報源一本化・検証非回帰）に最も素直に適合し、既存の章パターン・検証基盤をそのまま再利用できる。

---

## 4. 工数・リスク

- **工数: S〜M（2〜5 日）**。事実は確定済みで実装変更なし。作業の主体は (1) 章執筆（ボイス準拠・正確値）、(2) DEBUGGING.md 改稿+リダイレクト、(3) verify-content への最小追記と全検証の緑化。多ページ構成にすると M 寄り。
- **リスク: Low**。
  - 実装非依存（文書のみ）でリグレッション面積が小さい。
  - 唯一の判断点は R8.4 を満たす検証結線方法（verify-content 追記の粒度）— 既存パターン（GRAMMAR_CHAPTERS 配列方式）に倣えば低リスク。
  - 正確性リスクは確定済み事実表（§1.3）で軽減済み。

---

## 5. 設計フェーズへの推奨と Research Needed

### 推奨（design で確定すべき決定）
1. **章構成の粒度**: 単一 `debug/index.md` か、複数ページ（概要/接続/`.pasta`操作/制約）か。R1〜R5 の網羅と読みやすさのバランスで決定。Option B 採用前提。
2. **R8.4 の検証結線方式**: verify-content.mjs にデバッグ章の存在・本文（最小文字数）・ボイスを確認するアサートを追加するか、CONTENT-REVIEW.md の人手確認で代替するか。機械確認を要件が求めている（R8.4「検査可能な形で確認」）ため、最小アサート追加が有力。
3. **DEBUGGING.md リダイレクトの形式**: 1〜数行の誘導（GitHub から公開サイト URL と相対パス両方を案内）に統一。

### Research Needed（design 着手時に各 1 ファイルで確認・本ギャップでは深追いしない）
- **launch.json の正確な field**: `editors/vscode/package.json` の `contributes.debuggers`（type 識別子・request=attach・host/port/sourcePresentation・breakpoints 対応言語）を読み、利用者が誤記しない具体例を確定。`type` は `"pasta"` の見込み（要最終確認）。
- **サイドカー出力先パス・ファイル名規約**: `source_map_sidecar=true` 時に生成される `.lua.map` の出力場所/命名（`debug/source_map.rs` の `write_sidecar`）を確認し、章に正確に記す。
- **提示モード切替の利用者操作**: `sourcePresentation`（launch.json）/ `PASTA_DEBUG_SOURCE_MODE`（env）/ `present_as`（toml）の3経路と優先順位を、利用者向けに最短の説明へ整理。

---

## 6. 要件ディスカッション結果（2026-06-08）

要件精査の 1 対 1 ディスカッションで以下を確定。要件へ反映済み（コミット済み）。

- **C1 クライアント射程** → VSCode を主軸に具体記述し、DAP-over-TCP ホスト非依存ゆえ他 DAP クライアントからも接続しうる旨を一言補足（R3.5）。
- **C2 拡張の前提** → 未導入を前提に pasta VSCode 拡張の導入手順を章に含める。VSCode 本体のインストールは外部リンクのみ（R3.6 / R3.7）。
- **C3 章の性格** → 体系的ガイド＋短いウォークスルー（hello-pasta を実際に 1 箇所ブレーク→attach→停止/変数確認）（R4.7）。

### 設計フェーズへ追加で持ち越す判断（カテゴリ B 追補）
- **B5 ウォークスルーの検証範囲**: R4.7 のウォークスルーを `tutorial-check.mjs` の逐語一致ガード対象に含めるか、それとも getting-started のチュートリアルとは別扱い（逐語ガード非対象・通常の章本文として扱う）か。執筆量・保守負荷とのトレードオフで design にて決定。
- **B6 拡張導入手順の出典**: 既存の VSCode 拡張インストール説明（マーケットプレイス/既存章/README）が再利用できるかを design で確認し、二重記述を避ける導線を決める。

---

## 7. 設計フェーズ discovery 確定事項（2026-06-08・Light Discovery）

要件ディスカッション後の Research Needed を解消し、B1〜B6 を確定。

### 確定した実装事実（マニュアル記載値・ソース検証済み）

**launch.json / VSCode 拡張**（`editors/vscode/package.json` `contributes`）:
- `debuggers[].type = "pasta"`、`label = "Pasta Debug"`。
- `configurationAttributes.attach.properties`: `host`（既定 `"127.0.0.1"`）、`port`（既定 `9276`）、`sourcePresentation`（enum `["pasta","lua"]`・任意・省略時はバックエンド設定 env>toml>既定に委譲）。
- `request` は `attach` のみ（`initialConfigurations` / `configurationSnippets`「Pasta: Attach」も attach）。
- `breakpoints`: 言語 `pasta` と `lua` の両方を登録（`.pasta` 行・`.lua` 行いずれもブレーク可）。
- → 拡張の `contributes.debuggers` がデバッグ統合の提供元。拡張未導入ではデバッグ不可（R3.6 の根拠）。

**サイドカー出力**（`crates/pasta_lua/src/debug/source_map.rs`）:
- 出力先 = 生成 `.lua` の真隣 `<lua_path>.map`（例 `.../scene/sys.lua` → `.../scene/sys.lua.map`）。`sidecar_path_for_lua()`。
- 形式 = `serde_json` 直列化 JSON。決定論的（同入力→同バイト）。
- メモリ内マップが常に主、サイドカーは任意の追加出力。書込失敗は非致命（メモリ写像は不変）。
- 生成 `.lua` はローダのキャッシュディレクトリ配下（`CacheManager::source_to_cache_path`）。

**verify-content.mjs 構造**（`book/tools/verify-content.mjs`）:
- `checks[]` に `ok/fail/assert(id,cond,passMsg,failMsg)` で積む。`A`〜`F` の各カテゴリは独立ブロック。
- `GRAMMAR_CHAPTERS` 配列でループする「明示列挙」方式（doc/spec 由来章用）。
- ボイス検査: `hasVoice()`（広い `VOICE_MARKERS`・散文部）＋ コードフェンス内 `NARRATION_MARKERS`（狭い：`わたくし`/`おほほ`/`フンッ`/`ごきげんよう Claudia`）非混入。`isSubstantive(md, 800)` で本文実体。
- → デバッグ章用に **新カテゴリ `G`** を同一イディオムで追加すれば R8.4 を満たす（B2 確定）。

**CI**（`.github/workflows/manual.yml`）: `book/**` 変更で mdbook build → highlight → bigram → drift-check → tutorial-check → cargo test 構文ガード → Pages デプロイ。`verify-content.mjs` は CI 非搭載で DoD/ローカルゲート。

**既存拡張インストール章**: `book/src` に VSCode 拡張のインストール専用節は無い（B6: デバッグ章が導入手順を提供・二重記述リスクなし。マーケットプレイス/VSIX リンクは `reference/external-links.md` への追補も可）。

### 設計判断の確定（B1〜B6）
- **B1 章構成** → 複数ページ（`book/src/debug/` に index＋vscode-setup＋source-level＋constraints）。grammar/lua と同じ多ページ・イディオムを踏襲（Generalization）。
- **B2 R8.4 検証結線** → `verify-content.mjs` に新カテゴリ `G`（デバッグ章の存在・本文・ボイス・主要事実の登場）を追加。人手レビューは CONTENT-REVIEW.md に追補。
- **B3 DEBUGGING.md リダイレクト** → 本文を撤去し、公開サイト URL（`https://ekicyou.github.io/pasta/`）＋ 相対パス（`book/src/debug/`）＋ GitHub ソースパスへの数行誘導スタブに置換。
- **B4 launch.json 値** → 上記確定値を vscode-setup.md にそのまま記載。
- **B5 ウォークスルー検証範囲** → `tutorial-check.mjs` の逐語一致ガード対象外（同ガードは hello-pasta 辞書 .pasta 専用）。ウォークスルーは通常章本文として `G` で存在・ボイスのみ機械確認。
- **B6 拡張導入の出典** → デバッグ章 vscode-setup.md が単一の導入導線。マーケットプレイス/VSIX の外部 URL は external-links.md に追補可。

### 合成（synthesis）結論
- **Generalization**: デバッグ章は既存「コンテンツ章＋機械検証カテゴリ」の一変種。新カテゴリ G は A〜F の一般化適用で、新インフラ不要。
- **Build vs Adopt**: 既存パイプライン（mdbook build / bigram 検索 / verify-* / SUMMARY / AUTHORING ボイス規約）を全面 Adopt。新規ビルドは「章コンテンツ」＋「verify-content への G ブロック追記」のみ。
- **Simplification**: 新ツール・新依存・トランスクルージョン機構は持ち込まない。`doc/spec` 由来なしゆえ manual-sources.toml も触らない（R6.5）。
