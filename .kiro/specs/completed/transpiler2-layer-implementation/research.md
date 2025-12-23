# 調査レポート: transpiler2-layer-implementation

## 1. 調査スコープ

**フィーチャータイプ**: Extension（拡張型）
**ディスカバリー手法**: Light Discovery（軽量調査）

### 調査理由

transpiler2は既存システムの拡張であり、以下の理由から軽量調査が適切：
- 既存transpilerパターン（2パス戦略、TranspileContext）の踏襲
- 既存レジストリ（SceneRegistry, WordDefRegistry）のAST型非依存性
- parser2 ASTは既にparser2-filescope-bug-fixで安定化済み

---

## 2. 拡張ポイント分析

### 2.1 既存パターン調査

| コンポーネント | 場所 | パターン | 再利用可否 |
|---------------|------|---------|-----------|
| **Transpiler** | `src/transpiler/mod.rs` | 2パス変換エンジン | ✅ パターン踏襲 |
| **TranspileContext** | `src/transpiler/mod.rs:17-111` | 関数スコープ解決 | ⚠️ parser2用に再設計 |
| **SceneRegistry** | `src/transpiler/scene_registry.rs` | シーン登録・ID管理 | ✅ 完全再利用（AST型非依存） |
| **WordDefRegistry** | `src/transpiler/word_registry.rs` | 単語定義登録 | ✅ 完全再利用（AST型非依存） |

### 2.2 拡張ポイント

#### P1: モジュール構造
- **既存**: `src/transpiler/` - parser用transpiler
- **新規**: `src/transpiler2/` - parser2用transpiler2
- **共有**: `src/registry/` - 共有レジストリモジュール（Option B-改改）

#### P2: API拡張
- **既存API**: `Transpiler::transpile_pass1()`, `transpile_pass2()`
- **新規API**: `Transpiler2::transpile_pass1()`, `transpile_pass2()`
- **シグネチャ差異**: 入力型が `&parser::PastaFile` → `&parser2::PastaFile`

#### P3: エラー型拡張
- **既存**: `PastaError` (src/error.rs)
- **新規**: `TranspileError` (src/transpiler2/error.rs)
- **設計**: `thiserror` 2系使用、既存パターン踏襲

---

## 3. 依存関係チェック

### 3.1 直接依存

| 依存先 | バージョン | 用途 | リスク |
|--------|-----------|------|--------|
| parser2 | current | 入力AST提供 | ✅ 解決済（parser2-filescope-bug-fix） |
| registry | 新規作成 | シーン・単語登録 | ⚠️ 移動作業必要 |
| runtime | current | 生成コード実行 | ✅ 互換性維持 |

### 3.2 逆依存（影響分析）

| 依存元 | 影響 | 軽減策 |
|--------|------|--------|
| engine | 新API追加 | `transpile2()` メソッド追加 |
| tests | 新テスト追加 | 既存テスト影響なし |
| lib.rs | 公開API追加 | `pub mod transpiler2;` 追加 |

---

## 4. 技術検証

### 4.1 Parser2 AST構造確認

```rust
// src/parser2/ast.rs より抽出
PastaFile {
    path: PathBuf,
    items: Vec<FileItem>,  // 統一配列（記述順保持）
    span: Span
}

enum FileItem {
    FileAttr(Attr),           // file-level属性
    GlobalWord(KeyWords),      // file-level単語
    GlobalSceneScope(GlobalSceneScope)
}

GlobalSceneScope {
    name: String,
    attrs: Vec<Attr>,
    words: Vec<KeyWords>,
    code_blocks: Vec<CodeBlock>,
    local_scenes: Vec<LocalSceneScope>,
    span: Span
}

LocalSceneScope {
    name: Option<String>,  // None = __start__
    attrs: Vec<Attr>,
    items: Vec<LocalSceneItem>,  // VarSet/CallScene/ActionLine/ContinueAction
    code_blocks: Vec<CodeBlock>,
    span: Span
}
```

