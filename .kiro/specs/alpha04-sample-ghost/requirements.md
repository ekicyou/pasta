# Requirements Document

## Introduction

本仕様は pasta アルファリリースに向けた **動作するサンプルゴーストの完全実装** を定義する。

### 背景

- **親仕様**: alpha-release-planning（アルファリリース計画）
- **依存**: alpha01（SHIORI EVENT）, alpha02（仮想イベント）, alpha03（さくらスクリプト）
- **目的**: pasta エンジンの動作を体験できる完全なサンプルゴーストを提供

### サンプルゴースト概要

- **キャラクター**: 女の子（sakura）と男の子（kero）の2体
- **シェル**: ピクトグラム風のシンプルなPNG画像
- **機能**: 起動挨拶、ダブルクリック反応、終了挨拶、ランダムトーク、時報

---

## Requirements

### Requirement 1: ディレクトリ構成

**Objective:** As a ゴースト開発者, I want 標準的なディレクトリ構成テンプレートがほしい, so that ゴースト開発を始められる

#### Acceptance Criteria

1. The alpha04-sample-ghost shall 以下のディレクトリ構成を定義する:
   ```
   examples/sample-ghost/
   └── hello-pasta/            # ゴーストID
       ├── install.txt         # インストール設定
       ├── readme.txt          # 説明ファイル
       ├── thumbnail.png       # サムネイル画像
       ├── ghost/
       │   └── master/
       │       ├── pasta.toml  # pasta 設定ファイル
       │       ├── dic/        # Pasta DSL スクリプト
       │       │   ├── boot.pasta  # 起動・終了トーク
       │       │   ├── talk.pasta  # ランダムトーク
       │       │   └── click.pasta # クリック反応
       │       └── scripts/    # Lua スクリプト
       │           └── pasta/shiori/ # SHIORI エントリーポイント
       └── shell/
           └── master/         # シェル（見た目）
               ├── descript.txt    # シェル設定
               ├── surfaces.txt    # サーフェス定義
               ├── surface0.png    # sakura 通常
               ├── surface1.png    # sakura 笑顔
               └── ...
   ```
2. The alpha04-sample-ghost shall 各ファイルのテンプレート内容を定義する
3. The alpha04-sample-ghost shall install.txt に適切なインストール設定を定義する
4. The alpha04-sample-ghost shall `examples/sample-ghost/hello-pasta/` に配置される

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
2. The 反応 shall クリックしたキャラクター（sakura/kero）によって異なる
3. The 反応 shall Reference0（スコープ）を解析してキャラクターを判定する

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
2. The 時報 shall 現在時刻（`req.date.hour`）を含める
3. The 時報 shall 時間帯に応じたバリエーション（朝/昼/夕/夜）を持つ

---

### Requirement 6: シェル素材

**Objective:** As a 配布担当者, I want 著作権問題のないシェル素材がほしい, so that 自由に配布できる

#### Acceptance Criteria

1. The alpha04-sample-ghost shall 以下の仕様でシェル素材を作成する:
   - **サイズ**: 幅 96〜128 × 高さ 256 ピクセル（3頭身バランス）
   - **形式**: 透過 PNG
   - **キャラクター**: 2体
     - 女の子（sakura）: surface0-8（9種）
     - 男の子（kero）: surface10-18（9種）
2. The 表情 shall ピクトグラム風の記号表現とする:
   - `^ ^` 笑顔, `- -` 通常, `> <` 照れ, `o o` 驚き, `; ;` 泣き, `@ @` 困惑, `* *` キラキラ, `= =` 眠い, `# #` 怒り
3. The シェル shall descript.txt でサーフェス定義を行う

---

### Requirement 7: 設定ファイル

**Objective:** As a ゴースト開発者, I want pasta.toml で基本設定を定義したい, so that ゴーストの動作をカスタマイズできる

#### Acceptance Criteria

1. The alpha04-sample-ghost shall `pasta.toml` に以下を定義する:
   - `[ghost]` セクション: ゴースト名、作者、バージョン
   - `[ghost.talk]` セクション: トーク間隔設定
   - `[shiori]` セクション: SHIORI 設定
2. The 設定 shall alpha02（仮想イベント）で読み込まれる

---

### Requirement 8: テスト要件

**Objective:** As a 開発者, I want サンプルゴーストの動作テストを実行したい, so that 品質を保証できる

#### Acceptance Criteria

1. The alpha04-sample-ghost shall 各イベントハンドラの動作を検証する統合テストを提供する
2. The テスト shall さくらスクリプト出力の正確性を検証する
3. The テスト shall pasta.toml 設定の読み込みを検証する

