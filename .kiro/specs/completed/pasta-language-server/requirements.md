# Requirements Document

## はじめに

本ドキュメントは、Pasta DSL向けランゲージサーバー（`pasta_lsp` クレート）の要件を定義する。

**目的**: VSCode等のエディタで`*.pasta`ファイルのシンタックスハイライトを提供するLanguage Server Protocol（LSP）サーバーを、WebAssembly（WASM）ビルド可能なRustクレートとして実装する。

**スコープ**:
- シンタックスハイライト（セマンティックトークン）の提供
- LSPプロトコル準拠のサーバー実装
- `wasm32-unknown-unknown` ターゲットへのビルド対応
- 将来の`pasta_vscode`拡張クレートへの統合を見据えた設計

**スコープ外**:
- デバッグ対応（将来フェーズで検討）
- 自動補完・コード補完機能（将来フェーズで検討）
- リファクタリング支援（将来フェーズで検討）

## 要件

### Requirement 1: LSPサーバー基盤

**目的:** 開発者として、VSCodeなどのLSP対応エディタからPasta DSLファイルを編集する際に、ランゲージサーバーからの言語支援を受けたい。エディタ連携により開発体験を向上させるため。

#### 受入基準

1. The pasta_lsp shall LSPプロトコル（Language Server Protocol 3.17+）に準拠したサーバーとして動作する
2. When エディタがLSPサーバーに接続した時, the pasta_lsp shall `initialize`リクエストに対してサーバーケーパビリティを返却する
3. When エディタが`*.pasta`ファイルを開いた時, the pasta_lsp shall `textDocument/didOpen`通知を受信しドキュメントを管理対象に追加する
4. When エディタが`*.pasta`ファイルの内容を変更した時, the pasta_lsp shall `textDocument/didChange`通知を受信しドキュメントの内容を同期する
5. When エディタが`*.pasta`ファイルを閉じた時, the pasta_lsp shall `textDocument/didClose`通知を受信しドキュメントを管理対象から除外する

### Requirement 2: セマンティックトークンによるシンタックスハイライト

**目的:** 開発者として、`*.pasta`ファイルをエディタで開いた際にDSL構文が色分けされて表示されたい。コードの可読性を向上させ、構文エラーの早期発見を容易にするため。

#### 受入基準

1. The pasta_lsp shall `textDocument/semanticTokens/full`リクエストに対して、ドキュメント全体のセマンティックトークンを返却する
2. The pasta_lsp shall 以下のトークンタイプを識別しエディタに通知する:
   - **コメント**: `＃` / `#` で始まるコメント行
   - **シーンマーカー**: `＊` / `*`（グローバルシーン）, `・` / `-`（ローカルシーン）
   - **属性マーカー**: `＆` / `&` で始まる属性定義行
   - **単語マーカー**: `＠` / `@` で始まる単語定義・参照
   - **変数マーカー**: `＄` / `$` で始まる変数宣言・参照
   - **Call マーカー**: `＞` / `>` で始まるシーン呼び出し
   - **アクター辞書マーカー**: `％` / `%` で始まるアクター辞書定義
   - **アクター名**: アクション行のアクター名（`：` / `:` の前）
   - **Luaコードブロック**: ` ```lua ` ～ ` ``` ` で囲まれた範囲
   - **文字列リテラル**: アクション行のテキスト部分（`Action::Talk`）
   - **さくらスクリプト**: `\s[]`, `\n`, `\_w[]` 等のさくらスクリプトタグ（`Action::SakuraScript`）
   - **エスケープシーケンス**: `@@`, `$$`, `\\\\` 等のエスケープ（`Action::Escape`）
   - **変数代入**: `＄変数名：値` / `＄＊変数名：値` の変数設定行（`VarSet`）
   - **コロン区切り**: `：` / `:` セパレータ
3. When ドキュメントの内容が変更された時, the pasta_lsp shall セマンティックトークンを再計算し最新状態を維持する
4. The pasta_lsp shall アクション行内の各要素（`Action::Talk`, `Action::WordRef`, `Action::VarRef`, `Action::SakuraScript`, `Action::Escape`）を個別のトークンとして識別し、インライン要素レベルの細粒度色分けを提供する
5. The pasta_lsp shall 全角マーカー（`＊`、`・`、`＠`、`＄`、`＞`、`＃`、`＆`、`％`、`：`）と半角マーカー（`*`、`-`、`@`、`$`、`>`、`#`、`&`、`%`、`:`）を同等に認識してトークン化する
6. When ドキュメント全体のパースが失敗した時, the pasta_lsp shall 部分的にパース成功した行・スコープのセマンティックトークンを提供し、エラー行はDiagnosticsとして報告する。これにより編集中もマーカー構造が視認可能な状態を保つ

### Requirement 3: pasta_dslパーサー統合

