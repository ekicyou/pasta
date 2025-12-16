# Requirements Document: pasta-transpiler-pass2-output

| 項目 | 内容 |
|------|------|
| **Document Title** | Pasta トランスパイラー 2パス目出力修正 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-12-14 |
| **Priority** | P1 |
| **Status** | Requirements Generated |

---

## Introduction

本要件定義書は、Pasta DSLトランスパイラーの2パス目（Pass 2）における出力の修正を定義する。現在のトランスパイラーは `pub mod pasta` 内に `pub fn jump()` と `pub fn call()` を生成しているが、これは設計仕様と異なる。正しい実装では、`pub mod __pasta_trans2__` 内に `pub fn label_selector()` 関数を生成し、`pasta::jump()` と `pasta::call()` はこの関数を呼び出す簡潔なラッパーとなるべきである。

### Background

Pasta DSLトランスパイラーは、以下の構造でRuneコードを生成する設計となっている：

**正しい設計（目標）：**

```rune
// トランスパイラー Pass 2 が生成するモジュール
pub mod __pasta_trans2__ {
    pub fn label_selector(label, filters) {
        let id = pasta_stdlib::select_label_to_id(label, filters);
        match id {
            1 => crate::会話_1::__start__,
            2 => crate::会話_1::選択肢_1,
            3 => crate::会話_1::選択肢_2,
            _ => |_ctx, _args| { yield pasta_stdlib::Error(`ラベルID ${id} が見つかりませんでした。`); },
        }
    }
}

// ユーザーが利用するAPI（簡潔なラッパー）
pub mod pasta {
    pub fn jump(ctx, label, filters, args) {
        let func = crate::__pasta_trans2__::label_selector(label, filters);
        for a in func(ctx, args) { yield a; }
    }

    pub fn call(ctx, label, filters, args) {
        let func = crate::__pasta_trans2__::label_selector(label, filters);
        for a in func(ctx, args) { yield a; }
    }
}
```

**現在の誤った実装：**

現在のトランスパイラーは、以下のような冗長なコードを生成している：

```rune
// ❌ 誤り: pasta モジュール内にmatchロジックが重複
pub mod pasta {
    pub fn jump(ctx, label, filters, args) {
        let id = pasta_stdlib::select_label_to_id(label, filters);
        match id {
            1 => { for a in crate::メイン_1::__start__(ctx, args) { yield a; } },
            2 => { for a in crate::メイン_1::自己紹介_1(ctx, args) { yield a; } },
            3 => { for a in crate::メイン_1::趣味紹介_1(ctx, args) { yield a; } },
            // ... (全ラベルを列挙)
            _ => { yield pasta_stdlib::Error(`ラベルID ${id} が見つかりませんでした。`); },
        }
    }

    pub fn call(ctx, label, filters, args) {
        // 同じmatchロジックが再度記述される（コード重複）
        let id = pasta_stdlib::select_label_to_id(label, filters);
        match id { /* ... */ }
    }
}
```

### Problem Statement

**課題1: 設計仕様との不一致**

関連仕様（`pasta-label-resolution-runtime`, `MEMO.md`）では、`label_selector()` 関数が関数ポインタを返し、それを `jump()`/`call()` が呼び出す設計が明記されている。現在の実装はこの設計に従っておらず、以下の問題がある：

1. **コード重複**: `jump()` と `call()` が同じmatchロジックを持ち、保守性が低下
2. **モジュール構造の不統一**: `__pasta_trans2__` モジュールが生成されず、トランスパイラーの責務が不明確
3. **拡張性の欠如**: 将来的なラベル解決ロジックの変更が困難

**課題2: 参照実装との食い違い**

添付ファイル `comprehensive_control_flow.transpiled.rn` には、正しい実装と誤った実装の両方が混在している：

