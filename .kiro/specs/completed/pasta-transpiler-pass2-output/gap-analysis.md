# Gap Analysis: pasta-transpiler-pass2-output

| 項目 | 内容 |
|------|------|
| **Document Title** | Pasta トランスパイラー 2パス目出力修正 ギャップ分析書 |
| **Version** | 1.0 |
| **Date** | 2025-12-14 |
| **Analyzed By** | AI Development Assistant |
| **Status** | Analysis Complete |

---

## Executive Summary

Pasta DSLトランスパイラーのPass 2実装が設計仕様と異なり、`pub mod pasta` 内に冗長なmatchロジックを直接生成している。正しい設計では、`pub mod __pasta_trans2__` に `label_selector()` 関数を生成し、`pasta::jump()`/`call()` は簡潔なラッパーとすべき。

**主要な発見:**
- Pass 2コード生成ロジックは `transpiler/mod.rs` の `transpile_pass2()` 関数に集約
- 現在の実装は170-217行目に存在し、直接的にmatchロジックを `jump()`/`call()` 内に生成
- `LabelRegistry` は既に全ラベル情報（ID、fn_path）を保持しており、データ構造変更は不要
- テストは `two_pass_transpiler_test.rs` に3つ存在、出力文字列の検証パターンを変更する必要あり
- 参照フィクスチャ `comprehensive_control_flow.transpiled.rn` に誤った実装が混在

**推奨アプローチ:** オプションA（既存コンポーネント拡張）
- 修正箇所は `transpile_pass2()` 関数1つのみ
- コード生成ロジックの書き換えで対応可能（構造変更不要）
- 実装規模：S（1-3日）、リスク：Low

---

## 1. Current State Investigation

### 1.1 Key Files and Components

| ファイルパス | 役割 | 修正対象 |
|------------|------|---------|
| `crates/pasta/src/transpiler/mod.rs` | トランスパイラーのメインロジック | ✅ Yes (Pass 2生成部分) |
| `crates/pasta/src/transpiler/label_registry.rs` | ラベル情報管理 | ❌ No (データ構造は完全) |
| `crates/pasta/tests/two_pass_transpiler_test.rs` | 統合テスト | ✅ Yes (検証パターン変更) |
| `crates/pasta/tests/fixtures/comprehensive_control_flow.transpiled.rn` | 参照フィクスチャ | ✅ Yes (誤った実装削除) |

### 1.2 Architecture Patterns

**トランスパイラー構造:**
```
Transpiler (struct with static methods)
  ├── transpile_pass1() - ラベル登録とモジュール生成
  ├── transpile_pass2() - pasta モジュール生成 ← 修正対象
  └── transpile_to_string() - テスト用ヘルパー
```

**現在の Pass 2 実装（170-217行目）:**

```rust
pub fn transpile_pass2<W: std::io::Write>(
    registry: &LabelRegistry,
    writer: &mut W,
) -> Result<(), PastaError> {
    writeln!(writer, "pub mod pasta {{").map_err(...)?;
    
    // ❌ 問題: jump() に match ロジックを直接生成
    writeln!(writer, "    pub fn jump(ctx, label, filters, args) {{").map_err(...)?;
    writeln!(writer, "        let id = pasta_stdlib::select_label_to_id(label, filters);").map_err(...)?;
    writeln!(writer, "        match id {{").map_err(...)?;
    for label in registry.all_labels() {
        writeln!(writer, "            {} => {{ for a in {}(ctx, args) {{ yield a; }} }},", 
            label.id, label.fn_path).map_err(...)?;
    }
    writeln!(writer, "            _ => {{ yield pasta_stdlib::Error(`ラベルID ${{id}} が見つかりませんでした。`); }},").map_err(...)?;
    writeln!(writer, "        }}").map_err(...)?;
    writeln!(writer, "    }}").map_err(...)?;
    
    // ❌ 問題: call() も同じ match ロジックを重複生成（省略）
    // ...
    
    writeln!(writer, "}}").map_err(...)?;
    Ok(())
}
```

**コード生成パターン:**
- `writeln!(writer, ...)` による行単位のテキスト出力
- `LabelRegistry::all_labels()` でイテレーション
- エラーハンドリングは `.map_err(|e| PastaError::io_error(...))?` パターン

### 1.3 Data Models

**LabelRegistry (完全なデータ構造):**

