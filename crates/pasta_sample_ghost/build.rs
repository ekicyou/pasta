//! Build script for pasta_sample_ghost
//!
//! ビルド時に ghosts/hello-pasta/ ディレクトリにサンプルゴーストを生成します。

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // ビルドスクリプト再実行トリガー
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=src/");

    // クレートルートを取得
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let crate_root = Path::new(&manifest_dir);
    let ghosts_dir = crate_root.join("ghosts").join("hello-pasta");

    // ゴーストディレクトリ生成
    if let Err(e) = generate_ghost_files(&ghosts_dir) {
        eprintln!("Warning: Failed to generate ghost files: {}", e);
        // ビルドを失敗させない（オプショナル生成）
    }
}

fn generate_ghost_files(output_dir: &Path) -> std::io::Result<()> {
    // ディレクトリ構造を作成
    fs::create_dir_all(output_dir.join("ghost/master/dic"))?;
    fs::create_dir_all(output_dir.join("shell/master"))?;

    // install.txt
    fs::write(
        output_dir.join("install.txt"),
        r#"charset,UTF-8
type,ghost
directory,hello-pasta
name,hello-pasta
"#,
    )?;

    // readme.txt
    fs::write(
        output_dir.join("readme.txt"),
        r#"# hello-pasta

pasta システムのサンプルゴーストです。
pasta DSL の学習用として使用できます。

## 機能
- 起動・終了時の挨拶
- ダブルクリック反応
- ランダムトーク
- 時報

## ライセンス
Apache-2.0 OR MIT

## 作者
どっとステーション駅長 (ekicyou)
https://github.com/ekicyou/pasta
"#,
    )?;

    // ghost/master/descript.txt
    fs::write(
        output_dir.join("ghost/master/descript.txt"),
        r#"charset,UTF-8
type,ghost
shiori,pasta.dll
name,hello-pasta
craftman,ekicyou
craftmanw,どっとステーション駅長
sakura.name,女の子
kero.name,男の子
homeurl,https://github.com/ekicyou/pasta
"#,
    )?;

    // ghost/master/pasta.toml
    fs::write(
        output_dir.join("ghost/master/pasta.toml"),
        r#"# pasta.toml - pasta ゴースト設定ファイル
#
# このファイルは pasta システムの動作を設定します。
# 必須項目は [package] と [loader] のみです。

[package]
# 基本情報（descript.txt と同期推奨）
name = "hello-pasta"
version = "1.0.0"
edition = "2024"  # pasta DSL エディション

[loader]
# 読み込み設定
pasta_patterns = ["dic/*.pasta"]  # DSLファイルのパターン

# Lua モジュール検索パス（優先順位順）
# デフォルト: ["profile/pasta/save/lua", "scripts", "profile/pasta/cache/lua", "scriptlibs"]
lua_search_paths = [
    "profile/pasta/save/lua",   # ユーザー保存スクリプト
    "scripts",                   # pasta 標準ランタイム（pasta_lua/scripts/をコピー）
    "profile/pasta/cache/lua",   # トランスパイル済みキャッシュ
    "scriptlibs",                # 追加ライブラリ
]

# トランスパイル出力ディレクトリ
transpiled_output_dir = "profile/pasta/cache/lua"

[ghost]
# ゴースト固有設定（オプション）
random_talk_interval = 180  # ランダムトーク間隔（秒）
"#,
    )?;

    // shell/master/descript.txt
    fs::write(
        output_dir.join("shell/master/descript.txt"),
        r#"charset,UTF-8
name,master
type,shell
craftman,ekicyou
craftmanw,どっとステーション駅長
seriko.use_self_alpha,1
sakura.balloon.offsetx,64
sakura.balloon.offsety,0
kero.balloon.offsetx,64
kero.balloon.offsety,0
sakura.surface.alias,表,surface0
sakura.surface.alias,笑,surface1
sakura.surface.alias,照,surface2
sakura.surface.alias,驚,surface3
sakura.surface.alias,泣,surface4
sakura.surface.alias,困,surface5
sakura.surface.alias,輝,surface6
sakura.surface.alias,眠,surface7
sakura.surface.alias,怒,surface8
kero.surface.alias,表,surface10
kero.surface.alias,笑,surface11
kero.surface.alias,照,surface12
kero.surface.alias,驚,surface13
kero.surface.alias,泣,surface14
kero.surface.alias,困,surface15
kero.surface.alias,輝,surface16
kero.surface.alias,眠,surface17
kero.surface.alias,怒,surface18
"#,
    )?;

    // shell/master/surfaces.txt
    fs::write(
        output_dir.join("shell/master/surfaces.txt"),
        r#"charset,UTF-8
descript
{
    version,1
}

// サーフェス定義（女の子: surface0-8、男の子: surface10-18）
// 各画像はピクトグラム風の人型アイコン

// 女の子（sakura）サーフェス
surface0 { element0,base,surface0.png,0,0 }
surface1 { element0,base,surface1.png,0,0 }
surface2 { element0,base,surface2.png,0,0 }
surface3 { element0,base,surface3.png,0,0 }
surface4 { element0,base,surface4.png,0,0 }
surface5 { element0,base,surface5.png,0,0 }
surface6 { element0,base,surface6.png,0,0 }
surface7 { element0,base,surface7.png,0,0 }
surface8 { element0,base,surface8.png,0,0 }

// 男の子（kero）サーフェス
surface10 { element0,base,surface10.png,0,0 }
surface11 { element0,base,surface11.png,0,0 }
surface12 { element0,base,surface12.png,0,0 }
surface13 { element0,base,surface13.png,0,0 }
surface14 { element0,base,surface14.png,0,0 }
surface15 { element0,base,surface15.png,0,0 }
surface16 { element0,base,surface16.png,0,0 }
surface17 { element0,base,surface17.png,0,0 }
surface18 { element0,base,surface18.png,0,0 }
"#,
    )?;

    // pasta DSL スクリプト
    fs::write(
        output_dir.join("ghost/master/dic/boot.pasta"),
        BOOT_PASTA_TEMPLATE,
    )?;

    fs::write(
        output_dir.join("ghost/master/dic/talk.pasta"),
        TALK_PASTA_TEMPLATE,
    )?;

    fs::write(
        output_dir.join("ghost/master/dic/click.pasta"),
        CLICK_PASTA_TEMPLATE,
    )?;

    Ok(())
}

