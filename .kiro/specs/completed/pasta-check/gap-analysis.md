# Gap Analysis: pasta-check

## 1. 現状調査

### 1.1 既存アセットマップ

| アセット | パス | 概要 |
|----------|------|------|
| `update_files.rs` | `crates/pasta_sample_ghost/src/update_files.rs` | `updates2.dau` / `updates.txt` 生成ロジック（約280行、テスト含む）|
| `lib.rs` (finalize) | `crates/pasta_sample_ghost/src/lib.rs` | `finalize_ghost()` — `update_files::generate_update_files()` のラッパー |
| `main.rs` (--finalize) | `crates/pasta_sample_ghost/src/main.rs` | `--finalize` CLI オプション、`run_finalize_mode()` |
| `release.ps1` | `crates/pasta_sample_ghost/release.ps1` | 9ステップリリーススクリプト（DLLビルド→配布物生成→NAR作成）|
| `release.bat` | `crates/pasta_sample_ghost/release.bat` | `release.ps1` のバッチラッパー |
| NAR 作成処理 | `release.ps1` Step 8 | PowerShell `Compress-Archive` + `.zip` → `.nar` リネーム |
| robocopy 処理 | `release.ps1` Step 2-4 | `dist-src/` コピー、DLL/scripts コピー |
| `dist-src/` | `crates/pasta_sample_ghost/dist-src/` | **廃止対象**。辞書・設定テキスト・`install.txt` を一時格納。内容は `ghosts/hello-pasta/` に統合 |
| `ghosts/hello-pasta/` | `crates/pasta_sample_ghost/ghosts/hello-pasta/` | ゴースト開発フォルダー（`--target` の指定先）。`dist-src/` 統合後はすべての辞書・テキストを git 管理 |

### 1.2 既存パターンと慣例

| 項目 | 現状 |
|------|------|
| CLI パーサー | プロジェクト内に `clap` 等の CLI フレームワークは使用されていない。`pasta_sample_ghost` は `std::env::args()` で手動解析。`--copy` は複数回指定可能なため array 対応が必要 |
| ZIP 圧縮 | Rust 側にはない。`flate2` は `pasta_lua` でキャッシュ用 gzip に使用されているが、ZIP アーカイブではない |
| 依存グラフ | `pasta_shiori` → `pasta_lua` → `pasta_dsl` + `pasta_core`。`pasta_sample_ghost` は `publish=false` で `pasta_shiori` と `pasta_lua` を `[dev-dependencies]` に持つ |
| ワークスペース版番 | `0.1.21`（`version.workspace = true` で全クレート共通）|
| publish パターン | `pasta_core`, `pasta_dsl`, `pasta_lua`, `pasta_shiori`, `pasta_lsp` → `publish = true`。`pasta_sample_ghost` → `publish = false` |

### 1.3 統合ポイント

- **release.ps1**: Step 5 で `cargo run -p pasta_sample_ghost -- --finalize` を呼び出し。Step 8 で PowerShell NAR 作成。これらが `pasta_check` に置き換わる。
- **dist-src/ 廃止**: `release.ps1` の Step 2（dist-src robocopy）は廃止。辞書・設定テキスト・`install.txt` は `ghosts/hello-pasta/` に直接配置し git 管理。DLL/scripts コピー（旧 Step 4）は `release.ps1` Step 3 として維持。
- **ghosts/hello-pasta/ = 永続開発フォルダー**: `--target` 引数の指定先。`dist-src/` 統合後はすべてのゴーストファイル（辞書・テキスト）を git 管理。DLL・生成画像はビルド成果物として git 管理外。
- **release-workflow 仕様**: タスク一覧の Step 2-A で `cargo publish` 対象を列挙。`pasta_check` を追加する必要あり。
- **Cargo.toml ワークスペース**: `[workspace.dependencies]` に `pasta_check` 内部依存の追加が必要（md5, encoding_rs 等がワークスペース共通化されていないため、個別指定か新規追加）。
- **リリース出力先**: 開発フォルダー（`ghosts/hello-pasta`）とは別に `release/hello-pasta/` および `release/hello-pasta.nar` にリリース成果物を出力する構成に変更。既存の `hello-pasta.nar`（クレートルート直下）は削除対象。