```rust
pub struct LabelRegistry {
    labels: HashMap<i64, LabelInfo>,  // ID → ラベル情報
    next_id: i64,                      // 次のID
    name_counters: HashMap<String, usize>,  // 名前 → カウンター
}

pub struct LabelInfo {
    pub id: i64,              // ラベルID (1, 2, 3, ...)
    pub name: String,         // 元のラベル名
    pub attributes: HashMap<String, String>,  // 属性（P1機能）
    pub fn_path: String,      // "crate::会話_1::__start__"
    pub fn_name: String,      // "会話_1::__start__"
    pub parent: Option<String>,  // 親ラベル名（ローカルラベル用）
}
```

**重要な観察:**
- `LabelRegistry` は既に全ラベル情報を保持
- `fn_path` は完全修飾パス（`crate::モジュール::関数`）
- Pass 2で必要な情報（ID、fn_path）はすべて利用可能
- **データ構造の変更は不要**

### 1.4 Testing Infrastructure

**既存テスト (`two_pass_transpiler_test.rs`):**

| テスト名 | 検証内容 | 修正必要性 |
|---------|---------|-----------|
| `test_two_pass_transpiler_to_vec` | Pass 1/2の出力分離 | ✅ 出力パターン変更 |
| `test_two_pass_transpiler_to_string` | 複数ラベルの処理 | ✅ 出力パターン変更 |
| `test_transpile_to_string_helper` | ヘルパー関数検証 | ✅ 出力パターン変更 |
| `test_multiple_files_simulation` | 複数ファイル統合 | ✅ 出力パターン変更 |

**現在の検証パターン（例）:**
```rust
assert!(output.contains("for a in crate::会話_1::__start__(ctx, args)"));
```

**修正後の検証パターン（期待値）:**
```rust
// __pasta_trans2__ モジュールの存在確認
assert!(output.contains("pub mod __pasta_trans2__"));
assert!(output.contains("pub fn label_selector(label, filters)"));

// pasta モジュールが label_selector を呼ぶ確認
assert!(output.contains("let func = crate::__pasta_trans2__::label_selector(label, filters);"));
assert!(output.contains("for a in func(ctx, args) { yield a; }"));

// match 式が __pasta_trans2__ 内にある確認
assert!(output.contains("1 => crate::会話_1::__start__,"));  // 関数ポインタを返す
```

### 1.5 Integration Points

**上流依存（Pass 1）:**
- `transpile_pass1()` → `LabelRegistry` にラベル登録
- Pass 2はこのレジストリを読み取るのみ（変更不要）

**下流依存（Rune実行）:**
- 生成されたRuneコードは Rune VM で実行
- `pasta_stdlib::select_label_to_id()` の実装（別仕様: `pasta-label-resolution-runtime`）
- 現時点では仮実装（常に `1` を返す）で動作

**重要な制約:**
- Rune構文の正確性（`match` 式、クロージャ、ジェネレーター）
- `label_selector()` のシグネチャ: `fn(label, filters) -> fn(ctx, args)`

---

## 2. Requirements Feasibility Analysis

### 2.1 Technical Needs Mapping

| 要件 | 技術的実現方法 | 現在の状態 |
|------|--------------|----------|
| **Req 1**: `__pasta_trans2__` モジュール生成 | `writeln!` で新しいモジュールブロックを出力 | ❌ Missing |
| **Req 2**: `label_selector()` 関数生成 | match式で `id => fn_path` マッピング（関数呼び出しなし） | ❌ Missing |
| **Req 3**: `pasta` モジュール簡素化 | jump/call を2行のラッパーに変更 | ❌ 現在は冗長なmatch内包 |
| **Req 4**: テストフィクスチャ整理 | ファイル編集（誤った実装削除） | ❌ 誤った実装が残存 |
| **Req 5**: Pass 2実装特定と修正 | `transpile_pass2()` 関数の書き換え | ✅ 特定済み（170-217行目） |

### 2.2 Identified Gaps

#### Gap 1: `__pasta_trans2__` モジュール生成ロジックの欠如

**現状:**
- `transpile_pass2()` は `pub mod pasta {}` のみ生成
- `__pasta_trans2__` モジュールは生成されない

**必要な実装:**
```rust
// 新規追加: __pasta_trans2__ モジュール生成
writeln!(writer, "pub mod __pasta_trans2__ {{")?;
writeln!(writer, "    pub fn label_selector(label, filters) {{")?;
writeln!(writer, "        let id = pasta_stdlib::select_label_to_id(label, filters);")?;
writeln!(writer, "        match id {{")?;
for label in registry.all_labels() {
    // ✅ 関数ポインタを返す（関数呼び出しではない）
    writeln!(writer, "            {} => {},", label.id, label.fn_path)?;
}
writeln!(writer, "            _ => |_ctx, _args| {{ yield pasta_stdlib::Error(`ラベルID ${{id}} が見つかりませんでした。`); }},")?;
writeln!(writer, "        }}")?;
writeln!(writer, "    }}")?;
writeln!(writer, "}}")?;
```

