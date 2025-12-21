# Requirements Document

## Project Description (Input)
「＞カウント表示（＄カウンタ）」の展開結果が期待と異なる。()でくくられた場合は「同名の関数をそのまま呼び出す」挙動であるべき。シーン呼び出しではなく、関数呼び出しになる必要がある。

「＞カウント表示（＄カウンタ）」　⇒「for a in カウント表示(ctx, [ctx.local.カウンタ]) { yield a; }」でなければならない。

---

## Requirements

### Requirement 1: 括弧付きCall行の直接関数呼び出し変換

**Objective:** Pasta開発者として、`＞シーン名（引数）` 構文を使用することで、シーン検索を経由せずに同名の関数を直接呼び出したい。これにより、パフォーマンスの向上と呼び出し意図の明確化を実現する。

#### Acceptance Criteria

1. When Callマーカー（`＞` または `>`）の後にローカルシーン名と括弧付き引数リスト `（引数）` が記述された場合（例：`＞関数名（＄x）`）、the Transpiler shall 生成されるRuneコードをローカル関数の直接呼び出し形式 `for a in 関数名(ctx, [ctx.local.x]) { yield a; }` に変換する

2. When Callマーカーの後にグローバルシーンマーカー（`＊` または `*`）付きのシーン名と括弧付き引数リストが記述された場合（例：`＞＊グローバル関数（＄y）`）、the Transpiler shall 生成されるRuneコードを親モジュール参照形式 `for a in super::グローバル関数(ctx, [ctx.local.y]) { yield a; }` に変換する

3. When Callマーカーの後にシーン名のみで括弧が無い場合（例：`＞シーン名`）、the Transpiler shall 従来通りシーン検索を経由する形式 `for a in crate::pasta::call(ctx, "シーン名", #{}, []) { yield a; }` に変換する

4. When 括弧付きCall行で複数の引数が指定された場合（例：`＞関数名（＄x　＄y）`）、the Transpiler shall すべての引数を配列形式 `[ctx.local.x, ctx.local.y]` で直接関数呼び出しに渡す

5. When 括弧付きCall行でグローバル変数が引数として指定された場合（例：`＞関数名（＄＊グローバル変数）`）、the Transpiler shall グローバルスコープの変数参照 `ctx.global.グローバル変数` として正しく展開する

6. When 括弧付きCall行でリテラル値が引数として指定された場合（例：`＞関数名（10　「文字列」）`）、the Transpiler shall リテラル値をそのまま引数配列に含める

7. The Transpiler shall 括弧付きCall行において、フィルター構文（`＆属性＝値`）を無視または警告する（直接関数呼び出しではフィルタリングは意味を持たないため）

8. When 括弧付きCall行で空の引数リスト（例：`＞関数名（）`）が指定された場合、the Transpiler shall 空配列 `[]` を引数として渡す形式 `for a in 関数名(ctx, []) { yield a; }` に変換する

9. The Transpiler shall ローカル関数呼び出しを「同一Runeモジュール内で定義された関数」への直接呼び出しとして扱う（シーン検索やモジュール越えの参照を行わない）

10. The Transpiler shall グローバル関数呼び出しを「mainから公開されたグローバル関数」への直接呼び出しとして扱い、`super::` プレフィックスで親モジュールに解決する

### Requirement 2: 既存Call行の動作維持（後方互換性）

**Objective:** Pasta開発者として、括弧なしのCall行は従来通りの動作を維持したい。これにより、既存のスクリプトが破壊されないことを保証する。

#### Acceptance Criteria

1. When 括弧なしCall行（例：`＞シーン名`）が記述された場合、the Transpiler shall 従来通り `crate::pasta::call()` 経由のシーン検索呼び出しを生成する

2. When 括弧なしCall行でフィルター構文が使用された場合（例：`＞シーン名＆author＝Alice`）、the Transpiler shall フィルター情報を正しくハッシュマップ形式 `#{"author": "Alice"}` に変換する

3. The Transpiler shall 括弧なしCall行の前方一致シーン検索機能を保持する

### Requirement 3: トランスパイラーレイヤーでの判定ロジック

**Objective:** Pastaエンジニアとして、Call行が括弧付きか否かを判定し、適切な変換ロジックを選択したい。

#### Acceptance Criteria

1. When トランスパイラーが `Statement::Call` を処理する際、the Transpiler shall `args`フィールドが空でない（`!args.is_empty()`）場合、または空の引数リスト `arg_list` が明示的に指定されている場合を「括弧付きCall」と判定する

2. When 括弧付きCallかつ `JumpTarget::Local` の場合、the Transpiler shall ローカル関数の直接呼び出しコード `for a in 関数名(ctx, [引数]) { yield a; }` を生成する

3. When 括弧付きCallかつ `JumpTarget::Global` の場合、the Transpiler shall グローバル関数の直接呼び出しコード `for a in super::グローバル関数名_N::__start__(ctx, [引数]) { yield a; }` を生成する（`_N` は当該グローバルシーンのモジュールサフィックス）

