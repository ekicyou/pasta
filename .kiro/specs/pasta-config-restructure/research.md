# Gap Analysis: pasta-config-restructure

> 要件（requirements.md R1〜R7）と既存コードベースの差分分析。設計フェーズの判断材料として、現状資産・欠落・実装アプローチ・リスクを整理する。

## 1. 現状調査（Current State）

### 1.1 設定パースの実体（Rust 側）
`crates/pasta_lua/src/loader/config.rs` に `PastaConfig` が集約。

| セクション | Rust での扱い | Default 実装 | 消費経路 |
|-----------|--------------|-------------|---------|
| `[loader]` | **即時型パース**（`LoaderConfig`） | あり | Rust ローダ |
| `[logging]` | 遅延型パース（`logging()`） | あり | Rust ロギング初期化 |
| `[persistence]` | 遅延型パース（`persistence()`） | あり | Rust 永続化 |
| `[lua]` | 遅延型パース（`lua()`） | あり | Rust ランタイム |
| `[talk]` | 遅延型パース（`talk()`） | あり（`#[serde(default)]`） | Rust さくらスクリプト |
| `[debug]` | 遅延型パース（`debug()`） | あり（`#[serde(default)]`） | Rust DAP バックエンド |
| `[ghost]` | **型パースなし**（`custom_fields` のまま） | **Rust 構造体なし** | Lua 側のみ |
| `[actor."名前"]` | **型パースなし**（`custom_fields` のまま） | なし | Lua 側のみ |
| `[package]` | **どこからも消費されない** | — | なし（実質デコラティブ） |

- `PastaConfig::parse()` は `[loader]` のみ `remove` して型化し、**残り全部を `custom_fields: toml::Table`** に格納（config.rs:59-78）。
- `[loader]` 欠落時は `LoaderConfig::default()` を適用（config.rs:64-68）→ **任意セクションは既に省略可能**。
- `PastaConfig::load()` は **ファイル不在で `ConfigNotFound` エラー**（config.rs:48-50）→ **R3.4「ファイル不在を許容しない」は既に満たされている**。

### 1.2 設定消費の実体（Lua 側）
`custom_fields` は `register_config_module()`（`runtime/module_registry.rs:54-64`）が `toml_to_lua` で **read-only Lua テーブル `@pasta_config`** として公開する。`[ghost]`/`[actor]` はこの経路でのみ参照される。

- `pasta/config.lua` … `CONFIG.get(section, key, default)` ラッパー（`@pasta_config` を `pcall` 保護）
- `pasta/shiori/act.lua:48` … `CONFIG.get("ghost", "spot_newlines", 1.5)` ← **デフォルト 1.5 は Lua リテラル**
- `pasta/shiori/event/virtual_dispatcher.lua:72-79` … `talk_interval_min`(180) / `talk_interval_max`(300) / `hour_margin`(30) ← **デフォルトは Lua リテラル**
- `pasta/store.lua:87-98` … `STORE.actors = CONFIG.actor`、`actor.spot`（数値）を `STORE.actor_spots` へ転送。**`CONFIG.actor` が table でなければ沈黙して何もしない**（＝ actor 不在でもエラーにならない）。

### 1.3 デフォルト値の分散（SSOT 不在の現状）
同一デフォルトが **複数箇所に重複定義**されている。

| 分類 | 権威 | 重複先 |
|------|------|--------|
| `[loader]`/`[logging]`/`[persistence]`/`[lua]`/`[talk]`/`[debug]` | Rust `Default`/`default_*()` 関数 | doc 表、サンプル toml |
| **`[ghost]`** | **Rust 構造体が無く、Lua リテラルが事実上の権威** | doc 表（pasta-toml.md）、サンプル toml |
| `[actor]` | デフォルト無し（`spot` 必須・固有） | — |

→ **`[ghost]` には Rust 側 SSOT が存在しない**点が R5（SSOT 化）の最大の構造的課題。

### 1.4 テンプレート／サンプル／ドキュメントの現状
- **テンプレート生成器は存在しない**。`pasta_sample_ghost/src/config_templates.rs` は名前に反し **`surfaces.txt` 生成専用**（pasta.toml とは無関係）。
- 事実上の「フルテンプレート」= 手書きの `crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/pasta.toml`（フル記述・SSOT サンプル）。`release/hello-pasta/.../pasta.toml` はそのビルド成果物コピー。
- **`pasta-toml.md`（pasta-ghost-authoring スキル参照）には既に「最小構成例」と全セクション表が存在**し、`[package]`/`[lua]` を「★上級者向け」と注記済み。ただし **3分類（必須/任意/将来予約）の正式な確立には至っていない**。
- 利用者マニュアル: `book/src/getting-started/first-ghost.md` ほかが pasta.toml に言及。README も入口として言及。

