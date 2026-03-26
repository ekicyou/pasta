# Research & Design Decisions: pasta-check

## Summary
- **Feature**: `pasta-check`
- **Discovery Scope**: Extension（既存システムからの機能抽出）
- **Key Findings**:
  - CLI フレームワーク: `clap` は過剰。`lexopt` v0.3.2 がプロジェクト規模に最適
  - ZIP ライブラリ: `zip` v2.6（deflate のみ有効化）で NAR 作成が実現可能
  - 既存 `update_files.rs` は自己完結型で、ほぼそのまま移植可能
  - ファイルコピーロジックは `std::fs` の再帰走査で十分（外部依存不要）

## Research Log

### CLI フレームワーク選定

- **Context**: 要件 2 で `release` サブコマンド + `--target`/`--release`/`--nar`/`--copy`（複数回）オプションが必要。既存プロジェクトに CLI フレームワークの依存なし。
- **Sources Consulted**:
  - crates.io: `clap` v4.6.0（181 SLoC wrapper、MSRV 1.85、依存多数）
  - crates.io: `lexopt` v0.3.2（1,316 SLoC、26.8 KiB、依存ゼロ、MSRV 1.31）
  - 既存コード: `pasta_sample_ghost/src/main.rs`（`std::env::args()` 手動解析）
- **Findings**:
  - `clap` derive: 自動ヘルプ生成・バリデーション完備だが、依存ツリーが大きい（proc-macro 含む）。プロジェクトの他のクレートが `clap` を使用していないため、ワークスペース全体のビルド時間に影響する
  - `lexopt`: 依存ゼロ、1ファイル構成。OsString ベースのペダンティックなパーサー。サブコマンドは手動実装だが、`release` 1つのみなので問題なし。`--copy` の複数回指定は while ループでの蓄積パターンで自然に対応
  - `std::env::args()` 手動: 依存ゼロだが、サブコマンド + 複数値オプションの解析が煩雑。既存パターンは `--finalize` フラグ1つのみの単純構成
- **Implications**: `lexopt` を採用。最小依存・MIT ライセンス・crates.io publish に最適。エラーメッセージのカスタマイズも容易

### ZIP ライブラリ選定

- **Context**: 要件 5 で NAR ファイル（ZIP 形式、拡張子 `.nar`）の Rust 生成が必要。現状は PowerShell の `Compress-Archive` → `.zip` → `.nar` リネーム
- **Sources Consulted**:
  - crates.io: `zip` v8.4.0（12K SLoC、144 KiB、MSRV 1.88）
  - 既存コード: `release.ps1` Step 8（`Compress-Archive` + `Rename-Item`）
  - 既存ワークスペース: `flate2` は gzip のみ（ZIP アーカイブ非対応）
- **Findings**:
  - `zip` v8.4.0: 成熟クレート（1.5億 DL）、読み書き両対応。デフォルトで多数の圧縮形式が有効
  - デフォルトフィーチャー: `aes-crypto`, `bzip2`, `deflate`, `deflate64`, `lzma`, `ppmd`, `time`, `xz`, `zstd` — NAR には `deflate` のみで十分
  - 最小構成: `default-features = false, features = ["deflate"]` で依存を大幅に削減可能
  - NAR の圧縮形式: SSP は標準的な ZIP deflate を想定。特殊な圧縮形式は不要
  - MSRV 1.88: プロジェクトは Rust 2024 edition で最新ツールチェーン使用のため問題なし
- **Implications**: `zip` クレートを `default-features = false, features = ["deflate"]` で採用。NAR は ZIP deflate のみで十分

### 既存コードの移植性分析

- **Context**: `update_files.rs` のコード移植方法を評価
- **Findings**:
  - `update_files.rs` は約 230 行（テスト除く）で完全に自己完結
  - 依存: `md5` (0.8)、`encoding_rs` (0.8)、`std::fs`/`std::io`/`std::path` のみ
  - パブリック API: `generate_update_files(root_dir: &Path) -> io::Result<usize>`、`FileEntry` 構造体
  - テスト: 3テスト関数（`test_calculate_md5`, `test_collect_files_excludes_update_files`, `test_generate_update_files`）— すべて移植可能
  - `lib.rs` の `finalize_ghost()` は単なるラッパー（`update_files::generate_update_files()` を呼ぶだけ）
  - `integration_test.rs` の 10 テストはすべて画像生成テスト — finalize/update_files を参照していない
- **Implications**: コードコピーによる直接移植が最適。共通ライブラリ化は不要（依存グラフが逆方向になるため）

### ファイルコピーの実装パターン

- **Context**: `release` サブコマンドの再帰コピー機能（`--target` → `--release`、`--copy` → `--release` 上書き）
- **Findings**:
  - 既存パターン: `update_files.rs` の `collect_files_recursive()` が再帰ディレクトリ走査のリファレンスとして利用可能
  - `release.ps1` の robocopy は `/MIR`（ミラー）と `/E`（空ディレクトリ含む）を使用
  - `walkdir` クレートは不要 — `std::fs::read_dir` + 再帰で十分（既存パターンと一致）
  - コピー対象: 全ファイル（除外なし）。除外は `updates2.dau`/`updates.txt` 生成時と NAR 作成時に `profile/` のみ
- **Implications**: `std::fs` のみで実装。外部クレート追加なし

## Design Decisions

### Decision: CLI フレームワークに `lexopt` を採用