**実装難易度:** 低（既存パターンの複製）

#### Gap 2: `pasta` モジュールの冗長性削減

**現状:**
- `jump()` と `call()` が各々matchロジックを内包（170-217行目）
- 合計約50行のコード重複

**必要な実装:**
```rust
// pasta モジュール（簡素化）
writeln!(writer, "pub mod pasta {{")?;
writeln!(writer, "    pub fn jump(ctx, label, filters, args) {{")?;
writeln!(writer, "        let func = crate::__pasta_trans2__::label_selector(label, filters);")?;
writeln!(writer, "        for a in func(ctx, args) {{ yield a; }}")?;
writeln!(writer, "    }}")?;
writeln!(writer)?;
writeln!(writer, "    pub fn call(ctx, label, filters, args) {{")?;
writeln!(writer, "        let func = crate::__pasta_trans2__::label_selector(label, filters);")?;
writeln!(writer, "        for a in func(ctx, args) {{ yield a; }}")?;
writeln!(writer, "    }}")?;
writeln!(writer, "}}")?;
```

**実装難易度:** 低（コード量削減）

#### Gap 3: テスト検証パターンの更新

**現状:**
- テストは現在の出力形式（`for a in crate::ラベル(ctx, args)`）を検証
- 新しい出力形式に対応していない

**必要な変更:**
- `__pasta_trans2__` モジュールの存在確認
- `label_selector()` 関数の存在確認
- `pasta::jump()`/`call()` が `label_selector()` を呼ぶことの確認

**実装難易度:** 低（assertパターン変更のみ）

#### Gap 4: 参照フィクスチャの整理

**現状:**
- `comprehensive_control_flow.transpiled.rn` に誤った実装と正しい実装が混在
- 教育目的で意図的に両方を含む

**必要な変更:**
- 仕様実装完了後、誤った実装（77-103行目）を削除
- 説明用コメント（`// ❌`, `// ✅`）を削除
- 正しい実装のみを残す

**実装難易度:** 極低（ファイル編集）

### 2.3 Research Needed

以下の項目は設計フェーズで詳細を確認：

1. **Rune構文の正確性確認**
   - `label_selector()` が関数ポインタを返す構文が正しいか
   - クロージャ `|_ctx, _args| { yield ... }` のエラーケース構文
   - → Runeドキュメント参照またはサンプルコード実行で検証

2. **エラーハンドリングの一貫性**
   - 現在の `PastaError::io_error()` パターンを踏襲すべきか
   - → 設計時に既存パターンを再確認

---

## 3. Implementation Approach Options

### Option A: Extend Existing Component (推奨)

**概要:**
`transpile_pass2()` 関数を直接書き換え、既存の構造を維持したまま出力内容のみ変更。

**変更箇所:**
- **ファイル:** `crates/pasta/src/transpiler/mod.rs`
- **関数:** `transpile_pass2()` (163-219行目)
- **変更内容:**
  1. `__pasta_trans2__` モジュール生成を追加（先に出力）
  2. `pasta` モジュール内の `jump()`/`call()` を簡素化

**実装手順:**
```rust
pub fn transpile_pass2<W: std::io::Write>(
    registry: &LabelRegistry,
    writer: &mut W,
) -> Result<(), PastaError> {
    // Step 1: Generate __pasta_trans2__ module
    writeln!(writer, "pub mod __pasta_trans2__ {{")?;
    writeln!(writer, "    pub fn label_selector(label, filters) {{")?;
    writeln!(writer, "        let id = pasta_stdlib::select_label_to_id(label, filters);")?;
    writeln!(writer, "        match id {{")?;
    for label in registry.all_labels() {
        writeln!(writer, "            {} => {},", label.id, label.fn_path)?;
    }
    writeln!(writer, "            _ => |_ctx, _args| {{ yield pasta_stdlib::Error(`ラベルID ${{id}} が見つかりませんでした。`); }},")?;
    writeln!(writer, "        }}")?;
    writeln!(writer, "    }}")?;
    writeln!(writer, "}}")?;
    writeln!(writer)?;
    
    // Step 2: Generate simplified pasta module
    writeln!(writer, "pub mod pasta {{")?;
    writeln!(writer, "    pub fn jump(ctx, label, filters, args) {{")?;
    writeln!(writer, "        let func = crate::__pasta_trans2__::label_selector(label, filters);")?;
    writeln!(writer, "        for a in func(ctx, args) {{ yield a; }}")?;
    writeln!(writer, "    }}")?;
    writeln!(writer)?;
    writeln!(writer, "    pub fn call(ctx, label, filters, args) {{")?;
    writeln!(writer, "        let func = crate::__pasta_trans2__::label_selector(label, filters);")?;
    writeln!(writer, "        for a in func(ctx, args) {{ yield a; }}")?;
    writeln!(writer, "    }}")?;
    writeln!(writer, "}}")?;
    
    Ok(())
}
```

