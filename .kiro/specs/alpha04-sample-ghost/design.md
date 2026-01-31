# Technical Design: alpha04-sample-ghost

## Overview

### 機能概要

hello-pasta ゴーストは、pasta システムの入門者向けサンプルとして、SHIORI/3.0 プロトコルで動作するミニマルなゴーストを提供する。画像生成を含む自己完結型ゴーストとして、インストール直後から動作可能な状態を実現する。

### 技術的アプローチ

```mermaid
flowchart TB
    subgraph Crate["pasta_sample_ghost"]
        direction TB
        GEN[Image Generator]
        TOML[Config Templates]
        UKADOC[ukadoc Files]
    end
    
    subgraph Output["ghosts/hello-pasta/"]
        direction TB
        INSTALL[install.txt]
        subgraph Ghost["ghost/master/"]
            DESCRIPT[descript.txt]
            PASTA_TOML[pasta.toml]
            SCRIPTS[*.pasta files]
        end
        subgraph Shell["shell/master/"]
            SHELL_DESC[descript.txt]
            SURFACES[surfaces.txt]
            PNG[surface*.png]
        end
    end
    
    GEN -->|generate| PNG
    TOML -->|template| PASTA_TOML
    UKADOC -->|generate| INSTALL
    UKADOC -->|generate| DESCRIPT
    UKADOC -->|generate| SHELL_DESC
    UKADOC -->|generate| SURFACES
```

### ユーザーストーリーマッピング

| ユーザーストーリー | 実現方法 |
|-------------------|---------|
| pasta入門者がすぐに動くゴーストを体験 | 自己完結型ディストリビューション |
| DSL学習のリファレンス取得 | イベント別に分離されたスクリプト |
| カスタマイズの出発点 | コメント付きの pasta.toml テンプレート |

---

## Architecture

### 高レベルアーキテクチャ

```mermaid
flowchart LR
    subgraph Build["ビルド時"]
        TEST[Integration Tests]
        GEN[Image Generation]
    end
    
    subgraph Distribution["配布物"]
        GHOST[hello-pasta/]
    end
    
    subgraph Runtime["実行時"]
        SSP[SSP / 伺か]
        SHIORI[pasta_shiori.dll]
        LUA[pasta_lua runtime]
    end
    
    TEST -->|cargo test| GEN
    GEN -->|output| GHOST
    GHOST -->|install| SSP
    SSP -->|SHIORI/3.0| SHIORI
    SHIORI -->|execute| LUA
```

### クレート構成

```
crates/pasta_sample_ghost/
├── Cargo.toml              # 依存: image 0.25, imageproc 0.25, pasta_lua (dev)
├── README.md               # クレートドキュメント
├── src/
│   ├── lib.rs              # 公開API: generate_ghost()
│   ├── image_generator.rs  # ピクトグラム画像生成
│   ├── config_templates.rs # pasta.toml/ukadocテンプレート
│   └── scripts.rs          # pastaスクリプトテンプレート（文字列定数）
├── tests/
│   └── integration_test.rs # 統合テスト
└── ghosts/                 # 配布物テンプレート（テスト時に生成）
    └── hello-pasta/        # ゴーストID
        ├── install.txt
        ├── ghost/master/
        │   ├── descript.txt
        │   ├── pasta.toml
        │   ├── pasta.dll       # SHIORI DLL（テスト時コピー）
        │   └── dic/        # Pasta DSLスクリプト（実行時ロード）
        │       ├── boot.pasta
        │       ├── talk.pasta
        │       └── click.pasta
        └── shell/master/
            ├── descript.txt
            ├── surfaces.txt
            └── surface*.png
```

**配布物の生成方針**:
- テスト実行時に `ghosts/hello-pasta/` 配下へファイル生成
- 画像ファイル（surface*.png）は `image_generator.rs` で動的生成
- 設定ファイル（*.txt, pasta.toml）は `config_templates.rs` で生成
- pastaスクリプト（*.pasta）は `scripts.rs` の文字列定数から出力
- SHIORI DLL（pasta.dll）はテストヘルパー `copy_pasta_shiori_dll()` でコピー

### 依存関係

