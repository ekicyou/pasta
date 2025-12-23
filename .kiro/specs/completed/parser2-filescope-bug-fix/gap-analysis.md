# 実装ギャップ分析: parser2-filescope-bug-fix

## 分析日
2025-12-23

## エグゼクティブサマリー

parser2-filescope-bug-fix実装は**中程度の複雑度（M: 3-5日）**・**中リスク**のバグ修正です。以下の理由から**Option A（AST構造の破壊的変更）を推奨**します：

- **文法準拠の必須性**: grammar.pest の `file = ( file_scope | global_scene_scope )*` 仕様に完全準拠する唯一の方法
- **transpiler2への影響**: 依存仕様（transpiler2-layer-implementation）が新AST構造を前提としている
- **既存コードへの影響範囲**: parser2関連のみ（transpilerは既存parserを使用、完全分離）
- **明確な移行パス**: PastaFile構造体の変更により、依存コードでコンパイルエラーが発生し、修正漏れを防止

**Critical Path**:
1. AST構造変更（PastaFile.items導入、file_scopeフィールド廃止）
2. パーサーロジック修正（上書き代入→push操作）
3. テストコード更新（parser2_integration_test.rs）
4. transpiler2実装の有効化

---

## 1. 現状調査

### 主要アセット - Parser2

| コンポーネント | 場所 | 目的 | サイズ |
|---------------|------|------|------|
| **Parser2 AST** | `src/parser2/ast.rs` | AST型定義 | 624行 |
| **Parser2 Module** | `src/parser2/mod.rs` | パーサー実装 | 906行 |
| **Parser2 Grammar** | `src/parser2/grammar.pest` | Pest文法定義 | 223行 |
| **Parser2 Tests** | `tests/parser2_integration_test.rs` | 統合テスト | 454行 |

### 既存AST構造

**現在のPastaFile構造**（バグを含む）:
```rust
// src/parser2/ast.rs:62-73
pub struct PastaFile {
    pub path: PathBuf,
    pub file_scope: FileScope,              // ← 単一フィールド（上書き問題）
    pub global_scenes: Vec<GlobalSceneScope>,
    pub span: Span,
}
```

**FileScope構造**:
```rust
// src/parser2/ast.rs:95-99
pub struct FileScope {
    pub attrs: Vec<Attr>,      // ファイルレベル属性
    pub words: Vec<KeyWords>,  // ファイルレベル単語定義
}
```

**GlobalSceneScope構造**:
```rust
// src/parser2/ast.rs:109-120
pub struct GlobalSceneScope {
    pub name: String,
    pub is_continuation: bool,
    pub attrs: Vec<Attr>,
    pub words: Vec<KeyWords>,
    pub code_blocks: Vec<CodeBlock>,
    pub local_scenes: Vec<LocalSceneScope>,
    pub span: Span,
}
```

### バグのある既存パーサーロジック

**問題コード**（src/parser2/mod.rs:135-138）:
```rust
for pair in pairs {
    match pair.as_rule() {
        Rule::file_scope => {
            file.file_scope = parse_file_scope(pair)?;  // ← BUG: 上書き代入
        }
        Rule::global_scene_scope => {
            let scene = parse_global_scene_scope(pair, &mut last_global_scene_name, filename)?;
            file.global_scenes.push(scene);  // ← OK: push操作
        }
        Rule::EOI => {}
        _ => {}
    }
}
```

**根本原因**:
- `file.file_scope = ...` は上書き代入のため、ループで複数回`Rule::file_scope`がマッチした場合、最後のもののみが残る
- `file.global_scenes.push(...)` は累積操作のため、複数`global_scene_scope`を正しく保持

### 文法仕様（正しい意図）

**grammar.pest:222**:
```pest
file = _{ SOI ~ ( file_scope | global_scene_scope )* ~ s ~ EOI }
```

**仕様の意図**:
- `( file_scope | global_scene_scope )*` = file_scopeとglobal_scene_scopeが**任意順序で複数回出現可能**
- 実装はこの順序と出現回数を正確に保持すべき

