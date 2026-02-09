# ギャップ分析レポート: pasta-language-server

**分析日**: 2026-02-09  
**対象仕様**: `.kiro/specs/pasta-language-server/requirements.md`

---

## 1. 現状調査

### 1.1 既存の再利用可能資産

| 資産            | パス                                         | LSP関連性                                                                     |
| --------------- | -------------------------------------------- | ----------------------------------------------------------------------------- |
| **PEGパーサー** | `crates/pasta_dsl/src/parser/grammar.pest`   | ✅ シンタックス解析のコア。そのまま利用可能                                    |
| **AST型定義**   | `crates/pasta_dsl/src/parser/ast.rs` (886行) | ✅ `FileItem`, `GlobalSceneScope`, `Action`等の完全な型定義                    |
| **Span型**      | `crates/pasta_dsl/src/parser/ast.rs`         | ✅ 行/列(1-based) + バイトオフセット(0-based)。LSPポジション変換に直接利用可能 |
| **parse_str()** | `crates/pasta_dsl/src/parser/mod.rs:106`     | ✅ 文字列からASTへの変換。ドキュメント同期後の再パースに使用                   |
| **ParseError**  | `crates/pasta_dsl/src/error.rs`              | ✅ ファイル名・行・列・メッセージ。LSP Diagnosticsに変換可能                   |

### 1.2 既存アーキテクチャパターン

- **ワークスペース構成**: `crates/*` 配下に独立クレート。`Cargo.toml` の `members = ["crates/*"]` で自動検出
- **依存方向**: `pasta_dsl` は最下層で外部依存なし（pest, thiserror のみ）
- **テスト規約**: `crates/*/tests/<feature>_test.rs`、fixture は `tests/fixtures/*.pasta`
- **CI**: GitHub Actions、`cargo test --all` で全クレートテスト、Windows x86/x64 マトリックス
- **ライセンス**: MIT OR Apache-2.0 デュアルライセンス

### 1.3 統合サーフェス

| 統合ポイント     | 既存API                                                                   | 備考                                              |
| ---------------- | ------------------------------------------------------------------------- | ------------------------------------------------- |
| パーサー呼び出し | `pasta_dsl::parse_str(source, filename)`                                  | `&str` 入力、`Result<PastaFile, ParseError>` 返却 |
| AST走査          | `PastaFile.items: Vec<FileItem>`                                          | 記述順序保持、match式で走査                       |
| ソース位置       | `Span { start_line, start_col, end_line, end_col, start_byte, end_byte }` | 全ノードがSpan持ち                                |
| エラー情報       | `ParseError::SyntaxError { file, line, column, message }`                 | LSP Diagnosticに1:1マッピング可能                 |

---

## 2. 要件実現可能性分析

### 要件→資産マッピング

| 要件                           | 既存資産                       | ギャップ                                       | 状態                        |
| ------------------------------ | ------------------------------ | ---------------------------------------------- | --------------------------- |
| **R1: LSPサーバー基盤**        | なし                           | LSPフレームワーク選定・実装が必要              | **Missing**                 |
| **R2: セマンティックトークン** | `pasta_dsl` AST + Span         | AST→SemanticTokenの変換ロジック実装が必要      | **Missing**（部分流用可能） |
| **R3: pasta_dsl統合**          | `parse_str()`, `ParseError`    | API完備。変換レイヤーのみ必要                  | **低ギャップ**              |
| **R4: WASMビルド**             | なし                           | WASM互換フレームワーク選定、条件コンパイル設計 | **Missing**（要リサーチ）   |
| **R5: クレート設計**           | ワークスペースパターン確立済み | 新クレート作成のみ。既存パターンに準拠         | **低ギャップ**              |
| **R6: ドキュメント管理**       | なし                           | テキスト同期・増分更新の実装が必要             | **Missing**                 |
| **R7: テスト**                 | テスト規約確立済み             | テストケース実装が必要                         | **Missing**                 |

### 技術的制約と未知数

1. **LSPポジションエンコーディング**: LSPは`UTF-16`オフセットがデフォルト。`pasta_dsl`のSpanはバイトオフセット（UTF-8）。変換レイヤーが必要
2. **pest の `parse_file()` はファイルI/O使用**: WASMでは`parse_str()`のみ利用可能（`parse_file()`は`std::fs`依存）
3. **増分パース**: pestパーサーは増分パースに非対応。変更のたびに全ドキュメント再パースが必要（小〜中規模ファイルでは許容範囲）

### 複雑性シグナル

- **AST→SemanticToken変換**: AST走査による分類は明確。マーカー種別ごとのトークンタイプ割当は決定論的
- **WASM対応**: フレームワーク選定とトランスポート層抽象化が最大の技術課題
- **ドキュメント同期**: LSP標準パターン。lsp-typesの型を使えば定型的な実装

---

## 3. 実装アプローチ検討

### Option A: tower-lsp ベース（単一クレート）

**概要**: `tower-lsp` (v0.20) + `runtime-agnostic` featureを使用し、単一の `pasta_lang_server` クレートで実装。

**構成**:
```
crates/pasta_lang_server/
├── Cargo.toml         # tower-lsp, lsp-types, pasta_dsl 依存
└── src/
    ├── lib.rs         # LSP Backend実装
    ├── semantic_tokens.rs  # AST→SemanticToken変換
    ├── document.rs    # ドキュメント管理
    └── main.rs        # ネイティブ stdio エントリポイント
```

