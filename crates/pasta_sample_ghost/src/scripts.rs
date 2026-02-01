//! pasta DSL スクリプトテンプレート
//!
//! サンプルゴースト用の pasta DSL スクリプトを生成します。

use crate::GhostError;
use std::fs;
use std::path::Path;

/// スクリプトファイルを生成
pub fn generate_scripts(dic_dir: &Path) -> Result<(), GhostError> {
    fs::create_dir_all(dic_dir)?;

    fs::write(dic_dir.join("actors.pasta"), ACTORS_PASTA)?;
    fs::write(dic_dir.join("boot.pasta"), BOOT_PASTA)?;
    fs::write(dic_dir.join("talk.pasta"), TALK_PASTA)?;
    fs::write(dic_dir.join("click.pasta"), CLICK_PASTA)?;

    Ok(())
}

/// actors.pasta - アクター辞書（共通定義）
pub const ACTORS_PASTA: &str = r#"＃ actors.pasta - アクター辞書（共通定義）
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

/// boot.pasta - 起動/終了イベント用スクリプト
pub const BOOT_PASTA: &str = r#"＃ boot.pasta - 起動/終了イベント用シーン定義
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

/// talk.pasta - ランダムトーク/時報用スクリプト
pub const TALK_PASTA: &str = r#"＃ talk.pasta - ランダムトーク/時報用シーン定義
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

/// click.pasta - ダブルクリック反応用スクリプト
///
/// 仕様準拠: design.md では7種以上のバリエーション
pub const CLICK_PASTA: &str = r#"＃ click.pasta - ダブルクリック反応用シーン定義
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actors_pasta_contains_all_characters() {
        // アクター辞書に両キャラクターが定義されていることを確認
        assert!(
            ACTORS_PASTA.contains("％女の子"),
            "女の子アクターがありません"
        );
        assert!(
            ACTORS_PASTA.contains("％男の子"),
            "男の子アクターがありません"
        );
        // 全9表情が定義されていることを確認
        assert!(ACTORS_PASTA.contains("＠笑顔"), "笑顔表情がありません");
        assert!(ACTORS_PASTA.contains("＠通常"), "通常表情がありません");
        assert!(ACTORS_PASTA.contains("＠怒り"), "怒り表情がありません");
    }

    #[test]
    fn test_boot_pasta_contains_events() {
        assert!(BOOT_PASTA.contains("＊OnBoot"));
        assert!(BOOT_PASTA.contains("＊OnFirstBoot"));
        assert!(BOOT_PASTA.contains("＊OnClose"));
    }

    #[test]
    fn test_talk_pasta_contains_events() {
        assert!(TALK_PASTA.contains("＊OnTalk"));
        assert!(TALK_PASTA.contains("＊OnHour"));
        assert!(TALK_PASTA.contains("＄時"));
    }

    #[test]
    fn test_click_pasta_contains_events() {
        assert!(CLICK_PASTA.contains("＊OnMouseDoubleClick"));
        // 7種以上のバリエーション確認
        let count = CLICK_PASTA.matches("＊OnMouseDoubleClick").count();
        assert!(count >= 7, "ダブルクリック反応は7種以上必要: {}", count);
    }

    #[test]
    fn test_event_files_do_not_contain_actor_dictionary() {
        // アクター辞書は actors.pasta のみに定義されることを確認
        assert!(
            !BOOT_PASTA.contains("％女の子"),
            "boot.pasta にアクター辞書が含まれています"
        );
        assert!(
            !BOOT_PASTA.contains("％男の子"),
            "boot.pasta にアクター辞書が含まれています"
        );
        assert!(
            !TALK_PASTA.contains("％女の子"),
            "talk.pasta にアクター辞書が含まれています"
        );
        assert!(
            !TALK_PASTA.contains("％男の子"),
            "talk.pasta にアクター辞書が含まれています"
        );
        assert!(
            !CLICK_PASTA.contains("％女の子"),
            "click.pasta にアクター辞書が含まれています"
        );
        assert!(
            !CLICK_PASTA.contains("％男の子"),
            "click.pasta にアクター辞書が含まれています"
        );
    }
}
