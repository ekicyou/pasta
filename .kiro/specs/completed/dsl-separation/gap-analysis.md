# ギャップ分析: dsl-separation

## 1. 現状調査

### 1.1 対象アセットとディレクトリレイアウト

#### pasta_core の構成（分離元）

```
crates/pasta_core/
├── Cargo.toml            # pest, pest_derive, thiserror, fast_radix_trie, rand, tracing
├── src/
│   ├── lib.rs            # pub mod error/parser/registry + 再エクスポート
│   ├── error.rs          # ParseError, SceneTableError, WordTableError（混在）
│   ├── parser/
│   │   ├── mod.rs        # PastaParser2, parse_str(), parse_file(), build_ast() (1406行)
│   │   ├── ast.rs        # AST型定義 (886行)
│   │   └── grammar.pest  # Pest PEG文法定義
│   └── registry/
│       ├── mod.rs        # pub use で全レジストリ型を公開
│       ├── scene_registry.rs
│       ├── scene_table.rs
│       ├── word_registry.rs
│       ├── word_table.rs
│       └── random.rs
└── tests/
    ├── actor_code_block_test.rs    (3テスト)
    ├── digit_id_var_test.rs        (4テスト)
    ├── sakura_symbol_tag_test.rs   (7テスト)
    └── span_byte_offset_test.rs    (12テスト)
```

#### 現在の依存グラフ

```
pasta_shiori → pasta_lua → pasta_core
                              ├── parser (DSL)    ← 分離対象
                              ├── registry        ← core に残留
                              └── error           ← 分割対象
```

### 1.2 モジュール間の結合度分析

| 関係                              | 依存あり | 詳細                                                                                                      |
| --------------------------------- | -------- | --------------------------------------------------------------------------------------------------------- |
| parser → registry                 | ❌ なし   | **完全独立**                                                                                              |
| registry → parser                 | ❌ なし   | **完全独立**                                                                                              |
| parser → error (ParseError)       | ✅ あり   | `use crate::error::ParseError` (mod.rs 内)                                                                |
| registry → error                  | ❌ なし   | registry は独自エラーなし（error.rs にレジストリエラーが定義されているが、registry 内部では使っていない） |
| pasta_lua → pasta_core::parser    | ✅ 大量   | 4ファイル、10箇所の `use` 文 + 多数のフルパス参照                                                         |
| pasta_lua → pasta_core::error     | ✅ あり   | フルパス `pasta_core::ParseError` 参照（1ファイル）                                                       |
| pasta_shiori → pasta_core::parser | ❌ なし   | pasta_shiori は独自の ParseError を定義                                                                   |
| pasta_shiori → pasta_core::error  | ❌ なし   |                                                                                                           |

**発見**: parser と registry は**完全に疎結合**。相互参照が一切なく、分離は安全に実行可能。

### 1.3 error.rs の構造分析

`error.rs`（130行）は以下の3カテゴリを混在して定義：

| カテゴリ             | 型                                            | 分離先              |
| -------------------- | --------------------------------------------- | ------------------- |
| DSLパースエラー      | `ParseError`, `ParseErrorInfo`, `ParseResult` | → **pasta_dsl**     |
| シーンテーブルエラー | `SceneTableError`, `SceneTableResult`         | → pasta_core に残留 |
| 単語テーブルエラー   | `WordTableError`, `WordTableResult`           | → pasta_core に残留 |

**依存方向**:
- `ParseError` は `std::collections::HashMap` と `thiserror` のみに依存（外部依存なし）
- `SceneTableError` は `HashMap` と `thiserror` のみに依存
- `WordTableError` は `thiserror` のみに依存
- 3つのエラー型間に**相互依存はない**

### 1.4 lib.rs の再エクスポート構造

