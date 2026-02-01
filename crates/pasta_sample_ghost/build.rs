//! Build script for pasta_sample_ghost
//!
//! ビルド時に ghosts/hello-pasta/ ディレクトリにサンプルゴーストを生成します。
//! pasta_shiori のソース変更を検知し、pasta.dll を自動的にコピーします。

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // ビルドスクリプト再実行トリガー
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=src/");

    // pasta_shiori のソース変更を監視
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let workspace_root = Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("Failed to get workspace root");

    let pasta_shiori_src = workspace_root.join("crates/pasta_shiori/src");
    if pasta_shiori_src.exists() {
        println!("cargo::rerun-if-changed={}", pasta_shiori_src.display());
    }

    // クレートルートを取得
    let crate_root = Path::new(&manifest_dir);
    let ghosts_dir = crate_root.join("ghosts").join("hello-pasta");

    // ゴーストディレクトリ生成
    if let Err(e) = generate_ghost_files(&ghosts_dir) {
        eprintln!("Warning: Failed to generate ghost files: {}", e);
        // ビルドを失敗させない（オプショナル生成）
    }

    // pasta.dll を自動コピー（存在する場合）
    if let Err(e) = copy_pasta_dll(workspace_root, &ghosts_dir) {
        eprintln!("Info: pasta.dll not copied: {}", e);
        eprintln!("      Run setup.ps1 to copy pasta.dll manually.");
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
        output_dir.join("ghost/master/dic/actors.pasta"),
        ACTORS_PASTA_TEMPLATE,
    )?;

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

