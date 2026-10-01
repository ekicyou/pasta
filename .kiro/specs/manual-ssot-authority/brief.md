# Brief: manual-ssot-authority

> **ステータス**: 未着手（discovery 完了・2026-10-01）。`/kiro-start manual-ssot-authority` で開始する。

## Problem

pasta の利用者向け情報（文法・Lua API・設定）が複数の場所へ重複して書かれ、権威の所在が混乱している。

- 文法は 4 箇所に存在する: `doc/spec/`（「実装判断の権威」を自称）・`book/src/grammar/`（mdBook、drift-check で doc/spec に追従）・`GRAMMAR.md`（ルート 831 行のクイックリファレンス）・`pasta-ghost-authoring` スキル `references/`（手書き写し・同期機構なし）。
- Lua API は `book/src/lua/`（679 行）と `pasta-lua-coding` スキル `references/`（`runtime-api` 710 行 + `shiori-handlers` 442 行）に並存し、**スキル側の方が圧倒的に詳しい**。
- `pasta.toml` リファレンスはスキル側（`pasta-toml.md` 409 行）にしかなく、マニュアルに章が無い。しかも `crates/pasta_lua/tests/loader/config_defaults_test.rs` がこのスキルファイルを SSOT として直接読む。
- `book/AUTHORING.md` はスキルを mdBook 章の「起草元」と位置付けており、権威の向きが逆転している。

ユーザーは 2026-06-08（pasta-manual-debugging の discovery）で「mdBook に書いている項目は mdBook を権威にしたい」と明言している。

スキル（`pasta-ghost-authoring` / `pasta-lua-coding`）は**別リポジトリへコピーしてゴースト開発に使う**ため、自己完結していなければならない（リポジトリ内パス参照は持ち出し先で切れる）。

## Current State

- mdBook マニュアル（`book/`）は GitHub Pages（ekicyou.github.io/pasta）へ公開済み。CI（`.github/workflows/manual.yml`）で `drift-check.mjs` / `verify-drift-gate.mjs` が `book/manual-sources.toml` のハッシュマーカーで doc/spec との乖離を検出している。
- mdBook 章は「お嬢様口調の導入段落 → `---` → 規範的本文」という統一構造を持つ。
- `doc/spec/` ch08（属性・未実装）と ch12（将来仕様・145 行）は mdBook に対応章がない。
- doc/spec を参照する箇所: steering 各種（grammar/product/structure/tech/workflow/roadmap）・README・SOUL.md・OPTIMIZATION.md・スキル・kiro-complete スキル・review-improvement-loop spec 等。

## Desired Outcome

- **mdBook が利用者向け情報の唯一の権威**である。`doc/spec/` と `GRAMMAR.md` は存在しない（GRAMMAR.md はマニュアルへの案内のみ残す）。
- スキル `references/` のうち利用者向け規範部分は **mdBook から自動生成**され、スキルは自己完結したまま別リポジトリへ持ち出せる。生成物の鮮度は CI で保証される。
- スキル固有の AI 作業手順（作例・パターン・コーディング規約・テスト/lint）はスキル手書きが権威として残り、生成部分と手書き部分の区別がファイル単位で明示されている。
- 権威の移行と生成への切替が**一度に完了**し、「mdBook が権威なのにスキルが古い手書き」という過渡状態を出荷しない。

## Approach

**読者で線引きする（乙案）＋ 1 spec で一括完了。**

| 内容 | 権威 | スキルでの扱い |
|---|---|---|
| 文法（doc/spec 吸収） | mdBook | 生成 |
| 公開 Lua API（`runtime-api`・`shiori-handlers` を mdBook へ移し厚くする） | mdBook | 生成 |
| `pasta.toml` リファレンス（mdBook に新章、テストの読み先も付け替え） | mdBook | 生成 |
| 作例・パターン・コーディング規約・テスト/lint | スキル | 手書き |
| `internal-modules`（内部モジュール解説） | スキル（暫定） | 手書き。将来 `pasta-runtime-internals-doc` へ申し送り |

- 生成はリポジトリ内スクリプトで mdBook 章から導入段落（先頭〜最初の `---`）を落とした規範本文をスキル `references/` へ書き出す（詳細は設計）。
- `drift-check` / `manual-sources.toml` は doc/spec 廃止に伴い撤去し、代わりに「生成物鮮度チェック」を CI に置く。
- 却下: 甲（スキル内容を全て mdBook に吸収＝マニュアルが内部向け内容で倍化）、丙（重なる部分のみ＝pasta.toml 等が利用者マニュアルに載らないまま）、スキルを mdBook パス参照化（持ち出し先で切れる）、手書き写し維持＋方針明記のみ（乖離リスク残存）、2 spec 分割（過渡期にスキルが陳腐化）。

## Scope

- **In**:
  - `doc/spec/` の mdBook への吸収と廃止（ch08/ch12 の行き先は設計で決定）
  - `GRAMMAR.md` の廃止（案内のみ残す）
  - mdBook Lua API 章の拡充（スキル `runtime-api` / `shiori-handlers` の内容を移す）
  - mdBook `pasta.toml` リファレンス章の新設と `config_defaults_test.rs` の読み先付け替え
  - mdBook → スキル `references/` 生成スクリプトと CI 鮮度チェック
  - スキル内の生成/手書き区分の明示（ファイル単位）と `SKILL.md` の更新
  - drift-check 機構（`drift-check.mjs`・`verify-drift-gate.mjs`・`manual-sources.toml`・manual.yml 該当ステップ）の撤去
  - doc/spec・GRAMMAR.md を参照する steering・README・スキル・`book/AUTHORING.md` 等の参照修正
- **Out**:
  - ランタイム内部設計の解説（`pasta-runtime-internals-doc`）
  - `internal-modules` の権威移動（`pasta-runtime-internals-doc` で扱う）
  - 文法・API そのものの変更（記述の移し替えのみ。挙動は変えない）
  - `.kiro/specs/completed/` 内の歴史的記述の書き換え

## Boundary Candidates

- 内容の移し替え（doc/spec 吸収・Lua API 拡充・pasta.toml 章・GRAMMAR.md 廃止・参照修正）
- 生成機構（mdBook → スキル生成・CI 鮮度チェック・drift-check 撤去）

## Out of Boundary

- コントリビュータ向け内部解説（`pasta-runtime-internals-doc` の領域）
- マニュアルのデザイン・シンタックスハイライト（`pasta-manual-syntax-highlight` で完了済み）

## Upstream / Downstream

- **Upstream**: `pasta-user-manual`（mdBook 基盤）、`pasta-manual-syntax-highlight`、`pasta-manual-debugging`
- **Downstream**: `pasta-runtime-internals-doc`（本 spec が確立した「mdBook 権威＋スキル生成」方式に従う）、スキルを持ち出す各ゴーストリポジトリ

## Existing Spec Touchpoints

- **Extends**: なし（completed の pasta-user-manual の drift-check 方式を置き換える）
- **Adjacent**: `review-improvement-loop`（doc/spec を参照しているため参照修正が必要）、`release-workflow`

## Constraints

- mdBook（サーバー不要の静的 HTML+JS）を維持。追加エコシステム依存を持ち込まない（生成スクリプトは既存の node ツールチェーン `book/tools/` 内で完結させる）。
- スキルは別リポジトリへコピーして使うため、生成後のスキルは自己完結（リポジトリ外参照なし）であること。
- 改行は LF 正規化で扱う（CRLF 作業コピーと CI の差で誤検出しない）。
- 部分出荷禁止：権威移行と生成切替は同一 spec 内で完了させる。