### 既存テストの依存状況

**parser2テスト** (tests/parser2_integration_test.rs):
- Line 22-23: `file.file_scope.attrs`, `file.file_scope.words` の直接アクセス（6箇所）
- Line 81-84: file_scopeフィールドへの依存
- **影響**: AST構造変更により、これらのテストは修正が必要

**transpilerテスト**:
- grep結果: `file.file_scope` の使用は**parser2テストのみ**
- 既存transpiler（src/transpiler/）は **parser::PastaFile** を使用（parser2::PastaFileとは別型）
- **影響**: transpilerへの影響なし（完全分離）

### 統合面

| インターフェース | 使用箇所 | 備考 |
|-----------------|---------|------|
| `lib.rs` | `pub mod parser2;` | 公開API（AST型を公開） |
| `PastaFile` | transpiler2（未実装） | **ブロック中**: transpiler2がPastaFile.itemsを期待 |
| Tests | parser2_integration_test.rs | file.file_scopeへの直接アクセス6箇所 |

---

## 2. 要件実現性分析

### 要件→技術要求のマッピング

| 要件 # | 要件名 | 技術要求 | 実現性 | ギャップ |
|-------|--------|----------|--------|---------|
| 1 | 複数FileScope保持 | Vec<FileScope>またはVec<FileItem> | ✅ Easy | None |
| 2 | FileScope/GlobalSceneScope交互出現 | Vec<FileItem>で順序保持 | ✅ Easy | None |
| 3 | ファイルレベル属性分離保持 | FileScope個別インスタンス | ✅ Easy | None |
| 4 | ファイルレベル単語定義累積 | 各FileScopeにwords保持 | ✅ Easy | None |
| 5 | AST構造破壊的変更 | PastaFile構造体変更 | ⚠️ Medium | **移行コスト** |
| 6 | パーサーロジック修正 | ループ内push操作 | ✅ Easy | None |
| 7 | テストケース追加 | 新規fixtureとテスト | ✅ Easy | None |
| 8 | transpiler2互換性 | FileItem列挙型API設計 | ✅ Medium | **設計調整** |
| 9 | エラーハンドリング保持 | Span情報維持 | ✅ Easy | None |
| 10 | ドキュメント更新 | docコメント、CHANGELOG | ✅ Easy | None |

### ギャップ評価

#### クリティカルギャップ（設計・研究必須）

**1. FileItem列挙型の設計**
- **要件**: file_scopeとglobal_scene_scopeを統一的に扱う
- **提案**:
  ```rust
  pub enum FileItem {
      FileScope(FileScope),
      GlobalSceneScope(GlobalSceneScope),
  }
  ```
- **設計課題**:
  - Span情報をどこに保持するか（各Scope型 vs. FileItemにラップ）
  - ヘルパーメソッド（`file_scopes()`, `global_scenes()`）の必要性
  - パターンマッチの利便性 vs. 型安全性

**2. 既存テストコードの移行戦略**
- **影響範囲**: parser2_integration_test.rs の6箇所
- **移行パターン**:
  ```rust
  // 修正前
  assert_eq!(file.file_scope.attrs.len(), 2);
  
  // 修正後（案1: ヘルパーメソッド）
  let file_scopes: Vec<&FileScope> = file.file_scopes();
  assert_eq!(file_scopes[0].attrs.len(), 2);
  
  // 修正後（案2: 直接パターンマッチ）
  let attrs: Vec<_> = file.items.iter()
      .filter_map(|item| match item {
          FileItem::FileScope(fs) => Some(&fs.attrs),
          _ => None
      })
      .flatten()
      .collect();
  assert_eq!(attrs.len(), 2);
  ```
- **決定事項**: 設計フェーズで利便性と明確性のバランスを評価

#### 中程度のギャップ（設計フェーズで詳細化）