**目的:** 開発者として、既存のpasta_dslクレートのPEGパーサーをランゲージサーバーで活用したい。パース精度の一貫性を保ち、実装の重複を避けるため。

#### 受入基準

1. The pasta_lsp shall `pasta_dsl`クレートの`parse_str()`関数を利用してDSLソースをASTに変換する
2. When `pasta_dsl`のパーサーがパースエラーを返却した時, the pasta_lsp shall エラー情報をLSP Diagnostics（`textDocument/publishDiagnostics`）としてエディタに通知する
3. The pasta_lsp shall ASTの各ノード（`GlobalSceneScope`, `FileItem::FileAttr`, `FileItem::GlobalWord`, `FileItem::ActorScope`等）からセマンティックトークンのタイプと範囲を算出する
4. If `pasta_dsl`のパーサーがクラッシュまたは予期しないエラーを発生させた時, the pasta_lsp shall サーバー全体を停止せず、該当ドキュメントについてエラーメッセージをログに記録する
5. The pasta_lsp shall `pasta_dsl`に部分パースAPI（`parse_str_partial()`）を追加し、パースエラー時も成功した部分のASTとエラー情報リストを取得する。以下の詳細要件に従って`pasta_dsl`クレートを拡張する:

#### R3.5.1: PartialParseResult型定義

The pasta_dsl shall 部分パース結果を表現する型を公開する:

```rust
/// 部分パース結果
pub struct PartialParseResult {
    /// パース成功した部分のASTアイテム
    pub items: Vec<FileItem>,
    /// 各行/スコープのパースエラー
    pub errors: Vec<PartialParseError>,
}

/// 部分パースエラー
pub struct PartialParseError {
    /// エラーが発生した行番号（1-based）
    pub line: usize,
    /// エラーメッセージ
    pub message: String,
    /// エラー範囲のSpan（取得できた場合）
    pub span: Option<Span>,
}
```

#### R3.5.2: parse_str_partial() API

The pasta_dsl shall `parse_str_partial(source: &str) -> PartialParseResult` 関数を公開し、以下の3段階フォールバック戦略でパースを試行する:

- **Phase 1 (Full Parse)**: `parse_str()`による全体パースを試行。成功時は完全なASTを返却し、失敗時はPhase 2へ
- **Phase 2 (Scope Boundary Split)**: 行頭マーカー（`＊`/`*`, `％`/`%`, `＆`/`&`, `＠`/`@`）でソースをチャンクに分割し、各チャンクを個別にpestのRule（`global_scene`, `actor`, `file_attr`, `file_word`等）でパース。成功したチャンクのASTを収集し、失敗したチャンクはPhase 3へ
- **Phase 3 (Line-by-Line Fallback)**: 失敗した各行について、行頭パターンから適用すべきpest Ruleを推論し、行単位でパースを試行。成功した行のASTを収集し、失敗した行は`PartialParseError`として記録

#### R3.5.3: Pest Rule個別適用メカニズム

The pasta_dsl shall pestの各Ruleを個別に適用可能な内部API（`parse_with_rule(source: &str, rule: Rule) -> Result<Pairs, ParseError>`）を実装する。Phase 2/3で使用し、`Rule::global_scene`, `Rule::local_scene_line`, `Rule::action_line`等を行/スコープ単位で適用する。

#### R3.5.4: 行指向文法特性の活用

The pasta_dsl shall Pasta DSLの行指向文法特性（各行が独立してパース可能）を活用し、以下のマッピングで行頭パターンからpest Ruleを推論する:

| 行頭パターン | 推論されるRule           | 例                  |
| ------------ | ------------------------ | ------------------- |
| `＊` / `*`   | `Rule::global_scene`     | `＊シーン名`        |
| `・` / `-`   | `Rule::local_scene_line` | `・サブシーン`      |
| `＆` / `&`   | `Rule::file_attr`        | `＆key：value`      |
| `＠` / `@`   | `Rule::file_word`        | `＠word：def1 def2` |
| `％` / `%`   | `Rule::actor`            | `％アクター名`      |
| `＄` / `$`   | `Rule::var_set`          | `＄var：value`      |
| `＞` / `>`   | `Rule::call`             | `＞next_scene`      |
| `＃` / `#`   | `Rule::or_comment_eol`   | `＃コメント`        |
| 識別子 `:`   | `Rule::action_line`      | `Alice：こんにちは` |

#### R3.5.5: テスト要件

The pasta_dsl shall 部分パース機能に対する以下のテストを備える:

1. Phase 1成功時の完全AST返却テスト
2. Phase 2スコープ境界分割の正確性テスト（複数グローバルシーン、アクタースコープ混在）
3. Phase 3行単位フォールバックの正確性テスト（構文エラー行と正常行の混在）
4. 全角/半角マーカー両対応のテスト
5. エラー行の`PartialParseError`生成テスト（line番号、message、span精度）

