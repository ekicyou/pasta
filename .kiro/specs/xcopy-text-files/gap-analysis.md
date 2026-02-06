# ギャップ分析: xcopy-text-files

## 1. 現状調査

### 1.1 テキストファイル生成の現在のフロー

現在、テキスト系配布ファイルは以下の **2系統** で生成されている：

| 系統 | ソース | 生成先 | 仕組み |
|------|--------|--------|--------|
| **設定ファイル** | `templates/*.template` + `config_templates.rs` | `ghosts/hello-pasta/` 各所 | `include_str!` でテンプレート読み込み → `{{name}}` 等をRustで置換 → `fs::write` |
| **pastaスクリプト** | `scripts.rs` 内の `const &str` | `ghosts/hello-pasta/ghost/master/dic/` | Rustソース内に直接ハードコード → `fs::write` |

**生成トリガー**: `cargo run -p pasta_sample_ghost`（release.ps1 の Step 2 で呼ばれる）

### 1.2 release.ps1 の現在のフロー

```
Step 1: pasta.dll ビルド（cargo build --release --target i686-pc-windows-msvc）
Step 2: cargo run -p pasta_sample_ghost  ← テキスト＋画像を全生成
Step 3: pasta.dll コピー + scripts/ コピー（robocopy）
Step 4: finalize（updates2.dau / updates.txt 生成）
Step 5-8: バリデーション → .nar パッケージ作成
```

### 1.3 影響を受けるファイル一覧

| ファイル | 現在の生成元 | パス（配布先） |
|----------|-------------|----------------|
| `install.txt` | `templates/install.txt.template` | `ghosts/hello-pasta/install.txt` |
| `ghost/master/descript.txt` | `templates/ghost_descript.txt.template` | `ghosts/hello-pasta/ghost/master/descript.txt` |
| `ghost/master/pasta.toml` | `templates/pasta.toml.template` | `ghosts/hello-pasta/ghost/master/pasta.toml` |
| `shell/master/descript.txt` | `templates/shell_descript.txt.template` | `ghosts/hello-pasta/shell/master/descript.txt` |
| `shell/master/surfaces.txt` | `config_templates.rs::generate_surfaces_txt()` | `ghosts/hello-pasta/shell/master/surfaces.txt` |
| `ghost/master/dic/actors.pasta` | `scripts.rs::ACTORS_PASTA` | `ghosts/hello-pasta/ghost/master/dic/actors.pasta` |
| `ghost/master/dic/boot.pasta` | `scripts.rs::BOOT_PASTA` | `ghosts/hello-pasta/ghost/master/dic/boot.pasta` |
| `ghost/master/dic/talk.pasta` | `scripts.rs::TALK_PASTA` | `ghosts/hello-pasta/ghost/master/dic/talk.pasta` |
| `ghost/master/dic/click.pasta` | `scripts.rs::CLICK_PASTA` | `ghosts/hello-pasta/ghost/master/dic/click.pasta` |

**対象外**: `shell/master/surface*.png`（18ファイル、`image_generator.rs` で動的生成 → 変更なし）

### 1.4 テスト依存関係

| テストファイル | 依存方式 | 影響 |
|---------------|---------|------|
| `scripts.rs` 内ユニットテスト（6テスト） | `ACTORS_PASTA`, `BOOT_PASTA` 等の定数を直接参照 | 定数の参照元変更が必要 |
| `config_templates.rs` 内ユニットテスト（4テスト） | `generate_*()` 関数の戻り値を検証 | 関数廃止なら削除/移行 |
| `integration_test.rs`（8テスト） | `generate_ghost()` → `fs::read_to_string()` でファイル内容検証 | `generate_ghost()` のロジック変更に追従 |
| `integration_test.rs::test_random_talk_patterns` | `scripts::TALK_PASTA` 定数を直接参照 | 定数参照方式の変更に追従 |
| `integration_test.rs::test_hour_chime_patterns` | `scripts::TALK_PASTA` 定数を直接参照 | 同上 |

### 1.5 既存の `templates/` ディレクトリ

現在 `templates/` には4つの `.template` ファイルが存在：
- `install.txt.template` — プレースホルダー: `{{name}}`
- `ghost_descript.txt.template` — プレースホルダー: `{{name}}`, `{{sakura_name}}`, `{{kero_name}}`, `{{craftman}}`, `{{craftman_w}}`, `{{shiori}}`, `{{homeurl}}`
- `shell_descript.txt.template` — プレースホルダー: `{{craftman}}`, `{{craftman_w}}`
- `pasta.toml.template` — プレースホルダー: `{{name}}`, `{{version}}`

## 2. 要件実現可能性分析

### 2.1 要件-資産マッピング

