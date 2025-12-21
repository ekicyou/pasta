# Implementation Gap Analysis: transpiler-args-tuple-conversion

## 分析サマリー

**スコープ**: Pastaトランスパイラーにおける関数呼び出し引数の配列リテラル`[]`からタプルリテラル`()`への変換

**主要な調査結果**:
- ✅ **Rune 0.14はすべてのタプル構文をサポート**: 先行テスト（`pasta_rune_tuple_syntax_test.rs`）により、0個・1個・複数要素のタプルすべてが正常に動作することを確認
- ✅ **影響範囲は明確**: `src/transpiler/mod.rs`の3箇所（Call文生成2箇所、アクション行の関数呼び出し1箇所）と`transpile_exprs_to_args`ヘルパー関数
- 🔴 **実装バグを発見**: `transpile_speech_part_to_writer` (L507-520) が`SpeechPart::FuncCall`を誤って単語展開として処理（`args`を無視）
- ⚠️ **テスト更新が必要**: 既存のトランスパイラーテスト（複数ファイル）が期待する出力文字列を更新する必要がある
- ✅ **後方互換性リスクは低い**: Rune側の関数シグネチャは変更不要と想定（Out of Scopeで明示済み）

**推奨アプローチ**: Option A（既存コンポーネントの拡張）- シンプルな文字列置換であり、新規ファイル作成は不要

---

## 1. Current State Investigation

### 1.1 影響を受けるファイルとモジュール

#### 主要ファイル
| ファイル | 役割 | 変更箇所 |
|---------|------|---------|
| `src/transpiler/mod.rs` | トランスパイラー本体 | 3箇所の配列リテラル生成 |
| `tests/pasta_transpiler_word_code_gen_test.rs` | 単語展開テスト | 期待値文字列の更新 |
| `tests/pasta_rune_tuple_syntax_test.rs` | **新規追加済み** | Runeタプルサポート検証 |

#### 具体的な変更箇所（`src/transpiler/mod.rs`）

**箇所1: Call文の動的ターゲット（L424）**
```rust
// 現在
"        for a in crate::pasta::call(ctx, `${{ctx.{}.{}}}`, {}, [{}]) {{ yield a; }}"

// 変更後
"        for a in crate::pasta::call(ctx, `${{ctx.{}.{}}}`, {}, ({})) {{ yield a; }}"
```

**箇所2: Call文の静的ターゲット（L433）**
```rust
// 現在
"        for a in crate::pasta::call(ctx, \"{}\", {}, [{}]) {{ yield a; }}"

// 変更後
"        for a in crate::pasta::call(ctx, \"{}\", {}, ({})) {{ yield a; }}"
```

**箇所3: アクション行の関数呼び出し（L507-520）🔴バグ修正**
```rust
// 現在（バグ: argsを無視して単語展開として誤処理）
SpeechPart::FuncCall {
    name,
    args: _,  // 無視されている！
    scope: _,
} => {
    writeln!(
        writer,
        "        yield Talk(pasta_stdlib::word(\"{}\", \"{}\", []));",
        context.current_module(),
        name
    )
    .map_err(|e| PastaError::io_error(e.to_string()))?;
}

// 変更後（正しく関数呼び出しとして処理）
SpeechPart::FuncCall { name, args, scope } => {
    let args_str = Self::transpile_exprs_to_args(args, &context)?;
    let function_call = match scope {
        FunctionScope::Auto => {
            // ローカル優先（resolve_functionでスコープ解決）
            let resolved_name = context.resolve_function(name, *scope)?;
            format!("{}(ctx, ({}))", resolved_name, args_str)
        }
        FunctionScope::GlobalOnly => {
            // グローバル指定（super::を付与）
            format!("super::{}(ctx, ({}))", name, args_str)
        }
    };
    writeln!(
        writer,
        "        for a in {} {{ yield a; }}",
        function_call
    )
    .map_err(|e| PastaError::io_error(e.to_string()))?;
}
```

#### ヘルパー関数の拡張（`transpile_exprs_to_args`）

**現在の実装（L546-555）**:
```rust
fn transpile_exprs_to_args(
    exprs: &[Expr],
    context: &TranspileContext,
) -> Result<String, PastaError> {
    exprs
        .iter()
        .map(|expr| Self::transpile_expr(expr, context))
        .collect::<Result<Vec<_>, _>>()
        .map(|v| v.join(", "))
}
```