```rust
pub mod error;
pub mod parser;
pub mod registry;

pub use error::{ParseError, ParseErrorInfo, ParseResult, SceneTableError, SceneTableResult, WordTableError, WordTableResult};
pub use parser::{FileItem, PastaFile, parse_file, parse_str};
pub use registry::{DefaultRandomSelector, MockRandomSelector, RandomSelector, SceneEntry, SceneId, SceneInfo, SceneRegistry, SceneScope, SceneTable, WordCacheKey, WordDefRegistry, WordEntry, WordTable};
```

### 1.5 移動対象テストの詳細

| テストファイル              | テスト数 | 依存                                                                                            | 分離安全性    |
| --------------------------- | -------- | ----------------------------------------------------------------------------------------------- | ------------- |
| `actor_code_block_test.rs`  | 3        | `pasta_core::parser::{FileItem, parse_str}`                                                     | ✅ parser のみ |
| `digit_id_var_test.rs`      | 4        | `pasta_core::parser::{Action, FileItem, GlobalSceneScope, LocalSceneItem, VarScope, parse_str}` | ✅ parser のみ |
| `sakura_symbol_tag_test.rs` | 7        | `pasta_core::parser::{FileItem, Action, parse_str}`                                             | ✅ parser のみ |
| `span_byte_offset_test.rs`  | 12       | `pasta_core::parser::{FileItem, GlobalSceneScope, Span, SpanError, parse_str}`                  | ✅ parser のみ |
| **合計**                    | **26**   |                                                                                                 |               |

全テストファイルが **parser モジュールのみ**に依存。レジストリ・エラー型への直接依存なし。

### 1.6 pasta_core のテスト分布

- **総テスト数**: 130
- **統合テスト（tests/ 配下、移動対象）**: 26テスト
- **ユニットテスト（src/ 内 #[cfg(test)]）**: 104テスト
  - parser 内テスト: doctest + mod 内テスト
  - registry 内テスト
- **移動後の pasta_core テスト数**: 104テスト（ユニットテストは移動不要）

---

## 2. 要件実現可能性分析

### 要件→アセットマッピング

| 要件                      | 既存アセット                                      | ギャップ                                       | ステータス |
| ------------------------- | ------------------------------------------------- | ---------------------------------------------- | ---------- |
| Req 1: DSLクレート抽出    | parser/, ast.rs, grammar.pest が完全に独立        | 新クレートの Cargo.toml とモジュール構造の作成 | **容易**   |
| Req 2: 再エクスポート互換 | lib.rs に既存の pub use パターン                  | `pub use pasta_dsl::parser` への差し替え       | **容易**   |
| Req 3: ワークスペース統合 | Cargo.toml に既存の members/dependencies パターン | pasta_dsl エントリの追加                       | **容易**   |
| Req 4: 独立利用性         | parser が registry 非依存                         | テスト移動と import パス変更                   | **容易**   |
| Req 5: エラー型分離       | error.rs に3カテゴリ混在だが相互依存なし          | error.rs の分割                                | **容易**   |
| Req 6: ドキュメント更新   | 8ファイルに構成図あり                             | テキスト修正                                   | **容易**   |

### 制約と複雑性

- **複雑性シグナル**: 全要件が「既存構造のリファクタリング」であり、新規アルゴリズムやビジネスロジックの追加はない
- **主なリスク**: pasta_coreの再エクスポートが正しく機能し、下流クレートが変更なしでコンパイルできるか
- **未知の領域**: なし（全構造が十分に理解されている）

---

## 3. 実装アプローチオプション

### Option A: 移動ベース（ファイル移動 + 再エクスポート）

**概要**: parser/ ディレクトリと error.rs の ParseError 部分を pasta_dsl に物理移動し、pasta_core は再エクスポートで互換維持

**手順**:
1. `crates/pasta_dsl/` を新規作成（Cargo.toml, src/lib.rs）
2. `parser/mod.rs`, `parser/ast.rs`, `parser/grammar.pest` を pasta_dsl/src/ に移動
3. error.rs から ParseError/ParseErrorInfo/ParseResult を pasta_dsl/src/error.rs に移動
4. pasta_core の parser モジュールを `pub use pasta_dsl::parser` で置き換え
5. pasta_core の error.rs から ParseError を削除し、`pub use pasta_dsl::error::{ParseError, ...}` で再エクスポート
6. テスト4ファイルを pasta_dsl/tests/ に移動、import パス変更
7. ドキュメント8ファイルを更新