```rune
// ❌ 間違ったpastaモジュール（77行目〜）
pub mod pasta {
    pub fn jump(ctx, label, filters, args) { /* matchロジックを内包 */ }
    pub fn call(ctx, label, filters, args) { /* matchロジックを内包 */ }
}

// ✅ 正しい__pasta_trans2__モジュール（138行目〜）
pub mod __pasta_trans2__ {
    pub fn label_selector(label, filters) { /* 関数ポインタを返す */ }
}

// ✅ 正しいpastaモジュール（152行目〜）
pub mod pasta {
    pub fn jump(ctx, label, filters, args) {
        let func = crate::__pasta_trans2__::label_selector(label, filters);
        for a in func(ctx, args) { yield a; }
    }
}
```

この不整合により、実行時に誤ったコードが呼び出される可能性がある。

**課題3: トランスパイラーのPass 2実装の明確化**

現在のトランスパイラーのどの部分がPass 2出力を生成しているのか、コードベースを確認する必要がある。

### Scope

**含まれるもの：**

1. **Pass 2出力の修正**
   - `pub mod __pasta_trans2__` の生成
   - `pub fn label_selector()` 関数の生成（関数ポインタを返すmatchロジック）
   - `pub mod pasta` の `jump()` / `call()` 実装の簡素化（ラッパー化）

2. **テストフィクスチャの更新**
   - `comprehensive_control_flow.transpiled.rn` の誤った実装部分の削除
   - テストケースの検証

**含まれないもの：**

- トランスパイラー Pass 1（ラベル登録、構文解析）の変更
- `pasta_stdlib::select_label_to_id()` のRust実装（別仕様: `pasta-label-resolution-runtime`）
- DSL構文の変更

---

## Requirements

### Requirement 1: __pasta_trans2__ モジュールの生成

**Objective:** 開発者として、トランスパイラーが `pub mod __pasta_trans2__` を生成し、ラベル解決ロジックを一元化することで、コードの保守性と拡張性を向上させる。

#### Acceptance Criteria

1. When トランスパイラー Pass 2 が実行される, the Pasta Transpiler shall `pub mod __pasta_trans2__` を生成する
2. When `__pasta_trans2__` モジュールが生成される, the Pasta Transpiler shall モジュール内に `pub fn label_selector(label, filters)` 関数を定義する
3. When `label_selector()` 関数が定義される, the Pasta Transpiler shall 引数として `label`（文字列値）と `filters`（キー・値ペアの配列）を受け取る
4. When `label_selector()` 関数が実装される, the Pasta Transpiler shall 戻り値として関数ポインタを返す
5. When 複数のPastaファイルをトランスパイルする場合, the Pasta Transpiler shall 各ファイルごとに独立した `__pasta_trans2__` モジュールを生成する

### Requirement 2: label_selector() 関数の実装

**Objective:** スクリプト作成者として、ラベル名から対応する関数が正しく解決され、適切な会話フローが実行されることを保証する。

#### Acceptance Criteria

1. When `label_selector()` 関数が呼ばれる, the Pasta Transpiler shall 生成したコード内で `pasta_stdlib::select_label_to_id(label, filters)` を呼び出してラベルIDを取得する
2. When ラベルIDが取得される, the Pasta Transpiler shall `match id` 構文を使用して、IDに対応する関数ポインタを返す
3. When matchの各armが生成される, the Pasta Transpiler shall `1 => crate::会話_1::__start__` のように、IDと関数パスのマッピングを記述する（関数呼び出しではなく、関数名そのものを記述）
4. When 無効なラベルIDが渡される, the Pasta Transpiler shall デフォルトarmとしてエラーを返すクロージャを生成する（`_ => |_ctx, _args| { yield pasta_stdlib::Error(...); }`）
5. When トランスパイラーが全ラベルを処理する, the Pasta Transpiler shall `LabelRegistry` に登録された全ラベルのID → 関数パスマッピングを `label_selector()` のmatch式に含める

#### 出力例（リファレンス実装）

