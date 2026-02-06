# Requirements Document

## Introduction

pasta_sample_ghost クレートでは、ゴースト配布物のテキスト系ファイル（pasta.toml、boot.pasta 等）が Rust ソースコード内の定数やテンプレート置換により動的生成されている。この仕様では、テキスト系ファイルをクレート内の専用ディレクトリに実ファイルとして配置し、release.ps1 が xcopy（robocopy）でコピーする方式に移行する。

これにより、テキストファイルの編集にRustの再コンパイルが不要になり、DSLスクリプトの開発サイクルが改善される。

## Project Description (Input)

pasta_sample_ghostのテキスト系ファイルはxcopyしたほうがよい。
pasta.tomlやboot.pastaなどのファイルは、pasta_sample_ghostクレートが生成するのではなく、どこかクレート内に良い感じの名前のディレクトリを作成してそこに配置し、release.ps1がxcopyするほうがよい。

## Requirements

### Requirement 1: テキストファイル配置ディレクトリの導入

**Objective:** As a ゴースト開発者, I want テキスト系配布ファイルがクレート内の専用ディレクトリに実ファイルとして存在すること, so that Rustの再コンパイルなしにテキストファイルを編集・確認できる。

#### Acceptance Criteria

1. The pasta_sample_ghost crate shall テキスト系配布ファイルを格納する専用ディレクトリを `crates/pasta_sample_ghost/` 配下に持つ。
2. The directory name shall ゴースト配布物の構造を反映した直感的な名称である（例: `dist/` や `ghost-files/` 等）。
3. The directory structure shall `ghosts/hello-pasta/` 配布時の最終ディレクトリ構造をそのままミラーする（例: `ghost/master/dic/`, `ghost/master/`, `shell/master/` 等）。

### Requirement 2: pasta DSL スクリプトの外部ファイル化

**Objective:** As a ゴースト開発者, I want .pasta スクリプト（actors.pasta, boot.pasta, talk.pasta, click.pasta）がRustソースコード内の定数ではなく実ファイルとして存在すること, so that DSLスクリプトの編集・プレビューが容易になる。

#### Acceptance Criteria

1. When release.ps1 がゴースト配布物を構築する際, the release script shall 専用ディレクトリ内の `.pasta` ファイルを `ghosts/hello-pasta/ghost/master/dic/` にコピーする。
2. The `scripts.rs` module shall `.pasta` ファイルの内容をハードコードした定数として保持せず、`include_str!` マクロで専用ディレクトリの外部ファイルから読み込む方式、またはRustコード生成を廃止して xcopy のみに依存する方式のいずれかを採用する。

### Requirement 3: 設定ファイルの外部ファイル化

**Objective:** As a ゴースト開発者, I want 設定ファイル（pasta.toml, descript.txt, install.txt 等）がテンプレート置換なしの最終形として実ファイルで存在すること, so that 配布物の内容を直接確認・編集できる。

#### Acceptance Criteria

1. The pasta.toml shall テンプレートプレースホルダー (`{{name}}` 等) を含まず、最終的な値が埋め込まれた状態で専用ディレクトリに配置される。
2. The ghost/master/descript.txt shall テンプレート置換済みの完成形として専用ディレクトリに配置される。
3. The install.txt shall テンプレート置換済みの完成形として専用ディレクトリに配置される。
4. The shell/master/descript.txt shall テンプレート置換済みの完成形として専用ディレクトリに配置される。
5. The surfaces.txt shall 専用ディレクトリに完成形として配置される。

### Requirement 4: release.ps1 の xcopy 統合

**Objective:** As a リリース担当者, I want release.ps1 がテキスト系ファイルを専用ディレクトリから xcopy（robocopy）でコピーすること, so that Rustコード実行なしに配布物を構築できる。

#### Acceptance Criteria

1. When release.ps1 が実行される際, the release script shall 専用ディレクトリの内容を `ghosts/hello-pasta/` に robocopy（ミラーリングまたはコピー）する。
2. The release script shall 既存の pasta.dll コピーおよび scripts/ コピーと同じ Step 3 内で、テキストファイルのコピーを実行する。
3. If 専用ディレクトリが存在しない場合, the release script shall エラーメッセージを表示して処理を中断する。

### Requirement 5: 既存テストの維持

**Objective:** As a 開発者, I want テキストファイル外部化後も既存のテストがすべてパスすること, so that リグレッションが発生しない。

#### Acceptance Criteria

1. While テキストファイルが外部化された後, the `cargo test -p pasta_sample_ghost` shall すべて成功する。
2. The existing tests in `scripts.rs` shall `.pasta` ファイルの内容検証を引き続き実行できる（ファイル読み込み方式の変更に対応）。
3. The existing tests in `config_templates.rs` and integration tests shall 新しいディレクトリ構造に対応して更新される。

### Requirement 6: 画像ファイルは対象外

**Objective:** As a 開発者, I want 画像ファイル（surface*.png）の生成方式は変更しないこと, so that 変更範囲を最小限に抑える。

#### Acceptance Criteria

1. The `image_generator.rs` module shall 既存のRustコードによる動的画像生成を維持する。
2. The surface*.png files shall 引き続き `cargo run -p pasta_sample_ghost` 実行時にRustコードで生成される。
