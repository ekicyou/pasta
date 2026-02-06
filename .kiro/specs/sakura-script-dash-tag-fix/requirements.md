# Requirements Document

## Introduction

ukadoc（https://ssp.shillest.net/ukadoc/manual/list_sakura_script.html）の公式タグリストを網羅分析した結果、現在のPasta DSLの `sakura_token` 文字クラス `[!_a-zA-Z0-9]` では以下の5文字が不足しており、対応するさくらスクリプトタグを認識できないことが判明した：

| 不足文字 | 該当タグ例 | 機能 |
|---------|-----------|------|
| `-` (U+002D) | `\-` | ゴースト終了 |
| `+` (U+002B) | `\+`, `\_+` | ランダム交代、順次交代 |
| `*` (U+002A) | `\*` | 選択タイムアウト無効 |
| `?` (U+003F) | `\_?` | タグ表示モード |
| `&` (U+0026) | `\&[ID]` | エンティティ参照 |

現在のPasta DSLでは以下の4箇所すべてでこれら5文字が許容文字に含まれていない：

1. **言語仕様書** (`doc/spec/07-sakura-script.md`): `sakura_token ::= [!_a-zA-Z0-9]+` — 5文字なし
2. **文法リファレンス** (`GRAMMAR.md`): `sakura_token ::= [!_a-zA-Z0-9]+` — 同上（複製）
3. **Pest文法定義** (`grammar.pest`): `sakura_id = @{ ... "_" | "!" ... }` — 5文字なし
4. **ランタイムregex** (`tokenizer.rs`): `r"\\[0-9a-zA-Z_!]+"` — 5文字なし

本仕様は、これら4箇所を一貫して修正し、ukadocに記載されたすべてのさくらスクリプトタグを正しく認識・透過させることを目的とする。

### スコープ決定根拠

ukadocの公式タグリストを権威的情報源とし、そこに存在しない記号は対応不要とする。分析の結果、上記5文字の追加で ukadoc記載の全タグを網羅できることを確認済み。

## Requirements

### Requirement 1: 仕様書・文法リファレンスのsakura_token定義に5文字を追加

**Objective:** 仕様書利用者として、`\-`, `\+`, `\*`, `\_?`, `\&[ID]` 等のukadoc記載タグが有効なさくらスクリプトタグであることを仕様レベルで明確にしたい。これにより実装と仕様の乖離を防ぐ。

#### Acceptance Criteria
1. The pasta DSL specification (`doc/spec/07-sakura-script.md`) and grammar reference (`GRAMMAR.md`) shall include `-` (U+002D), `+` (U+002B), `*` (U+002A), `?` (U+003F), `&` (U+0026) in the `sakura_token` character class definition.
2. When `\-`, `\+`, `\*`, `\_?`, `\&[ID]` 等が仕様書の `sakura_command` 定義に照らされたとき, the specification shall これらを有効なさくらスクリプトコマンドとして受理する.
3. The specification shall `sakura_token` の文字クラスを `[!\-+*?&_a-zA-Z0-9]+` のように定義し、5文字を明示的に含める.

### Requirement 2: Pest文法定義のsakura_idに5文字を追加

**Objective:** DSL開発者として、パーサーが `\-`, `\+`, `\*`, `\_?`, `\&[ID]` 等をさくらスクリプトとして正しくパースできるようにしたい。これによりPastaスクリプト内でこれらのタグが構文エラーにならなくなる。

#### Acceptance Criteria
1. When Pastaパーサーが `\-`, `\+`, `\*`, `\_?`, `\&[ID]` を含むアクション行をパースしたとき, the pasta_core parser shall これらを `sakura_script` ASTノードとして認識する.
2. When Pastaパーサーが `\-`, `\+`, `\*` 等（角括弧なし）をパースしたとき, the pasta_core parser shall 各sakura_idを正しく認識したノードを生成する.
3. When Pastaパーサーが `Alice：こんにちは\-。` をパースしたとき, the pasta_core parser shall `talk("こんにちは")`, `sakura_script("\-")`, `talk("。")` の3要素を正しく分割する.
4. The pasta_core parser shall 既存のさくらスクリプトタグ（`\h`, `\s[0]`, `\_w[500]`, `\![open,inputbox]` 等）に対してリグレッションを発生させない.

### Requirement 3: ランタイムregexのさくらスクリプトタグ判定に5文字を追加

**Objective:** ランタイム利用者として、`\-`, `\+`, `\*`, `\_?`, `\&[ID]` を含むトークテキストが正しくトークナイズされ、ウェイト挿入処理で適切にさくらスクリプトタグとして扱われるようにしたい。

#### Acceptance Criteria
1. When tokenizer が `\-`, `\+`, `\*`, `\_?`, `\&[ID]` を含むテキストをトークナイズしたとき, the pasta_lua tokenizer shall これらを `TokenKind::SakuraScript` として分類する.
2. When tokenizer が `こんにちは\-。` をトークナイズしたとき, the pasta_lua tokenizer shall `\-` の前後でテキストを正しく分割し、`\-` を1つのSakuraScriptトークンとして返す.
3. When ウェイト挿入モジュールが `\-`, `\+`, `\*` 等のトークンを処理したとき, the pasta_lua wait inserter shall これらのタグに対してウェイトを挿入せず透過する.
4. The pasta_lua tokenizer shall 既存のさくらスクリプトタグ正規表現パターンに対してリグレッションを発生させない.

### Requirement 4: 4箇所の定義の一貫性保証

**Objective:** プロジェクト管理者として、仕様書・文法リファレンス・Pest文法・ランタイムregexの4箇所が常に同じ文字クラスを使用することを保証したい。

#### Acceptance Criteria
1. The sakura_token character class shall 仕様書（`doc/spec/07-sakura-script.md`）、文法リファレンス（`GRAMMAR.md`）、Pest文法（`grammar.pest`）、ランタイムregex（`tokenizer.rs`）の4箇所で同一の文字セットを許容する.
2. The test suite shall 追加5文字（`-`, `+`, `*`, `?`, `&`）を使用するタグのパースとトークナイズの両方を検証するテストケースを含む.
3. The specification shall `sakura_token` の文字クラス変更時に更新すべき4箇所をドキュメントに明記し、将来の拡張時に同じパターンで対応できるようにする.