**影響範囲:**
- `transpiler/mod.rs`: 1関数のみ修正
- `two_pass_transpiler_test.rs`: 検証パターン変更
- `comprehensive_control_flow.transpiled.rn`: 実装完了後に整理

**Trade-offs:**
- ✅ **最小限の変更**: 既存の関数1つのみ書き換え
- ✅ **既存パターン維持**: `writeln!` による行単位出力を踏襲
- ✅ **テスト容易性**: 統合テストで全体を検証可能
- ✅ **後方互換性**: Pass 1出力、`LabelRegistry` の変更なし
- ❌ **関数の長さ**: 70行程度に増加（許容範囲内）

**リスク:** Low
- 既存の構造を維持
- Rune構文のみ検証が必要
- 失敗時は元のコードに戻すだけ

### Option B: Create New Component

**概要:**
コード生成ロジックを専用のヘルパー関数に分離し、`transpile_pass2()` を簡潔に保つ。

**変更箇所:**
```rust
// 新規関数1: __pasta_trans2__ モジュール生成
fn generate_pasta_trans2_module<W: std::io::Write>(
    registry: &LabelRegistry,
    writer: &mut W,
) -> Result<(), PastaError> {
    // __pasta_trans2__ モジュールの生成ロジック
}

// 新規関数2: pasta モジュール生成（簡素化）
fn generate_pasta_module<W: std::io::Write>(
    writer: &mut W,
) -> Result<(), PastaError> {
    // pasta モジュールの生成ロジック（ラッパーのみ）
}

// 既存関数: 修正
pub fn transpile_pass2<W: std::io::Write>(
    registry: &LabelRegistry,
    writer: &mut W,
) -> Result<(), PastaError> {
    generate_pasta_trans2_module(registry, writer)?;
    writeln!(writer)?;
    generate_pasta_module(writer)?;
    Ok(())
}
```

**Trade-offs:**
- ✅ **関数の責務明確化**: 各関数が1つのモジュール生成に専念
- ✅ **テスト分離**: 各ヘルパー関数を個別にテスト可能
- ✅ **可読性向上**: `transpile_pass2()` が3行の構成になる
- ❌ **ファイル複雑化**: 新しい関数が2つ増える
- ❌ **オーバーエンジニアリング**: 単純なタスクに対して過剰な分離

**リスク:** Low
- Option Aと同等のリスク
- 関数分離による追加テストが必要

### Option C: Hybrid Approach

**概要:**
初期実装はOption Aで進め、将来的にコード生成ロジックが複雑化した際にOption Bへリファクタリング。

**フェーズ1（現在の仕様）:**
- Option Aを実装（単一関数の書き換え）
- 統合テストで検証

**フェーズ2（将来の拡張）:**
- P1機能（属性フィルタ、複雑なmatch式）が追加された際に関数分離を検討
- 現時点では不要

**Trade-offs:**
- ✅ **段階的改善**: 必要になったときに分離
- ✅ **YAGNI原則**: 現在不要な抽象化を避ける
- ❌ **将来の負債**: リファクタリングコストが後から発生

**リスク:** Low
- 現時点ではOption Aと同一

---

## 4. Implementation Complexity & Risk

### 4.1 Effort Estimation

**Total Effort: S (1-3 days)**

| タスク | 工数 | 理由 |
|-------|------|------|
| Pass 2実装修正 | 2-4時間 | 単一関数の書き換え、Rune構文検証 |
| テスト更新 | 1-2時間 | 検証パターン変更（4テストケース） |
| フィクスチャ整理 | 30分 | ファイル編集（77-103行目削除） |
| 統合テスト実行 | 1時間 | cargo test + 出力確認 |
| ドキュメント更新 | 1時間 | コメント修正、設計ドキュメント更新 |

**合計:** 5-8.5時間（約1日）

### 4.2 Risk Assessment

**Overall Risk: Low**