```rune
pub mod __pasta_trans2__ {
    pub fn label_selector(label, filters) {
        let id = pasta_stdlib::select_label_to_id(label, filters);
        match id {
            1 => crate::メイン_1::__start__,
            2 => crate::メイン_1::自己紹介_1,
            3 => crate::メイン_1::趣味紹介_1,
            4 => crate::メイン_1::カウント表示_1,
            5 => crate::メイン_1::会話分岐_1,
            6 => crate::メイン_1::別の話題_1,
            _ => |_ctx, _args| { yield pasta_stdlib::Error(`ラベルID ${id} が見つかりませんでした。`); },
        }
    }
}
```

### Requirement 3: pasta モジュールの簡素化

**Objective:** 開発者として、`pasta::jump()` と `pasta::call()` を簡潔なラッパー関数として実装し、コード重複を排除する。

#### Acceptance Criteria

1. When トランスパイラー Pass 2 が `pub mod pasta` を生成する, the Pasta Transpiler shall `pub fn jump(ctx, label, filters, args)` 関数を定義する
2. When `pasta::jump()` 関数が実装される, the Pasta Transpiler shall 関数内で `crate::__pasta_trans2__::label_selector(label, filters)` を呼び出し、戻り値の関数ポインタを取得する
3. When 関数ポインタが取得される, the Pasta Transpiler shall `for a in func(ctx, args) { yield a; }` 構文を使用して、取得した関数を実行する
4. When `pasta::call()` 関数が実装される, the Pasta Transpiler shall `jump()` と同じロジックを持つ（現時点では挙動に差異なし）
5. When `pasta` モジュールが生成される, the Pasta Transpiler shall `jump()` と `call()` 以外に match ロジックやラベルマッピングを含めない

#### 出力例（リファレンス実装）

```rune
pub mod pasta {
    pub fn jump(ctx, label, filters, args) {
        let func = crate::__pasta_trans2__::label_selector(label, filters);
        for a in func(ctx, args) { yield a; }
    }

    pub fn call(ctx, label, filters, args) {
        let func = crate::__pasta_trans2__::label_selector(label, filters);
        for a in func(ctx, args) { yield a; }
    }
}
```

### Requirement 4: テストフィクスチャの最終整理（実装タスクに含む）

**Objective:** 開発者として、テストフィクスチャから説明用の誤った実装と注釈コメントをすべて削除し、正しい実装のみを残すことで、実際のトランスパイラー出力を正確に反映する。

#### Context

現在の `comprehensive_control_flow.transpiled.rn` は教育目的で誤った実装と正しい実装を並べて記載している。本仕様の実装完了時には、このファイルを実際のトランスパイラー出力（正しい実装のみ）に更新する。比較用の参照資料は `.kiro/specs/pasta-transpiler-pass2-output/reference_comparison.rn` に保存済み。

**実施タイミング:** トランスパイラー本体の修正（要件5）と同じ実装タスク内で自動実施。

#### Acceptance Criteria

1. When 本仕様の実装が完了する, the Test Suite shall `comprehensive_control_flow.transpiled.rn` から77行目〜103行目の誤った `pub mod pasta` 実装（matchロジックを内包）を完全に削除する
2. When 誤った実装が削除される, the Test Suite shall 正しい `pub mod __pasta_trans2__` と `pub mod pasta` 実装のみを残す
3. When ファイルが整理される, the Test Suite shall 説明用のコメント（`// ❌ 間違った...`, `// ✅ 正しい...`）をすべて削除する
4. When 最終的なファイルが完成する, the Test Suite shall トランスパイラーが実際に出力するコードと完全に一致する内容のみを含む
5. When トランスパイラーのテストが実行される, the Test Suite shall 更新されたフィクスチャに基づいて、正しい出力が生成されることを検証する

#### 注記

比較用の参照資料（誤った実装と正しい実装の両方を含む）は `.kiro/specs/pasta-transpiler-pass2-output/reference_comparison.rn` として保存されており、本仕様の実装中に参照可能。

### Requirement 5: Pass 2 実装の特定と修正

**Objective:** 開発者として、トランスパイラーのコードベース内でPass 2出力を生成している箇所を特定し、仕様に準拠した実装に修正する。

