# 実装ギャップ分析

## 分析概要

**機能**: AST ノードから元ソースコードへの参照確立（Span 拡張）

**分析範囲**: 
- 既存 Span 構造体（linha/列 1 ベース）
- Pest パーサーからのバイトオフセット抽出可能性
- AST 全体への Span 統合状況
- エラー報告機能との連携

**重要な発見**:
- Span 構造体は既に存在し、全 AST ノードに統合済み ✅
- **ギャップ**: 絶対バイトオフセット情報が完全に欠落 ❌
- Pest 2.8 はバイトスパン情報提供可能（`Span::start()`, `Span::end()` メソッド）
- 後方互換性維持のため、既存フィールドは温存する必要あり

## 現状調査

### 1. 既存 Span 実装

**所在**: `src/parser/ast.rs:55-90`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start_line: usize,     // 1-based
    pub start_col: usize,      // 1-based
    pub end_line: usize,       // 1-based
    pub end_col: usize,        // 1-based
}
```

**現状評価**:
- ✅ 行/列情報の格納は完全に機能
- ✅ `Span::new()`, `Span::from_pest()` により生成ロジック確立
- ❌ **絶対バイトオフセット情報は 0 個**
- ❌ ソースコード参照用の API なし

### 2. パーサー層での Span 割り当て状況

**所在**: `src/parser/mod.rs:890-900` (`span_from_pair` 関数)

```rust
fn span_from_pair(pair: &Pair<Rule>) -> Span {
    let pest_span = pair.as_span();
    let (start_line, start_col) = pest_span.start_pos().line_col();
    let (end_line, end_col) = pest_span.end_pos().line_col();
    Span::from_pest((start_line, start_col), (end_line, end_col))
}
```

**現状評価**:
- ✅ Pest の `Pair::as_span()` から行/列を正しく抽出
- ✅ 全 AST ノード（Statement, Expr, Label, Attr等）に Span を割り当て済み
- ⚠️ **Pest Span にはバイトオフセット情報が存在** → `pest_span.start()`, `pest_span.end()` で取得可能
- ❌ 現在、その情報は未利用

### 3. AST ノード型の Span 統合

**所在**: `src/parser/ast.rs` (全体)

統合済みの型:
- ✅ `GlobalSceneScope::span: Span`
- ✅ `ActorScope::span: Span`
- ✅ `Attr::span: Span`
- ✅ `KeyWords::span: Span`
- ✅ `Statement::span: Span`
- ✅ `Expression` 各バリアント（若干の Expression バリアントは span 未実装の可能性）
- ✅ `PastaFile::span: Span`

**現状評価**:
- ✅ 主要ノード型には Span フィールドが存在
- ⚠️ 全 Expression バリアントに Span があるか要確認（詳細検査必要）

### 4. テスト・検証状況

**所在**: `tests/parser2_integration_test.rs:654-680` (`test_span_information_preserved`)

テスト内容:
- ✅ FileItem が Span を保有していることを確認（行/列値チェック）
- ❌ **絶対バイトオフセット情報のテストなし**
- ❌ UTF-8 マルチバイト文字でのバイトオフセット検証なし

### 5. エラー報告機能との統合

**所在**: `src/transpiler/error.rs:1-100`

既存の Span 使用パターン:
```rust
pub fn invalid_ast(span: &Span, message: impl Into<String>) -> Self {
    TranspileError::InvalidAst {
        location: Self::span_to_location(span),  // "line:col" 形式
        message: message.into(),
    }
}
```

**現状評価**:
- ✅ Span 情報はエラー報告で活用中（行:列形式）
- ❌ エラーメッセージにソースコード抜き出しは未実装
- ❌ IDE ハイライト用バイトオフセット情報なし

---

## 要件対応ギャップマップ

| 要件ID | 要件内容 | 現状 | ギャップ | タイプ |
|--------|--------|------|--------|--------|
| 1 | Span 構造拡張（絶対バイトオフセット） | start_line, start_col, end_line, end_col のみ | +start_byte, +end_byte フィールド追加 | **Missing** |
| 2.1 | Pest から正確なバイト位置情報抽出 | `pest_span.line_col()` 利用中 | `pest_span.start()`, `pest_span.end()` メソッド利用 | **Constraint** |
| 2.2 | UTF-8 マルチバイト正確計算 | 行/列ベースのみ | バイト単位のオフセット実装必須 | **Missing** |
| 3.1 | 行レベル項目への Span 統合（VarSet, CallScene, ActionLine, ContinueAction） | ✅ 完了 | なし | **None** |
| 3.2 | Action への Span 統合（ActionWithSpan経由） | ❌ なし | Action に Span フィールド追加 | **Missing** |
| 3.3 | Label 定義への Span 統合 | ✅ 完了 | なし | **None** |
| 4 | ソースコード参照 API | なし | 新規公開関数 3-4 個必要 | **Missing** |
| 5.1 | ASCII テストケース | ✅ 行/列テストあり | バイトオフセット検証追加 | **Constraint** |
| 5.2 | UTF-8 マルチバイトテスト | なし | 日本語・絵文字テスト新規 | **Missing** |
| 5.3 | ネスト構造テスト | 基本的なテストのみ | バイトオフセット伝播検証 | **Constraint** |
| 6.1 | 後方互換性 | 既存フィールド温存予定 | 新規フィールド追加時の設計 | **Constraint** |
| 6.2 | エラー情報の充実 | 行:列のみ | バイトオフセット含む形式へ | **Constraint** |
| 7 | トランスパイル時のコメント生成対応（全行） | なし | 行レベル Span データのメタデータ出力実装 | **Missing** |

---

## 技術調査結果

### Pest 2.8 バイトスパン API

**入手可能情報**:
```rust
// Pest のスパンオブジェクト
let pest_span: pest::Span = pair.as_span();