| 要件 | 既存資産 | ギャップ |
|------|---------|---------|
| Req 1: 専用ディレクトリ | `templates/` が部分的に近い | **Missing**: 配布構造をミラーしたディレクトリは存在しない |
| Req 2: .pasta 外部ファイル化 | `scripts.rs` に4定数 | **Missing**: 実ファイルが存在しない（定数→ファイル化が必要） |
| Req 3: 設定ファイル最終形化 | `templates/*.template` | **Missing**: テンプレートは存在するがプレースホルダー未解決 |
| Req 4: release.ps1 xcopy統合 | Step 3 で robocopy パターン確立済み | **Low Gap**: 既存パターンを拡張するだけ |
| Req 5: Rust生成ロジック簡素化 | `config_templates.rs`, `scripts.rs` | **設計判断**: 廃止の範囲を決定する必要あり |
| Req 6: テスト維持 | 18テスト（ユニット10、統合8） | **Medium Gap**: テスト参照方式の変更が必要 |
| Req 7: 画像対象外 | `image_generator.rs` | **No Gap**: 変更不要 |

### 2.2 設計フェーズで要検討事項

1. **ディレクトリ名の選定**: `dist/`, `ghost-files/`, `content/` 等。`templates/` との関係整理
2. **`templates/` ディレクトリの扱い**: 廃止するか、`dist/` に統合するか
3. **`cargo run -p pasta_sample_ghost` の責務**: 画像生成のみ残すか、テキストコピーも行うか
4. **`GhostConfig` の扱い**: テンプレート不要になると `GhostConfig` の用途が縮小する

## 3. 実装アプローチ選択肢

### Option A: 最小変更（`include_str!` 方式）

**概要**: 専用ディレクトリにファイルを配置し、`scripts.rs` は `include_str!` で外部ファイルを参照。`config_templates.rs` はテンプレートを最終形ファイルに置換して同様に `include_str!` 参照。release.ps1 には変更なし（`cargo run` がファイルコピー）。

- **変更ファイル**: `scripts.rs`（定数の参照先変更）、新規ディレクトリ＋実ファイル作成
- **`config_templates.rs`**: テンプレート→最終形に差し替え、`include_str!` で読み込み
- **release.ps1**: 変更不要（`cargo run` が引き続き全生成）

**Trade-offs**:
- ✅ 変更範囲が最小（Rustコード構造を維持）
- ✅ テストが最も影響を受けにくい
- ❌ `cargo run` が依然としてテキストコピーを実行（要件4の趣旨と乖離）
- ❌ `GhostConfig` とテンプレート置換が残る矛盾

### Option B: 完全外部化（xcopy 専用方式）

**概要**: 専用ディレクトリに最終形ファイルを配置。`cargo run` は画像生成のみ。release.ps1 がテキストファイルを robocopy。`config_templates.rs` と `scripts.rs` は大幅に簡素化/廃止。

- **変更ファイル**: `scripts.rs`, `config_templates.rs`, `lib.rs`, `main.rs`, release.ps1, 統合テスト全般
- **新規ディレクトリ**: テキストファイル配置（配布構造ミラー）
- **テンプレート**: 最終形に変換して `templates/` を廃止 → 新ディレクトリに統合

**Trade-offs**:
- ✅ 要件の趣旨に最も忠実（Rust生成→xcopy移行）
- ✅ `config_templates.rs` のテンプレート置換ロジック全廃止でコード量削減
- ✅ DSLスクリプトの編集→リリースサイクルが最速（Rust再コンパイル不要）
- ❌ テスト大幅書き換え（`generate_ghost()` の責務縮小に伴う）
- ❌ `GhostConfig` の一部フィールドが不要に

### Option C: ハイブリッド方式

**概要**: 専用ディレクトリに最終形ファイルを配置。`cargo run` は画像生成に加え、専用ディレクトリからのファイルコピーも実行（テンプレート置換は廃止）。release.ps1 もテキストを robocopy。テストは `generate_ghost()` を引き続き使用可能。

- **`cargo run`**: 画像生成 + 専用ディレクトリから `ghosts/` へコピー（`fs::copy`）
- **release.ps1**: 画像生成の `cargo run` 後に robocopy でテキスト上書き
- **テスト**: `generate_ghost()` がコピー方式に切り替わるため、既存テスト構造を概ね維持

**Trade-offs**:
- ✅ テスト影響を最小化しつつ要件を満たす
- ✅ `cargo run` 単独でも動作する（開発時利便性）
- ❌ 二重コピー（`cargo run` と release.ps1 の両方）の冗長性
- ❌ 設計の一貫性がやや弱い

## 4. 複雑度・リスク評価

| 項目 | 評価 | 理由 |
|------|------|------|
| **工数** | **S（1-3日）** | 既存パターン踏襲、ファイル配置＋コピーロジック変更のみ |
| **リスク** | **Low** | 全テスト既存で検証可能、ロールバック容易、外部依存なし |

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ: **Option B（完全外部化）**

要件の趣旨は「Rustコード生成をやめて xcopy にする」であり、Option B が最も忠実。工数は S であり、テスト書き換えのコストも低い。

### 設計フェーズで決定すべき事項

1. **専用ディレクトリ名**: ユーザーに候補を提示して選定（`dist/`, `ghost-files/` 等）
2. **`templates/` の扱い**: 廃止して新ディレクトリに統合するか
3. **`GhostConfig` の残存範囲**: 画像生成で使うフィールドのみ残すか
4. **`cargo run` の新しい責務**: 画像生成のみ or 画像＋ディレクトリ構造検証
5. **`surfaces.txt` の扱い**: コード生成（サーフェスID列挙）を維持するか、最終形ファイル化するか