#### Acceptance Criteria

1. When トランスパイラーのコードベースを調査する, the Development Team shall Pass 2 で `pub mod pasta` を生成している関数またはメソッドを特定する
2. When Pass 2 実装が特定される, the Development Team shall `label_selector()` 関数を生成するロジックが欠落していることを確認する
3. When 修正が実施される, the Pasta Transpiler shall `generate_pasta_trans2_module()` などの関数を追加し、`__pasta_trans2__` モジュールを生成する
4. When 既存の `pasta` モジュール生成ロジックが修正される, the Pasta Transpiler shall matchロジックを削除し、`label_selector()` 呼び出しに変更する
5. When 修正が完了する, the Pasta Transpiler shall 単体テストおよび統合テストがすべてパスすることを確認する

---

## Technical Context

### 現在の実装状況

**トランスパイラー構造（推定）：**

```
crates/pasta/src/transpiler/
  ├── mod.rs          # トランスパイラーのエントリポイント
  ├── pass1/          # Pass 1: 構文解析、ラベル登録
  ├── pass2/          # Pass 2: Rune コード生成 ← 修正対象
  └── label_registry.rs  # ラベル情報の管理
```

Pass 2 のコード生成では、以下のような関数が存在すると推定される：

- `generate_pasta_module()` - `pub mod pasta` の生成（修正対象）
- `generate_label_functions()` - 各ラベル関数の生成（変更なし）
- `generate_pasta_trans2_module()` - **新規追加が必要**

### 修正箇所の推定

**修正前（現在の実装）：**

```rust
// crates/pasta/src/transpiler/pass2/mod.rs (推定)
fn generate_pasta_module(label_registry: &LabelRegistry) -> String {
    let mut code = String::from("pub mod pasta {\n");
    
    // ❌ 誤り: jump() 内に match ロジックを直接生成
    code.push_str("    pub fn jump(ctx, label, filters, args) {\n");
    code.push_str("        let id = pasta_stdlib::select_label_to_id(label, filters);\n");
    code.push_str("        match id {\n");
    
    for (id, label_info) in label_registry.iter() {
        code.push_str(&format!("            {} => {{ for a in crate::{}(ctx, args) {{ yield a; }} }},\n", 
            id, label_info.fn_path));
    }
    
    code.push_str("            _ => { yield pasta_stdlib::Error(`ラベルID ${id} が見つかりませんでした。`); },\n");
    code.push_str("        }\n    }\n");
    
    // call() も同様のロジック（省略）
    
    code.push_str("}\n");
    code
}
```

**修正後（目標実装）：**

```rust
// crates/pasta/src/transpiler/pass2/mod.rs (目標)
fn generate_pasta_trans2_module(label_registry: &LabelRegistry) -> String {
    let mut code = String::from("pub mod __pasta_trans2__ {\n");
    code.push_str("    pub fn label_selector(label, filters) {\n");
    code.push_str("        let id = pasta_stdlib::select_label_to_id(label, filters);\n");
    code.push_str("        match id {\n");
    
    for (id, label_info) in label_registry.iter() {
        // ✅ 関数ポインタを返す（関数呼び出しではない）
        code.push_str(&format!("            {} => crate::{},\n", id, label_info.fn_path));
    }
    
    code.push_str("            _ => |_ctx, _args| { yield pasta_stdlib::Error(`ラベルID ${id} が見つかりませんでした。`); },\n");
    code.push_str("        }\n    }\n");
    code.push_str("}\n");
    code
}

fn generate_pasta_module_wrapper() -> String {
    // ✅ 簡潔なラッパー関数のみ生成
    r#"pub mod pasta {
    pub fn jump(ctx, label, filters, args) {
        let func = crate::__pasta_trans2__::label_selector(label, filters);
        for a in func(ctx, args) { yield a; }
    }

    pub fn call(ctx, label, filters, args) {
        let func = crate::__pasta_trans2__::label_selector(label, filters);
        for a in func(ctx, args) { yield a; }
    }
}"#.to_string()
}
```

