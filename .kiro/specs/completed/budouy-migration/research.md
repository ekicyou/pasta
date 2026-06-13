# Gap Analysis: budouy-migration

分かち書きライブラリ依存を `budoux 0.1.1` → `budouy 0.2.2`（`vendored-models`）へ差し替える移行作業のギャップ分析。要件（requirements.md）と既存コードベースの溝を整理し、設計フェーズへ引き継ぐ判断材料を提供する。

## 1. 現状調査（Current State）

### 影響範囲（実コード）
| 資産 | パス | 現状 | budoux 依存箇所 |
| ---- | ---- | ---- | --------------- |
| workspace 依存定義 | `Cargo.toml:40` | `budoux = "0.1.1"` を `[workspace.dependencies]` で集中管理 | 1 行 |
| pasta_lua 依存参照 | `crates/pasta_lua/Cargo.toml` | `budoux.workspace = true` で参照（要確認・後述） | 1 行 |
| 状態保持 | `crates/pasta_lua/src/sakura_script/mod.rs:39,64` | `SakuraScriptState.budoux_model: budoux::Model`、`budoux::models::default_japanese_model().clone()` で初期化 | 2 箇所 |
| 改行本体 | `crates/pasta_lua/src/sakura_script/line_breaker.rs:101-150` | `break_lines_impl(..., model: &budoux::Model)` 内で `budoux::parse(model, &plaintext)` を呼び `Vec<String>` を幅閾値ループへ | シグネチャ1 + 呼出1 + ループ |
| ユニットテスト | `line_breaker.rs:171-379` | `model()` ヘルパが `budoux::models::default_japanese_model()` を返す。型 `&budoux::Model` 直書き | ヘルパ1 + 各テスト呼出 |
| 統合テスト | `crates/pasta_lua/tests/sakura_script/budoux_test.rs` | Lua API 経由（`SAKURA.break_lines` / `talk_to_script`）。budoux 型に**非依存**（Lua 文字列入出力のみ） | なし（API 不変なら無修正） |

### 重要なアーキテクチャ事実
- **改行アルゴリズム本体（トークン化 `tokenize_plain_chars`・幅閾値ロジック Phase 3・タグ保持 Phase 4）は budoux に非依存**。budoux が触れるのは Phase 2 の「平文 → 単語分割」1 行（`line_breaker.rs:120`）のみ。
- 依存は **workspace で集中管理**。差し替えは `Cargo.toml` 1 箇所で全 crate に波及。
- Lua 公開 API（`break_lines` / `talk_to_script`）は文字列 in/out で、内部の分割ライブラリ型を一切露出しない → **要件 2.4（公開 API 不変）は構造的に保証されやすい**。
- 統合テスト `budoux_test.rs` は Lua レベルのブラックボックス。ライブラリ差し替えで型修正は不要、挙動（`\n` 挿入）のみ検証 → 模型差で結果が変われば assert（`result.contains("\\n")` 等の緩い検査が主体）が壊れるか要確認。

## 2. 新旧 API 差分（budouy 0.2.2、docs.rs 確認済み）

| 項目 | budoux 0.1.1（現状） | budouy 0.2.2（移行先） |
| ---- | -------------------- | ---------------------- |
| 既定モデル取得 | `budoux::models::default_japanese_model() -> &'static Model`（`.clone()` で所有） | `budouy::model::load_default_japanese_parser() -> Parser`（Parser がモデルを所有） |
| 状態保持型 | `budoux::Model` | `budouy::Parser` |
| 分割呼出 | `budoux::parse(&model, &plaintext)` | `parser.parse(&plaintext)`（`&self` メソッド） |
| 戻り値型 | `Vec<String>`（所有） | `Vec<&str>`（**入力 `plaintext` を借用**） |
| feature | （なし／モデル同梱） | `vendored-models` で既定モデルを同梱（`load_default_japanese_parser` はこの feature 前提の公算大 → 要確認） |
| ライセンス | MIT | **Apache-2.0** |
| 提供 feature | — | `std`(default), `alloc`, `vendored-models`, `html`, `cli`, `wasm` |

### この差分が生む唯一の非自明ポイント
現状 Phase 3 の幅計算ループは `for word in &words { word.word_width(word.as_str()); word.chars().count() }`（`words: Vec<String>` → `word: &String`、`as_str()` 可）。budouy は `Vec<&str>` を返すため `word: &&str` となり **`as_str()` が使えない**。`UnicodeWidthStr::width_cjk(word)`（`&str` を直接）/ `word.chars().count()` へ最小修正が必要（`line_breaker.rs:131,149` 周辺）。これが brief 記載「出力 chunk の型に合わせ幅計算ループを最小限調整」の実体。借用元 `plaintext` は同一スコープのローカルで `words` と寿命が一致するためライフタイム問題なし。

