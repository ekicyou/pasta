# Requirements Document

## Introduction

本仕様は pasta アルファリリースに向けた **動作するサンプルゴーストの完全実装** を定義する。

### 背景

- **親仕様**: alpha-release-planning（アルファリリース計画）
- **依存**: 
  - alpha01（SHIORI EVENT）
  - alpha02（仮想イベント）
  - alpha03（さくらスクリプト）
  - act-req-parameter（シーン関数への act 引き渡し）
  - onhour-date-var-transfer（OnHour 時の日時変数自動設定）
- **目的**: pasta エンジンの動作を体験できる完全なサンプルゴーストを提供

### サンプルゴースト概要

- **キャラクター**: 女の子（sakura）と男の子（kero）の2体
- **シェル**: ピクトグラム風のシンプルなPNG画像
- **機能**: 起動挨拶、ダブルクリック反応、終了挨拶、ランダムトーク、時報

### キャラクター設定

| キャラ | 一人称 | 口調 | 性格 |
|--------|--------|------|------|
| **女の子** | わたし | 標準語、丁寧めでかわいい | 明るく元気、ちょっと天然 |
| **男の子** | ぼく | 標準語、少し生意気 | ちょっとマセガキ、でも憎めない |

**トーク例**:
- 女の子: 「こんにちは～、今日もよろしくね！」
- 男の子: 「へえ、また来たんだ。ちゃんと使ってよね。」

---

## Requirements

### Requirement 1: ディレクトリ構成

**Objective:** As a ゴースト開発者, I want 標準的なディレクトリ構成テンプレートがほしい, so that ゴースト開発を始められる

#### Acceptance Criteria

1. The alpha04-sample-ghost shall 専用クレートとして以下のディレクトリ構成を定義する:
   ```
   crates/pasta_sample_ghost/     # Rustクレート（pasta_luaから責務分離）
   ├── Cargo.toml                 # クレート設定
   ├── README.md                  # クレート説明
   ├── src/
   │   └── lib.rs                 # シェル画像生成ロジック
   ├── tests/
   │   └── integration_test.rs    # 統合テスト
   └── ghosts/                    # ゴースト配布物ルート
       └── hello-pasta/           # ゴーストID
           ├── install.txt        # インストール設定
           ├── readme.txt         # 説明ファイル
           ├── ghost/
           │   └── master/
           │       ├── pasta.toml # pasta 設定ファイル
           │       ├── descript.txt # ゴースト設定（ukadoc準拠）
           │       ├── pasta.dll  # SHIORI DLL（テスト時コピー）
           │       └── dic/       # Pasta DSL スクリプト
           │           ├── boot.pasta
           │           ├── talk.pasta
           │           └── click.pasta
           └── shell/
               └── master/        # シェル（見た目）
                   ├── descript.txt
                   ├── surfaces.txt
                   └── surface*.png # サーフェス画像（build時生成）
   ```
2. The alpha04-sample-ghost shall Rustクレート部分（src/, tests/, Cargo.toml）とゴースト配布物（ghosts/）を明確に分離する
3. The alpha04-sample-ghost shall `ghosts/` を配布物ルート、`ghosts/hello-pasta/` をゴーストIDディレクトリとする
4. The alpha04-sample-ghost shall `crates/pasta_sample_ghost/` に配置される（ルート汚染回避）
5. The alpha04-sample-ghost shall pasta_luaから完全に独立したクレートとする（責務分離）

---

### Requirement 2: 起動・終了トーク

**Objective:** As a エンドユーザー, I want ゴースト起動時に挨拶してほしい, so that ゴーストが動作していることがわかる

#### Acceptance Criteria

1. The alpha04-sample-ghost shall OnFirstBoot 時に初回起動メッセージを表示する
2. The alpha04-sample-ghost shall OnBoot 時に起動挨拶を表示する（時間帯に応じた挨拶）
3. The alpha04-sample-ghost shall OnClose 時に終了挨拶を表示する
4. The トーク shall `pasta.shiori.act` を使用してさくらスクリプトを生成する

---

### Requirement 3: ダブルクリック反応

**Objective:** As a エンドユーザー, I want キャラクターをダブルクリックして反応がほしい, so that インタラクションを楽しめる

#### Acceptance Criteria

1. The alpha04-sample-ghost shall OnMouseDoubleClick 時に反応トークを表示する
2. The 反応 shall pasta DSL のみで実装し、ランダム選択により反応の多様性を確保する
3. The 反応 shall 複数バリエーション（5種以上）を用意し、クリック毎に異なる反応を示すこと
4. The alpha04-sample-ghost shall シンプルさを優先し、入門者が理解しやすい実装とすること

---

### Requirement 4: ランダムトーク

**Objective:** As a エンドユーザー, I want ゴーストが時々話しかけてきてほしい, so that 賑やかさを感じられる

#### Acceptance Criteria

1. The alpha04-sample-ghost shall OnTalk 仮想イベント時にランダムトークを表示する
2. The ランダムトーク shall 複数パターン（5〜10種）を用意する
3. The トーク shall sakura と kero の掛け合いを含む