## 2. 要件 → 資産マップ（Requirement-to-Asset Map）

| 要件 | 既存資産 | ギャップ判定 |
|------|---------|-------------|
| **R1** 3分類モデル | pasta-toml.md の2分類（型解析/カスタム）＋★注記 | **Missing**: 「SHIORIコア必須/任意/将来予約」の正式分類成果物が未確立 |
| **R2.1/2.2** actor 必須・最小起動 | `STORE.actors=CONFIG.actor` 経路 | 任意省略時の起動は既に可（Constraint）。actor 必須化は**仕様・検証として未定義** |
| **R2.3** actor 不在を判別可能に | **なし**（store.lua は沈黙） | **Missing（本仕様の中核的振る舞いギャップ）** |
| **R2.4/R3.1-3.3** ghost 任意・デフォルト適用・両経路一貫 | Lua リテラルデフォルト＋ Rust Default | 概ね既存。ただし**両経路一貫の保証根拠（SSOT）が弱い** |
| **R3.4** ファイル不在不許容 | `ConfigNotFound`（config.rs:48） | **充足済み（変更不要）** |
| **R4.1** `[package]` 将来予約分類 | pasta-toml.md ★注記 | 分類の格上げ（Constraint） |
| **R4.2/4.3** テンプレ・サンプルから除去 | サンプル toml は **`[package]` を含み、かつコメントで「必須項目は `[package]` と `[loader]` のみ」と明記** | **Missing＋矛盾**（後述 3.4） |
| **R4.4** 既存 `[package]` を無視起動 | どのコードパスも未消費 | **充足済み（後方互換確認のみ）** |
| **R5** テンプレ2層＋SSOT | テンプレ生成器なし、デフォルト分散 | **Missing（最大の設計判断対象）** |
| **R6** 完全後方互換 | 既存テスト群（下記2.1） | 追加的変更が中心で低リスク。R2.3 検証の追加のみ要注意 |
| **R7** ドキュメント反映 | pasta-toml.md / book / README | 既存土台あり。3分類・最小例の反映が必要 |

### 2.1 後方互換の検証資産（既存テスト）
- `tests/loader/config_test.rs`（設定パース）
- `tests/loader/startup_test.rs`（起動）
- `tests/loader/config_actors_initialization_test.rs`（`CONFIG.actor → STORE.actors`）
- `tests/shiori/virtual_event_config_test.rs`（`[ghost]` 由来のトーク間隔）
- `tests/lua_specs/config_get_test.lua` / `config_actor_init_test.lua`
- フィクスチャ `tests/fixtures/loader/with_ghost_config/pasta.toml`
→ これらの **意図を壊さないこと**が R6.3 の合否基準。

## 3. 重要な発見・要研究事項（Research Needed / リスク）

### 3.1 ★ `[loader]` 省略と `pasta_patterns` 既定値の不整合（高リスク）
- 既定 `pasta_patterns = ["dic/*/*.pasta"]`（config.rs:159-161、サブディレクトリ前提）。
- 一方 hello-pasta サンプルは **`["dic/*.pasta"]`**（直下前提）に上書きしている。
- **「actor のみの最小テンプレート」で `[loader]` を省略すると `dic/*/*.pasta` が既定適用され、`dic/直下` に辞書を置く作者は辞書を読み込めず沈黙失敗**する恐れ。
- → 最小必須セクションの確定（R2）と既定 glob の妥当性は **セットで設計判断が必要**。「最小テンプレートに `[loader].pasta_patterns` を含めるか」「既定 glob を `dic/**/*.pasta` 等に広げるか」を design で検討。

### 3.2 R2.3「判別可能な扱い」の実現手段（要設計判断）
- 現状 store.lua は actor 不在で沈黙。「沈黙して誤起動しない」をどう実装するか:
  - 案: `@pasta_log` で warn/error ／ OnBoot で `RES` 経由のバルーン通知 ／ Rust ローダ段での検出。
- ただし Out-of-scope に「**設定値のバリデーション強化（型安全ラッパー等）は対象外**」とあるため、**過剰実装を避けた軽量な判別手段**に留める設計境界の見極めが必要。

