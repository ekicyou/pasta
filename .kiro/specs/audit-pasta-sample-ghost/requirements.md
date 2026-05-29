# 要件ドキュメント

## 導入

pasta_sample_ghostクレート（~300行、5ソースファイル + build.rs）を対象とした脆弱性監査・コード簡素化仕様。画像処理ライブラリ（image 0.25 / imageproc 0.26）の安全な使用、build.rsのファイルI/O安全性、デッドコード除去を実施する。外部振る舞い（生成画像・ゴーストデータ）は不変を保証し、既存テスト全パスを維持する。

## プロジェクト記述（入力）

pasta_sample_ghostはサンプルゴースト「hello-pasta」のビルド・画像生成を行う小規模クレート。image/imageprocによる画像処理を含む。publish=falseのため直接配布はされないが、リリースNARの元データを生成するため品質は重要。画像処理ライブラリ使用箇所の安全性確認、build.rsのファイルI/O安全性検証、デッドコード除去・冗長表現削減を行い、既存テスト全パス・外部振る舞い不変を維持する。

## 境界コンテキスト

- **スコープ内**: pasta_sample_ghost/src/ 全ファイル（lib.rs, main.rs, image_generator.rs, config_templates.rs, scripts.rs）の脆弱性調査、build.rs の安全性確認、デッドコード除去、冗長コード削減
- **スコープ外**: ゴーストデータ（ghosts/）の内容変更、画像デザインの変更、新しいサンプルゴースト追加、image/imageprocクレートの内部実装修正
- **隣接する期待**: release-workflow仕様がNARビルド時にこのクレートの出力を参照する。生成物の形式・パスは変更しない

## 要件

### 要件 1: 画像処理の安全性検証

**目的:** 監査者として、image/imageprocライブラリの使用箇所に脆弱性や不安全なパターンがないことを確認したい。生成画像の品質と安全性を保証するために。

#### 受け入れ基準

1. When image_generator.rs内のピクセル操作（`put_pixel`, `draw_filled_circle_mut`等）が画像境界外の座標を受け取った場合, the pasta_sample_ghost shall パニックせずに安全に処理すること
2. When 画像サイズ定数（WIDTH, HEIGHT）やオフセット計算の結果が整数オーバーフローを起こす可能性がある場合, the pasta_sample_ghost shall オーバーフローしない計算方法を使用すること
3. The pasta_sample_ghost shall 全てのピクセル座標計算において画像境界チェックを実施すること
4. When 全サーフェス画像（surface0-8, surface10-18）を生成した場合, the pasta_sample_ghost shall 既存テストの期待値と同一の結果を生成すること

### 要件 2: build.rsのファイルI/O安全性

**目的:** 監査者として、build.rsのファイルシステム操作が安全であることを確認したい。ビルド時の予期しないエラーを防ぐために。

#### 受け入れ基準

1. When CARGO_MANIFEST_DIR環境変数が設定されていない場合, the build.rs shall パニックメッセージで原因を明示すること
2. When パス結合で親ディレクトリ（`parent()`）が存在しない場合, the build.rs shall 適切なエラーメッセージで失敗すること
3. The build.rs shall パストラバーサル攻撃を受けないよう、外部入力に基づくパス構築を行わないこと
4. When ghosts/hello-pastaディレクトリが存在しない場合, the build.rs shall ビルド失敗ではなく警告メッセージのみを出力すること

### 要件 3: デッドコード・冗長表現の除去

**目的:** 開発者として、不要なコードや冗長な表現を除去したい。コードの可読性と保守性を向上させるために。

#### 受け入れ基準

1. The pasta_sample_ghost shall 未使用の関数・定数・型定義を含まないこと
2. The pasta_sample_ghost shall 未使用のimport文を含まないこと
3. When コード削減を行った場合, the pasta_sample_ghost shall 既存テスト全てにパスすること
4. The pasta_sample_ghost shall Rustコンパイラの `dead_code` 警告が0件であること

### 要件 4: 外部振る舞いの不変性保証

**目的:** リリース担当者として、監査後もゴースト配布物の生成結果が変わらないことを保証したい。既存のリリースワークフローに影響を与えないために。

#### 受け入れ基準

1. When `cargo run -p pasta_sample_ghost` を実行した場合, the pasta_sample_ghost shall 監査前と同一のサーフェス画像ファイル（surface0-8.png, surface10-18.png）を生成すること
2. When surfaces.txtを生成した場合, the pasta_sample_ghost shall 監査前と同一の内容を出力すること
3. The pasta_sample_ghost shall 公開API（`generate_ghost`, `GhostConfig`, `GhostError`）のシグネチャを変更しないこと
4. When `cargo test -p pasta_sample_ghost` を実行した場合, the pasta_sample_ghost shall 全テストにパスすること

### 要件 5: main.rsのエラーハンドリング安全性

**目的:** 監査者として、CLI実行時のエラーハンドリングが適切であることを確認したい。ユーザーに分かりやすいエラー情報を提供するために。

#### 受け入れ基準

1. When 出力先ディレクトリへの書き込み権限がない場合, the pasta_sample_ghost shall 適切なエラーメッセージを表示して終了すること
2. When コマンドライン引数に無効なパスが指定された場合, the pasta_sample_ghost shall パニックせずにエラーを報告すること
3. The pasta_sample_ghost shall 全てのエラーケースで `Box<dyn std::error::Error>` を通じて適切にエラーを伝播すること
