# Requirements Document

## はじめに

本ドキュメントは、Pasta DSL向けランゲージサーバー（`pasta_lang_server` クレート）の要件を定義する。

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

1. The pasta_lang_server shall LSPプロトコル（Language Server Protocol 3.17+）に準拠したサーバーとして動作する
2. When エディタがLSPサーバーに接続した時, the pasta_lang_server shall `initialize`リクエストに対してサーバーケーパビリティを返却する
3. When エディタが`*.pasta`ファイルを開いた時, the pasta_lang_server shall `textDocument/didOpen`通知を受信しドキュメントを管理対象に追加する
4. When エディタが`*.pasta`ファイルの内容を変更した時, the pasta_lang_server shall `textDocument/didChange`通知を受信しドキュメントの内容を同期する
5. When エディタが`*.pasta`ファイルを閉じた時, the pasta_lang_server shall `textDocument/didClose`通知を受信しドキュメントを管理対象から除外する

### Requirement 2: セマンティックトークンによるシンタックスハイライト

**目的:** 開発者として、`*.pasta`ファイルをエディタで開いた際にDSL構文が色分けされて表示されたい。コードの可読性を向上させ、構文エラーの早期発見を容易にするため。

#### 受入基準

1. The pasta_lang_server shall `textDocument/semanticTokens/full`リクエストに対して、ドキュメント全体のセマンティックトークンを返却する
2. The pasta_lang_server shall 以下のトークンタイプを識別しエディタに通知する:
   - **コメント**: `＃` / `#` で始まるコメント行
   - **シーンマーカー**: `＊` / `*`（グローバルシーン）, `・` / `-`（ローカルシーン）
   - **属性マーカー**: `＆` / `&` で始まる属性定義行
   - **単語マーカー**: `＠` / `@` で始まる単語定義・参照
   - **変数マーカー**: `＄` / `$` で始まる変数宣言・参照
   - **Call マーカー**: `＞` / `>` で始まるシーン呼び出し
   - **アクター辞書マーカー**: `％` / `%` で始まるアクター辞書定義
   - **アクター名**: アクション行のアクター名（`：` / `:` の前）
   - **Luaコードブロック**: ` ```lua ` ～ ` ``` ` で囲まれた範囲
   - **文字列リテラル**: アクション行のテキスト部分
   - **コロン区切り**: `：` / `:` セパレータ
3. When ドキュメントの内容が変更された時, the pasta_lang_server shall セマンティックトークンを再計算し最新状態を維持する
4. The pasta_lang_server shall 全角マーカー（`＊`、`・`、`＠`、`＄`、`＞`、`＃`、`＆`、`％`、`：`）と半角マーカー（`*`、`-`、`@`、`$`、`>`、`#`、`&`、`%`、`:`）を同等に認識してトークン化する

### Requirement 3: pasta_dslパーサー統合

**目的:** 開発者として、既存のpasta_dslクレートのPEGパーサーをランゲージサーバーで活用したい。パース精度の一貫性を保ち、実装の重複を避けるため。

#### 受入基準

1. The pasta_lang_server shall `pasta_dsl`クレートの`parse_str()`関数を利用してDSLソースをASTに変換する
2. When `pasta_dsl`のパーサーがパースエラーを返却した時, the pasta_lang_server shall エラー情報をLSP Diagnostics（`textDocument/publishDiagnostics`）としてエディタに通知する
3. The pasta_lang_server shall ASTの各ノード（`GlobalSceneScope`, `FileItem::FileAttr`, `FileItem::GlobalWord`, `FileItem::ActorScope`等）からセマンティックトークンのタイプと範囲を算出する
4. If `pasta_dsl`のパーサーがクラッシュまたは予期しないエラーを発生させた時, the pasta_lang_server shall サーバー全体を停止せず、該当ドキュメントについてエラーメッセージをログに記録する

### Requirement 4: WebAssembly（WASM）ビルド対応

**目的:** 開発者として、ランゲージサーバーを`wasm32-unknown-unknown`ターゲットにビルドしたい。将来的に`pasta_vscode` VSCode拡張としてブラウザ環境でも動作させるため。

#### 受入基準

1. The pasta_lang_server shall `cargo build --target wasm32-unknown-unknown` でコンパイルエラーなくビルドできる
2. The pasta_lang_server shall ファイルシステムI/O・ネットワークI/O・スレッド生成などのWASM非互換APIを直接使用しない
3. The pasta_lang_server shall 依存クレート（`pasta_dsl`含む）がすべて`wasm32-unknown-unknown`ターゲットでビルド可能であることを保証する
4. While WASMビルドモードの時, the pasta_lang_server shall LSPトランスポート層をプラットフォーム抽象化し、ネイティブ（stdio）とWASM（メッセージパッシング）の両方に対応する
5. The pasta_lang_server shall `#[cfg(target_arch = "wasm32")]` による条件コンパイルでWASM固有コードとネイティブ固有コードを分離する

### Requirement 5: クレート設計とワークスペース統合

**目的:** 開発者として、ランゲージサーバーが既存のpastaワークスペース構成に適合する形で実装されたい。プロジェクトの一貫性を維持し、保守性を高めるため。

#### 受入基準

1. The pasta_lang_server shall 独立したクレートとして`crates/pasta_lang_server/`に配置される
2. The pasta_lang_server shall `pasta_dsl`クレートにのみ依存し、`pasta_lua`・`pasta_core`・`pasta_shiori`には依存しない
3. The pasta_lang_server shall ワークスペースの`Cargo.toml`に`members`として登録される
4. The pasta_lang_server shall 既存のCI/CDパイプライン（GitHub Actions）で`cargo test --workspace`に含まれてテストされる
5. The pasta_lang_server shall `MIT OR Apache-2.0`デュアルライセンスに従う

### Requirement 6: ドキュメント管理

**目的:** 開発者として、エディタで編集中の`*.pasta`ファイルの内容をランゲージサーバーが正確に追跡したい。リアルタイムなハイライト更新を実現するため。

#### 受入基準

1. The pasta_lang_server shall 開かれた各ドキュメントのテキスト内容をメモリ上に保持する
2. When エディタから増分テキスト変更通知（`TextDocumentContentChangeEvent`）を受信した時, the pasta_lang_server shall ドキュメント内容を正確に更新する
3. When ドキュメント内容が更新された時, the pasta_lang_server shall 再パースを実行しセマンティックトークンとDiagnosticsを更新する
4. The pasta_lang_server shall UNICODEテキスト（日本語識別子・全角マーカー含む）のバイトオフセット計算を正確に行う
5. If ドキュメントの文字エンコーディングがUTF-8でない時, the pasta_lang_server shall エラーDiagnosticsを発行し、処理をスキップする

### Requirement 7: テスト要件

**目的:** 開発者として、ランゲージサーバーの品質を自動テストで保証したい。リグレッション防止と継続的な品質維持のため。

#### 受入基準

1. The pasta_lang_server shall 各トークンタイプの識別に対するユニットテストを備える
2. The pasta_lang_server shall LSPリクエスト/レスポンスの統合テストを備える
3. The pasta_lang_server shall 全角・半角マーカー両方のパターンに対するテストケースを含む
4. The pasta_lang_server shall 日本語識別子（シーン名・変数名・単語名）を含むテストケースを備える
5. The pasta_lang_server shall WASMターゲットビルドの成功を検証するCIテストを備える
6. The pasta_lang_server shall テストファイルを`crates/pasta_lang_server/tests/`配下に`<feature>_test.rs`の命名規則で配置する