### 3.3 `[ghost]` の SSOT 化方針（要設計判断）
- `[ghost]` は Rust 構造体が無く Lua リテラルが権威。R3.3「Rust 側と Lua 側で一貫したデフォルト」と R5.4「SSOT から導出」を満たすには:
  - 案A: Rust に `GhostConfig`（typed・Default）を新設し SSOT 化（ただし custom_fields 経路の互換維持が条件）。
  - 案B: Lua を権威とし、doc/テンプレートをドリフト検出で同期（repo 既存の `book/tools/drift-check.mjs`＋`manual-sources.toml` パターンを流用）。
- 本リポジトリには既に **版マーカー＋ドリフト検出でドキュメント同期を担保する基盤**が存在するため、テンプレ／doc の SSOT 整合に再利用できる。

### 3.4 ★ サンプル toml のコメント矛盾（要修正・後方互換に影響なし）
- hello-pasta の `pasta.toml` 冒頭コメントが **「必須項目は `[package]` と `[loader]` のみ」** と明記（pasta.toml:4, 11-14）。新モデル（actor 必須・package 予約・loader 任意）と**正面衝突**。R4.2/4.3/R7 の一部として **コメント文言ごと是正**が必要。

### 3.5 テンプレート提供形態（要設計判断）
- 2層テンプレート（最小／フルリファレンス）の **物理的提供先**が未定:
  - 案: doc（pasta-toml.md / book 章）に記載 ／ サンプルゴーストに最小版を実体配置 ／ Rust 生成器で出力。
- `pasta-toml.md` に既に最小例があるため、**doc 主体＋サンプル実体の2点同期**が現実的。SSOT 整合（R5.4/5.5）の担保方法と合わせて決定する。

## 4. 実装アプローチ（Options）

### Option A: ドキュメント／スキーマ中心（拡張最小）
3分類を **仕様・ドキュメントのメタ情報**として確立し、actor 必須判別を軽量に追加、サンプル／doc を是正。Rust の型追加なし、`[ghost]` SSOT は doc/Lua 同期で対応。
- ✅ 後方互換リスク最小（パース挙動を変えない）／ brief のアプローチ A に忠実
- ✅ 既存ドリフト検出基盤を流用可能
- ❌ `[ghost]` の Rust 側 SSOT は得られず、一貫性は「同期」担保に依存
- ❌ R2.3 の判別を Lua 側に置くと検出層が分散

### Option B: 型付き構造体＋テンプレート生成器（新規作成）
Rust に `GhostConfig`（SSOT）と分類メタの enum、pasta.toml テンプレート生成モジュールを新設。
- ✅ `[ghost]` を含む全デフォルトの Rust 単一 SSOT を実現、R5 を強く満たす
- ✅ 生成器でテンプレ2層を機械生成（乖離不能）
- ❌ `[ghost]` を型パースに移すと custom_fields 経路・`@pasta_config` 公開との二重管理／互換検証コスト
- ❌ Out-of-scope の「バリデーション強化」に踏み込むリスク、工数大

### Option C: ハイブリッド（推奨・brief 整合）
**A を核**に、**C 思想（ティア化・SSOT）**を取り込む。
- 3分類は仕様＋ doc メタとして確立（A）。
- デフォルト値の SSOT は **単一ソース（Rust 既存 `default_*()` ＋ `[ghost]` 用の最小限の Rust 定数/構造体、あるいは Lua 権威）から1本化**し、テンプレ／doc は **ドリフト検出（既存 `manual-sources.toml` 方式）で同期**。
- R2.3 は **軽量な判別手段**（log/通知）で実装し、過剰なバリデーションは避ける。
- 最小テンプレートは **3.1 の glob 問題を踏まえ `[loader]` の扱いを明示**。
- ✅ brief 明記のアプローチ（A 核＋C 思想）と一致／互換リスクと SSOT 要求のバランス
- ❌ SSOT の物理的1本化先（Rust か Lua か）の決定が前提、計画がやや複雑

## 5. 工数・リスク（Effort / Risk）