**変更の必要性**:
- 現在の実装はカンマ区切り文字列を返す（例: `"arg1, arg2"`）
- 1個の引数の場合、末尾カンマを追加する必要がある（例: `"arg,"`）
- 呼び出し側で括弧`()`を追加する

### 1.2 既存のアーキテクチャパターン

#### トランスパイラーの構造
- **2パス変換**: Pass1（シーン登録）→ Pass2（コード生成）
- **文字列ベース生成**: `writeln!`マクロでRuneコードを直接文字列として出力
- **IR型**: 変更不要（トランスパイル時の内部表現のみ影響）

#### テストの構造
- **文字列マッチング**: `assert!(code.contains(r#"expected_output"#))`パターンを使用
- **ゴールデンテスト**: 完全一致ではなく部分一致で検証
- **統合テスト**: `tests/`配下に配置、エンドツーエンド実行

### 1.3 依存関係と統合ポイント

#### Rune VM側（変更不要と仮定）
- `pasta::call(ctx, scene, filters, args)` - 第4引数はタプルとして受け取る設計を想定
- アクション行の関数: `function_name(ctx, args)` - 第2引数はタプルとして受け取る設計を想定

#### テストの依存性
- **複数ファイルが影響を受ける**:
  1. `pasta_transpiler_word_code_gen_test.rs` - 期待値文字列の更新
  2. その他のトランスパイラーテスト（要grep検索で特定）

---

## 2. Requirements Feasibility Analysis

### 2.1 技術要件マップ

| 要件 | 現在の状態 | ギャップ | 実装難易度 |
|------|-----------|---------|-----------|
| Req 1.1-1.6: Call文・アクション行のタプル変換 | 配列`[]`を使用 | 文字列置換とロジック修正 | **中** |
| Req 2.1-2.4: `transpile_exprs_to_args`修正 | カンマ区切り文字列返す | 1個の引数に末尾カンマ追加 | **中** |
| Req 3.1-3.3: 後方互換性 | 全テスト合格 | テスト期待値を更新 | **中** |
| Req 4.1-4.2: ドキュメント更新 | コメントに配列リテラル例 | タプルリテラル例に更新 | **低** |

### 2.2 検証済みの前提条件

#### ✅ Runeタプルサポート（先行調査完了）

**検証方法**: 新規テストファイル `tests/pasta_rune_tuple_syntax_test.rs` を作成し実行

**検証結果**:
```
running 4 tests
test test_rune_tuple_as_function_argument ... ok
test test_rune_single_element_tuple ... ok
test test_rune_multi_element_tuple ... ok
test test_rune_zero_element_tuple ... ok
```

**結論**:
- ✅ **0個の引数**: 空タプル `()` がサポートされている
- ✅ **1個の引数**: 単一要素タプル `(arg,)` がサポートされている（末尾カンマ必須）
- ✅ **2個以上の引数**: 通常のタプル `(arg1, arg2, ...)` がサポートされている
- ✅ **関数引数として渡す**: タプルを関数の引数として渡すことが可能

### 2.3 ギャップと制約

| ギャップ | 重要度 | 調査状況 |
|---------|--------|---------|
| 1個の引数に末尾カンマが必要 | **高** | ✅ 検証済み - Runeは`(arg,)`をサポート |
| 既存テストの期待値更新 | **中** | ⚠️ 要確認 - 全テストファイルをgrep検索必要 |
| Rune側関数シグネチャの互換性 | **中** | ✅ 仮定確認 - Out of Scopeで明示済み |

#### 制約
- **Rune 0.14に固定**: バージョンアップによる破壊的変更リスクなし
- **文字列ベース生成**: AST変換不要、文字列置換のみで対応可能
- **既存テストパターン**: 部分一致検証のため、テスト更新は局所的

---

## 3. Implementation Approach Options

### Option A: 既存コンポーネントの拡張（推奨）

#### 対象ファイル
- **`src/transpiler/mod.rs`**:
  - L424: 動的Call文の配列→タプル
  - L433: 静的Call文の配列→タプル
  - L507-520: アクション行の関数呼び出し（バグ修正）
  - L546-555: `transpile_exprs_to_args`にタプル生成ロジック追加

#### 変更の詳細

