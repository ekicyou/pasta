# Requirements Document

## Project Description (Input)
パスタトランスパイラーは、関数呼び出しを展開するとき、argsを配列にしているが、配列ではなく、タプルに展開すべき。該当する箇所を確認し、すべて修正する仕様を立ち上げよ。

## Introduction

Pastaトランスパイラーは現在、関数呼び出しの引数をRune配列リテラル `[]` として生成していますが、Runeのタプル構文 `()` を使用すべきです。Runeにおいてタプルは固定サイズの値シーケンスであり、関数の引数渡しに適した型です。配列は可変長コレクションであり、引数リストとしては意味的に不適切です。

本仕様では、トランスパイラーが生成する以下の箇所における配列リテラルをタプルリテラルに変換します：
1. Call/Jump文の引数リスト（`pasta::call`関数呼び出し）
2. アクション行の関数呼び出し（`＠関数（引数）`構文）

**アクション行の処理区分**:
- `＠XXX`（括弧なし）: 単語検索 → `pasta_stdlib::word(...)` （本仕様のスコープ外）
- `＠XXX()`（括弧あり）: 関数呼び出し → `for a in XXX(ctx, args) { yield a; }` （本仕様の対象）
- `＠＊XXX()`（グローバル指定）: グローバル関数呼び出し → `for a in super::XXX(ctx, args) { yield a; }` （本仕様の対象）

**注意事項**:
- 単語展開（`pasta_stdlib::word`）の第3引数は`_filters`（フィルター用）であり、`args`（引数リスト）ではないため本仕様のスコープ外
- **実装バグ**: 現在`transpile_speech_part_to_writer` (L507-520) は`SpeechPart::FuncCall`の`args`を無視して単語展開として誤処理している。正しくは関数呼び出しとして`for a in function_name(ctx, args) { yield a; }`を生成すべき。

## Requirements

### Requirement 1: 関数呼び出しの引数変換（Call/Jump文およびアクション行）
**Objective:** トランスパイラー開発者として、すべての関数呼び出し生成時に引数を配列ではなくタプルとして展開したい。これにより、Rune側の関数シグネチャと意味的に一致したコードが生成される。

#### Acceptance Criteria
1. When Statement::Callを処理するとき、Pastaトランスパイラーは引数リストをタプルリテラル `(arg1, arg2, ...)` 形式で生成する
2. When SpeechPart::FuncCallを処理するとき、Pastaトランスパイラーは`pasta::call`を使用して関数を呼び出し、引数をタプル形式で渡す
3. When 引数が0個のとき、Pastaトランスパイラーは空タプル `()` を生成する
4. When 引数が1個のとき、Pastaトランスパイラーは単一要素タプル `(arg,)` を生成する（末尾カンマ必須）
5. When 引数が2個以上のとき、Pastaトランスパイラーは通常のタプル `(arg1, arg2, ...)` を生成する
6. The Pastaトランスパイラーは、動的ターゲット（テンプレートリテラル使用）と静的ターゲット両方でタプル構文を使用する

#### Implementation Notes

**Call/Jump文の現在のコード生成**:
```rune
for a in crate::pasta::call(ctx, "scene", #{}, [arg1, arg2]) { yield a; }
```

**Call/Jump文の変更後**:
```rune
for a in crate::pasta::call(ctx, "scene", #{}, (arg1, arg2)) { yield a; }
```

**アクション行の現在のコード生成（バグ）**:
```rune
yield Talk(pasta_stdlib::word("module", "func_name", []));  // 引数を無視
```

**アクション行の変更後（ローカル優先）**:
```rune
for a in func_name(ctx, (arg1, arg2)) { yield a; }
```

**アクション行の変更後（グローバル指定）**:
```rune
for a in super::func_name(ctx, (arg1, arg2)) { yield a; }
```

**空引数の場合**:
```rune
for a in func_name(ctx, ()) { yield a; }
```

### Requirement 2: 引数トランスパイル関数の修正
**Objective:** トランスパイラー開発者として、`transpile_exprs_to_args`関数がタプルリテラル構文を生成するように修正したい。これにより、引数リストの変換ロジックが一元化される。

#### Acceptance Criteria
1. When `transpile_exprs_to_args`が空の引数リスト `&[]` を受け取るとき、Pastaトランスパイラーは空文字列 `""` を返す（呼び出し側で `()` に展開）
2. When `transpile_exprs_to_args`が1個の引数を受け取るとき、Pastaトランスパイラーは `"arg,"` を返す（単一要素タプルの末尾カンマ）
3. When `transpile_exprs_to_args`が2個以上の引数を受け取るとき、Pastaトランスパイラーは `"arg1, arg2, ..."` を返す
4. The Pastaトランスパイラーは、呼び出し側で引数文字列を括弧 `()` で囲んでタプルを形成する

### Requirement 3: 後方互換性の保証
**Objective:** 開発者として、既存のテストが引き続き合格することを確認したい。これにより、変更が既存機能を破壊しないことが保証される。

#### Acceptance Criteria
1. When 全テストを実行するとき、Pastaプロジェクトは既存の全テストが合格する
2. If テストが失敗するとき、then Pastaトランスパイラーは期待されるRune出力を配列からタプルに更新する
3. The Pastaトランスパイラーは、変更前と同じ意味的な動作を維持する

### Requirement 4: ドキュメント更新
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