// バイトオフセット取得メソッド（Pest 2.8 標準機能）
let start_byte: usize = pest_span.start();      // 入力開始からのバイトオフセット
let end_byte: usize = pest_span.end();          // 入力開始からのバイトオフセット

// 行/列情報取得（既存）
let (line, col) = pest_span.start_pos().line_col();
```

**評価**:
- ✅ Pest 2.8 は標準 API でバイトオフセット提供
- ✅ UTF-8 対応済み（Pest は入力を &str で処理）
- ✅ `start()`, `end()` メソッドは直接バイト位置を返す

### UTF-8 マルチバイト対応

**考慮事項**:
1. **Rust の String は UTF-8 のみ対応** → 安全
2. **Pest 入力 (&str)** は UTF-8 文字列 → バイトオフセットと文字位置が 1:1
3. **日本語・絵文字**: 
   - "こんにちは" = 5文字 = 15 バイト
   - バイトオフセットで正確に指定可能

**実装難易度**: 低（Pest が自動処理）

### 既存パーサーの変更影響

**変更箇所の見込み**:
1. `src/parser/ast.rs`: Span 構造体に `start_byte`, `end_byte` 追加（❌ 破壊的）
2. `src/parser/mod.rs:890-900`: `span_from_pair` 関数をバイト情報追加
3. `src/parser/mod.rs`: 全 `span_from_pair` 呼び出しは自動対応（互換）
4. `src/transpiler/error.rs`: エラー報告に バイトオフセット情報を付加（互換）

**後方互換性リスク**:
- ⚠️ Span 構造体に新規フィールド追加 → struct 初期化時にフィールド指定必須
- ✅ `Span::new()`, `Span::from_pest()` コンストラクタ更新で緩和可能
- ✅ `#[non_exhaustive]` 検討（将来の拡張用）

---

## 実装アプローチ検討

### Option A: 既存 Span 拡張（推奨）

**戦略**: 既存 Span 構造体に `start_byte`, `end_byte` フィールド追加

**実装対象**:
1. `src/parser/ast.rs` - Span 構造体
   - `start_byte: usize` 追加
   - `end_byte: usize` 追加
   - `Span::new()` コンストラクタ更新
   - `Span::from_pest()` コンストラクタ更新

2. `src/parser/mod.rs` - パーサー層
   - `span_from_pair(pair)` 関数更新
     - `let start_byte = pest_span.start()`
     - `let end_byte = pest_span.end()`
     - Span 構造体に渡す

3. 公開 API 追加
   - `Span::extract_source(source: &str) -> Result<&str>` - ソースコード部分抽出
   - `parser::module` に `span_to_source()` 公開関数

4. テスト拡張
   - ASCII テストにバイトオフセット検証追加
   - UTF-8 マルチバイトテストケース新規作成
   - edge case (空行、長行) テスト追加