const ACTORS_PASTA_TEMPLATE: &str = r#"＃ actors.pasta - アクター辞書（共通定義）
＃ 全ての .pasta ファイルで共有されるアクター定義
＃ pasta DSL ローダーが dic/*.pasta パターンで自動読み込み

＃ 女の子（sakura）- 赤色ピクトグラム surface0-8
％女の子
　＠笑顔：\s[0]
　＠通常：\s[1]
　＠照れ：\s[2]
　＠驚き：\s[3]
　＠泣き：\s[4]
　＠困惑：\s[5]
　＠キラキラ：\s[6]
　＠眠い：\s[7]
　＠怒り：\s[8]

＃ 男の子（kero）- 青色ピクトグラム surface10-18
％男の子
　＠笑顔：\s[10]
　＠通常：\s[11]
　＠照れ：\s[12]
　＠驚き：\s[13]
　＠泣き：\s[14]
　＠困惑：\s[15]
　＠キラキラ：\s[16]
　＠眠い：\s[17]
　＠怒り：\s[18]
"#;

const BOOT_PASTA_TEMPLATE: &str = r#"＃ boot.pasta - 起動/終了イベント用シーン定義
＃ pasta DSL では「シーン関数フォールバック」機能を利用
＃ シーン名とSHIORIイベント名を一致させることで、自動ディスパッチされる
＃ ※アクター辞書は actors.pasta で共通定義

＃ グローバル単語定義（ランダム選択用）
＠起動挨拶：おはよう！今日もよろしくね！、やっほー、また会えたね！、起動完了！準備OKだよ。
＠終了挨拶：またね～！、お疲れ様！、ばいばーい！

＃ OnBoot イベント - 通常起動時（シーン関数フォールバックで呼び出し）
＊OnBoot
　女の子：＠笑顔　＠起動挨拶
　男の子：＠元気　へえ、また来たんだ。

＃ OnBoot イベント - 別パターン（同名シーンでランダム選択）
＊OnBoot
　女の子：＠通常　起動したよ～。
　男の子：＠通常　さあ、始めようか。

＃ OnFirstBoot イベント - 初回起動時
＊OnFirstBoot
　女の子：＠笑顔　初めまして！\nわたしは女の子、よろしくね。
　男の子：＠元気　ぼくは男の子。ちゃんと使ってよね。

＃ OnClose イベント - 終了時
＊OnClose
　女の子：＠通常　＠終了挨拶
　男の子：＠通常　また呼んでよね。

＃ OnClose イベント - 別パターン
＊OnClose
　女の子：＠眠い　おやすみなさい...
　男の子：＠通常　じゃあね。
"#;

const TALK_PASTA_TEMPLATE: &str = r#"＃ talk.pasta - ランダムトーク/時報用シーン定義
＃ OnSecondChange (毎秒) → 仮想イベントディスパッチャ → ランダムトーク/時報
＃ ※アクター辞書は actors.pasta で共通定義

＃ ランダムトーク用単語（ランダム選択）
＠雑談：何か用？、暇だなあ...、ねえねえ、聞いてる？、うーん、眠くなってきた...

＃ ランダムトーク - 仮想イベント OnTalk
＊OnTalk
　女の子：＠通常　＠雑談

＊OnTalk
　女の子：＠笑顔　Pasta DSL、使ってみてね！
　男の子：＠元気　Lua 側も触ってみなよ。

＊OnTalk
　女の子：＠考え　今日は何しようかな...
　男の子：＠通常　宿題やったの？

＊OnTalk
　女の子：＠通常　ねえ、今日の天気どう思う？
　男の子：＠考え　さあ、外見てないからわかんないや。

＊OnTalk
　女の子：＠笑顔　一緒にお話しよう！
　男の子：＠元気　しょうがないなあ。

＊OnTalk
　女の子：＠眠い　ふわあ...ちょっと眠いかも。
　男の子：＠通常　寝てていいよ、ぼくが見てるから。

＃ 時報 - 仮想イベント OnHour
＃ ＄時１２ 変数は onhour-date-var-transfer により自動設定される（12時間表記）
＊OnHour
　女の子：＠笑顔　＄時１２　だよ！時報だよ～。
　男の子：＠元気　もう　＄時１２　か、早いね。

＊OnHour
　女の子：＠通常　今　＄時１２　だって。
　男の子：＠通常　へえ、そうなんだ。

＊OnHour
　女の子：＠考え　＄時１２　...時間が経つのって不思議だね。
　男の子：＠考え　哲学的だね。
"#;

const CLICK_PASTA_TEMPLATE: &str = r#"＃ click.pasta - ダブルクリック反応用シーン定義
＃ OnMouseDoubleClick イベントに反応
＃ ※アクター辞書は actors.pasta で共通定義

＃ ダブルクリック反応（ランダム選択）7種以上
＊OnMouseDoubleClick
　女の子：＠驚き　わっ、びっくりした！
　男の子：＠元気　どうしたの？

＊OnMouseDoubleClick
　女の子：＠笑顔　なあに？呼んだ？
　男の子：＠通常　こっちに用があるんじゃない？

＊OnMouseDoubleClick
　女の子：＠照れ　え、なに？
　男の子：＠元気　照れてるの？

＊OnMouseDoubleClick
　男の子：＠驚き　うわっ！なに！？
　女の子：＠笑顔　反応してくれたね。

＊OnMouseDoubleClick
　女の子：＠怒り　もう、そんなにクリックしないで！
　男の子：＠驚き　お、怒った怒った。

＊OnMouseDoubleClick
　女の子：＠笑顔　わ〜い、遊んでくれるの？
　男の子：＠通常　まあ、暇だしね。

＊OnMouseDoubleClick
　男の子：＠元気　ふふん、ぼくのことが気になる？
　女の子：＠驚き　えっ？そんなんじゃないよ！
"#;

/// pasta.dll を自動的にコピーする
///
/// 32bit Windows ビルドの pasta.dll が存在する場合、
/// ghosts/hello-pasta/ghost/master/ にコピーします。
fn copy_pasta_dll(workspace_root: &Path, ghosts_dir: &Path) -> std::io::Result<()> {
    // pasta.dll のソースパス
    let dll_src = workspace_root
        .join("target")
        .join("i686-pc-windows-msvc")
        .join("release")
        .join("pasta.dll");

    if !dll_src.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("pasta.dll not found at {}", dll_src.display()),
        ));
    }

    // pasta.dll のコピー先
    let dll_dest = ghosts_dir.join("ghost").join("master").join("pasta.dll");

    // ディレクトリが存在することを確認
    if let Some(parent) = dll_dest.parent() {
        fs::create_dir_all(parent)?;
    }

    // ファイルをコピー
    fs::copy(&dll_src, &dll_dest)?;

    println!(
        "cargo::warning=Copied pasta.dll from {} to {}",
        dll_src.display(),
        dll_dest.display()
    );

    Ok(())
}