4. When 括弧なしCallと判定された場合、the Transpiler shall 従来のシーン検索ロジック（`crate::pasta::call()`）を使用する

5. The Transpiler shall 括弧付きCallにおいて、`JumpTarget::Dynamic` が指定された場合はエラーを報告する（動的ターゲットは直接関数呼び出しと互換性がないため）

6. The Transpiler shall 括弧付きCallにおいて、`JumpTarget::LongJump` が指定された場合はエラーを報告する（長距離ジャンプは直接関数呼び出しと互換性がないため）

### Requirement 4: 引数展開の正確性

**Objective:** Pastaエンジニアとして、括弧付きCall行の引数が正しくRune形式の配列に変換されることを保証したい。

#### Acceptance Criteria

1. When 引数リストに変数参照が含まれる場合、the Transpiler shall 変数スコープ（local/global）を正しく判定し、`ctx.local.変数名` または `ctx.global.変数名` に展開する

2. When 引数リストにリテラル値が含まれる場合、the Transpiler shall 数値リテラル、文字列リテラル、真偽値リテラルをそのままRuneコードに含める

3. When 引数リストに関数呼び出しが含まれる場合（例：`＞関数名（＠計算（）　＄x）`）、the Transpiler shall 関数呼び出しを適切に展開する（ネストした関数呼び出しのサポート）

4. When 空の引数リスト（例：`＞関数名（）`）が指定された場合、the Transpiler shall 空配列 `[]` を引数として渡す

5. The Transpiler shall 引数配列の要素を位置引数として解釈し、Rune関数の `args` パラメータに渡す形式で生成する

### Requirement 5: エラーハンドリングと検証

**Objective:** Pastaユーザーとして、括弧付きCall行で不正な構文が使用された場合に適切なエラーメッセージを受け取りたい。

#### Acceptance Criteria

1. If 括弧付きCall行でフィルター構文が同時に使用された場合、the Transpiler shall 警告メッセージを出力する（「直接関数呼び出しではフィルターは無効です」）

2. If 括弧付きCall行で `JumpTarget::Dynamic`（動的ターゲット）が指定された場合、the Transpiler shall エラーメッセージを出力する（「動的ターゲットは括弧付きCallと互換性がありません」）

3. If 括弧付きCall行で `JumpTarget::LongJump`（長距離ジャンプ）が指定された場合、the Transpiler shall エラーメッセージを出力する（「長距離ジャンプは括弧付きCallと互換性がありません」）

4. If 括弧付きCall行で指定された関数名が存在しない場合、the Runtime (Rune VM) shall エラーメッセージを生成する（コンパイル時検証は将来拡張）

5. The Transpiler shall 括弧付きCall行の引数展開において、不正な構文（例：名前付き引数の誤用）を検出してエラーを報告する

### Requirement 6: ドキュメントと仕様更新

**Objective:** Pastaドキュメント管理者として、括弧付きCall行の新しい動作を仕様書に反映したい。

#### Acceptance Criteria

1. The Documentation shall `SPECIFICATION.md` の「4. Call の詳細仕様」セクションに、括弧付きCall行の動作説明を追加する

2. The Documentation shall 括弧付きCall行と括弧なしCall行の違いを明確に示す使用例を提供する（ローカル関数呼び出しとグローバル関数呼び出しの両方）

3. The Documentation shall パフォーマンス上の利点（シーン検索のオーバーヘッド削減）を説明する

4. The Documentation shall グローバル関数呼び出し時の `super::` プレフィックスとモジュール構造について説明する

---

## Out of Scope

以下は本仕様の対象外とする：

- **単語呼び出し（`＠単語名（引数）`）の直接関数呼び出し対応**: 単語は前方一致によるランダム選択が本質的な機能であるため、直接呼び出しは対象外
- **ローカルシーン名の重複解決**: 括弧付きCallは同名の関数を一意に特定できることが前提（重複定義がある場合の動作は未定義）
- **コンパイル時の関数存在チェック**: 現時点ではRune VMに依存した実行時エラーで対応（将来の静的解析で拡張可能）
- **名前付き引数のサポート**: 括弧付きCall行では位置引数のみをサポート（名前付き引数は将来拡張）
- **動的ターゲット（`＞＄変数（）`）の括弧付きCall対応**: 変数名による動的呼び出しは直接関数呼び出しと互換性がないため対象外
- **長距離ジャンプ（`＞＊グローバル・ローカル（）`）の括弧付きCall対応**: 長距離ジャンプ構文は直接関数呼び出しと互換性がないため対象外

---

## Success Metrics

- 既存のテストスイート（`cargo test --all`）が全て合格する
- 括弧付きCall行を使用した新規テストケースが3件以上追加され、すべて合格する
- `SPECIFICATION.md` に新しい構文の説明が追加される
- 既存のスクリプト（括弧なしCall行）の動作に変更がないことが検証される