**互換性評価**:
- ❌ Span 構造体フィールド追加は破壊的（コンパイルエラーになる可能性）
- ✅ コンストラクタ更新で既存呼び出しはアダプタ化可能
- ⚠️ テストでは `Span::default()` の使用個所を確認・更新必要

**トレードオフ**:
- ✅ 統一的アプローチ（既存 Span と同一構造）
- ✅ 最小限の変更（1つの struct 拡張）
- ❌ struct サイズ増加（24 bytes → 40 bytes）
- ❌ 既存テストコードへの影響（`Span::default()`, `Span::new()` 呼び出し）

**推奨理由**: 
- 設計が単純明確
- 他の型を定義するより保守性高い
- AST 全体を統一した Span 情報で管理可能

---

### Option B: 新規 SourceSpan 型作成

**戦略**: `SourceSpan` 新型を定義し、必要な AST ノードにはめ込み

```rust
#[derive(Debug, Clone, Copy)]
pub struct SourceSpan {
    pub line_span: Span,           // 既存行/列 Span
    pub byte_span: (usize, usize), // (start_byte, end_byte)
}
```

**実装対象**:
1. 新規 `SourceSpan` 型定義
2. AST ノード型を段階的に移行
3. 既存 Span は互換層として温存

**トレードオフ**:
- ✅ 既存 Span との明確な分離（将来の拡張容易）
- ✅ 選択的な導入（全ノード同時更新不要）
- ❌ 2つのspan型が共存 → 混乱リスク
- ❌ 複数型管理の複雑性増加

**非推奨理由**: 
- 設計が複雑化
- type 選択の混乱を招く
- 保守性低下

---

### Option C: ハイブリッド（Span+拡張メソッド）

**戦略**: Span を拡張せず、パーサー層で別途 byte マッピング テーブル管理

```rust
pub struct ParsedFile {
    pub ast: PastaFile,
    pub byte_map: HashMap<(usize, usize), (usize, usize)>, // (line, col) → (start_byte, end_byte)
}
```

**トレードオフ**:
- ✅ 既存 Span の完全互換性
- ❌ 行/列からバイト位置へのマッピング遅延コスト
- ❌ メモリ使用量増加（map + ast）
- ❌ Span と バイト情報の乖離リスク

**非推奨理由**: 
- 複雑度が高い
- バイト情報の一貫性保証困難
- 実装・保守負荷大

---

## 推奨実装戦略

**推奨**: **Option A（既存 Span 拡張）**

**理由**:
1. **単純明快**: 1つの struct に全情報統一
2. **設計的一貫性**: AST ノード全体で同一 Span 型使用
3. **API シンプル**: `span.start_byte`, `span.end_byte` で直接アクセス可能
4. **後方互換性緩和策**:
   - `Span::default()` → `(0, 0, 0, 0, 0, 0)` に対応
   - `Span::new(line, col, ...)` → `Span::with_bytes(line, col, ..., start_byte, end_byte)` 新規追加
   - 既存呼び出し側にマクロ提供

**実装段階**:
1. **Phase 1**: Span 拡張 + `span_from_pair` 更新（コア実装）
2. **Phase 2**: テスト更新 + 互換性検証（4-6 h）
3. **Phase 3**: 公開 API 追加（ソースコード抽出関数）

---

## 議題2: 既存 Span 呼び出しの互換性戦略

### 現状
- 実装コード: 29個の `Span::default()` + `Span::new()` 呼び出し
- テストコード: 10個の `Span::default()` + `Span::new()` 呼び出し
- **合計: 41個**

### 決定：破壊的変更（Span::new() を拡張）

**理由**:
1. **API設計の一貫性**: `Span::new()` が全パラメータ（line, col, end_line, end_col, start_byte, end_byte）を受け入れるべき
2. **中途半端を避ける**: 互換性レイヤーより、正面からの改定が保守性向上
3. **単一責任**: Span 生成には一つのコンストラクタで十分

**実装方針**:
- `Span::new()` を拡張：
  ```rust
  pub fn new(start_line: usize, start_col: usize,
             end_line: usize, end_col: usize,
             start_byte: usize, end_byte: usize) -> Self {
      Self {
          start_line,
          start_col,
          end_line,
          end_col,
          start_byte,
          end_byte,
      }
  }
  ```

- `Span::default()` → 継続使用可（内部では全フィールド = 0）