**トレードオフ**:
- ✅ 最もクリーンな分離（pasta_dsl がソースの正本）
- ✅ pasta_dsl の依存は最小限（pest, pest_derive, thiserror のみ）
- ✅ parser の `use crate::error::ParseError` が `use crate::error::ParseError` のまま（同一クレート内で完結）
- ❌ pasta_core の `pub mod parser` が消えるため、再エクスポートの設計が重要

### Option B: コピーベース（コピー + 段階的移行）

**概要**: parser をコピーして pasta_dsl を作成し、pasta_core は一時的に両方を保持した後、段階的に移行

**トレードオフ**:
- ✅ 段階的移行で安全性が高い
- ❌ 一時的にコード重複が発生
- ❌ 同期の手間が増える
- ❌ 追加の移行ステップが必要

### Option C: 薄い wrapper（pasta_core の parser を pub re-export のみに）

**概要**: Option A と同じだが、pasta_core に `pub mod parser` を薄い wrapper として残す

```rust
// pasta_core/src/parser.rs (薄いwrapper)
pub use pasta_dsl::parser::*;
```

**トレードオフ**:
- ✅ 下流クレートの `use pasta_core::parser::*` がそのまま動作
- ✅ Option A とほぼ同じクリーンさ
- ✅ 最もシンプルな移行パス
- ❌ なし（実質的にデメリットがない）

### 推奨: **Option A（移動ベース・完全分離）** ← 要件レビューで確定

> **要件レビューでの決定**: パターンB（移行措置）が採用され、Option Cの薄いwrapperは不採用。
> 下流クレートは `pasta_dsl` に直接依存し、`pasta_core::parser` パスは完全に消滅する。

**理由**:
1. parser と registry が完全に独立しているため、分離自体にリスクがない
2. 下流クレートが `pasta_dsl` に直接依存することで、依存関係が明確になる
3. pasta_core から parser を完全除去し、再エクスポートを行わないことでクリーンな責務分離を実現
4. pasta_core は将来的に registry 等のユーティリティに特化

---

## 4. 実装複雑度とリスク

### 工数見積もり: **S（1〜3日）**

**根拠**: 
- 既存パターン（Cargoワークスペース）の適用のみ
- 新規ロジック・外部連携なし
- parser/registry が完全に独立しており、分離時の意図しない副作用リスクが極めて低い
- 下流クレートの import パス変更は機械的な作業

### リスク: **Low**

**根拠**:
- 全構造が十分に理解されている（未知の技術なし）
- parser↔registry 間に相互依存がない（調査で確認済み）
- `cargo test --all` で即座に回帰検証可能
- 下流クレートの変更は import パスの書き換えのみ

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ

**Option A（移動ベース・完全分離）** を採用し、以下の順序で実装：

1. pasta_dsl クレート新規作成（Cargo.toml, src/lib.rs, src/error.rs）
2. parser ソースファイル移動（mod.rs, ast.rs, grammar.pest）
3. ParseError 型の移動（error.rs の分割）
4. pasta_core から parser モジュールを完全除去（再エクスポートなし）
5. 下流クレート（pasta_lua 等）の import パスを `pasta_dsl` に変更
6. テスト4ファイル（26テスト）の移動と import パス変更
7. `cargo test --all` で全テスト通過を確認
8. ドキュメント8ファイルの構成図更新

### 設計フェーズで決定すべき事項

1. **pasta_dsl の `pub mod` 構成**: `parser` と `error` を別モジュールにするか、フラットにするか
2. **pasta_dsl の Cargo.toml 設定**: `tracing` 依存を含めるか（現在 parser 内で `tracing` を使用しているか要確認）

### Research Needed 項目

- **なし**: 全構造が明確であり、追加調査は不要