### 関連仕様との整合性

| 仕様 | 関連性 | 状態 |
|------|--------|------|
| `pasta-label-resolution-runtime` | `label_selector()` の呼び出し先として `select_label_to_id()` を使用 | 🔄 実装中 |
| `.kiro/specs/MEMO.md` | API設計書に `label_selector()` の仕様が記載 | ✅ 参照済み |
| `pasta-declarative-control-flow` | トランスパイラーの全体設計（Pass 1/2の分離） | ✅ Completed |

---

## Testing Strategy

### Unit Tests

| テストケース | 入力 | 期待される出力 |
|-------------|------|--------------|
| **__pasta_trans2__生成** | `LabelRegistry` with 3 labels | `pub mod __pasta_trans2__ { ... }` を含むコード |
| **label_selector生成** | 同上 | `pub fn label_selector(label, filters)` 関数定義を含む |
| **matchロジック** | `LabelRegistry` with IDs 1-6 | 各IDに対応する `crate::モジュール::関数` マッピング |
| **pastaラッパー生成** | 空の `LabelRegistry` | `pub mod pasta { pub fn jump(...) { ... } }` のみ生成 |
| **エラーハンドリング** | 無効なID（99） | デフォルトarmでエラークロージャを生成 |

### Integration Tests

1. **エンドツーエンドトランスパイル:**
   - Pasta DSL → トランスパイル → 生成されたRuneコードが正しい構造を持つ
   
2. **Runeコンパイル検証テスト（簡易的な実行テスト）:**
   - 生成されたコードをRune VMでコンパイル → 構文エラーなくコンパイルが通る
   - 全関数が定義されていることを確認（関数呼び出しの解決が成功する）
   - **注意:** `pasta_stdlib::select_label_to_id()` は現在ダミー実装（常に1を返す）のため、完全な実行検証は行わない
   
3. **フィクスチャ検証:**
   - `comprehensive_control_flow.transpiled.rn` をパース → 誤った実装が存在しないことを確認

---

## Implementation Notes

### 実装の優先順位

1. **Phase 1: コード生成ロジックの修正**（必須）
   - `generate_pasta_trans2_module()` の実装
   - `generate_pasta_module()` の簡素化

2. **Phase 2: テストフィクスチャの整理**（必須）
   - `comprehensive_control_flow.transpiled.rn` の誤った実装削除

3. **Phase 3: テストケースの追加**（推奨）
   - Pass 2 出力の単体テスト
   - 生成されたコードのRuneコンパイル検証テスト（構文チェック）

### パフォーマンス考慮事項

- トランスパイル時のコード生成は1回のみ実行されるため、パフォーマンスへの影響は無視できる
- 生成されるコードサイズは若干増加（`__pasta_trans2__` モジュール分）するが、実行時パフォーマンスは変わらない

---

## Dependencies

| 依存仕様/クレート | 理由 | 状態 |
|------------------|------|------|
| `pasta-label-resolution-runtime` | `select_label_to_id()` 関数の実装 | 🔄 実装中 |
| `rune` (0.14) | 生成されたコードの実行環境 | ✅ 既存依存 |

---

## Future Work

- **Pass 2 最適化:** ラベル数が多い場合の `match` 式の最適化（ハッシュマップ利用など）
- **デバッグ情報:** 生成されたコードに元のPasta DSL行番号をコメントとして埋め込む
- **エラーメッセージ改善:** `label_selector()` で無効なIDが渡された際の詳細なエラーメッセージ

---

## References

- **関連仕様:** `.kiro/specs/pasta-label-resolution-runtime/`
- **設計メモ:** `.kiro/specs/MEMO.md` (セレクターAPI設計)
- **テストフィクスチャ:** `crates/pasta/tests/fixtures/comprehensive_control_flow.transpiled.rn`
- **トランスパイラー:** `crates/pasta/src/transpiler/` (推定パス)
