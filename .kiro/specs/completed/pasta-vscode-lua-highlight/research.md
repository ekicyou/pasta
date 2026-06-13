# Gap Analysis: pasta-vscode-lua-highlight

> 既存コードベースと要件（requirements.md）の差分を分析し、設計フェーズの判断材料を提供する。
> 本書は「情報と選択肢」を提示するものであり、最終実装方針を決定しない。

## 1. 分析サマリー

- **本質は2層協調の小規模改修**: 「TextMate 層で `source.lua` を埋め込み注入」＋「LSP 層で Lua ブロック本文を覆う `codeBlock` セマンティックトークンをフェンスのみへ縮小」の2点。新規コンポーネントは不要で、いずれも既存ファイルの拡張で成立する（Option A/C）。
- **最大リスクは SSOT 文法の隣接結合**: 本仕様が改変対象とする `editors/vscode/syntaxes/pasta.tmLanguage.json` を、**完了済み隣接仕様 pasta-manual-syntax-highlight（book 側）が同一ファイルとして読み込み、しかも「`include: source.lua` を持たないこと」を前提に二段トークナイズしている**（`book/tools/highlight/tokenizer.mjs` に「改変不可・5.2」と明記）。brief では book を Out of scope としているが、SSOT 文法を介して境界を跨いで結合しており、設計で必ず調停が要る。
- **回帰ベースラインの欠落**: LSP の codeBlock セマンティックトークンに対する専用テストが存在せず（フィクスチャは `analyze_robustness_test.rs:78` の堅牢性確認のみ）、R3/R4 の検証には新規テストの追加が前提。
- **修正点の特定は完了**: 改変対象3ファイル（tmLanguage.json / visitors.rs / 必要なら book tokenizer.mjs）と、AST・文法構造（`cb.span` がフェンス込み全域、`cb.content` が本文のみ）を確認済み。フェンス行座標は `span.start_line`／`span.end_line` から決定可能。
- **推奨はフェーズ分割した拡張（Option C）**: LSP トークン縮小 → TextMate 注入 → book 調停の順で、各段に回帰ゲートを置く。Effort=M / Risk=Medium。

## 2. 現状調査（Current State）