| 依存 | バージョン | 用途 | 種別 |
|-----|-----------|------|------|
| `image` | 0.25 | PNG画像生成 | runtime |
| `imageproc` | 0.25 | 図形描画（円、楕円） | runtime |
| `pasta_lua` | workspace | テスト用トランスパイラ | dev-dependencies |
| `tempfile` | 3.x | テスト用一時ディレクトリ | dev-dependencies |

---

## System Flows

### シーケンス: ゴースト生成フロー

```mermaid
sequenceDiagram
    participant Test as Integration Test
    participant Gen as ImageGenerator
    participant Cfg as ConfigTemplates
    participant FS as FileSystem
    
    Test->>FS: create TempDir
    Test->>Gen: generate_surfaces(path)
    Gen->>Gen: draw_pictogram(sakura)
    Gen->>Gen: draw_pictogram(kero)
    Gen->>FS: write surface0.png
    Gen->>FS: write surface10.png
    Test->>Cfg: generate_configs(path)
    Cfg->>FS: write install.txt
    Cfg->>FS: write ghost/master/descript.txt
    Cfg->>FS: write shell/master/descript.txt
    Cfg->>FS: write shell/master/surfaces.txt
    Test->>FS: copy pasta scripts
    Test->>Test: verify structure
```

### シーケンス: イベント処理フロー

```mermaid
sequenceDiagram
    participant SSP as SSP
    participant SHIORI as pasta_shiori
    participant Lua as pasta_lua
    participant Script as *.pasta
    
    SSP->>SHIORI: OnBoot event
    SHIORI->>Lua: dispatch_event("OnBoot")
    Lua->>Script: load boot.pasta
    Script->>Script: select random talk
    Script-->>Lua: Sakuraトーク
    Lua-->>SHIORI: response
    SHIORI-->>SSP: Value: \0\s[0]...
```

---

## Requirements Traceability

| Req ID | 要件 | 設計要素 | 検証方法 |
|--------|-----|---------|---------|
| REQ-001 | ディレクトリ構造 | `config_templates.rs::generate_structure()` | `test_directory_structure()` |
| REQ-002 | 起動/終了トーク | `scripts.rs` (boot.pasta) | `test_boot_close_events()` |
| REQ-003 | ダブルクリック | `scripts.rs` (click.pasta) | `test_doubleclick_event()` |
| REQ-004 | ランダムトーク | `scripts.rs` (talk.pasta) | `test_random_talk()` |
| REQ-005 | 時報 | `scripts.rs` (talk.pasta OnHour) | `test_hourly_chime()` |
| REQ-006 | シェル画像 | `image_generator.rs` | `test_shell_images()` |
| REQ-007 | pasta.toml | `config_templates.rs::pasta_toml()` | `test_pasta_toml()` |
| REQ-008 | 結合テスト | `tests/integration_test.rs` | CI成功 |
| REQ-009 | ukadoc準拠 | `config_templates.rs::ukadoc_*()` | `test_ukadoc_compliance()` |

---

## Components

### Component 1: ImageGenerator

**責務**: ピクトグラムスタイルのシェル画像生成

**インターフェース**:

```rust
/// シェル画像を生成し、指定パスに出力
pub fn generate_surfaces(output_dir: &Path) -> Result<(), ImageError>;

/// 個別サーフェス生成（テスト用）
pub fn generate_surface(
    width: u32,
    height: u32,
    character: Character,
) -> RgbaImage;
```

**設計詳細**:

```mermaid
flowchart TB
    subgraph ImageGenerator
        GEN[generate_surfaces]
        DRAW[draw_pictogram]
        SAVE[save_png]
    end
    
    subgraph Output
        S0[surface0.png - sakura]
        S10[surface10.png - kero]
    end
    
    GEN --> DRAW
    DRAW --> SAVE
    SAVE --> S0
    SAVE --> S10
```

**ピクトグラム仕様**:

