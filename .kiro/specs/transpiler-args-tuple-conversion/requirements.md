# Requirements Document

## Project Description (Input)
パスタトランスパイラーは、関数呼び出しを展開するとき、argsを配列にしているが、配列ではなく、タプルに展開すべき。該当する箇所を確認し、すべて修正する仕様を立ち上げよ。

## Introduction

Pastaトランスパイラーは現在、関数呼び出しの引数をRune配列リテラル `[]` として生成していますが、Runeのタプル構文 `()` を使用すべきです。Runeにおいてタプルは固定サイズの値シーケンスであり、関数の引数渡しに適した型です。配列は可変長コレクションであり、引数リストとしては意味的に不適切です。

本仕様では、トランスパイラーが生成する以下の箇所における配列リテラルをタプルリテラルに変換します：
1. Call/Jump文の引数リスト（`pasta::call`関数呼び出し）
2. 単語展開の引数リスト（`pasta_stdlib::word`関数呼び出し）
3. その他の関数呼び出しにおける引数リスト

## Requirements

### Requirement 1: Call/Jump文の引数変換
**Objective:** トランスパイラー開発者として、Call/Jump文生成時に引数を配列ではなくタプルとして展開したい。これにより、Rune側の関数シグネチャと意味的に一致したコードが生成される。

#### Acceptance Criteria
1. When Statement::Callを処理するとき、Pastaトランスパイラーは引数リストをタプルリテラル `(arg1, arg2, ...)` 形式で生成する
2. When 引数が0個のとき、Pastaトランスパイラーは空タプル `()` を生成する
3. When 引数が1個のとき、Pastaトランスパイラーは単一要素タプル `(arg,)` を生成する（末尾カンマ必須）
4. When 引数が2個以上のとき、Pastaトランスパイラーは通常のタプル `(arg1, arg2, ...)` を生成する
5. The Pastaトランスパイラーは、動的ターゲット（テンプレートリテラル使用）と静的ターゲット両方でタプル構文を使用する

#### Implementation Notes
現在のコード生成：
```rune
for a in crate::pasta::call(ctx, "scene", #{}, [arg1, arg2]) { yield a; }
```

変更後のコード生成：
```rune
for a in crate::pasta::call(ctx, "scene", #{}, (arg1, arg2)) { yield a; }
```

### Requirement 2: 単語展開の引数変換
**Objective:** トランスパイラー開発者として、単語展開（Word expansion）生成時に引数を配列ではなくタプルとして展開したい。これにより、将来的に引数付き単語呼び出しをサポートする際の一貫性が保たれる。

#### Acceptance Criteria
1. When SpeechPart::FuncCallを処理するとき、Pastaトランスパイラーは`pasta_stdlib::word`関数の第3引数を空タプル `()` で生成する
2. The Pastaトランスパイラーは、グローバル単語とローカル単語両方でタプル構文を使用する

#### Implementation Notes
現在のコード生成：
```rune
yield Talk(pasta_stdlib::word("module", "word_key", []));
```

変更後のコード生成：
```rune
yield Talk(pasta_stdlib::word("module", "word_key", ()));
```

**補足**: `pasta_stdlib::word`関数の第3引数は現在未使用（`_filters`として無視）だが、将来の拡張のために予約されている。

### Requirement 3: 引数トランスパイル関数の修正
**Objective:** トランスパイラー開発者として、`transpile_exprs_to_args`関数がタプルリテラル構文を生成するように修正したい。これにより、引数リストの変換ロジックが一元化される。

#### Acceptance Criteria
1. When `transpile_exprs_to_args`が空の引数リスト `&[]` を受け取るとき、Pastaトランスパイラーは空文字列 `""` を返す（呼び出し側で `()` に展開）
2. When `transpile_exprs_to_args`が1個の引数を受け取るとき、Pastaトランスパイラーは `"arg,"` を返す（単一要素タプルの末尾カンマ）
3. When `transpile_exprs_to_args`が2個以上の引数を受け取るとき、Pastaトランスパイラーは `"arg1, arg2, ..."` を返す
4. The Pastaトランスパイラーは、呼び出し側で引数文字列を括弧 `()` で囲んでタプルを形成する

### Requirement 4: 後方互換性の保証
**Objective:** 開発者として、既存のテストが引き続き合格することを確認したい。これにより、変更が既存機能を破壊しないことが保証される。

#### Acceptance Criteria
1. When 全テストを実行するとき、Pastaプロジェクトは既存の全テストが合格する
2. If テストが失敗するとき、then Pastaトランスパイラーは期待されるRune出力を配列からタプルに更新する
3. The Pastaトランスパイラーは、変更前と同じ意味的な動作を維持する

### Requirement 5: ドキュメント更新
**Objective:** 開発者として、トランスパイラーの出力仕様が最新の実装を反映していることを確認したい。これにより、ドキュメントと実装の乖離を防ぐ。

#### Acceptance Criteria
1. When 実装が完了したとき、Pastaプロジェクトは関連ドキュメント内の配列リテラル例をタプルリテラルに更新する
2. The Pastaプロジェクトは、コード内コメントでタプル構文の使用を明記する

## Out of Scope

以下は本仕様の対象外です：
- Rune VM側の`pasta::call`や`pasta_stdlib::word`関数のシグネチャ変更
  - **理由**: `pasta::call`は引数を透過的に渡すだけ、`pasta_stdlib::word`は第3引数を無視するため、変更不要
- 新しい関数呼び出し構文の追加
- 配列とタプルの両方をサポートする互換レイヤーの実装