**1. transpiler2への影響**
- **現状**: transpiler2-layer-implementation仕様が`PastaFile.items`を前提
- **gap-analysis.md確認**: "parser2のFileScope複数出現バグは修正済み"と記載
- **状況**: transpiler2はこのバグ修正を**ブロッキング依存**として認識
- **調整必要**:
  - transpiler2の設計書で期待するAPI（`file.items`のイテレーション）を確認
  - FileItem列挙型のパターンマッチパターンを文書化

**2. パフォーマンス影響**
- **要件**: 複数file_scopeの処理時間は線形増加を超えない
- **分析**: Vec<FileItem>のpush操作はO(1)償却、イテレーションはO(n)
- **結論**: 現実的なPastaファイル（file_scope数 < 100）では無視可能
- **懸念なし**: 非機能要件を満たす

**3. メモリ効率**
- **要件**: Vec<FileItem>のメモリ使用量は合計数に比例
- **分析**: 
  - 既存: `FileScope` (単一) + `Vec<GlobalSceneScope>`
  - 新規: `Vec<FileItem>` (FileScope + GlobalSceneScope統合)
- **オーバーヘッド**: FileItem列挙型のタグ（8バイト/要素）
- **結論**: 実用上問題なし（典型的ファイル: 数十要素程度）

---

## 3. 実装アプローチのオプション

### Option A: AST構造の破壊的変更（推奨）⭐

**根拠**: PastaFile構造体を変更し、`items: Vec<FileItem>`を導入、`file_scope`フィールドを廃止

**AST設計**:
```rust
// 新規: src/parser2/ast.rs
pub enum FileItem {
    FileAttr(Attr),                    // file_scope 内の属性
    GlobalWord(KeyWords),              // file_scope 内の単語定義
    GlobalSceneScope(GlobalSceneScope), // グローバルシーン
}

pub struct PastaFile {
    pub path: PathBuf,
    pub items: Vec<FileItem>,  // ← 新フィールド（3バリアント統合）
    pub span: Span,
    // 廃止予定:
    // pub file_scope: FileScope,  ← 廃止
    // pub global_scenes: Vec<GlobalSceneScope>,  ← 廃止
}

impl PastaFile {
    // ヘルパーメソッド（transpiler2での利便性向上）
    pub fn file_attrs(&self) -> Vec<&Attr> {
        self.items.iter().filter_map(|item| match item {
            FileItem::FileAttr(attr) => Some(attr),
            _ => None,
        }).collect()
    }
    
    pub fn words(&self) -> Vec<&KeyWords> {
        self.items.iter().filter_map(|item| match item {
            FileItem::GlobalWord(word) => Some(word),
            _ => None,
        }).collect()
    }
    
    pub fn global_scene_scopes(&self) -> Vec<&GlobalSceneScope> {
        self.items.iter().filter_map(|item| match item {
            FileItem::GlobalSceneScope(gs) => Some(gs),
            _ => None,
        }).collect()
    }
}
```

**パーサーロジック修正**:
```rust
// 修正後: src/parser2/mod.rs
fn build_ast(pairs: Pairs<Rule>, filename: &str) -> Result<PastaFile, PastaError> {
    let mut file = PastaFile::new(std::path::PathBuf::from(filename));
    let mut last_global_scene_name: Option<String> = None;

    for pair in pairs {
        match pair.as_rule() {
            Rule::file_scope => {
                // file_scope 内の attrs と words を個別の FileItem として追加
                let fs = parse_file_scope(pair)?;
                for attr in fs.attrs {
                    file.items.push(FileItem::FileAttr(attr));
                }
                for word in fs.words {
                    file.items.push(FileItem::GlobalWord(word));
                }
            }
            Rule::global_scene_scope => {
                let scene = parse_global_scene_scope(pair, &mut last_global_scene_name, filename)?;
                file.items.push(FileItem::GlobalSceneScope(scene));  // ← FIX: push操作
            }
            Rule::EOI => {}
            _ => {}
        }
    }

    Ok(file)
}
```

**利点**:
- ✅ grammar.pest仕様に**完全準拠**（file_scopeとglobal_scene_scopeの順序保持）
- ✅ transpiler2が期待する構造を直接提供
- ✅ 直感的（ファイル記述順序 = items配列順序）
- ✅ コンパイルエラーにより修正漏れを防止（`file.file_scope`アクセスがエラー）
- ✅ 将来の拡張性（新しいfile-levelアイテム追加が容易）