**Step 1: `transpile_exprs_to_args`の修正**
```rust
fn transpile_exprs_to_args(
    exprs: &[Expr],
    context: &TranspileContext,
) -> Result<String, PastaError> {
    let args: Vec<String> = exprs
        .iter()
        .map(|expr| Self::transpile_expr(expr, context))
        .collect::<Result<Vec<_>, _>>()?;
    
    Ok(match args.len() {
        0 => String::new(),           // 空タプル用: "()" は呼び出し側で付与
        1 => format!("{},", args[0]), // 単一要素タプル: 末尾カンマ必須
        _ => args.join(", "),         // 複数要素タプル: カンマ区切り
    })
}
```

**Step 2: Call文生成の修正**
```rust
// L424（動的ターゲット）
writeln!(
    writer,
    "        for a in crate::pasta::call(ctx, `${{ctx.{}.{}}}`, {}, ({})) {{ yield a; }}",
    scope_str, name, filters_str, args_str
)

// L433（静的ターゲット）
writeln!(
    writer,
    "        for a in crate::pasta::call(ctx, \"{}\", {}, ({})) {{ yield a; }}",
    search_key, filters_str, args_str
)
```

**Step 3: アクション行の関数呼び出し修正**
```rust
// L507-520（バグ修正）
SpeechPart::FuncCall { name, args, scope } => {
    let args_str = Self::transpile_exprs_to_args(args, &context)?;
    let function_call = match scope {
        FunctionScope::Auto => {
            let resolved_name = context.resolve_function(name, *scope)?;
            format!("{}(ctx, ({}))", resolved_name, args_str)
        }
        FunctionScope::GlobalOnly => {
            format!("super::{}(ctx, ({}))", name, args_str)
        }
    };
    writeln!(writer, "        for a in {} {{ yield a; }}", function_call)
        .map_err(|e| PastaError::io_error(e.to_string()))?;
}
```

#### 後方互換性評価
- ✅ **既存インターフェース不変**: 公開APIは変更なし
- ✅ **既存ロジック保持**: 引数の解釈ロジックは同じ
- ⚠️ **テスト更新必要**: 期待値文字列を`[]`から`()`に変更

#### 複雑性と保守性
- ✅ **認知負荷**: 低 - 文字列置換のみ
- ✅ **単一責任原則**: 維持 - トランスパイラーの責務は不変
- ✅ **ファイルサイズ**: 変更なし（数行の修正のみ）

#### Trade-offs
- ✅ 新規ファイル不要、最小限の変更
- ✅ 既存パターンを活用、学習コスト0
- ✅ リグレッションリスク低（テストで検証可能）
- ❌ テスト更新が必須（3ファイル程度と想定）

### Option B: 新規コンポーネント作成

**評価**: ❌ **不採用** - 文字列置換のみの変更に対して過剰設計

**理由**:
- 新規ファイル作成の必要性なし
- 責務分離の必要性なし
- コードの複雑性が増すだけ

### Option C: ハイブリッドアプローチ

**評価**: ❌ **不採用** - 本仕様には不要

**理由**:
- 段階的実装の必要性なし（変更が小規模）
- フィーチャーフラグ不要（破壊的変更ではない）

---

## 4. Implementation Complexity & Risk

### 工数見積もり: **S（1-3日）**

**根拠**:
- ✅ 既存パターンを活用（文字列ベース生成）
- ✅ 依存関係が少ない（トランスパイラー内完結）
- ✅ 統合が容易（文字列置換のみ）
- ✅ Runeサポート確認済み（先行テスト完了）

**内訳**:
- Day 1: `transpile_exprs_to_args`修正 + Call文・単語展開の文字列置換
- Day 2: テスト更新（期待値文字列の変更）
- Day 3: 全テスト実行 + ドキュメント更新

### リスク評価: **Low（低）**

**根拠**:
- ✅ **技術的に確立**: Runeタプルサポート検証済み
- ✅ **統合が明確**: トランスパイラー内完結、外部API変更なし
- ✅ **スコープが明確**: 3箇所の文字列置換のみ
- ✅ **テストカバレッジ**: 既存テストで回帰検証可能

**リスク項目**:
| リスク | 発生確率 | 影響度 | 対策 |
|--------|---------|--------|------|
| テスト更新漏れ | 中 | 低 | `cargo test --all`で全件検証 |
| 1個の引数の末尾カンマ漏れ | 低 | 中 | `transpile_exprs_to_args`のロジックで保証 |
| Rune側の非互換性 | 低 | 高 | 先行テストで検証済み（可能性ほぼなし） |

---

## 5. Recommendations for Design Phase

### 推奨アプローチ: **Option A（既存コンポーネント拡張）**

