# Research & Design Decisions

## Summary
- **Feature**: `xcopy-text-files`
- **Discovery Scope**: Extension（既存システムの配布フロー変更）
- **Key Findings**:
  - テキストファイル生成は2系統（テンプレート置換4ファイル＋定数ハードコード4ファイル）、合計8ファイルが対象
  - release.ps1 は既に robocopy パターンを確立済みで、追加ステップの挿入は低リスク
  - `surfaces.txt` はサーフェスID（0-8, 10-18）の機械的列挙であり、画像生成と不可分のためコード生成を維持する
  - `GhostConfig` はテンプレート廃止後も画像生成では不使用のため、大幅に簡素化可能

## Research Log

### 専用ディレクトリ名の選定
- **Context**: Req 1 で「直感的な名称」を要求。候補: `dist/`, `ghost-files/`, `content/`, `hello-pasta-files/`
- **Sources Consulted**: 既存ディレクトリ構造（`templates/`, `ghosts/hello-pasta/`）、Rust エコシステムの慣例
- **Findings**:
  - `dist/` — 一般的だが「ビルド出力」の印象が強い。ここはソース側なので誤解を招く
  - `ghost-files/` — 意味は明確だが `-` 区切りは既存の `hello-pasta` と混在し冗長
  - `content/` — 内容を示すが抽象的すぎる
  - `dist-src/` — 「配布物のソース」を明示、`ghosts/hello-pasta/`（配布先）と対になる
- **Implications**: `dist-src/` を推奨。配布先 `ghosts/hello-pasta/` と「ソース↔出力」の関係が直感的

### `templates/` ディレクトリの扱い
- **Context**: 現在 `templates/` に4つの `.template` ファイルが存在。外部化後はテンプレート置換が不要になるため役割が消失する
- **Findings**:
  - `.template` ファイルは `{{name}}` 等のプレースホルダーを含むが、`GhostConfig::default()` の値は**固定**（実行時変更なし）
  - テンプレート置換は形式的なもので、実質的には固定値の埋め込み
  - `dist-src/` に最終形を直接配置すれば、テンプレート自体が不要
  - `templates/` を残すと二重管理になり保守コスト増
- **Implications**: `templates/` を即座に削除（dist-src 作成と同時）。最終形ファイルを `dist-src/` に直接作成。PowerShell スクリプト等の自動化も不要

### `GhostConfig` の残存範囲
- **Context**: テンプレート置換廃止により `GhostConfig` の多くのフィールドが不要になる
- **Findings**:
  - 現在のフィールド: `name`, `version`, `sakura_name`, `kero_name`, `craftman`, `craftman_w`, `shiori`, `homeurl`
  - テンプレート廃止後の使用箇所: なし（画像生成は `image_generator.rs` で `GhostConfig` を参照していない）
  - `generate_ghost()` のシグネチャは `(output_dir: &Path, config: &GhostConfig)` だが、画像生成のみなら `config` 引数は不要
  - ただし、API 互換性を維持するため `GhostConfig` を即時削除するよりは段階的に検討するべき
- **Implications**: 今回のスコープでは `GhostConfig` の構造体は維持するが、テンプレート関連メソッドを削除。`generate_ghost()` のシグネチャは維持しつつ、内部でテキスト生成を呼ばない形に変更

### `cargo run` の新しい責務
- **Context**: ユーザー確認済み — `cargo run` は画像生成と surfaces.txt のみを担当
- **Findings**:
  - 現在の `generate_ghost()`: (1) `generate_structure()` テキスト生成 → (2) `generate_surfaces()` 画像生成 → (3) `generate_scripts()` スクリプト生成
  - 変更後: (1) ディレクトリ作成 → (2) `generate_surfaces()` 画像生成 → (3) `generate_surfaces_txt()` surfaces.txt 生成
  - `generate_structure()` はテキストファイル書き込みとディレクトリ作成を兼ねているため、ディレクトリ作成のみを残すか、`generate_ghost()` 内でインライン化
- **Implications**: `generate_ghost()` からテキスト生成呼び出しを削除。ディレクトリ作成と画像生成＋surfaces.txt のみに責務を縮小

### テスト影響分析
- **Context**: 18テスト（ユニット10、統合8）の変更範囲を特定
- **Findings**:
  - **`scripts.rs` ユニットテスト（6テスト）**: 定数 `ACTORS_PASTA` 等を直接参照 → `include_str!` でファイル読み込みに変更、またはテスト内で `fs::read_to_string` を使用
  - **`config_templates.rs` ユニットテスト（4テスト）**: `generate_install_txt()` 等を検証 → テンプレート関連関数の廃止に伴い削除。`test_surfaces_txt` のみ残す
  - **`integration_test.rs`（8テスト）**: `generate_ghost()` 経由でファイル内容を検証 → `generate_ghost()` がテキストを生成しなくなるため、テキスト系テストは外部ファイルを直接読み込む方式に変更
  - **`test_random_talk_patterns`, `test_hour_chime_patterns`**: `scripts::TALK_PASTA` 定数を直接参照 → 外部ファイル読み込みに変更