**欠点**:
- ❌ **破壊的変更**: セマンティックバージョン上はメジャーバージョンアップ必要
- ❌ 既存テストコード修正（parser2_integration_test.rs の6箇所）
- ❌ ヘルパーメソッド設計の追加工数（1-2時間）

**推定工数**: M (3-4日)
- AST構造変更: 0.5日
- パーサーロジック修正: 0.5日
- テストコード更新（既存6箇所 + 新規3件）: 1日
- ヘルパーメソッド実装・テスト: 0.5日
- ドキュメント更新: 0.5日
- 統合テスト・検証: 1日

**推定リスク**: Medium
- **コンパイルエラー検出**: `file.file_scope`への直接アクセスはコンパイルエラー → 修正漏れなし
- **テスト範囲**: parser2のみ、transpiler（既存parser使用）への影響なし
- **ランタイムリスク**: 低（AST構造のみ、ロジック変更最小）

---

### Option B: FileScope累積（影響範囲最小化）

**根拠**: `file_scope`フィールドを`file_scopes: Vec<FileScope>`に変更、`global_scenes`はそのまま

**AST設計**:
```rust
pub struct PastaFile {
    pub path: PathBuf,
    pub file_scopes: Vec<FileScope>,  // ← 複数保持
    pub global_scenes: Vec<GlobalSceneScope>,
    pub span: Span,
}
```

**パーサーロジック**:
```rust
Rule::file_scope => {
    file.file_scopes.push(parse_file_scope(pair)?);
}
```

**利点**:
- ✅ 変更範囲が小さい（1フィールドのみ）
- ✅ global_scenes分離維持（既存パターンに近い）
- ✅ 複数file_scopeを保持可能

**欠点**:
- ❌ **file_scopeとglobal_scene_scopeの交互出現順序が保持されない**（要件2違反）
- ❌ transpiler2側で順序復元が必要（困難かつエラーの原因）
- ❌ grammar.pest仕様の意図を正確に反映しない

**推定工数**: S (2-3日)
**推定リスク**: High（要件2未達、transpiler2でのバグリスク）

**非推奨理由**: 要件2（交互出現順序保持）を満たさないため、transpiler2実装を困難にする

---

### Option C: ハイブリッド（段階的移行）

**根拠**: 初期実装でOption Bを採用、transpiler2実装後にOption Aへリファクタリング

**フェーズ1**: FileScope累積のみ（Option B）
**フェーズ2**: transpiler2実装時に順序問題が顕在化
**フェーズ3**: Option Aへ全面移行

**利点**:
- ✅ 段階的リスク分散

**欠点**:
- ❌ **2度手間**（フェーズ3で再び破壊的変更）
- ❌ フェーズ2で技術的負債が蓄積
- ❌ transpiler2実装が複雑化（順序復元ロジック実装 → 後で削除）