## 3. Requirement → Asset マップ（ギャップタグ）

| 要件 | 対応資産・作業 | ギャップ |
| ---- | -------------- | -------- |
| 1.1–1.3 依存差し替え | `Cargo.toml:40`、`pasta_lua/Cargo.toml` | Constraint（pasta_lua 側の参照記述要確認） |
| 1.4 Cargo.lock から budoux 除去 | `Cargo.lock` 再生成 | Missing（lock 更新の確実な実施） |
| 1.5 vendored-models 同梱 | feature 指定 | Constraint（`load_default_japanese_parser` の feature 依存を要確認） |
| 2.1–2.3 改行挙動互換 | `line_breaker.rs` Phase 2 差し替え＋Phase 3 最小調整 | **Unknown**（模型差で分割位置が変化しうる） |
| 2.4 公開 API 不変 | `mod.rs` register／Lua 関数 | なし（型非露出で構造的に保証） |
| 2.5 空入力ガード | `mod.rs:102,191`、`line_breaker.rs:107` | なし（既存ロジック流用） |
| 3.1–3.3 テスト緑化 | `line_breaker.rs` ユニット＋`budoux_test.rs` | **Unknown**（模型差で期待値ズレの可能性） |
| 4.1–4.3 build/clippy/test | 全体 | Constraint（`Vec<&str>` 化に伴う warning 解消） |
| 5.1–5.2 ドキュメント同期 | 後述「ドキュメント波及」一覧 | Missing（記載漏れ防止） |
| 6.1 ライセンス | `deny.toml` | なし（Apache-2.0 は `deny.toml:5` で許可済み・GPL 非該当） |
| 6.2 MSRV 1.88.0 | CI/開発環境 | なし（CI は `dtolnay/rust-toolchain@stable` で常時最新・pin ファイル不在） |

### ドキュメント波及（要件 5 の作業対象、grep 確認済み）
- `.kiro/steering/tech.md:43`（`budoux 0.1.1` 記載）
- `.kiro/steering/product.md:55`（`budoux-line-breaker` / BudouX）
- `.claude/skills/pasta-lua-coding/references/runtime-api.md`（複数箇所：説明文・`budoux = [10,12]` 例）
- `.claude/skills/pasta-ghost-authoring/SKILL.md:314`、`references/pasta-toml.md`（複数箇所）
- `book/src/lua/modules.md:164`
- **判断要**: `actor.budoux` という pasta.toml の**設定フィールド名**は Lua 公開仕様（要件で「公開 API 不変」）。クレート名変更がフィールド名 `budoux` の改称を含むかは設計判断。本 spec の scope は「クレート名・バージョン記載の同期」であり、ユーザー向け設定キー改称は公開 API 変更（scope 外）に該当しうる → **設計で要明確化**。

## 4. 実装アプローチ（Options）

分割ライブラリ呼出が 1 モジュールに局所化されているため、構造的選択肢は限定的。実質は「既存実装の最小改変（Option A）」が妥当で、B/C は過剰。

### Option A: 既存実装を最小改変（推奨）
`Cargo.toml` 依存 1 行 + `sakura_script` モジュール 3 点（状態型・初期化・parse 呼出）+ 幅計算ループ 1 点 + テストヘルパを修正。改行アルゴリズム・タグ保持・幅閾値ロジックはそのまま再利用。
- ✅ 変更が局所・差分最小、既存テスト構造を流用（要件 3.2 と整合）
- ✅ 公開 API・設定フィールド・Lua スクリプトに無影響
- ❌ 模型差による期待値ズレの吸収は別途必要（Option 非依存の課題）

### Option B: 改行処理を抽象化（分割 backend を trait 化）
`Segmenter` trait を切り、budoux/budouy を差し替え可能にする。
- ✅ 将来の再差し替え・A/B 比較が容易
- ❌ 本 spec の scope（単純差し替え）を超える over-engineering。要件外の抽象化は brief「変更を局所化」方針に反する → **非推奨**

### Option C: ハイブリッド（段階移行・両ライブラリ併存）
feature flag で budoux/budouy を切替可能にして段階移行。
- ✅ ロールバック容易
- ❌ 要件 1.1/1.4「budoux を Cargo.lock 含め完全除去」と矛盾。併存は scope 外 → **非推奨**