---

## 2. 要件 → アセットマッピング

| 要件 | 既存アセット | ギャップ |
|------|-------------|---------|
| **Req 1**: クレート構成 | — | **Missing**: `crates/pasta_check/` ディレクトリ、`Cargo.toml`、`src/main.rs` を新規作成。ワークスペース `[workspace.dependencies]` に内部依存追加 |
| **Req 2**: CLI インターフェース | `pasta_sample_ghost/main.rs`（手動 args 解析） | **Missing**: `release` サブコマンド、`--target`/`--release`/`--nar`/`--copy` オプション解析。CLI フレームワーク選定が必要 |
| **Req 3**: release 実行フロー | `release.ps1` Step 2-5, 8（PowerShell） | **Missing**: フォルダー削除→再帰コピー→上書きコピーのロジックを Rust で実装 |
| **Req 4**: 更新ファイル生成 | `update_files.rs`（完全な実装+テスト） | **移植可能**: 既存コードをほぼそのまま移植可能。依存: `md5 0.8`, `encoding_rs 0.8` |
| **Req 5**: NAR 作成 | `release.ps1` Step 8（PowerShell） | **Missing**: Rust の ZIP ライブラリ（`zip` クレート）による NAR 作成が必要。`flate2` は gzip のみで ZIP アーカイブ非対応 |
| **Req 6**: pasta_sample_ghost 分離 | `update_files.rs`, `lib.rs` finalize, `main.rs` --finalize, `hello-pasta.nar` | **削除対象**: `update_files.rs` モジュール全体、`finalize_ghost()` 関数、`--finalize` CLI 処理、`md5`/`encoding_rs` 依存、`hello-pasta.nar` |
| **Req 7**: release.ps1 簡素化 | `release.ps1` Step 2, 4, 5, 7, 8 | **変更**: Step 2（dist-src robocopy）は廃止（`ghosts/hello-pasta/` に直接統合）。Step 5（finalize）、Step 8（NAR作成）を `pasta_check release` に置換。旧 Step 4（DLL/scripts コピー）は `release.ps1` の新 Step 3 として維持（`GhostDir` へ直接コピー）。Step 7（バリデーション）は不要（`pasta_check` の正常終了で保証）。`--copy` 引数は hello-pasta フローでは使用しない。9ステップ→6ステップに簡素化 |
| **Req 8**: release.bat 移動 | `crates/pasta_sample_ghost/release.bat` | **変更**: リポジトリルートに移動、パス解決を調整 |
| **Req 9**: crates.io 公開 | publish パターン確立済み | **Missing**: `Cargo.toml` で `publish = true` + `description`。`release-workflow` タスクリストへの追加 |

---

## 3. 実装アプローチ評価

### Option A: 最小移植（更新ファイル生成のみ移植、NAR はスクリプト継続）

**概要**: `update_files.rs` を `pasta_check` に移植し、`release` サブコマンドで更新ファイル生成のみ担当。NAR 作成は引き続き `release.ps1` で実施。

**トレードオフ**:
- ✅ 最小工数、既存 NAR 作成ロジックの動作保証
- ✅ ZIP ライブラリの新規依存を回避
- ❌ 要件 5（NAR 作成の Rust 化）を未達成
- ❌ `release.ps1` の簡素化が限定的

**評価**: 要件 5 を満たさないため、**不採用**。

### Option B: フル移植（推奨）

**概要**: `update_files.rs` の移植 + ファイルコピーロジック + NAR（ZIP）作成の全てを `pasta_check` に実装。`release.ps1` は DLL ビルド・画像生成・バリデーション・リリース手順表示に専念。

**トレードオフ**:
- ✅ 全要件を完全達成
- ✅ `release.ps1` から robocopy/Compress-Archive 依存を除去し、クロスプラットフォーム性向上
- ✅ 将来的に `release.ps1` 自体の不要化への道筋
- ❌ `zip` クレートの新規依存追加（ただし成熟したクレート）
- ❌ ファイルコピーロジックの Rust 実装が必要（中程度の工数）