const BOOT_PASTA_TEMPLATE: &str = r#"## boot.pasta - 起動・終了時のイベントハンドラ

@OnFirstBoot
---
\\1\\s[10]やあ、初めまして！\\n\\n[half]\\0\\s[0]初めまして〜！\\n\\nわたしたち、pasta システムのサンプルゴーストです。\\n\\n[half]よろしくね！\\e

@OnBoot
---
\\1\\s[10]また来たね。\\n\\n[half]\\0\\s[1]おかえりなさ〜い！\\e

@OnClose[act]
---
\\0\\s[0]またね〜！\\n\\n[half]\\1\\s[10]じゃあね。\\e
"#;

const TALK_PASTA_TEMPLATE: &str = r#"## talk.pasta - ランダムトーク・時報

@OnTalk
---
\\0\\s[0]今日もいい天気だね〜。\\n\\n[half]\\1\\s[10]そうだね、外に出たいな。\\e
---
\\0\\s[1]ねえねえ、pasta って面白いよね！\\n\\n[half]\\1\\s[11]まあ、ぼくには簡単だけどね。\\e
---
\\0\\s[2]えへへ〜、なんだか照れちゃう。\\n\\n[half]\\1\\s[10]なに照れてるのさ。\\e
---
\\1\\s[10]暇だな〜。\\n\\n[half]\\0\\s[0]じゃあ、お話しようよ！\\e
---
\\0\\s[6]わあ、きらきら〜！\\n\\n[half]\\1\\s[16]なにがそんなに楽しいの？\\e
---
\\0\\s[0]pasta DSL、覚えてくれた？\\n\\n[half]\\1\\s[11]ま、すぐ覚えられるよ。\\e
---
\\1\\s[17]眠い...\\n\\n[half]\\0\\s[7]わたしも眠くなってきちゃった...\\e

@OnHour[act]
---
\\0\\s[0]【act.var.時】時だよ〜！\\n\\n[half]\\1\\s[10]時報か。\\e
"#;

const CLICK_PASTA_TEMPLATE: &str = r#"## click.pasta - クリック反応

@OnMouseDoubleClick
---
\\0\\s[3]わっ、びっくりした！\\e
---
\\0\\s[1]えへへ、くすぐったいな〜。\\e
---
\\0\\s[2]そ、そんなに見つめないで...。\\e
---
\\1\\s[13]うわっ！なに！？\\e
---
\\1\\s[18]ちょっと、やめてよ！\\e
---
\\0\\s[6]わ〜い、遊んでくれるの？\\e
---
\\1\\s[11]ふふん、ぼくのことが気になる？\\e
"#;