---

### Requirement 5: 時報

**Objective:** As a エンドユーザー, I want 正時に時報を聞きたい, so that 時間を意識できる

#### Acceptance Criteria

1. The alpha04-sample-ghost shall OnHour 仮想イベント時に時報トークを表示する
2. The 時報 shall 現在時刻（`act.var.時` または `act.var.時１２`）を含める
3. The 時報 shall 時間帯に応じたバリエーション（24時間制/12時間制）を持つ
4. The alpha04-sample-ghost shall `onhour-date-var-transfer` 仕様により、OnHour 発火時に日時変数が自動設定されること

---

### Requirement 6: シェル素材

**Objective:** As a 配布担当者, I want 著作権問題のないシェル素材がほしい, so that 自由に配布できる

#### Acceptance Criteria

1. The alpha04-sample-ghost shall 以下の仕様でシェル素材を自動生成する:
   - **サイズ**: 幅 128 × 高さ 256 ピクセル（固定）
   - **比率**: 3頭身（頭部半径 42px、頭部は全体の約1/3）
   - **形式**: 透過 PNG
   - **キャラクター**: 2体
     - 女の子（sakura）: surface0-8（9種）- 赤色 `#DC3545`
     - 男の子（kero）: surface10-18（9種）- 青色 `#007BFF`
   - **生成方法**: Rustコードによるプログラマティック生成（`src/image_generator.rs`）
   - **依存**: `image`, `imageproc`（塗りつぶし円・ポリゴン描画）

2. The ピクトグラム shall トイレマーク風の人型アイコンとする:
   - **頭部**: 塗りつぶし円（半径 42px）
   - **胴体**: 純粋な三角形のみ（台形は不可、手足なし）
     - 女の子: `○ + △`（正三角形、スカート風、頂点が上）
     - 男の子: `○ + ▽`（逆三角形、頂点が下）
   - **装飾**: 手足・耳などの装飾は一切付けない（シンプルさ優先）

3. The 表情 shall 顔部分に線描画で重ねる:
   - **線の太さ**: 3px（視認性確保）
   - **目の間隔**: 36px
   - **表情種類**（9種）:
     - `^ ^` 笑顔, `- -` 通常, `> <` 照れ, `o o` 驚き, `; ;` 泣き
     - `@ @` 困惑, `* *` キラキラ, `= =` 眠い, `# #` 怒り
   - フォント不要（CI再現性確保）

4. The シェル shall `shell/master/descript.txt` でサーフェス定義を行う（ukadoc準拠）

5. The 画像生成 shall CIで再現可能であること（外部依存なし）

---

### Requirement 7: 設定ファイル

**Objective:** As a ゴースト開発者, I want pasta.toml で基本設定を定義したい, so that ゴーストの動作をカスタマイズできる

#### Acceptance Criteria

1. The alpha04-sample-ghost shall `ghosts/hello-pasta/ghost/master/pasta.toml` に以下を定義する:
   ```toml
   [package]
   name = "hello-pasta"
   version = "1.0.0"
   edition = "2024"

   [loader]
   pasta_patterns = ["dic/*.pasta"]
   lua_search_paths = [
       "profile/pasta/save/lua",
       "scripts",
       "profile/pasta/cache/lua",
       "scriptlibs"
   ]
   transpiled_output_dir = "profile/pasta/cache/lua"

   [ghost]
   random_talk_interval = 180
   ```

2. The `lua_search_paths` shall 以下の順序で Lua モジュールを検索する:
   - `profile/pasta/save/lua`: ユーザー保存スクリプト
   - `scripts`: pasta 標準ランタイム（`crates/pasta_lua/scripts/` からコピー）
   - `profile/pasta/cache/lua`: トランスパイル済みキャッシュ
   - `scriptlibs`: 追加ライブラリ

3. The `[package]` セクション shall 教育的コメント付きで含め、伺かゴーストでは省略可能であることを説明する

4. The 設定 shall [pasta.toml設定仕様書](research/pasta-toml-spec.md) に準拠する

5. The 設定 shall alpha02（仮想イベント）で読み込まれる

---

### Requirement 8: テスト要件

**Objective:** As a 開発者, I want サンプルゴーストの動作テストを実行したい, so that 品質を保証できる

#### Test Prerequisites

1. The テスト実行前 shall pasta_shiori をリリースビルドすること:
   ```powershell
   cargo build --release --target i686-pc-windows-msvc -p pasta_shiori
   ```
2. The ビルド成果物 shall `target/i686-pc-windows-msvc/release/pasta.dll` として出力される
   - 注: `pasta_shiori` クレートの `[lib] name = "pasta"` により、出力ファイル名は `pasta.dll`
3. The テスト shall DLL不在時に明確なエラーメッセージを表示する

#### Acceptance Criteria