**選択理由**:
- 最小限の変更で要件を満たす
- リグレッションリスクが低い
- 工数が最小（S: 1-3日）
- 既存パターンを活用し、保守性が高い

### 主要な設計決定事項

1. **`transpile_exprs_to_args`のロジック**:
   - 0個: 空文字列を返す（呼び出し側で`()`を付与）
   - 1個: `"{arg},"`を返す（末尾カンマ必須）
   - 2個以上: `"{arg1}, {arg2}, ..."`を返す

2. **テスト戦略**:
   - 既存テストの期待値を配列`[]`からタプル`()`に更新
   - 新規テスト不要（`pasta_rune_tuple_syntax_test.rs`で検証済み）

3. **ドキュメント更新**:
   - `src/transpiler/mod.rs`内のコメント更新
   - 関連する仕様書（該当する場合）の例を更新

### 設計フェーズでの調査項目

| 項目 | 優先度 | 詳細 |
|------|--------|------|
| 全テストファイルのgrep検索 | **高** | `[]`を含むテスト期待値をすべて特定 |
| Rune側関数シグネチャの確認 | 中 | `pasta::call`と`pasta_stdlib::word`の実装確認（Out of Scopeだが念のため） |
| エッジケースの洗い出し | 中 | ネストされた関数呼び出し、式評価の順序など |

### Next Steps

1. **設計フェーズ開始**:
   ```
   /kiro-spec-design transpiler-args-tuple-conversion
   ```

2. **設計で詳細化すべき項目**:
   - `transpile_exprs_to_args`の具体的な実装コード
   - テスト更新の完全なリスト（grep検索結果に基づく）
   - コメント更新箇所の特定
   - 実装順序の決定（関数修正 → テスト更新 → ドキュメント）

---

## 6. Gap Analysis Output Checklist

### ✅ Requirement-to-Asset Map

| 要件 | 現在の資産 | ギャップ | タグ |
|------|-----------|---------|------|
| Req 1: Call文・アクション行のタプル変換 | `src/transpiler/mod.rs` L424, L433, L507-520 | 配列`[]`→タプル`()`置換 + バグ修正 | **修正必要** |
| Req 2: ヘルパー関数修正 | `src/transpiler/mod.rs` L546-555 | 1個の引数に末尾カンマ追加ロジック | **拡張必要** |
| Req 3: 後方互換性 | 既存テスト | 期待値文字列の更新 | **更新必要** |
| Req 4: ドキュメント | コード内コメント | タプル例への更新 | **更新必要** |

### ✅ Options A/B/C with Rationale

- **Option A（推奨）**: 既存コンポーネント拡張 - 最小変更、低リスク、高保守性
- **Option B（不採用）**: 新規コンポーネント - 過剰設計
- **Option C（不採用）**: ハイブリッド - 不要

### ✅ Effort & Risk

- **Effort**: **M（3-5日）** - 文字列置換に加え、アクション行の関数呼び出しロジック修正が必要
- **Risk**: **Medium（中）** - L507-520のバグ修正により既存動作が変わるため、慎重なテストが必要

### ✅ Recommendations

- **Preferred Approach**: Option A（既存コンポーネント拡張）
- **Key Decisions**:
  1. `transpile_exprs_to_args`で末尾カンマロジックを実装
  2. テスト期待値を配列からタプルに一括更新
  3. コメント内の例をタプルに更新

- **Research Items**:
  - 全テストファイルのgrep検索（`[]`を含む期待値）
  - エッジケースの洗い出し（ネスト、式評価順序）

---

## Appendix: 先行調査結果

### Runeタプルサポート検証テスト

**ファイル**: `tests/pasta_rune_tuple_syntax_test.rs`（新規作成）

**テスト項目**:
1. `test_rune_zero_element_tuple`: 空タプル`()`のサポート検証
2. `test_rune_single_element_tuple`: 単一要素タプル`(arg,)`のサポート検証
3. `test_rune_multi_element_tuple`: 複数要素タプル`(a, b, c)`のサポート検証
4. `test_rune_tuple_as_function_argument`: タプルを関数引数として渡す検証

**実行結果**:
```
running 4 tests
test test_rune_tuple_as_function_argument ... ok
test test_rune_single_element_tuple ... ok
test test_rune_multi_element_tuple ... ok
test test_rune_zero_element_tuple ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured
```

**結論**: Rune 0.14は要件で必要なすべてのタプル構文をサポートしている。