### 4.2 既存Transpilerパターン確認

```rust
// src/transpiler/mod.rs:124-150 より抽出
pub fn transpile_pass1<W: std::io::Write>(
    file: &PastaFile,  // parser::PastaFile
    scene_registry: &mut SceneRegistry,
    word_registry: &mut WordDefRegistry,
    writer: &mut W,
) -> Result<(), PastaError> {
    // 1. グローバル単語登録
    for word_def in &file.global_words { ... }
    
    // 2. シーン登録・モジュール生成
    for scene in &file.scenes { ... }
}
```

### 4.3 Runeコード生成確認

```rune
// test_combined_code.rn より抽出
pub mod 会話_1 {
    use pasta_stdlib::*;
    use crate::actors::*;
    
    pub fn __start__(ctx, args) {
        yield Talk("こんにちは");
    }
    
    pub fn 選択肢_1(ctx, args) {
        yield Talk("選択肢です");
    }
}

pub mod __pasta_trans2__ {
    pub fn scene_selector(scene, filters) {
        let id = pasta_stdlib::select_scene_to_id(scene, filters);
        match id {
            1 => crate::会話_1::__start__,
            _ => |_ctx, _args| { yield Error(`シーンID ${id} が見つかりませんでした。`); },
        }
    }
}

pub mod pasta {
    pub fn call(ctx, scene, filters, args) {
        let func = crate::__pasta_trans2__::scene_selector(scene, filters);
        for a in func(ctx, args) { yield a; }
    }
}
```

---

## 5. 統合リスク評価

### 5.1 リスクマトリクス

| リスク | 発生確率 | 影響度 | 軽減策 |
|--------|---------|--------|--------|
| AST型不整合 | 中 | 高 | 設計段階で完全マッピング |
| Rune生成コードバグ | 中 | 高 | 単体テスト + Runtime実行テスト |
| シンボル解決エラー | 低 | 中 | Phase 1登録ロジック厳密化 |
| 属性マージ不整合 | 中 | 中 | File/Scene属性マージルールのテスト網羅 |
| CodeBlock配置エラー | 低 | 中 | 出力位置仕様化 + 構文検証 |

### 5.2 総合リスク評価

**リスクレベル**: Medium-High

**根拠**:
- 5つの新機能（Req 11-15）が既存transpilerに存在しない
- ただし、既存パターン踏襲とAST型非依存レジストリ再利用でリスク軽減可能

---

## 6. 主要な設計決定

### D1: Option B-改改（新規Transpiler2 + 共有Registry）

**決定**: 採用
**根拠**:
- 完全なモジュール独立性（parser/transpiler と parser2/transpiler2 の分離）
- レジストリ100%再利用（AST型非依存）
- 既存テストへの影響ゼロ
- 段階的レガシー削除が可能

### D2: TranspileError型

**決定**: `src/transpiler2/error.rs` に独立定義
**根拠**:
- transpiler2専用のエラーコンテキスト
- 既存PastaErrorとの明確な分離
- 将来の統合時にFrom変換を提供

### D3: FileItem統一配列アクセス

**決定**: 記述順序保持アクセス（`for item in &file.items`）
**根拠**:
- file-level属性の順次積算が必要（Req 11）
- GlobalWordとGlobalSceneScopeの相互位置関係が重要
- ヘルパーメソッドは型別フィルタリングのみに使用

### D4: ContinueAction処理

**決定**: 直前ActionLineへの連結（同一yield文）
**根拠**:
- pasta2.pest仕様：継続行は`：`プレフィックス必須
- 継続行のactionsは直前のActionLine出力に結合

---

## 7. 次のステップ

1. **design.md作成**: 本調査結果に基づく詳細設計書
2. **コンポーネント図**: transpiler2内部構造のMermaid図
3. **データモデル**: TranspileContext2、TranspileError型定義
4. **フロー図**: 2パス変換フローの詳細化
5. **テスト戦略**: 15カテゴリテストの構成定義