1. The alpha04-sample-ghost shall `crates/pasta_sample_ghost/tests/` に統合テストを配置する
2. The テスト shall `tests/common/mod.rs` にヘルパー関数 `copy_pasta_shiori_dll()` を実装する
3. The テスト shall PastaLoaderを使用して各イベントハンドラの動作を検証する
4. The テスト shall さくらスクリプト出力の正確性を検証する
5. The テスト shall pasta.toml 設定の読み込みを検証する
6. The テスト shall `cargo test --workspace` でCI実行可能であること（事前ビルド必須）
7. The テスト shall 実SSP不要（モックSHIORI環境）で完結すること
8. The 配布物 shall `ghost/master/pasta.dll` にSHIORI DLLを含むこと

---

### Requirement 9: ukadoc設定ファイル

**Objective:** As a SSPユーザー, I want SSP標準の設定ファイルを持つゴーストがほしい, so that SSPで正常に動作できる

#### Acceptance Criteria

1. The alpha04-sample-ghost shall `ghosts/hello-pasta/install.txt` に以下を定義する:
   - `charset`: UTF-8
   - `type`: ghost
   - `name`: hello-pasta
   - `directory`: hello-pasta

2. The alpha04-sample-ghost shall `ghosts/hello-pasta/ghost/master/descript.txt` に以下を定義する:
   - `charset`: UTF-8
   - `type`: ghost（**必須**）
   - `shiori`: pasta.dll（SHIORI DLL指定 - **必須**）
   - `name`: hello-pasta
   - `sakura.name`: 女の子
   - `kero.name`: 男の子
   - `craftman`: ekicyou
   - `craftmanw`: どっとステーション駅長
   - `homeurl`: https://github.com/ekicyou/pasta

3. The alpha04-sample-ghost shall `ghosts/hello-pasta/shell/master/descript.txt` に以下を定義する:
   - `charset`: UTF-8
   - `name`: master
   - `type`: shell（**必須**）
   - `craftman`: ekicyou
   - `craftmanw`: どっとステーション駅長
   - `seriko.use_self_alpha`: 1
   - `sakura.balloon.offsetx`: 64（画像幅の半分）
   - `sakura.balloon.offsety`: 0
   - `kero.balloon.offsetx`: 64
   - `kero.balloon.offsety`: 0

4. The 設定ファイル shall [ukadoc設定ファイル仕様書](research/ukadoc-config-spec.md) に準拠する

---

### Requirement 10: 配布ビルド自動化

**Objective:** As a ゴースト開発者, I want ワンコマンドで配布可能なゴーストをビルドしたい, so that 手作業を減らせる

#### Acceptance Criteria

1. The alpha04-sample-ghost shall `scripts/build-ghost.ps1` PowerShell スクリプトを提供する

2. The スクリプト shall 以下を自動実行する:
   - 32bit Windows ターゲット（`i686-pc-windows-msvc`）でのDLLビルド
   - テンプレートディレクトリ（`ghosts/hello-pasta/`）のコピー
   - ビルド成果物の配置
   - Lua ランタイムのコピー（後述）

3. The DLLコピー shall 以下の仕様に従う:
   - **ソースパス**: `target/i686-pc-windows-msvc/release/pasta.dll`
   - **出力パス**: `dist/hello-pasta/ghost/master/pasta.dll`
   - **重要**: Cargo.tomlの `[lib] name = "pasta"` により出力ファイル名は `pasta.dll`（`pasta_shiori.dll` ではない）

4. The Luaランタイムコピー shall 以下の仕様に従う:
   - **ソースディレクトリ**: `crates/pasta_lua/scripts/`（全サブディレクトリ含む再帰コピー）
   - **出力ディレクトリ**: `dist/hello-pasta/ghost/master/scripts/`
   - **必須ファイル**: `pasta/*.lua`（コアモジュール）、`hello.lua`（サンプル）

5. The 出力 shall `dist/hello-pasta/` に配布可能な完全なゴーストとして生成する:
   - `ghost/master/pasta.dll` - SHIORI DLL
   - `ghost/master/pasta.toml` - 設定ファイル
   - `ghost/master/scripts/` - Lua ランタイム
   - `shell/master/` - シェル素材（surfaces, descript.txt）
   - `install.txt` - インストール情報

6. The 自動化 shall Rust と PowerShell のみを使用する（Makefile 不使用）

---

## Out of Scope

- 高度な会話ロジック（コンテキスト保持、学習等）
- SAORI/MAKOTO 連携
- ネットワーク更新機能
- 複雑なシェルアニメーション
- 手動でのシェル素材準備（自動生成する）

---

## References

| ドキュメント | 説明 |
|-------------|------|
| [pasta.toml設定仕様書](research/pasta-toml-spec.md) | pasta.toml の全セクション仕様 |
| [ukadoc設定ファイル仕様書](research/ukadoc-config-spec.md) | SSP標準設定ファイル仕様 |

---

## Glossary

| 用語 | 説明 |
|------|------|
| hello-pasta | サンプルゴーストのゴーストID |
| sakura | メインキャラクター（女の子） |
| kero | サブキャラクター（男の子） |
| サーフェス | キャラクターの表情画像 |
| descript.txt | シェル設定ファイル |
| pasta.toml | ゴースト設定ファイル |