**推定工数**: L (6-8日、累積）
**推定リスク**: High（技術的負債、リファクタリングコスト）

**非推奨理由**: 総工数増加、技術的負債蓄積

---

## 4. 実装複雑度とリスク評価

### 工数見積もり

**Option A（推奨）**: **M (3-4日)**
- AST設計・実装: 0.5日
- パーサーロジック: 0.5日
- テスト更新: 1日
- ヘルパーメソッド: 0.5日
- ドキュメント: 0.5日
- 統合検証: 1日

**根拠**:
- 既存パターン（GlobalSceneScope処理）の流用可能
- コンパイラがエラー検出を支援（修正漏れなし）
- parser2スコープ限定（transpiler影響なし）

### リスク評価

**Option A**: **Medium**

**リスクポイント**:
1. **破壊的変更の影響範囲**: PastaFile公開API変更
   - **緩和策**: parser2は新モジュール、既存transpilerは完全分離
   - **検証**: コンパイルエラーで未修正箇所を検出
   
2. **ヘルパーメソッドの使いやすさ**: file_scopes(), global_scenes()の設計
   - **緩和策**: 設計フェーズでAPI使用例を検証
   - **検証**: doctest追加

3. **テストカバレッジの維持**: 既存テスト修正時のリグレッション
   - **緩和策**: 段階的テスト修正（既存1件修正→実行→次へ）
   - **検証**: cargo test --package pasta --lib parser2

**一行理由**: 変更範囲限定（parser2のみ）、コンパイラ支援あり、既存パターン流用可能

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ

**Option A（AST構造の破壊的変更）** を推奨

**理由**:
1. **文法仕様準拠**: grammar.pestの意図を正確に実装する唯一の方法
2. **transpiler2有効化**: ブロッキング依存解除に必須
3. **長期的メンテナンス**: 技術的負債なし、直感的なデータ構造
4. **リスク管理**: 影響範囲限定（parser2のみ）、コンパイラ支援

### 主要決定事項

以下を設計フェーズで詳細化：

1. **FileItem列挙型のAPI設計**
   - Span情報の保持方法
   - ヘルパーメソッドの必要性と実装
   - パターンマッチの推奨パターン

2. **テストコード移行戦略**
   - ヘルパーメソッド vs. 直接パターンマッチ
   - テストケースの可読性維持

3. **transpiler2との調整**
   - FileItem列挙型のイテレーションパターン文書化
   - Pass1でのfile_scope順次処理の実装例

### 研究項目（設計フェーズで実施）

1. **Span情報の最適配置**
   - 各Scope型にSpan（現状） vs. FileItemにSpan（ラップ）
   - エラーメッセージの品質への影響

2. **ヘルパーメソッドのシグネチャ**
   - `&FileScope` vs. `FileScope`（所有権）
   - `Vec<&FileScope>` vs. `impl Iterator<Item = &FileScope>`（パフォーマンス）

3. **CHANGELOG記述方針**
   - 破壊的変更の明確な記載
   - 移行ガイドの詳細度

---

## 6. 実装チェックリスト（設計書用）

設計フェーズで以下を具体化：

- [ ] FileItem列挙型の完全な型定義（Span配置決定）
- [ ] PastaFile構造体の新定義（ヘルパーメソッド含む）
- [ ] build_ast()関数の修正ロジック
- [ ] parse_file_scope()の変更有無確認
- [ ] テストコード移行パターン（6箇所の具体例）
- [ ] 新規テストケース3件の詳細仕様
- [ ] エラーメッセージ変更の影響評価
- [ ] docコメント更新箇所リスト
- [ ] CHANGELOG.mdエントリ案

---

## 参考資料

### 関連コードベース

- **AST定義**: src/parser2/ast.rs (Lines 62-99: PastaFile, FileScope)
- **パーサーロジック**: src/parser2/mod.rs (Lines 135-138: バグ箇所)
- **文法定義**: src/parser2/grammar.pest (Line 222: file rule)
- **テストコード**: tests/parser2_integration_test.rs (Lines 22-23, 81-84: file_scope依存)

### 依存仕様

- **transpiler2-layer-implementation**: .kiro/specs/transpiler2-layer-implementation/gap-analysis.md
  - Lines 9-16: "parser2のFileScope複数出現バグは修正済み"想定
  - Requirements 11, 15がこのバグ修正に依存

### 文法仕様

- **SPECIFICATION.md**: 第5章 File Scope（詳細未確認、設計フェーズで参照）
- **grammar.pest**: `file = ( file_scope | global_scene_scope )*` の意図

---

## 結論

parser2-filescope-bug-fixは**Option A（AST構造の破壊的変更）**で実装すべきです。

**判断根拠**:
- grammar.pest仕様への完全準拠が必須
- transpiler2実装のブロッキング依存解除
- 影響範囲限定（parser2のみ、transpiler無影響）
- 長期的なメンテナンス性向上

**次ステップ**: `/kiro-spec-design parser2-filescope-bug-fix` で詳細設計を実施