| 領域 | Effort | Risk | 根拠 |
|------|--------|------|------|
| R1 3分類モデルの確立（仕様/doc） | S | Low | 既存 pasta-toml.md 表を分類拡張するだけ |
| R2.3 actor 不在の判別実装 | S–M | **Medium** | 新規振る舞い。検出層の選定と既存起動フロー非回帰が要件 |
| R3.1（glob 既定の整合・3.1） | S | **Medium** | 最小構成で辞書未読込となる沈黙失敗の回避が必要 |
| R4 `[package]` 除去・コメント是正 | S | Low | 未消費のため挙動変化なし、文言/サンプル修正中心 |
| R5 テンプレ2層＋SSOT | M | **Medium** | SSOT 物理化先の決定とドリフト同期設計 |
| R6 完全後方互換（回帰確認） | S | Low | 追加的変更中心、既存テスト群で担保 |
| R7 ドキュメント反映 | S | Low | book/README/skill 参照への分類反映 |
| **全体** | **M（3–7日）** | **Medium** | 中核は仕様/doc＋サンプル是正＋軽量検証。SSOT と glob 整合が要注意点 |

## 6. 設計フェーズへの申し送り（Recommendations）

1. **推奨アプローチ: Option C**（brief のアプローチ A 核＋C 思想に整合）。
2. **先に決める設計判断**:
   - (a) **SSOT の物理的1本化先**: `[ghost]` を Rust typed 化（案A）か Lua 権威＋同期（案B）か。
   - (b) **R2.3 の判別手段**: log / OnBoot 通知 / Rust 検出のいずれか（Out-of-scope の過剰バリデーション回避と両立）。
   - (c) **最小テンプレートの `[loader]` 扱い**（3.1 の glob 不整合をどう解消するか）。
   - (d) **テンプレ2層の提供先**（doc 主体 / サンプル実体 / 生成器）と SSOT 同期方式（既存 drift-check 流用可否）。
3. **持ち越し調査（Research Needed）**:
   - 既定 `pasta_patterns`（`dic/*/*.pasta`）と実運用（`dic/*.pasta`）の整合方針。
   - `@pasta_config` read-only 公開と typed 化を両立させる場合の二重管理コスト。
   - ドリフト検出基盤（`book/tools/drift-check.mjs` / `manual-sources.toml`）を pasta.toml テンプレ同期へ流用する適合性。
4. **後方互換の必達ライン**: §2.1 の既存テスト意図を破壊しないこと、`[package]` 含むフル記述が無警告で起動すること（R6.1/6.2）。

---

## 7. 要件ディスカッション結果（2026-06-12 反映）

要件ディスカッション（`/kiro-requirements-discussion`）で以下を決定し、requirements.md を再構成した。設計フェーズはこの決定を前提とする。

### 7.1 採用した核モデル（議題1）
- **「3分類（必須/任意/将来予約）」を「単一デフォルト適用ステップ＋2プロファイルのデフォルト表（SSOT）」モデルへ再構成**（R1 全面改訂）。
  - 分類は「**SHIORI プロファイルにデフォルトを持つ（省略可）/ デフォルトを持たない（必須）/ エンジンプロファイル専用（SHIORI では適用不要）**」の3区分へ。
  - `pasta.toml` ロード後に **SSOT のデフォルト表に基づき省略項目を補完する単一ステップ**を通す（明示値は上書きしない／R3.1・R3.4）。
- **スコープ**: 今回は **SHIORI プロファイルのデフォルト値のみ確定**。エンジンプロファイルは**概念＋予約**に留置し、値確定・実装は将来仕様へ（R1.4・R4.2／Out-of-scope 据え置き）。
- 既存実装の含意: Rust serde `Default` と Lua リテラル（`CONFIG.get(...,default)`）に**散在**する補完を、SSOT＋単一適用へ**一本化**する方向（§1.3・§3.3 の SSOT 課題に対する回答）。

### 7.2 `[actor]` の扱い（議題2）
- `[actor]` を**唯一の「デフォルト不能＝必須」**とする（アクター名は `descript.txt` 一致必須・`spot` はゴースト固有）。
- 不在時は **起動を停止せず、軽量な警告（ログまたは通知）で判別可能**にする（R2.3）。fail-fast は採らない。Out-of-scope の「バリデーション強化」を超えない軽量実装に限定。

### 7.3 議題3（R6.4 の検証水準・自明修正）
- 「起動確認」を **既存のローダ／統合テスト水準**（`startup_test.rs` / `config_actors_initialization_test.rs` 等の層）と明確化。フル DLL の e2e 起動は要求せず、CI で実行可能な回帰で担保（R6.4 文言を是正）。