## 5. 工数・リスク

- **Effort: S（1–3 日）** — 変更点は依存1 + コード4 + テストヘルパ + ドキュメント数ファイル。アルゴリズム本体は無改変。既存パターンの踏襲。
- **Risk: Low〜Medium** — コード差し替え自体は Low（局所・型差のみ）。**Medium 要因は「模型差による分割位置の変化」**（要件 2.1/3.3 の Unknown）。budoux→budouy で既定日本語モデルの分割境界が一致する保証はなく、`line_breaker.rs` の固定期待値テスト（特に `\n` 位置を厳密検証する箇所）が割れる可能性。ただし大半のテストは「`\n` を含む／平文保持」という緩い検査で、厳密な分割位置を固定する assert は限定的。

## 6. Research Needed（設計フェーズへ持ち越す未確定事項）
1. **模型差の影響実測**: budouy 既定日本語モデルの分割結果が budoux と一致するか。差異が出るテストの特定と、期待値更新の妥当性判断（要件 3.3）。→ 設計後の実装で実テスト実行が確実。
2. **`vendored-models` と `load_default_japanese_parser()` の関係**: 既定パーサー取得がこの feature gate 下にあるか（feature 無効時にコンパイルエラーにならないか）を docs/ソースで確定。
3. **pasta_lua/Cargo.toml の参照記述**: `budoux.workspace = true` の実記述を確認し、`budouy.workspace = true`（+ feature 指定を workspace 側へ）へ正しく置換。feature `vendored-models` を workspace 集中定義（`budouy = { version = "0.2.2", features = ["vendored-models"] }`）に置くか各 crate で有効化するかの方針決定。
4. **設定フィールド名 `budoux` の扱い**: pasta.toml の `actor.budoux` キーを改称するか維持するか（公開 API 不変 vs ドキュメント整合）。→ scope 解釈の明確化が必要。
5. **`budouy::Parser` の Send/Sync**: `Arc<SakuraScriptState>` 経由で Lua クロージャに `move` 共有するため、Parser が必要な auto-trait を満たすか（budoux::Model は問題なかった。Parser も plain data 想定だが要確認）。

## 7. 設計フェーズへの提言
- **採用アプローチ**: Option A（最小改変）。trait 抽象化・feature 併存は scope 外として明示排除。
- **主要設計判断**: (a) `vendored-models` の feature 指定位置（workspace 集中 vs crate 個別）、(b) pasta.toml `actor.budoux` キー名の維持/改称の確定、(c) 模型差で割れるテストの期待値更新ポリシー（個別妥当性判断の手順を tasks に組込む）。
- **検証戦略**: `cargo build/clippy/test --workspace` の三点緑化を完了ゲートに。模型差テストは「平文保持 + `\n` 存在」の不変条件を優先し、厳密な分割位置 assert は実測後に個別更新。
- **完全除去の確認**: `Cargo.lock` から `budoux` エントリ消失を grep で確認する手順を tasks に明記（要件 1.4）。

---

## 8. 要件ディスカッション結果（2026-06-13）と設計判断の引き継ぎ

要件ディスカッション（`/kiro-requirements-discussion`）で以下を確定。要件側へ反映済み。

### 確定事項（要件へ反映済み）
- **C1: 互換の定義 = 機能的同等**。分割位置の文字単位一致は要さず、「自然な分かち書き位置で改行が挿入され、平文・タグが保持される」ことを基準とする。模型差で位置が変わるテストは Req 3.3 に従い個別判断で期待値更新（Req 2.1/2.2 を緩和済み）。
- **C2/C3: 内部クレート名と外部機構名の区別**。外部呼称「budoux / BudouX」は正式名として**維持**、内部依存クレートのみ `budouy` へ差し替える。文書更新対象は実質 `tech.md` の依存一覧1箇所。公開設定キー `actor.budoux`、`book/` の機構記載、完了 spec 名 `budoux-line-breaker` はすべて維持（Req 5 を内部/外部区別で再構成、Boundary Out of scope に明記済み）。