### 2.1 TextMate 層（VS Code 拡張のシンタックス定義責務）
- ファイル: `editors/vscode/syntaxes/pasta.tmLanguage.json`（`scopeName: source.pasta`、SSOT）
- `lua-code-block`（L129-147）:
  - `begin: ^(\s*)(```)(lua)?\s*$` / `end: ^(\s*)(```)\s*$`
  - `name: meta.embedded.block.lua`、`contentName: meta.embedded.block.lua.content`
  - フェンス言語名キャプチャ: `entity.name.type.language.pasta`（R2 対象）
  - **`patterns`（content への `{ "include": "source.lua" }`）が無い** → 本文は色付けされず単一スコープのまま。**ここが R1 の埋め込み注入ポイント。**
- マニフェスト `editors/vscode/package.json`:
  - `semanticTokenScopes.codeBlock → ["meta.embedded.block.lua"]`（L175-177）
  - `semanticTokenTypes` に `codeBlock`（superType: string、凡例 index 9）。**R4.4 によりこの凡例・並び順は不変であること。**
  - エンジン `^1.85.0`（embedded languages 仕様準拠）。`source.lua` は VS Code 組み込み（追加同梱不要）。

### 2.2 セマンティックトークン層（pasta_lsp 解析責務）
- ファイル: `crates/pasta_lsp/src/analysis/visitors.rs`
  - `visit_code_block`（L109-113）: `cb.span` 全域を `CODE_BLOCK` トークンとして `add_token_from_span` で出力。
  - `add_token_from_span`（L844-）: 複数行スパンを **start_line..=end_line の全行**にわたってトークン化（L876 以降の multiline 分岐）。→ **本文全域が `codeBlock` で覆われ、TextMate の Lua ハイライトを上書きする。ここが R3 の縮小ポイント。**
- TypeScript 提供層: `editors/vscode/src/semanticTokensProvider.ts`
  - `PASTA_TOKEN_TYPES`（L12-30）に `codeBlock`（index 9）。WASM の delta 出力を絶対座標へ変換して push するのみ。**凡例固定（R4.4）なので本ファイルは原則無改変。**

### 2.3 AST・文法構造（改修の前提）
- `crates/pasta_dsl/src/parser/grammar.pest`（L216-219）:
  ```
  code_block    =  { code_open ~ code_contents ~ code_close }
  code_open     = _{ PUSH("`"{3,}) ~ id? ~ eol }   # 開始フェンス＋言語＋改行（silent）
  code_contents = @{ (!PEEK ~ ANY)+ }              # 本文のみ
  code_close    = _{ POP ~ or_comment_eol }        # 終了フェンス（silent）
  ```
- `crates/pasta_dsl/src/parser/ast/action.rs`（L48-55）`CodeBlock { language, content, span }`:
  - `span` = `code_block` ルール全体 = **開始フェンス〜終了フェンス込みの全域**。
  - `content` = `code_contents` = **本文のみ**（ただし行座標スパンは別途保持されない）。
  - → フェンス行のみのトークン化には、`span.start_line`（開始フェンス行）と `span.end_line`（終了フェンス行）から行座標を算出する必要がある。本文専用スパンは AST に無いため、**本文行を「トークン非出力」にする実装**が素直（フェンス行のみ別途出力 or 全行スキップ）。

### 2.4 テスト資産（回帰の土台）
- TextMate: `editors/vscode/src/test/tmGrammar.test.ts`
  - Lua ブロックテスト済み（L227-248）。テスト環境では `source.lua` を**ロードしない**（`loadGrammar` が source.pasta 以外 null）。判定は `meta.embedded OR source.lua` を許容（L237-248）。
  - → `include: source.lua` 追加後も、未解決 include を vscode-textmate が無害化するため既存テストは通る見込み。設計では vendored lua をロードして実スコープを検証する強化が選択肢。
- LSP: `crates/pasta_lsp/tests/semantic_token_test.rs`（基盤・**codeBlock 専用ケース無し**）、`analyze_robustness_test.rs:78`（`＊ｓ\n```lua\nreturn 1\n```\n` の堅牢性フィクスチャ）。
  - → R3（フェンスのみ）・R4（本文非出力・無回帰）の**新規回帰テスト追加が必須**。

## 3. 隣接結合リスク（最重要・Research Needed）

**book ハイライタとの SSOT 文法共有**
- `book/tools/highlight/highlight-html.mjs`（L38-41）: `GRAMMAR_PATHS.pasta = editors/vscode/syntaxes/pasta.tmLanguage.json`。**VS Code 拡張と完全同一の文法ファイルを読む。**
- `book/tools/highlight/tokenizer.mjs`（L3-9）: 明示コメント「**pasta 文法は lua-code-block に `include: source.lua` を持たない（改変不可・5.2）**」を前提に、`meta.embedded.block.lua.content` 区間を検出して**自前 vendored lua 文法（`book/tools/highlight/grammars/lua.tmLanguage.json`）で二段トークナイズ**している。
- **衝突**: 本仕様が `include: source.lua` を文法へ追加すると、この「改変不可」前提が崩れる。book は registry に vendored `source.lua` を登録済み（tokenizer.mjs L131-132）のため、pasta パスの時点で textmate が lua を注入し、二段処理が冗長化／境界相違を生む可能性がある。
- **影響範囲**: book の `tokenizer-test.mjs` / `highlight-html-test.mjs` / `scope-map-test.mjs` が回帰検知。`manual-sources.toml` のドリフトマーカーは doc/spec↔book 章の対応のみで**文法ファイル変更は捕捉しない**ため、ここは別途人手/テストでの確認が要る。
- **設計の決定事項**: (a) tokenizer.mjs の二段ロジックと「改変不可」コメントをどう更新するか、(b) book テストの再実行と期待値更新、(c) そもそも本文注入を VS Code 限定にする回避策が TextMate 埋め込みで成立しないこと（文法は単一 SSOT、別文法フォークは SSOT 原則違反）の確認。
- **要件ディスカッション決定（2026-06-13）**: 本結合リスクへのスコープ判断として、**book ハイライタの無回帰を本仕様の In scope とする**ことが確定した（requirements.md R4.AC5 を追加、Boundary Context を更新）。book ハイライト機能仕様そのものは pasta-manual-syntax-highlight が引き続き担当するが、SSOT 文法改変による book の無回帰調停は本仕様が担保する。したがって設計では上記 (a)/(b) を本仕様の作業として計画すること。

## 4. 要件→資産マッピング（Requirement-to-Asset Map）

| 要件 | 対象資産 | ギャップ種別 | メモ |
| --- | --- | --- | --- |
| R1 Lua 自動ハイライト | tmLanguage.json `lua-code-block` content | **Missing** | `patterns: [{include: source.lua}]` 追加。`source.lua` は VS Code 組み込み |
| R2 フェンスの pasta スコープ保持 | tmLanguage.json begin/endCaptures | Constraint | `entity.name.type.language.pasta` 等を不変保持。content 注入が境界を侵さないこと |
| R3 codeBlock トークン範囲縮小 | visitors.rs `visit_code_block` / `add_token_from_span` | **Missing** | 本文全域→フェンスのみ。`cb.span` 始終行から算出 |
| R4 無回帰（ブロック外・凡例） | semanticTokensProvider.ts 凡例、既存 LSP テスト群 | Constraint / **Unknown** | 凡例不変。codeBlock 専用回帰テストが無く baseline 不足 |
| R5 範囲外不提供（トグル/インライン/言語サービス） | （既存に該当機能なし） | Constraint | 追加しないことの確認のみ。`pasta.debug.toggleSourcePresentation` は別系統で混同回避 |
| 隣接 book 調停 | book/tools/highlight/tokenizer.mjs | **Unknown（最重要）** | §3 参照。SSOT 文法共有による境界跨ぎ結合 |

## 5. 実装アプローチ案

### Option A: 既存コンポーネント拡張（最小改修）
- tmLanguage.json `lua-code-block.patterns` に `source.lua` を include、visitors.rs `visit_code_block` をフェンスのみ出力へ変更。
- ✅ 新規ファイル無し・確立された embedded language 手法・brief 採用方針と一致。
- ❌ SSOT 文法共有のため book 側へ波及。book 調停を Out of scope のまま放置すると回帰。

### Option B: 新規コンポーネント分離
- 別 lua 注入文法や別 visitor を新設。
- ❌ 文法は単一 SSOT（book と共有）であり、フォークは SSOT 原則違反・二重管理。embedded language は既存 `lua-code-block` の改変が不可避。**本ケースには不適。**

### Option C: フェーズ分割した拡張（推奨）
- **Phase 1**: LSP の `visit_code_block` をフェンスのみ出力へ縮小＋R3/R4 回帰テスト追加（TextMate と独立に検証可能）。
- **Phase 2**: tmLanguage.json に `source.lua` 注入＋tmGrammar.test.ts 強化（vendored lua ロードで実スコープ検証）。
- **Phase 3**: book tokenizer.mjs の二段ロジックと「改変不可」前提を調停し、book テスト再実行・期待値更新。
- ✅ 各段に回帰ゲート。最大リスク（book 結合）を明示的に最終段で封じ込め。
- ❌ 計画がやや複雑。Phase 間の整合管理が必要。

## 6. Effort / Risk

- **Effort: M（3–7 日）** — コード差分自体は小さいが、文法(JSON)＋Rust LSP＋book(JS) と3つのテストスイート（tmGrammar / pasta_lsp / book highlight）を横断し、合成結果（セマンティック×TextMate）の可視検証が必要。
- **Risk: Medium** — 主たる未知は §3 の book SSOT 結合。加えて「セマンティック優先合成で Lua 色が実際に見える」ことの自動検証手段が単体テストでは捉えにくい（手動 VS Code 確認 or vendored lua を用いた合成検証の設計が要る）。

## 7. Research Needed（設計フェーズへ持ち越し）

1. **book 二段トークナイザの調停**: pasta 文法が `include: source.lua` を獲得した際の book 挙動（冗長二重トークナイズ／スコープ境界相違）を実測し、tokenizer.mjs のロジックと「改変不可・5.2」コメントの更新要否・book テスト期待値の改訂方針を確定する。
2. **フェンスのみトークン出力の精密化**: `cb.span` の始終行からフェンス行範囲を算出し、本文行に codeBlock を出さない実装の境界条件（空本文、フェンス末尾の余分文字、4 本以上のバッククォート、`or_comment_eol` の扱い）を確認。
3. **合成可視性の検証戦略**: 「Lua スコープが実際に表示される（codeBlock に隠れない）」ことを自動検証する手段（vendored lua をロードした合成テスト、または LSP が本文行に codeBlock を出さないことの厳密アサート）を設計。
4. **R2 フェンス言語スコープの保持確認**: content 注入後も `entity.name.type.language.pasta` 等のフェンス側スコープが維持されること。

## 8. 設計フェーズへの推奨

- **推奨アプローチ**: Option C（フェーズ分割した拡張）。brief 採用方針（TextMate 注入＋セマンティック縮小）を踏襲しつつ、book 調停を独立フェーズとして明示し回帰ゲートで封じる。
- **主要決定事項**: (1) フェンスのみ出力の実装形（フェンス行別出力 vs 全行非出力）、(2) book 二段ロジックの去就、(3) 合成可視性の自動検証手段。
- **持ち越し研究**: §7 の 1〜4。特に #1（book 結合）は設計開始時の最優先調査項目。

---

## 9. 設計判断（Design Decisions・2026-06-13 design フェーズ）

### 9.1 注入方式: VS Code 注入文法（injection grammar）を採用（方式A・ユーザー承認 2026-06-13）

- **決定**: 共有 SSOT 文法 `editors/vscode/syntaxes/pasta.tmLanguage.json` は **改変しない**。代わりに**新規の注入文法ファイル**（`editors/vscode/syntaxes/pasta-lua-injection.tmLanguage.json`）を追加し、`package.json` の `grammars[].injectTo: ["source.pasta"]` で `meta.embedded.block.lua.content` スコープへ `source.lua` を注入する。
- **brief 当初案からの変更**: brief は「`lua-code-block` content に `{ include: source.lua }` を直接注入」（方式B）を採用していたが、要件ディスカッション #1 で **book 無回帰を In scope** とした結果、book が読む SSOT 文法を改変しない方式Aの方が目的（book 無回帰）を**構造的に**達成でき、総変更量・リスクとも小さいため方式Aへ更新。
- **Build vs Adopt**: VS Code 標準の injection grammar 機構（`injectTo` / `injectionSelector`）をそのまま採用。独自ハイライト実装は行わない。`source.lua` はランタイムで VS Code 組み込み、テストでは **editors/vscode 配下に vendored したテスト専用 lua 文法**（`src/test/fixtures/lua.tmLanguage.json`・設計ディスカッション #1 で決定）で代替し、`getInjections` で注入を結線する。book の vendored lua へはツリー間依存しない。
- **book への影響**: SSOT 文法も book コードも無改変ゆえ、book の二段トークナイザ（自前 vendored lua）は**現状のまま無影響**。R4.5 は構造的に充足（検証として book ハイライトテストを実行）。tokenizer.mjs の「改変不可・5.2」前提は**維持される**（§3 の調停作業は方式A採用により不要化）。

### 9.2 セマンティックトークン縮小: フェンス行のみ出力（R3）

- **決定**: `visit_code_block` を、`cb.span` 全域出力から **開始フェンス行（`span.start_line`）・終了フェンス行（`span.end_line`）のみ** codeBlock トークンを出力する形へ変更。本文行（フェンス間）には codeBlock を一切出力しない（全行非出力ではなくフェンス保持を採用＝フェンスの視覚マーカーを維持しつつ本文を解放）。
- **凡例不変（R4.4）**: トークン種別の並び順・凡例（`token_types.rs` / `semanticTokensProvider.ts` の `PASTA_TOKEN_TYPES`）は変更しない。

### 9.3 Simplification / 並行性

- 注入文法（TextMate・editors/vscode）と LSP 縮小（Rust・pasta_lsp）は**相互依存なし**＝独立タスクとして並行実装可能。新規抽象レイヤーは設けない（最小構成）。