- **Context**: `pasta_check` は crates.io に publish する CLI ツール。サブコマンド1つ + オプション4種（うち1つは複数回指定可能）
- **Alternatives Considered**:
  1. `clap` v4.6 — フル機能 CLI フレームワーク。derive マクロで宣言的定義
  2. `lexopt` v0.3.2 — ミニマル CLI レキサー。依存ゼロ、手動マッチング
  3. `std::env::args()` 手動解析 — 依存ゼロ、完全手動
- **Selected Approach**: `lexopt` v0.3.2
- **Rationale**:
  - プロジェクト内に `clap` 使用実績なし → ビルド時間への影響を最小化
  - サブコマンド1つ + オプション4種の規模では `clap` は過剰
  - `lexopt` は `--copy` の複数回指定を `while let Some(arg) = parser.next()` ループで自然に処理
  - 依存ゼロで `cargo publish` 時の依存ツリーが最小
  - MIT ライセンスでプロジェクトの MIT OR Apache-2.0 と互換
- **Trade-offs**: 自動ヘルプ生成なし（手動で `--help` を実装する必要あり）。ただしオプション数が少ないため負担は軽微
- **Follow-up**: `--help` メッセージのフォーマットを実装時に確定

### Decision: ZIP ライブラリに `zip` (deflate のみ) を採用

- **Context**: NAR ファイルは ZIP アーカイブ。現状は PowerShell `Compress-Archive` で作成
- **Alternatives Considered**:
  1. `zip` v8.4 (default features) — 全圧縮形式対応
  2. `zip` v8.4 (deflate のみ) — 最小機能セット
  3. `flate2` + 手動 ZIP 実装 — 既存依存の活用
- **Selected Approach**: `zip` v8.4 (default-features = false, features = ["deflate"])
- **Rationale**:
  - NAR は標準 ZIP deflate で十分。AES、bzip2、zstd 等は不要
  - `default-features = false` で依存サイズを最小化
  - `flate2` は gzip ストリーム用で ZIP アーカイブのディレクトリ構造を持たない — 手動実装は非現実的
  - `zip` クレートは 1.5 億 DL の実績があり、セキュリティ問題の早期発見が期待される
- **Trade-offs**: 新規外部依存の追加。ただし deflate のみの最小構成で影響を抑制
- **Follow-up**: 圧縮レベルの選定（デフォルト deflate で十分か）

### Decision: `update_files.rs` はコードコピーで直接移植

- **Context**: `update_files.rs` を `pasta_check` に移植する方法
- **Alternatives Considered**:
  1. コードコピー — ファイルごと `pasta_check/src/` にコピーして適合
  2. 共通ライブラリ化 — `pasta_core` に移動して両クレートから参照
  3. `pasta_sample_ghost` を依存として参照
- **Selected Approach**: コードコピー
- **Rationale**:
  - `update_files.rs` は自己完結型（外部クレートは `md5` と `encoding_rs` のみ）
  - 共通ライブラリ化は依存グラフを複雑化（`pasta_core` に `md5`/`encoding_rs` 依存を追加することになる）
  - `pasta_sample_ghost` は `publish = false` のため `pasta_check` から依存できない
  - 移植後に `pasta_sample_ghost` から該当コードを削除するため、重複の長期維持は発生しない
- **Trade-offs**: 一時的なコード重複（移植完了後に解消）

### Decision: `md5`/`encoding_rs` をワークスペース依存に昇格しない

- **Context**: `md5` 0.8 と `encoding_rs` 0.8 は現在 `pasta_sample_ghost` の個別依存
- **Selected Approach**: `pasta_check` の個別依存として追加。`[workspace.dependencies]` には追加しない
- **Rationale**:
  - これらのクレートは `pasta_check` のみが使用する（移植後、`pasta_sample_ghost` からは削除される）
  - ワークスペース共通化は 2 クレート以上で共有する場合のパターン
  - 1 クレートのみの使用ではワークスペース依存に追加する動機が薄い

## Risks & Mitigations

- **zip クレートの MSRV (1.88)**: プロジェクトが Rust 2024 edition を使用しているため問題なし。ただしツールチェーンが古い環境では注意 → CI でツールチェーンバージョンを固定
- **NAR 形式の互換性**: `Compress-Archive` (PowerShell) と `zip` クレートで生成される ZIP のバイナリ互換性は保証されないが、SSP は標準 ZIP パーサーを使用するためフォーマット互換性は確保される → 実装後に SSP での動作確認テストを実施
- **release.ps1 の並行変更**: `pasta_check` 実装中に `release.ps1` を変更するとコンフリクトのリスク → `pasta_check` 実装を先に完了し、その後 `release.ps1` を 1 回の変更で更新

## References

- [lexopt v0.3.2 ドキュメント](https://docs.rs/lexopt/0.3.2) — CLI パーサー
- [zip v8.4.0 ドキュメント](https://docs.rs/zip/8.4.0) — ZIP アーカイブ作成
- [md5 v0.8 ドキュメント](https://docs.rs/md5/0.8) — MD5 ハッシュ計算
- [encoding_rs v0.8 ドキュメント](https://docs.rs/encoding_rs/0.8) — Shift_JIS エンコーディング
- `crates/pasta_sample_ghost/src/update_files.rs` — 移植元コード
- `crates/pasta_sample_ghost/release.ps1` — 統合対象スクリプト