### 設計フェーズへ先送りする設計判断（カテゴリB）
- **B1**: `vendored-models` feature と `budouy::model::load_default_japanese_parser()` の依存関係を確定（feature 無効時にコンパイルエラーとなるか、既定パーサー取得がこの feature gate 下にあるか）。
- **B2**: 幅計算ループの `Vec<&str>` 対応。budouy `parser.parse()` は `Vec<&str>`（入力借用）を返すため、現状 `Vec<String>` 前提の `word.as_str()`（`line_breaker.rs:131,149` 周辺）を `&str` 直接参照へ最小修正。借用元 `plaintext` と寿命一致でライフタイム問題なし。
- **B3**: `budouy::Parser` の `Send`/`Sync` 充足確認（`Arc<SakuraScriptState>` 経由で Lua クロージャへ `move` 共有するため）。
- **B4**: 模型差で割れるテストの期待値更新「手順」を tasks に具体化（「平文保持 + `\n` 存在」の不変条件を優先、厳密な分割位置 assert は実測後に個別更新）。

### 確定済み（設計判断ではなく既知の実装事実）
- `crates/pasta_lua/Cargo.toml:29` の `budoux.workspace = true` → `budouy.workspace = true` へ置換。`vendored-models` feature は workspace 集中定義（`budouy = { version = "0.2.2", features = ["vendored-models"] }`）に置く。

---

## 9. 設計フェーズ discovery: budouy 0.2.2 API 確定（docs.rs ソース確認・2026-06-13）

設計フェーズで docs.rs のソース・型ページを精査し、B1〜B3 を確定。**移行はギャップ分析想定よりさらに小さい**。

| 確認項目 | 確定結果 | 出典 | 設計影響 |
| -------- | -------- | ---- | -------- |
| `Parser::parse` 戻り値型 | **`pub fn parse(&self, sentence: &str) -> Vec<String>`**（所有権つき、budoux と同型） | docs.rs `src/budouy/parser.rs` | **B2 解消**: 幅計算ループ（`word.as_str()` / `word.chars().count()`）は**変更不要**。`Vec<&str>` 想定の調整は不要だった。 |
| 既定パーサー取得 | `budouy::model::load_default_japanese_parser()`（`vendored-models` feature gate 下、`Parser` を返す） | docs.rs `budouy::model` | **B1 解消**: 既定パーサー取得は `vendored-models` 前提。同 feature を有効化すれば取得可。 |
| `Parser` の auto-trait | **`Send + Sync + Clone + Debug + Unpin`** | docs.rs `struct.Parser` | **B3 解消**: `Arc<SakuraScriptState>` 経由の `move` 共有・スレッド間共有に問題なし。 |
| `Model` | 型エイリアス（BudouX モデルデータ）。`Parser::new(Model)` でパーサーが所有 | docs.rs `budouy::model` | `SakuraScriptState` は `Model` を直接持たず `Parser` を保持（パーサーが模型を所有） |

**確定した最小変更点（コンパイル不通箇所のみ・要件 2.7 準拠）**:
1. `Cargo.toml`: `budoux = "0.1.1"` → `budouy = { version = "0.2.2", features = ["vendored-models"] }`。
2. `crates/pasta_lua/Cargo.toml:29`: `budoux.workspace = true` → `budouy.workspace = true`。
3. `sakura_script/mod.rs`: フィールド型 `budoux::Model` → `budouy::Parser`、初期化 `budoux::models::default_japanese_model().clone()` → `budouy::model::load_default_japanese_parser()`、2 呼出箇所の参照渡し。
4. `sakura_script/line_breaker.rs`: `break_lines_impl` の引数型 `&budoux::Model` → `&budouy::Parser`、`budoux::parse(model, &plaintext)` → `parser.parse(&plaintext)`。**幅計算ループ・タグ保持・再構築は無変更**。
5. `line_breaker.rs` テストヘルパ: `fn model() -> &'static budoux::Model` → 所有 `Parser` を返すヘルパへ（budouy は `&'static` でなく所有を返すため、呼出側は `&parser` 渡しへ追従）。
6. `tests/sakura_script/budoux_test.rs`: budoux 型に非依存（Lua API 経由）のため**コード変更なし**。模型差で挙動が変わればのみ Req 3.3。
7. `Cargo.lock` 再生成（budoux エントリ除去）、`tech.md:43` 依存記載更新。

**残る唯一の実装時確認**: `load_default_japanese_parser()` の戻りが `Parser` 直か `Result<Parser, _>` か（vendored JSON パース失敗の表現）。いずれにせよコンパイラが差分を指摘するため、最小修正（`Result` なら `expect`）で吸収。これは設計を阻害しない。

---
**Adjacent spec 注記**: 母体 spec `ukagaka-desktop-mascot`（`budoux-line-breaker` 機能内包）は本移行の裏側差し替えのみで機能境界は不変。公開 API・設定キー・外部機構名を維持する限り回帰なし。