### 7.4 設計フェーズへ持ち越す判断（更新）
§6 の申し送りのうち、ディスカッションで**確定したもの**と**設計へ残すもの**を区別する。
- 確定済み: 核モデル（7.1）／`[actor]` 唯一必須＋警告方針（7.2）／検証水準（7.3）。
- 設計で決める（残置）:
  - (a) **SSOT の物理的1本化先**: Rust 側にデフォルト表を集約するか、Lua 権威＋同期か（§3.3）。「単一適用ステップ」をどの層（Rust ローダ / Lua 補完）に置くか。
  - (b) **`pasta_patterns` の SHIORI デフォルト値の具体確定**: 最小構成で慣例配置（`dic` 直下 `*.pasta`）の辞書が読み込まれること（R2.5）を満たす glob を決定（§3.1）。`dic/**/*.pasta` 等への拡張可否と後方互換影響を精査。
  - (c) **`[actor]` 不在警告の実装手段**: `@pasta_log` warn / OnBoot 通知 / Rust 検出のいずれか（§3.2）。
  - (d) **テンプレ2層の提供先と SSOT 同期方式**: doc 主体／サンプル実体／生成器、および既存 drift-check 基盤の流用可否（§3.5）。

---

## 8. 設計フェーズ Synthesis 結果（2026-06-12 反映）

設計（`/kiro-spec-design`）で discovery（light）と synthesis を実施し、§7.4 の設計判断を確定した。design.md を正本とし、本節は決定の根拠ログ。

### 8.1 統合点の確定（discovery）
- 設定生成の唯一の関所は `PastaConfig::parse`（`config.rs:59`、`loader/mod.rs:110` の Phase 1 で `load` 経由呼出）。ここで `[loader]` 抽出後に `custom_fields` を構築するため、**`parse` 戻り直前が単一補完ステップの正しい置き場**。Rust 消費・Lua 公開（`register_config_module` → `@pasta_config`）の双方が補完後を見る。
- glob `**` の挙動: `discovery.rs` の既存テスト（`**/*.pasta`）で動作確認済み。`dic/**/*.pasta` は直下 `*.pasta`（現行既定 `dic/*/*.pasta` では除外＝`test_discover_excludes_root_dic`）も拾うため、§3.1 の沈黙失敗を既定変更のみで解消できる。

### 8.2 Synthesis（3レンズ）
- **Generalization**: R1/R2/R3/R5 はいずれも「デフォルトの一元管理」の変種。単一の `apply_shiori_defaults`＋SSOT で同時に満たす。2プロファイルは**インターフェースの一般化**（プロファイル選択）として持ち、エンジンは実装しない（予約）。
- **Build vs Adopt**: 補完は既存 serde `Default` 規約を踏襲（`GhostConfig` は `TalkConfig` を模倣）。警告は既存 `tracing::warn`（`RuntimeConfig::validate_and_warn` と同型）。glob は `**` を adopt。SSOT ガードは `insta`/等価テストで足り、生成器は作らない。
- **Simplification**: `[ghost]` を `custom_fields` から**抽出しない**（`GhostConfig` は値供給のみ）。Lua フォールバックリテラルは test-env 安全のため残置（Rust 権威）。エンジンプロファイル・テンプレ生成器は作らない。

### 8.3 §7.4 設計判断の確定
- (a) **SSOT 物理化先 / 補完層**: **Rust（`config.rs`）に集約**。`GhostConfig`＋`default_*()` が値の SSOT、補完は `parse` 内 `apply_shiori_defaults`。
- (b) **`pasta_patterns` 既定値**: `["dic/**/*.pasta"]`（flat+nested 網羅）。既定依存ゴーストへは加算的影響（Revalidation Trigger 記載）。
- (c) **`[actor]` 不在警告**: Rust ローダで `tracing::warn`（起動継続）。Lua `store.lua` は従来どおり no-op で安全。
- (d) **テンプレ2層 / SSOT 同期**: テンプレは doc（`pasta-toml.md` リファレンス）主体＋サンプル実体（hello-pasta）。値は SSOT 由来とし `config_defaults_test` で乖離を固定。文章同期は既存 book drift-check を任意流用。

### 8.4 要確認の回帰（実装時フック）
- `pasta_sample_ghost` の配布検証テスト（`integration_test.rs` / `dist_src_validation_test.rs`）が `[package]` 除去後も通るか確認（`[package]` 存在を前提する assert があれば是正）。
- `discovery.rs` テスト群の期待値更新（`dic/**` 既定化で直下 `*.pasta` が読込対象化）。