**評価**: 全要件を達成し、プロジェクトの方向性と合致。**推奨**。

### Option C: ハイブリッド（段階的移行）

**概要**: Phase 1 で更新ファイル生成 + NAR 作成を移植。Phase 2 でリリースフォルダー構築（`--target`/`--release`/`--copy`）を追加。

**トレードオフ**:
- ✅ 段階的リスク低減
- ✅ 初期リリースを早期に達成可能
- ❌ Phase 1 では `release.ps1` の robocopy がまだ必要
- ❌ 2フェーズの管理コスト

**評価**: Option B の工数が許容範囲のため、段階化のメリットが薄い。Option B が困難な場合のフォールバックとして有用。

---

## 4. 技術的課題と Research Needed

### 4.1 CLI フレームワーク選定

**ステータス**: Research Needed

現在のプロジェクトに CLI フレームワーク（`clap` 等）の依存はない。選択肢：

| 候補 | メリット | デメリット |
|------|---------|----------|
| `clap` (derive) | 業界標準、自動ヘルプ/バリデーション | 依存サイズ大（コンパイル時間増） |
| `std::env::args()` 手動解析 | 依存ゼロ、既存パターンと一致 | サブコマンド対応が煩雑 |
| `lexopt` / `pico-args` | 軽量、最小依存 | エコシステムが小さい |

**設計フェーズへの課題**: `pasta_check` は `crates.io` に publish するため、依存サイズと使いやすさのバランスを検討する必要あり。

### 4.2 ZIP ライブラリ選定

**ステータス**: Research Needed

NAR ファイルは ZIP アーカイブ（拡張子 `.nar`）。候補：

| 候補 | メリット | デメリット |
|------|---------|----------|
| `zip` クレート | 成熟、広く使用 | 依存サイズ中 |
| `flate2` + 手動 ZIP | 既にワークスペースに存在 | ZIP フォーマット手動実装は非現実的 |

**結論（暫定）**: `zip` クレートが最も現実的。

### 4.3 ファイルコピーの再帰実装

**ステータス**: 低リスク

`std::fs::copy` + `walkdir`（または手動再帰）で対応可能。既存の `collect_files_recursive` パターンが参考になる。`--copy` の上書きコピーも同様のパターン。

### 4.4 release-workflow 仕様への統合

**ステータス**: 制約

`release-workflow` の `tasks.md` タスクリスト Step 2-A に `cargo publish` 対象クレートが列挙されている。`pasta_check` の追加が必要。設計フェーズで具体的な統合方法を定義する。

---

## 5. 複雑度とリスク評価

| 項目 | 評価 | 根拠 |
|------|------|------|
| **工数** | **M（3-7日）** | 既存 `update_files.rs` の移植がベース。新規実装は CLI 解析、ファイルコピー、ZIP 作成。パターンは既存コードで確立済み |
| **リスク** | **Low** | 既知の技術（Rust fs 操作、ZIP クレート）、明確なスコープ、既存テストが移植可能。CLI/ZIP ライブラリは成熟 |

---

## 6. 設計フェーズへの推奨事項

### 推奨アプローチ: Option B（フル移植）

### 設計フェーズで決定すべき事項

1. **CLI フレームワーク**: `clap` vs 手動解析 vs 軽量ライブラリ — publish するクレートとしてのトレードオフ
2. **ZIP ライブラリ**: `zip` クレートの具体的バージョンと設定（圧縮レベル等）
3. **update_files.rs の移植方法**: コードコピー vs 共通ライブラリ化（`pasta_check` に直接移植が妥当か）
4. **release.ps1 の変更範囲**: Step 5/8 の置換方法の詳細設計
5. **pasta_sample_ghost のテスト影響**: `finalize_ghost` を使用するテストの有無と対応

### キャリーフォワード Research Items

- CLI フレームワーク選定（§4.1）
- ZIP ライブラリ選定（§4.2）
