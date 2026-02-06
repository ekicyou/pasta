# Requirements Document

## Introduction

さくらスクリプトにはバックスラッシュ直後にハイフン（`-`）のみが続く `\-` タグが存在する。これは伺か（ukagaka）ベースラインにおいて有効なタグであるが、現在のPasta DSLでは以下の4箇所すべてで `-` が許容文字に含まれておらず、`\-` をさくらスクリプトタグとして認識できない：

1. **言語仕様書** (`doc/spec/07-sakura-script.md`): `sakura_token ::= [!_a-zA-Z0-9]+` — `-` なし
2. **文法リファレンス** (`GRAMMAR.md`): `sakura_token ::= [!_a-zA-Z0-9]+` — `-` なし（上記の複製）
3. **Pest文法定義** (`grammar.pest`): `sakura_id = @{ ... "_" | "!" ... }` — `-` なし
4. **ランタイムregex** (`tokenizer.rs`): `r"\\[0-9a-zA-Z_!]+"` — `-` なし

本仕様は、これら4箇所を一貫して修正し、`\-` を正しくさくらスクリプトタグとして認識・透過させることを目的とする。

## Requirements

### Requirement 1: 仕様書・文法リファレンスのsakura_token定義にハイフンを追加

**Objective:** 仕様書利用者として、`\-` が有効なさくらスクリプトタグであることを仕様レベルで明確にしたい。これにより実装と仕様の乖離を防ぐ。

#### Acceptance Criteria
1. The pasta DSL specification (`doc/spec/07-sakura-script.md`) and grammar reference (`GRAMMAR.md`) shall include `-` (hyphen, U+002D) in the `sakura_token` character class definition.
2. When `\-` が仕様書の `sakura_command` 定義に照らされたとき, the specification shall `\-` を有効なさくらスクリプトコマンドとして受理する.
3. The specification shall `sakura_token` の文字クラスを `[!\-_a-zA-Z0-9]+` のように定義し、ハイフンを明示的に含める.

### Requirement 2: Pest文法定義のsakura_idにハイフンを追加

**Objective:** DSL開発者として、パーサーが `\-` をさくらスクリプトとして正しくパースできるようにしたい。これによりPastaスクリプト内で `\-` が構文エラーにならなくなる。

#### Acceptance Criteria
1. When Pastaパーサーが `\-` を含むアクション行をパースしたとき, the pasta_core parser shall `\-` を `sakura_script` ASTノードとして認識する.
2. When Pastaパーサーが `\-` 単体（角括弧なし）をパースしたとき, the pasta_core parser shall sakura_id が `"-"` であるノードを生成する.
3. When Pastaパーサーが `Alice：こんにちは\-。` をパースしたとき, the pasta_core parser shall `talk("こんにちは")`, `sakura_script("\-")`, `talk("。")` の3要素を正しく分割する.
4. The pasta_core parser shall 既存のさくらスクリプトタグ（`\h`, `\s[0]`, `\_w[500]`, `\![open,inputbox]` 等）に対してリグレッションを発生させない.

### Requirement 3: ランタイムregexのさくらスクリプトタグ判定にハイフンを追加

**Objective:** ランタイム利用者として、`\-` を含むトークテキストが正しくトークナイズされ、ウェイト挿入処理で適切にさくらスクリプトタグとして扱われるようにしたい。

#### Acceptance Criteria
1. When tokenizer が `\-` を含むテキストをトークナイズしたとき, the pasta_lua tokenizer shall `\-` を `TokenKind::SakuraScript` として分類する.
2. When tokenizer が `こんにちは\-。` をトークナイズしたとき, the pasta_lua tokenizer shall `\-` の前後でテキストを正しく分割し、`\-` を1つのSakuraScriptトークンとして返す.
3. When ウェイト挿入モジュールが `\-` トークンを処理したとき, the pasta_lua wait inserter shall `\-` タグに対してウェイトを挿入せず透過する.
4. The pasta_lua tokenizer shall 既存のさくらスクリプトタグ正規表現パターンに対してリグレッションを発生させない.

### Requirement 4: 4箇所の定義の一貫性保証

**Objective:** プロジェクト管理者として、仕様書・文法リファレンス・Pest文法・ランタイムregexの4箇所が常に同じ文字クラスを使用することを保証したい。

#### Acceptance Criteria
1. The sakura_token character class shall 仕様書（`doc/spec/07-sakura-script.md`）、文法リファレンス（`GRAMMAR.md`）、Pest文法（`grammar.pest`）、ランタイムregex（`tokenizer.rs`）の4箇所で同一の文字セットを許容する.
2. The test suite shall `\-` タグのパースとトークナイズの両方を検証するテストケースを含む.
3. The specification shall `sakura_token` の文字クラス変更時に更新すべき4箇所をドキュメントに明記し、将来の拡張時に同じパターンで対応できるようにする.