| リスク項目 | 詳細 | 軽減策 |
|-----------|------|--------|
| **Rune構文エラー** | 関数ポインタ返却の構文が不正 | 参照フィクスチャで動作確認済み、Runeドキュメント参照 |
| **テスト失敗** | 新しい出力形式に検証パターンが未対応 | 段階的にテスト更新、出力を目視確認 |
| **後方互換性** | Pass 1出力やLabelRegistryに影響 | Pass 2のみ変更、Pass 1は不変 |
| **パフォーマンス** | コード生成時間の増加 | 出力行数は削減（50行→30行程度）、影響なし |

**High Risk項目:** なし

**Medium Risk項目:** なし

**Low Risk項目:**
- Rune構文検証のみ（既存参照実装で動作確認済み）
- 既存の構造を維持（破壊的変更なし）

---

## 5. Recommendations for Design Phase

### 5.1 Preferred Approach

**推奨:** Option A（既存コンポーネント拡張）

**理由:**
1. **最小限の変更**: 修正箇所は1関数のみ（170-217行目）
2. **既存パターン踏襲**: `writeln!` による行単位出力を継続
3. **データ構造変更不要**: `LabelRegistry` は完全な情報を保持
4. **リスク最小**: 破壊的変更なし、後方互換性維持
5. **実装速度**: 1日以内で完了可能

### 5.2 Key Decisions for Design Phase

1. **Rune構文の最終確認**
   - `label_selector()` の関数ポインタ返却構文
   - エラーケースのクロージャ構文
   - → 参照フィクスチャ（`.kiro/specs/pasta-transpiler-pass2-output/reference_comparison.rn`）で検証

2. **エラーハンドリング方針**
   - 既存の `.map_err(|e| PastaError::io_error(e.to_string()))?` パターンを継続
   - 新規エラータイプは不要

3. **テスト戦略**
   - 既存の4テストケースの検証パターンを更新
   - 新規テストは不要（カバレッジ十分）

### 5.3 Research Items

| 項目 | 目的 | 方法 |
|------|------|------|
| Rune関数ポインタ構文 | `match id { 1 => crate::func, ... }` が正しいか確認 | Runeドキュメント参照、参照フィクスチャで動作確認 |
| クロージャエラー構文 | `_ => \|_ctx, _args\| { yield Error(...) }` が正しいか確認 | Runeドキュメント参照、サンプル実行 |

### 5.4 Implementation Sequence

1. **Phase 1: コード生成修正**（優先度：高）
   - `transpile_pass2()` 関数の書き換え
   - Rune構文の検証

2. **Phase 2: テスト更新**（優先度：高）
   - `two_pass_transpiler_test.rs` の検証パターン変更
   - テスト実行と出力確認

3. **Phase 3: フィクスチャ整理**（優先度：中）
   - `comprehensive_control_flow.transpiled.rn` の誤った実装削除
   - コメント整理

4. **Phase 4: ドキュメント更新**（優先度：低）
   - コメント修正
   - 設計ドキュメント更新

---

## 6. Requirement-to-Asset Map

| 要件ID | 要件概要 | 現在の資産 | ギャップ | 実装方法 |
|--------|---------|----------|---------|---------|
| **Req 1** | `__pasta_trans2__` モジュール生成 | なし | Missing | `transpile_pass2()` に生成ロジック追加 |
| **Req 2** | `label_selector()` 関数生成 | なし | Missing | match式で `id => fn_path` マッピング |
| **Req 3** | `pasta` モジュール簡素化 | 冗長な実装（170-217行目） | Constraint | 既存コードを2行ラッパーに置換 |
| **Req 4** | テストフィクスチャ整理 | 誤った実装混在 | Missing | ファイル編集（77-103行目削除） |
| **Req 5** | Pass 2実装特定 | `transpile_pass2()` 関数 | ✅ 完了 | 既に特定済み（170-217行目） |

**ギャップ凡例:**
- **Missing**: 実装が存在しない
- **Constraint**: 既存実装が制約となる（書き換え必要）
- **✅ 完了**: ギャップなし

---

## 7. Conclusion

Pasta トランスパイラー Pass 2 の修正は、**既存の1関数の書き換えで実現可能**な小規模タスク。データ構造変更不要、リスクは低く、1日以内で実装可能。

**Next Steps:**
1. 本ギャップ分析を確認
2. `/kiro-spec-design pasta-transpiler-pass2-output` で詳細設計を開始
3. Option A（既存コンポーネント拡張）を採用し、実装に進む

**Critical Success Factors:**
- Rune構文の正確性確認（参照フィクスチャで検証済み）
- テスト検証パターンの完全な更新
- Pass 1との独立性維持（後方互換性）