| 要素 | sakura (surface0) | kero (surface10) |
|-----|-------------------|------------------|
| サイズ | 256x512 px | 256x512 px |
| 頭部 | 円（半径40px） | 円（半径40px） |
| 胴体 | 台形 | 台形 |
| 装飾 | なし | 三角形の耳 |
| 色 | ライトブルー (#4A90D9) | ライトグリーン (#4AD98A) |
| 背景 | 透明 | 透明 |

**実装方針**:

```rust
use image::{RgbaImage, Rgba};
use imageproc::drawing::{draw_filled_circle_mut, draw_polygon_mut};

pub fn draw_pictogram(img: &mut RgbaImage, character: Character) {
    let color = match character {
        Character::Sakura => Rgba([74, 144, 217, 255]),
        Character::Kero => Rgba([74, 217, 138, 255]),
    };
    
    // 頭部（円）
    draw_filled_circle_mut(img, (128, 80), 40, color);
    
    // 胴体（台形 - ポリゴンとして描画）
    let body = [
        Point::new(88, 150),
        Point::new(168, 150),
        Point::new(180, 350),
        Point::new(76, 350),
    ];
    draw_polygon_mut(img, &body, color);
    
    // keroの場合は耳を追加
    if matches!(character, Character::Kero) {
        draw_ear(img, color);
    }
}
```

---

### Component 2: ConfigTemplates

**責務**: ukadoc準拠の設定ファイル生成

**インターフェース**:

```rust
/// ゴースト配布構造全体を生成
pub fn generate_ghost(output_dir: &Path, config: &GhostConfig) -> Result<()>;

/// 個別ファイル生成
pub fn generate_install_txt(config: &GhostConfig) -> String;
pub fn generate_ghost_descript(config: &GhostConfig) -> String;
pub fn generate_shell_descript(config: &GhostConfig) -> String;
pub fn generate_surfaces_txt() -> String;
pub fn generate_pasta_toml(config: &GhostConfig) -> String;
```

**GhostConfig構造**:

```rust
pub struct GhostConfig {
    pub name: String,           // "hello-pasta"
    pub sakura_name: String,    // "Pasta"
    pub kero_name: String,      // "Lua"
    pub craftman: String,       // "ekicyou"
    pub craftman_w: String,     // "どっとステーション駅長"
    pub shiori: String,         // "pasta.dll"
    pub homeurl: String,        // "https://github.com/ekicyou/pasta"
}
```

**ghost/master/descript.txt テンプレート**:

```
charset,UTF-8
type,ghost
name,{name}
sakura.name,{sakura_name}
kero.name,{kero_name}
craftman,{craftman}
craftmanw,{craftman_w}
shiori,{shiori}
homeurl,{homeurl}
```

**生成例**:

```
charset,UTF-8
type,ghost
name,hello-pasta
sakura.name,Pasta
kero.name,Lua
craftman,ekicyou
craftmanw,どっとステーション駅長
shiori,pasta.dll
homeurl,https://github.com/ekicyou/pasta
```

**ファイル生成フロー**:

```mermaid
flowchart TD
    CFG[GhostConfig] --> GEN[generate_ghost]
    GEN --> INSTALL[install.txt]
    GEN --> GHOST_DESC[ghost/master/descript.txt]
    GEN --> SHELL_DESC[shell/master/descript.txt]
    GEN --> SURFACES[shell/master/surfaces.txt]
    GEN --> PASTA[ghost/master/pasta.toml]
```

---

### Component 3: Scripts（Pasta DSLスクリプト）

**責務**: サンプルイベントハンドラの提供

**配置**: `ghosts/hello-pasta/ghost/master/dic/*.pasta`（要件定義で確定済み）

#### pasta DSL とイベントシステムの連携

pasta DSL は行指向の宣言的言語であり、SHIORI イベントハンドラを直接定義する構文を持ちませんわ。代わりに、**シーン関数フォールバック**機能を活用して、pasta DSL のみでイベント処理を記述できますの。

```mermaid
sequenceDiagram
    participant SSP as SSP/ベースウェア
    participant SHIORI as pasta_shiori
    participant EVENT as EVENT.fire()
    participant REG as REG テーブル
    participant SCENE as SCENE.search()
    participant DSL as *.pasta シーン

    SSP->>SHIORI: OnBoot イベント
    SHIORI->>EVENT: fire({id:"OnBoot"})
    EVENT->>REG: REG.OnBoot 検索
    REG-->>EVENT: nil（未登録）
    EVENT->>SCENE: search("OnBoot")
    SCENE-->>EVENT: シーン関数発見
    EVENT->>DSL: シーン実行
    DSL-->>EVENT: アクション実行
    EVENT-->>SHIORI: レスポンス
    SHIORI-->>SSP: Value: \0\s[0]...
```

**イベント処理の流れ**:

1. **ベースウェア** が SHIORI イベント（OnBoot, OnClose 等）を送信
2. **EVENT.fire()** が `REG.イベント名` でハンドラを検索
3. **未登録の場合**、`SCENE.search(req.id)` でシーン名を前方一致検索
4. **シーン発見時**、そのシーン関数を実行
5. アクター発言がさくらスクリプトに変換され、レスポンス返却

**pasta DSL の基本構文**:

| マーカー | 意味 | 例 |
|---------|------|-----|
| `＃` | コメント | `＃ これはコメント` |
| `％` | アクター辞書定義 | `％Pasta` |
| `＠` | 単語定義/表情指定 | `＠挨拶：こんにちは、やあ` （値は読点区切り） |
| `＊` | グローバルシーン定義 | `＊OnBoot` |
| `・` | ローカルシーン定義 | `・挨拶` |
| `アクター：` | アクション行 | `Pasta：こんにちは！` |
| `＄` | 変数参照/代入 | `＄カウンタ＝１０` |
| `＞` | シーン呼び出し | `＞挨拶` |

**ファイル構成**:

| ファイル | イベント | 内容 |
|---------|---------|------|
| `boot.pasta` | OnFirstBoot, OnBoot, OnClose | 起動/終了トーク |
| `talk.pasta` | OnTalk, OnHour | ランダムトーク、時報 |
| `click.pasta` | OnMouseDoubleClick | ダブルクリック反応 |

**生成方法**: `config_templates.rs` からテンプレート文字列として出力

**boot.pasta 設計**:

```pasta
＃ boot.pasta - 起動/終了イベント用シーン定義
＃ pasta DSL では「シーン関数フォールバック」機能を利用
＃ シーン名とSHIORIイベント名を一致させることで、自動ディスパッチされる

＃ アクター辞書（このファイルで使用するアクター）
％Pasta
　＠通常：\s[0]
　＠笑顔：\s[1]
　＠眠い：\s[2]

％Lua
　＠通常：\s[10]
　＠元気：\s[11]

＃ グローバル単語定義（ランダム選択用）
＠起動挨拶：おはよう！今日も頑張ろう！、やあ、また会えたね。、起動完了。準備OKだよ。
＠終了挨拶：またね！、お疲れ様！、See you!

＃ OnBoot イベント - 通常起動時（シーン関数フォールバックで呼び出し）
＊OnBoot
　Pasta：＠笑顔　＠起動挨拶
　Lua　：＠元気　よろしくやで！

＃ OnBoot イベント - 別パターン（同名シーンでランダム選択）
＊OnBoot
　Pasta：＠通常　起動したよー。
　Lua　：＠通常　さあ、始めよか。

＃ OnFirstBoot イベント - 初回起動時
＊OnFirstBoot
　Pasta：＠笑顔　初めまして！\n私は Pasta、よろしくね。
　Lua　：＠元気　ワイは Lua や！一緒に頑張ろな！

＃ OnClose イベント - 終了時
＊OnClose
　Pasta：＠通常　＠終了挨拶
　Lua　：＠通常　ほな、また！
```

**シーン関数フォールバックの仕組み**:
- SHIORI イベント発生時、`REG.イベント名` ハンドラを検索
- 未登録の場合、`SCENE.search(req.id)` でシーン名と一致するシーンを検索
- 見つかった場合、そのシーン関数を実行
- これにより pasta DSL のみでイベントハンドラを記述可能

**talk.pasta 設計**:

```pasta
＃ talk.pasta - ランダムトーク/時報用シーン定義
＃ OnSecondChange (毎秒) → 仮想イベントディスパッチャ → ランダムトーク/時報

＃ アクター辞書
％Pasta
　＠通常：\s[0]
　＠笑顔：\s[1]
　＠眠い：\s[2]
　＠考え：\s[3]

％Lua
　＠通常：\s[10]
　＠元気：\s[11]

＃ ランダムトーク用単語（ランダム選択）
＠雑談：何か用？、暇だなあ...、ねえねえ、聞いてる？、うーん、眠くなってきた...

＃ ランダムトーク - 仮想イベント OnAITalk（ベースウェア設定による）
＊OnAITalk
　Pasta：＠通常　＠雑談

＊OnAITalk
　Pasta：＠笑顔　Pasta DSL、使ってみてね。
　Lua　：＠元気　Lua 側も触ってみてや！

＊OnAITalk
　Pasta：＠眠い　ふぁ〜あ...
　Lua　：＠通常　寝るなや。

＊OnAITalk
　Pasta：＠考え　今日は何しようかな...
　Lua　：＠通常　ワイに任せとき！

＃ 時報 - 毎時0分に発生
＊OnHour
　Pasta：＠笑顔　今＄時だよ！\n＠時報メッセージ
　Lua　：＠通常　時間の確認は大事やで。

＃ 時報メッセージ（時間帯別に追加可能）
＠時報メッセージ：お昼ごはん食べた？、おやつの時間かも！、そろそろ休憩しよう。
```

**仮想イベントディスパッチ**:
- `OnSecondChange` イベントで仮想イベントディスパッチャが動作
- `OnAITalk`（ランダムトーク）、`OnHour`（時報）等を時間条件で発火
- pasta DSL 側はシーン名を一致させるだけで連携可能

**click.pasta 設計**:

```pasta
＃ click.pasta - クリックイベント用シーン定義
＃ OnMouseDoubleClick イベントをシーン関数フォールバックで処理

＃ アクター辞書
％Pasta
　＠通常：\s[0]
　＠驚き：\s[2]
　＠照れ：\s[1]

％Lua
　＠通常：\s[10]
　＠元気：\s[11]

＃ ダブルクリック反応の単語定義
＠Pasta反応：なになに？、はいはい。、ダブルクリックされた！
＠Lua反応：呼んだ？、ん？、なんや？

＃ OnMouseDoubleClick イベント - Pasta側クリック
＃ Reference0 = 0 で Pasta（\0側）がクリックされた場合
＊OnMouseDoubleClick
　Pasta：＠驚き　＠Pasta反応

＊OnMouseDoubleClick
　Pasta：＠照れ　え、なに？
　Lua　：＠通常　こっちに用があるんちゃうん？

＃ 注意: Reference による分岐は alpha04 時点では未実装
＃ 将来的には条件分岐（＄０ == "0"）や属性（＆where:0）で分岐可能予定
```

**現在の制約と将来の拡張**:
- alpha04 時点では Reference 値による条件分岐は pasta DSL に未実装
- Lua ハンドラで req.reference[0] を判定する高度なパターンも可能
- 将来の拡張で属性ベースの条件分岐 `＆where:0` 等を検討

---

## Data Models

### ディレクトリ構造モデル

**配布物構造**（`crates/pasta_sample_ghost/ghosts/hello-pasta/`）:

```
ghosts/hello-pasta/
├── install.txt                     # REQ-009
├── ghost/
│   └── master/
│       ├── descript.txt            # REQ-009
│       ├── pasta.toml              # REQ-007
│       ├── pasta.dll               # REQ-008 (SHIORI DLL - テスト時コピー)
│       └── dic/                    # Pasta DSLスクリプト配置ディレクトリ
│           ├── boot.pasta          # REQ-002
│           ├── talk.pasta          # REQ-004, REQ-005
│           └── click.pasta         # REQ-003
└── shell/
    └── master/
        ├── descript.txt            # REQ-009
        ├── surfaces.txt            # REQ-009
        ├── surface0.png            # REQ-006 (sakura)
        └── surface10.png           # REQ-006 (kero)
```

**配置ルール**:
- 基準パス: `crates/pasta_sample_ghost/ghosts/`
- 配布時: ZIP圧縮して `hello-pasta.zip` として配布
- テスト時: この構造を TempDir に再現して検証

### pasta.toml 設定モデル

```toml
# hello-pasta ゴースト設定ファイル
# pasta alpha04 サンプル

[loader]
# スクリプトファイルパターン
patterns = ["**/*.pasta"]
# 起動時自動ロード
auto_load = true

[logging]
# ログレベル: off, error, warn, info, debug, trace
level = "info"
# ログ出力先（ベースシェルのログフォルダ）
output = "log"

[persistence]
# 永続化ファイル名
filename = "save.lua"
# 自動保存間隔（秒）- OnCloseでも保存
auto_save_interval = 300

[lua]
# メモリ制限（MB）- 0で無制限
memory_limit = 128
# 追加モジュール検索パス
module_path = ["./scripts", "./lib"]

# [package] セクション（将来の拡張用サンプル）
# 議題 #1 で決定: 省略可能だが将来的な拡張サンプルとして含める
[package]
name = "hello-pasta"
version = "1.0.0"
authors = ["pasta-team"]
description = "pasta入門用サンプルゴースト"
```

---

## Error Handling

### エラー種別

| エラー | 発生箇所 | 処理方法 |
|--------|---------|---------|
| `ImageError` | 画像生成 | `Result` で伝播、テスト失敗 |
| `IoError` | ファイル書き込み | `Result` で伝播 |
| `TemplateError` | 設定生成 | コンパイル時検証（定数埋め込み） |

### エラー型定義

```rust
#[derive(Debug, thiserror::Error)]
pub enum SampleGhostError {
    #[error("画像生成エラー: {0}")]
    Image(#[from] image::ImageError),
    
    #[error("IO エラー: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("ディレクトリ作成失敗: {path}")]
    DirectoryCreation { path: PathBuf },
}
```

---

## Testing Strategy

### テスト環境構成

```mermaid
flowchart TB
    subgraph TestEnv["テスト環境"]
        TEMP[TempDir]
        COPY[pasta_lua scripts/scriptlibs コピー]
        GEN[ゴースト生成]
    end
    
    subgraph Verify["検証"]
        STRUCT[構造検証]
        IMAGE[画像検証]
        SCRIPT[スクリプト検証]
        E2E[E2Eイベント検証]
    end
    
    TEMP --> COPY
    COPY --> GEN
    GEN --> STRUCT
    GEN --> IMAGE
    GEN --> SCRIPT
    SCRIPT --> E2E
```

### テストケース一覧

| テスト | 対象 | 検証内容 |
|--------|-----|---------|
| `test_directory_structure` | REQ-001 | 必須ディレクトリ/ファイル存在 |
| `test_boot_close_events` | REQ-002 | OnBoot/OnClose トーク出力 |
| `test_doubleclick_event` | REQ-003 | OnMouseDoubleClick 反応 |
| `test_random_talk` | REQ-004 | OnTalk ランダム選択 |
| `test_hourly_chime` | REQ-005 | OnHour 時報出力 |
| `test_shell_images` | REQ-006 | PNG 存在/サイズ/透過背景 |
| `test_pasta_toml` | REQ-007 | 設定パース可能/キー存在 |
| `test_actor_dictionary` | REQ-002/003/004 | アクター辞書定義→表情参照→さくらスクリプト変換 |
| `test_ukadoc_compliance` | REQ-009 | 必須フィールド存在 |

**test_actor_dictionary 検証内容**:
- アクター辞書定義（`％Pasta`, `％Lua`）のパース成功
- 表情指定（`＠笑顔：\s[1]`）の登録確認
- アクション行での表情参照（`Pasta：＠笑顔　こんにちは`）
- さくらスクリプトへの正しい変換（`\s[1]こんにちは`）
- alpha01-shiori-alpha-events との統合確認

### テストパターン（TempDir + コピー方式）

```rust
// 議題 #3 で決定: 既存パターン踏襲
// 議題 #5 で決定: ヘルパー関数による堅牢化
// 議題 #6 で決定: pasta.dll 自動コピー（オプションA）

// tests/common/mod.rs - テスト用ヘルパー

/// pasta_shiori DLL をコピー（テスト前提: リリースビルド済み）
pub fn copy_pasta_shiori_dll(dest_ghost_master: &Path) -> Result<(), std::io::Error> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "ワークスペースルートが見つかりません"
        ))?;
    
    #[cfg(target_os = "windows")]
    let dll_src = workspace_root
        .join("target/i686-pc-windows-msvc/release/pasta_shiori.dll");
    
    #[cfg(target_os = "linux")]
    let dll_src = workspace_root
        .join("target/release/libpasta_shiori.so");
    
    if !dll_src.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "pasta_shiori DLL not found: {:?}\n\n\
                テスト前に以下を実行してください:\n\
                cargo build --release --target i686-pc-windows-msvc -p pasta_shiori",
                dll_src
            )
        ));
    }
    
    #[cfg(target_os = "windows")]
    let dll_dest = dest_ghost_master.join("pasta.dll");
    
    #[cfg(target_os = "linux")]
    let dll_dest = dest_ghost_master.join("pasta.so");
    
    std::fs::copy(&dll_src, &dll_dest)?;
    Ok(())
}

pub fn copy_pasta_lua_runtime(dest_ghost_master: &Path) -> Result<(), std::io::Error> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "ワークスペースルートが見つかりません"
        ))?;
    
    let scripts_src = workspace_root.join("crates/pasta_lua/scripts");
    let scriptlibs_src = workspace_root.join("crates/pasta_lua/scriptlibs");
    
    if !scripts_src.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("pasta_lua scripts not found: {:?}", scripts_src)
        ));
    }
    
    copy_dir_all(&scripts_src, &dest_ghost_master.join("scripts"))?;
    copy_dir_all(&scriptlibs_src, &dest_ghost_master.join("scriptlibs"))?;
    
    Ok(())
}

// tests/integration_test.rs - 統合テスト
#[test]
fn test_sample_ghost_generation() {
    let temp = tempfile::tempdir().unwrap();
    let ghost_root = temp.path().join("hello-pasta");
    let ghost_master = ghost_root.join("ghost/master");
    
    // pasta_shiori DLL をコピー（議題 #6: オプションA）
    common::copy_pasta_shiori_dll(&ghost_master)
        .expect("pasta_shiori.dll のコピーに失敗。テスト前にビルドしてください。");
    
    // pasta_lua ランタイムをコピー（ヘルパー使用）
    common::copy_pasta_lua_runtime(&ghost_master).unwrap();
    
    // ゴースト生成（画像、設定ファイル、pastaスクリプト）
    pasta_sample_ghost::generate_ghost(&ghost_root, &default_config()).unwrap();
    
    // 検証
    assert!(ghost_root.join("install.txt").exists());
    assert!(ghost_root.join("ghost/master/descript.txt").exists());
    assert!(ghost_root.join("ghost/master/pasta.toml").exists());
    assert!(ghost_root.join("ghost/master/pasta.dll").exists()); // DLL存在確認
    assert!(ghost_root.join("ghost/master/dic/boot.pasta").exists());
    assert!(ghost_root.join("shell/master/surface0.png").exists());
    assert!(ghost_root.join("shell/master/surface10.png").exists());
}
```

**パス解決の信頼性向上**:
- ヘルパー関数 `copy_pasta_lua_runtime()` でエラーハンドリング明示
- ワークスペースルート取得失敗時に具体的なエラーメッセージ
- `scripts/scriptlibs` の存在確認を実施
- CI環境での再現性を保証

### CI/CD統合

```yaml
# .github/workflows/test.yml（既存に追加）
- name: Test sample ghost
  run: cargo test -p pasta_sample_ghost --all-features
```

---

## 付録: 議題決定事項

### 議題 #1: [package] セクション仕様

- **決定**: 省略可能だが、将来的な拡張用サンプルとして含める
- **根拠**: 入門者がパッケージメタデータの書き方を学べる
- **コミット**: `e462fb1`

### 議題 #2: 画像生成クレート選定

- **決定**: `image` 0.25 + `imageproc` 0.25
- **根拠**: フォント依存なし、トイレマーク風ピクトグラムで十分
- **コミット**: `9617a76`

### 議題 #3: テスト環境構築

- **決定**: TempDir + コピー方式（既存 pasta_lua パターン踏襲）
- **根拠**: 実績あるパターン、依存関係の明確化
- **コミット**: `595d868`