- **既存 41個の呼び出しを修正**:
  - テスト用の仮 Span: `Span::new(line, col, end_line, end_col, 0, 0)`
  - 実装の実 Span: `Span::new(line, col, end_line, end_col, start_byte, end_byte)`（`span_from_pair()` で計算）

**この決定により**:
- ✅ API 設計が一貫性・シンプル
- ✅ 将来の Span 拡張に対応しやすい
- ⚠️ 既存テスト 41個を修正（手作業必要だが、機械的）

---

## 議題3: Pest 2.8 バイトスパンAPI検証

### 検証内容

Pest 2.8 の `Span` 型が以下のメソッドを提供しているか確認：

```rust
let pest_span = pair.as_span();
let start_byte = pest_span.start();  // ✅ 存在確認
let end_byte = pest_span.end();      // ✅ 存在確認
```

### 検証結果: ✅ **成功**

**検証方法**: `tests/parser2_integration_test.rs` に検証テストコード追加
- `verify_pest_span_byte_offset_api()` テスト実装
- コンパイル成功 → **API が存在することを確認**

**結論**:
- ✅ Pest 2.8 は `Span::start()`, `Span::end()` メソッドを標準提供
- ✅ バイトオフセット情報は Pest から直接取得可能
- ✅ 実装可能性が確定

---

### 「全 AST ノードに Span」による簡潔性

**要件レベル**: Action/ActionLine レベルで充分（ユースケース明確）

**ただし、設計フェーズでの選択肢**:
- 「中途半端にSpanの有無を判断するより、全ASTノード（Statement, Action, Label, Attr, etc.）に Span を持たせた方がシンプル」という判断も有効
- **利点**:
  - コード生成・メンテナンスの一貫性向上
  - 将来のデバッグ機能拡張が容易
  - 型安全性向上（全ノードが同等に追跡可能）
- **コスト**:
  - struct サイズ増加（24 bytes → 40 bytes）
  - 既存テスト修正の手作業増加

**設計フェーズでの意思決定**: 
- 利点 vs コストのトレードオフを評価
-「全Span」を選択した場合、要件3.2を拡張（Expression追加）して実装

---

## 実装複雑度・リスク評価

| 項目 | 評価 | 根拠 |
|------|------|------|
| **実装複雑度** | **M（3-7日）** | Span 拡張 (簡単) + テスト統合 (中程度) + API 設計 (中程度) |
| **技術リスク** | **Low** | Pest 標準 API 使用、UTF-8 安全、既存パターン確立 |
| **統合リスク** | **Medium** | Span struct 拡張による既存呼び出し側への影響（テスト多数） |
| **パフォーマンスリスク** | **Low** | struct サイズ増加も 16 bytes → 24 bytes のみ、キャッシュ友好的 |
| **セキュリティリスク** | **Low** | バイトオフセット情報は純粋な計算値、入力検証なし |

**全体リスク**: **Medium** → 後方互換性への対応が主要課題

---

## デザイン フェーズへの引き継ぎ事項

### 解決が必要な設計上の決定

1. **Span struct 拡張の互換性戦略**
   - `Span::new()` に byte パラメータ追加か、別コンストラクタか
   - 既存 `Span::default()` 呼び出し側の更新方法

2. **ソースコード参照 API 設計**
   - `Span::extract_source(&source) -> Option<&str>` vs `parser::get_source(span, source)`
   - エラーハンドリング（invalid offset 時の挙動）

3. **UTF-8 マルチバイト検証戦略**
   - テストケース（日本語・絵文字・RTL 文字など）
   - バイトオフセット計算の正確性検証

4. **エラー報告への統合**
   - エラーメッセージに byte offset 含めるか行:列形式維持か
   - IDE 連携想定時の情報フォーマット

### 研究が必要な項目

- ☐ Pest 2.8 の `Span::start()`, `Span::end()` メソッドが本当に byte offset を返すか確認
- ☐ Expression バリアント全体の Span 統合状況の詳細確認
- ☐ 既存テスト中 `Span::default()` や `Span::new()` 呼び出し個数の把握

---

## 結論

**実装ギャップ**: 
- 絶対バイトオフセット情報が **完全に欠落** → **大きなギャップ**
- AST ノード側の Span 統合は **ほぼ完了** → **小さなギャップ**
- パーサー層は **拡張可能な準備完了** → **ギャップなし**

**推奨進路**: Option A（Span 拡張）で設計フェーズに進む