### Requirement 4: WebAssembly（WASM）ビルド対応

**目的:** 開発者として、ランゲージサーバーを`wasm32-unknown-unknown`ターゲットにビルドしたい。VSCode拡張機能に同梱し、追加インストール不要でシンタックスハイライトを提供するため。将来的にvscode.dev（ブラウザ版VSCode）でも動作させる選択肢を残す。

#### 受入基準

1. The pasta_lsp shall `cargo build --target wasm32-unknown-unknown --release` でコンパイルエラーなくビルドできる。ビルド成功は以下で検証する:
   - ビルドコマンドの終了コード0を確認
   - `target/wasm32-unknown-unknown/release/pasta_lsp.wasm`の生成を確認（`crate-type = ["cdylib", "rlib"]` 設定により`.wasm`バイナリを生成）
   - wasmバイナリサイズが10MB以下であることを確認（wasm-opt最適化前）
2. The pasta_lsp shall ファイルシステムI/O・ネットワークI/O・スレッド生成などのWASM非互換APIを直接使用しない
3. The pasta_lsp shall 依存クレート（`pasta_dsl`含む）がすべて`wasm32-unknown-unknown`ターゲットでビルド可能であることを保証する
4. While WASMビルドモードの時, the pasta_lsp shall LSPトランスポート層をプラットフォーム抽象化し、ネイティブ（stdio）とWASM（メッセージパッシング）の両方に対応する
5. The pasta_lsp shall `#[cfg(target_arch = "wasm32")]` による条件コンパイルでWASM固有コードとネイティブ固有コードを分離する

### Requirement 5: クレート設計とワークスペース統合

**目的:** 開発者として、ランゲージサーバーが既存のpastaワークスペース構成に適合する形で実装されたい。プロジェクトの一貫性を維持し、保守性を高めるため。

#### 受入基準

1. The pasta_lsp shall 独立したクレートとして`crates/pasta_lsp/`に配置される
2. The pasta_lsp shall `pasta_dsl`クレートにのみ依存し、`pasta_lua`・`pasta_core`・`pasta_shiori`には依存しない
3. The pasta_lsp shall ワークスペースの`Cargo.toml`に`members`として登録される
4. The pasta_lsp shall 既存のCI/CDパイプライン（GitHub Actions）で`cargo test --workspace`に含まれてテストされる
5. The pasta_lsp shall `MIT OR Apache-2.0`デュアルライセンスに従う

### Requirement 6: ドキュメント管理

**目的:** 開発者として、エディタで編集中の`*.pasta`ファイルの内容をランゲージサーバーが正確に追跡したい。リアルタイムなハイライト更新を実現するため。

#### 受入基準

1. The pasta_lsp shall 開かれた各ドキュメントのテキスト内容をメモリ上に保持する
2. When エディタから増分テキスト変更通知（`TextDocumentContentChangeEvent`）を受信した時, the pasta_lsp shall ドキュメント内容を正確に更新する
3. When ドキュメント内容が更新された時, the pasta_lsp shall 再パースを実行しセマンティックトークンとDiagnosticsを更新する
4. The pasta_lsp shall UNICODEテキスト（日本語識別子・全角マーカー・BMP外文字含む）のバイトオフセット計算を正確に行う。以下の文字種で正確なUTF-8→UTF-16位置変換を保証する:
   - ASCII（1バイト → 1コードユニット）
   - 日本語BMP内文字（3バイト → 1コードユニット）
   - BMP外文字（絵文字、CJK拡張B等、4バイト → 2コードユニット/サロゲートペア）
   - 結合文字シーケンス（可変バイト → code point単位でカウント）

### Requirement 7: テスト要件

**目的:** 開発者として、ランゲージサーバーの品質を自動テストで保証したい。リグレッション防止と継続的な品質維持のため。

#### 受入基準

1. The pasta_lsp shall 各トークンタイプの識別に対するユニットテストを備える
2. The pasta_lsp shall LSPリクエスト/レスポンスの統合テストを備える
3. The pasta_lsp shall 全角・半角マーカー両方のパターンに対するテストケースを含む
4. The pasta_lsp shall 日本語識別子（シーン名・変数名・単語名）を含むテストケースを備える
5. The pasta_lsp shall WASMターゲットビルドの成功を検証するCIテストを備える。CIパイプライン（GitHub Actions）で以下を検証:
   - `cargo build -p pasta_lsp --target wasm32-unknown-unknown --release` が終了コード0で成功
   - 生成された`.wasm`ファイルの存在確認
   - wasmバイナリサイズが閾値（10MB）以下であることの確認
6. The pasta_lsp shall テストファイルを`crates/pasta_lsp/tests/`配下に`<feature>_test.rs`の命名規則で配置する