**WASM対応**: `tower-lsp` は v0.16.0 でWASM公式対応。`runtime-agnostic` featureでtokio依存を除去可能。`wasm-bindgen` でJavaScript側にエクスポート。

**トレードオフ**:
- ✅ 実績のあるフレームワーク（3.6M+ downloads）
- ✅ `LanguageServer` traitによる構造化された実装
- ✅ WASM公式対応の実績あり（tower-lsp-web-demo）
- ✅ 単一クレートでシンプル
- ❌ async/await必須（WASMではシングルスレッド制約あり）
- ❌ tower-lspの抽象化がやや重い（セマンティックトークンのみなら過剰）

### Option B: lsp-server ベース（ネイティブ専用）+ 将来WASM分離

**概要**: rust-analyzer由来の `lsp-server` (v0.7) でネイティブLSPサーバーを実装。WASMは将来別クレートで対応。

**構成**:
```
crates/pasta_lang_server/
├── Cargo.toml         # lsp-server, lsp-types, pasta_dsl 依存
└── src/
    ├── lib.rs         # 言語解析コア（WASM互換層）
    ├── semantic_tokens.rs
    ├── document.rs
    └── main.rs        # stdio サーバー
```

**トレードオフ**:
- ✅ rust-analyzer実績のある軽量フレームワーク
- ✅ 同期的（async不要）でシンプル
- ❌ **WASM非互換**（`crossbeam-channel`, `std::thread` 依存）
- ❌ 要件R4（WASMビルド）を即座に満たせない
- ❌ 将来的にWASM版を別クレートで作る必要あり

### Option C: ハイブリッド（解析コア分離 + プラットフォーム別サーバー）

**概要**: 解析コアをWASM互換ライブラリとして分離し、プラットフォーム別のサーバー実装を提供。

**構成**:
```
crates/pasta_lang_server/
├── Cargo.toml         # pasta_dsl, lsp-types 依存（WASM互換）
└── src/
    ├── lib.rs         # 解析コア（セマンティックトークン、Diagnostics）
    ├── semantic_tokens.rs  # AST→Token変換
    ├── document.rs    # ドキュメント管理
    ├── capabilities.rs    # LSPケーパビリティ定義
    └── transport/
        ├── mod.rs     # トランスポート抽象
        ├── native.rs  # #[cfg(not(target_arch = "wasm32"))] tower-lsp stdio
        └── wasm.rs    # #[cfg(target_arch = "wasm32")] wasm-bindgen
```

**トレードオフ**:
- ✅ 解析コアの完全WASM互換性
- ✅ 将来の`pasta_vscode`統合に最も適した設計
- ✅ テスタビリティ最高（コアは純関数的にテスト可能）
- ❌ 初期実装コストがやや高い
- ❌ 条件コンパイルの管理が複雑

---

## 4. 工数・リスク評価

| 観点       | 評価            | 根拠                                                                                              |
| ---------- | --------------- | ------------------------------------------------------------------------------------------------- |
| **工数**   | **M（3〜7日）** | LSPフレームワーク統合 + AST→Token変換 + ドキュメント管理。pasta_dsl資産の流用でパーサー実装不要   |
| **リスク** | **中**          | WASM+LSPの組み合わせは実績あるが、tower-lspのWASMモードは比較的新しい。pestのWASM互換性は確認済み |

### リスク詳細

| リスク                               | 影響度 | 軽減策                                                           |
| ------------------------------------ | ------ | ---------------------------------------------------------------- |
| tower-lsp WASM統合の技術的障壁       | 中     | tower-lsp-web-demo の実績を参考に実装。fallbackとしてOption B    |
| LSP UTF-16ポジション変換の精度       | 低     | Span型に既にバイトオフセット情報あり。UTF-8→UTF-16変換は定型処理 |
| 増分パース非対応によるパフォーマンス | 低     | pastaファイルは通常小〜中規模。全体再パースでも十分高速          |
| CI WASMビルド追加の複雑性            | 低     | `wasm32-unknown-unknown` ターゲットをmatrixに追加するだけ        |

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ: **Option A（tower-lsp単一クレート）**

**理由**:
1. **要件R4（WASMビルド）を直接満たせる**唯一のオプション
2. tower-lspの`LanguageServer` traitがLSPプロトコル実装を大幅に簡素化
3. 単一クレートで管理コストを最小化
4. 必要に応じてOption Cへの段階的移行が可能（lib.rsの解析コアを分離するだけ）

### 設計フェーズで決定すべき事項

1. **トランスポート層設計**: ネイティブ（stdio）とWASM（wasm-bindgen message passing）の切り替え方法
2. **セマンティックトークンマッピング表**: 各AST ノード → LSP SemanticTokenType/Modifierの対応表
3. **UTF-8 → UTF-16 ポジション変換**: `Span`からLSP `Position`への変換ユーティリティ設計
4. **エラー回復戦略**: パースエラー時にも部分的なトークンを返すか、エラー箇所のみDiagnostics化するか

### 設計フェーズへの持ち越しリサーチ項目

| 項目                                               | 理由                                                                    |
| -------------------------------------------------- | ----------------------------------------------------------------------- |
| tower-lsp `runtime-agnostic` の具体的設定          | WASMビルド時のasyncランタイム選択（wasm-bindgen-futures等）             |
| pasta_dsl の `pest` WASM ビルド実機検証            | `default-features = false` の設定確認。`grammar-extras` featureの互換性 |
| VSCode拡張 (pasta_vscode) との統合インターフェース | WASMバイナリのロード方法、LSPトランスポートの接続方式                   |