---

## Out of Scope

- 高度な会話ロジック（コンテキスト保持、学習等）
- SAORI/MAKOTO 連携
- ネットワーク更新機能
- 複雑なシェルアニメーション
- 手動でのシェル素材準備（自動生成する）

---

## Discussion Items

### 議題 #1: Ghost配置場所とID ✅ 解決済み

**決定事項:**
- 配置場所: `crates/pasta_sample_ghost/`（専用クレート）
- ゴーストID: `hello-pasta`
- 責務: pastaエンジンの利用例・動作検証（pasta_luaから完全独立）

### 議題 #2: Shell素材準備 ✅ 解決済み

**決定事項:**
- **Option D: プロジェクト内で自動生成**
- Rustで専用クレート作成（`pasta_shell_generator` 等）
- プログラマティックにシェル画像を生成
- 依存: `image`, `imageproc` 等の画像処理クレート
- 生成内容:
  - surface0.png (sakura側デフォルト)
  - surface10.png (kero側デフォルト、透明可)
  - 最小限の当たり判定用のシンプルな図形

**利点:**
- ビルドプロセスに統合可能
- CIで再現可能
- 外部依存なし
- プレースホルダとして十分

### 議題 #3: 設定ファイル記載内容の確定 ✅ 解決済み

**既に確定している項目（要件より）:**
- `sakura.name`: **女の子**
- `kero.name`: **男の子**

**決定事項:**
- `craftman`: `ekicyou`
- `craftmanw`: `どっとステーション駅長`
- `homeurl`: `https://github.com/ekicyou/pasta`
- バルーンoffset: 自動生成画像サイズに基づき後で調整

**参照:** [ukadoc設定ファイル仕様書](research/ukadoc-config-spec.md)

### 議題 #4: サーフェス構成の最小仕様 ✅ 要件で確定済み

**確定事項（Requirement 6より）:**
- 女の子（sakura）: surface0-8（9種）
- 男の子（kero）: surface10-18（9種）
- 表情仕様: ピクトグラム風記号表現（`^ ^`, `- -`, `> <`, `o o`, `; ;`, `@ @`, `* *`, `= =`, `# #`）
- サイズ: 96〜128 × 256ピクセル（3頭身バランス）
- 形式: 透過PNG

### 議題 #5: pasta.toml 設定項目の詳細仕様 ✅ 解決済み

**調査結果:** [pasta.toml設定仕様書](research/pasta-toml-spec.md) を参照

**決定事項:**

```toml
[package]
name = "hello-pasta"
version = "0.1.0"
authors = ["どっとステーション駅長"]

[loader]
debug_mode = true

[ghost]
spot_switch_newlines = 1.5
talk_interval_min = 60   # 1分（テスト用に短縮）
talk_interval_max = 120  # 2分（テスト用に短縮）
hour_margin = 30
```

**採用構成:** 案B（必要最小限 + ゴースト動作設定）  
**[package] セクション:** 正式導入（Cargo.toml準拠）  
**参照:** [pasta.toml設定仕様書](research/pasta-toml-spec.md)

### 議題 #6: テスト要件の実装範囲 ✅ 解決済み

**決定事項:**

専用クレート構成を採用し、責務を明確に分離：

```
crates/pasta_sample_ghost/
├─ Cargo.toml
├─ README.md
├─ src/
│  └─ lib.rs (シェル画像生成ロジック)
├─ tests/
│  └─ integration_test.rs (統合テスト)
├─ ghost/
│  └─ master/
│     ├─ descript.txt
│     └─ dic/ (*.pasta)
├─ shell/
│  └─ master/
│     ├─ descript.txt
│     └─ surface*.png (build時生成)
├─ install.txt
└─ pasta.toml
```

| 項目 | 決定内容 | 理由 |
|------|---------|------|
| **クレート配置** | `crates/pasta_sample_ghost/` | pasta_luaから責務分離、ルート汚染回避 |
| **ゴースト配置** | クレート内（`ghost/`, `shell/`等） | 完全な独立性、配布物として完結 |
| **テスト配置** | `crates/pasta_sample_ghost/tests/` | クレート標準構成 |
| **テスト環境** | 統合テスト（PastaLoader使用） | 実SSP不要、CI実行可能 |
| **画像生成** | `src/lib.rs` + build時生成 | 再現性・自動化 |
| **CIでの実行** | 有効（`cargo test --workspace`） | リグレッション検出 |

**責務定義:**
- pasta_sample_ghost: pastaエンジンの**利用例**および**動作検証**
- pasta_lua: エンジン本体（サンプルゴーストへの依存なし）

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
