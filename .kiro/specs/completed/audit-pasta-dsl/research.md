# リサーチ＆設計判断

## サマリ
- **機能**: `audit-pasta-dsl`
- **ディスカバリースコープ**: 拡張（既存システムの監査・簡素化）
- **主要知見**:
  - `unwrap()`は1箇所のみ（parse_scene.rs:388）、ドキュメントコメント内の2箇所は非コード
  - `#[allow(dead_code)]`や`#[allow(unused)]`は0件、Rustコンパイラ警告レベルでの未使用コード検出が必要
  - 全13ソースファイル合計約2,675行（grammar.pest 214行含む）、最大ファイルはparse_scene.rs（424行）

## リサーチログ

### unwrap/expect/panicの使用状況
- **背景**: 外部入力パスにおけるパニックリスクの評価
- **調査結果**:
  - `parse_scene.rs:388` — `raw[colon_pos..].chars().next().unwrap()` — コロン位置が事前検証済みのためパニックリスクは低いが、防御的コーディングに置換すべき
  - `lib.rs:19`, `mod.rs:42` — ドキュメントコメント内のサンプルコードのみ（実行パスではない）
- **影響**: unwrap排除は1箇所のみで低リスク

### ファイルサイズと複雑度分布
- **背景**: パーサーモジュールの複雑度ホットスポット特定
- **調査結果**:
  | ファイル | 行数 | 備考 |
  |---------|------|------|
  | parse_scene.rs | 424 | シーン解析、最大ファイル |
  | parse_action.rs | 394 | アクション行解析 |
  | mod.rs | 303 | パーサーエントリーポイント |
  | action.rs (ast) | 267 | アクションAST型定義 |
  | grammar.pest | 214 | Pest文法（変更対象外） |
  | scene.rs (ast) | 215 | シーンAST型定義 |
  | partial.rs | 200 | パーシャルパース |
  | parse_elements.rs | 185 | 要素解析 |
  | mod.rs (ast) | 157 | AST再エクスポート |
  | span.rs | 126 | Span型定義 |
  | cue.rs | 92 | キューコマンドAST |
  | error.rs | 66 | エラー型 |
  | lib.rs | 32 | クレートエントリーポイント |
- **影響**: parse_scene.rs と parse_action.rs が複雑度削減の主要候補

### 公開API・型の安定性
- **背景**: 監査での変更不可範囲の確認
- **調査結果**:
  - 公開関数: `parse_str`, `parse_file`, `parse_str_partial`, `parse_with_rule`, `infer_rule_from_line`
  - 公開型: `PastaFile`, `FileItem`, `SceneScope`, `SceneActorItem`, `ActionLine`, `Span`, `ParseError`, `ParseErrorInfo`, `PartialParseResult`, `PartialParseError` 等
  - pasta_lua, pasta_lsp が依存（公開インターフェース不変が必須）
- **影響**: 内部リファクタリングのみ許可、公開シグネチャ変更禁止

### デッドコード検出戦略
- **背景**: Rustのデッドコード検出メカニズムの確認
- **調査結果**:
  - `cargo clippy` で `unused_imports`, `dead_code`, `unreachable_patterns` 等を検出可能
  - `pub` 可視性が過剰な場合、クレート外部から参照されないアイテムは `pub(crate)` に縮小可能
  - Pest生成コードの `Rule` 列挙体は自動生成のため、未使用バリアント警告は抑制対象
- **影響**: clippy + 手動レビューの2段階アプローチ

## 設計判断

### 判断: 監査アプローチ — ファイル単位の独立監査
- **背景**: 13ファイル・約2,500行をどのような粒度で監査するか
- **代替案**:
  1. 全ファイル一括監査 — 一度に全体を見通せるが、変更の追跡が困難
  2. ファイル単位の独立監査 — 各ファイルを独立に監査、変更を局所化
  3. 機能ドメイン単位 — パーサー/AST/エラーで分割
- **選択**: ファイル単位の独立監査（機能ドメインでグルーピング）
- **理由**: 変更の影響範囲を局所化しやすく、レビュー・テストが容易
- **トレードオフ**: ファイル横断の共通パターン抽出は最後に実施する必要がある

### 判断: grammar.pestは変更対象外
- **背景**: brief.mdのスコープ定義
- **理由**: Pest文法定義の変更は構文変更を意味し、監査スコープ外。grammar.pestは「正規仕様」として不変
- **トレードオフ**: 文法定義に起因する複雑度は許容する