- **Implications**: テスト変更は「定数参照 → ファイル読み込み」のパターン統一で対応可能。テスト構造の大幅な再設計は不要

### release.ps1 フロー変更の詳細
- **Context**: ユーザー確認済みの実行順序: DLLビルド → テキストxcopy → cargo run（画像）→ DLL/scripts コピー → finalize
- **Findings**:
  - 現行 Step 2（`cargo run`）の前に新ステップ「テキストファイル robocopy」を挿入
  - robocopy パターンは Step 3（scripts/ コピー）で既に確立済み: `/MIR`, `/NJH`, `/NJS`, `/NDL`, `/NC`, `/NS`, `/NP`
  - `dist-src/` の構造が `ghosts/hello-pasta/` をミラーするため、robocopy の `/MIR` は不適切（画像ファイルやDLLを削除してしまう）
  - 代替: `/E`（サブディレクトリ含むコピー）+ 個別ファイル指定、または `/XD shell` で画像ディレクトリを除外
  - 最も安全: `robocopy dist-src ghosts\hello-pasta /E /NJH /NJS /NDL /NC /NS /NP`（既存ファイルを上書きするが削除しない）
- **Implications**: `/E` フラグで再帰コピー（削除なし）を使用。`/MIR` は使わない

## Design Decisions

### Decision: 専用ディレクトリ名 `dist-src/`
- **Context**: テキスト系配布ファイルを格納するディレクトリの命名（Req 1.2）
- **Alternatives Considered**:
  1. `dist/` — 一般的だがビルド出力と誤認される
  2. `ghost-files/` — 明確だが冗長
  3. `content/` — 抽象的
  4. `dist-src/` — 配布物ソースの意味が明確
- **Selected Approach**: `dist-src/`
- **Rationale**: 配布先 `ghosts/hello-pasta/` と対になる「配布物のソース」を明示。Rustクレート内での位置 `crates/pasta_sample_ghost/dist-src/` として直感的
- **Trade-offs**: ハイフン付きだが Rust のモジュール名には無関係（ファイルシステムのみ）
- **Follow-up**: ディレクトリ構造は `ghosts/hello-pasta/` の構造をミラーする

### Decision: `templates/` ディレクトリの即時削除
- **Context**: テンプレート置換の存在意義の再検証
- **Alternatives Considered**:
  1. `templates/` を残して別用途に転用
  2. `dist-src/` 初期化完了後に削除
  3. `dist-src/` 作成と同時に即座に削除
- **Selected Approach**: `dist-src/` 作成と同時に即座に削除
- **Rationale**: `GhostConfig::default()` の値は固定であり、テンプレート置換は形式的。最終形ファイルを `dist-src/` に直接作成すれば、テンプレート自体が不要。PowerShell/Rust による自動化スクリプトも不要
- **Trade-offs**: `templates/` に歴史的参照価値があるが、git 履歴で参照可能。初期化スクリプトが不要になり実装がシンプル化
- **Follow-up**: `dist-src/` の8ファイルは実装タスクで固定値を直接埋め込んで作成

### Decision: `config_templates.rs` の簡素化（surfaces.txt 生成のみ残す）
- **Context**: テンプレート置換コードの廃止範囲（Req 3, 6）
- **Alternatives Considered**:
  1. モジュール自体を削除し `generate_surfaces_txt()` を `lib.rs` か別モジュールに移動
  2. モジュール名を変更して `generate_surfaces_txt()` のみ残す
  3. モジュール名はそのままで不要な関数・定数を削除
- **Selected Approach**: モジュール名 `config_templates.rs` を維持し、`generate_surfaces_txt()` とそのテストのみを残す
- **Rationale**: ファイル名変更は git diff のノイズが大きい。関数削除のみで十分明確
- **Trade-offs**: モジュール名が実態（surfaces.txt 生成のみ）と乖離するが、将来的な設定生成の拡張点として維持可能

### Decision: robocopy の `/E` フラグ使用
- **Context**: `dist-src/` → `ghosts/hello-pasta/` へのコピー方式（Req 4）
- **Alternatives Considered**:
  1. `/MIR` — ミラーリング（ソースにないファイルを削除）
  2. `/E` — サブディレクトリ含む再帰コピー（既存ファイル保持）
  3. `Copy-Item -Recurse` — PowerShell ネイティブ
- **Selected Approach**: `robocopy /E`
- **Rationale**: `dist-src/` には画像ファイルが含まれないため、`/MIR` を使うと後続の画像生成結果が消える。`/E` なら既存ファイルを保持しつつテキストをコピー
- **Trade-offs**: 孤立したテキストファイルがクリーンアップされないが、release.ps1 は毎回 `ghosts/hello-pasta/` をクリーンビルドする前提
