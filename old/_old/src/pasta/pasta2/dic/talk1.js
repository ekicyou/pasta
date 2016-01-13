﻿
/* UTF-8 ＠ぱすた＠　会話辞書１
*/


(function() {
  "use strict";

  define(["engine/aglio_dic"], function(dic) {
    var story;
    story = function(func) {
      return func();
    };
    story(function() {
      var title;
      title = "今日も暑いですね";
      return dic.scrap([title, "会話", "おじさん", "夏"], function(essence) {
        this.section();
        this.period("mister p1");
        this.emote("パスタ：ノーマル");
        this.talk("パスタさん、今日も暑いですね。");
        this.period("pasta p1");
        this.talk("そーですねえ。");
        this.period("mister p1");
        this.talk("汗かかない？一枚脱ぐ？");
        this.section();
        this.period("pasta p1");
        this.emote("パスタ：ジトー");
        this.talk("それって、逆に汗かきませんか？");
        this.period("mister p1");
        this.talk("‥‥‥‥‥‥。");
        return this.close();
      });
    });
    story(function() {
      var title;
      title = "出来損ないのパスタ";
      dic.scrap([title, "会話", "パスタ"], function(essence) {
        this.section();
        this.period("pasta p2");
        this.emote("パスタ：ノーマル");
        this.talk("みんながもってる、記憶の糸。");
        this.period("pasta p1");
        this.talk("生まれてから、続いている、");
        this.br();
        this.talk("長い長い、一本の道。");
        this.section();
        this.period("pasta p1");
        this.emote("パスタ：よそみ");
        this.talk("そう、きっと、一本道。");
        this.br();
        this.emote("パスタ：ノーマル");
        this.talk("いつか来る、");
        this.emote("パスタ：まばたき");
        this.talk("終わりの日まで。");
        return this.separate();
      });
      return dic.scrap([title + "#3"], function(essence) {
        this.section();
        this.period("pasta p1");
        this.emote("パスタ：ノーマル");
        this.talk("でもね。");
        this.br();
        this.emote("パスタ：よそみ");
        this.talk("私の糸はカッコ悪いの。");
        this.period("pasta p1");
        this.emote("パスタ：うわのそら");
        this.talk("撚れたり、結んだり、絡み合ったり。");
        this.section();
        this.period("pasta p2");
        this.emote("パスタ：ノーマル");
        this.talk("出来損ないのパスタみたい、");
        this.period("pasta p1");
        this.talk("でもきっと、");
        this.emote("パスタ：ほほえみ");
        this.talk("そのほうが美味しいよ？");
        return this.close();
      });
    });
    return console.log("[talk1] 辞書の登録完了");
  });

  /*
  #===================================================================
  .aglio.story("ずっとは無い")
  #===# ==============================================================
      .パスタ２(  -> """こないだのコト、なんですけど。""")
      .パスタ１(  -> """雨降ってたんですよね。
                        ずっと降ったら大変だなって、""")
      .$_______()
      .パスタ１(  -> """でも、３日ほどしたら止みました。
                        「ずっと」ってコトは、無いんですねぇ。""")
  
      .$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$()
      .ソルト１(  -> """でも、ずっとは飽きるよね。""")
      .パスタ１(  -> """少年は大志を抱くんだね。
                        女の子はね、永遠のしあわせを願うの。""")
      .$_______()
      .ソルト１(  -> """ずっと続くしあわせは、幸せだと思う？""")
      .パスタ１(  -> """‥‥多分、不幸せ。""")
  
  
  #===================================================================
  .aglio.story("きっと何者にもなれない")
  #===# ==============================================================
      .パスタ２(  -> """きっと何者にもなれないお前たちに告げる。""")
      .$_______()
      .呼出("きっと何者にもなれない")
  
   .分岐("きっと何者にもなれない")
      .パスタ１(  -> """安心しろ、きっとだれも、
                        お前になることなど出来ない。""")
  
   .分岐("きっと何者にもなれない")
      .呼出("言ってみたかったセリフ")
  
  
  
  #===================================================================
   .aglio.story("言ってみたかったセリフ")
  #===# ==============================================================
      .パスタ２( -> $.random [
          """まだまだだね。"""
          """真実はいつもひとつ。"""
          """逃げちゃダメだ、逃げちゃダメだ、
             逃げちゃダメだ！"""
          """いっぺん‥‥
             死んでみる？"""
          """それでも
             守りたい世界があるんだ。"""
      ])
      .$_______()
      .呼出("言ってみたかったセリフ")
  
  
  #===================================================================
  .aglio.story("言ってみたかったセリフ")
  #===# ==============================================================
      .パスタ１( ->  """‥‥言ってみたかったんです。""")
      .ifend(0.5) # 数値     -> 0.5の確率で、ここでトークを打ち切り
                  # 関数     -> trueの場合に打ち切り
                  # 引数なし -> 無条件打ち切り。通常は不要
  
  #===================================================================
   .aglio.story("一句")
  #===# ==============================================================
      .パスタ２( -> $.random [
          """木枯らしに"""
          """ちはやふる"""
          """サラリーマン"""
          """振り向けば"""
          """逃げちゃダメだ"""
      ])
      .パスタ１( -> " " + $.random [
          """鐘が鳴るなり"""
          """いつもそこには"""
          """逃げちゃダメだ"""
      ])
      .パスタ１( -> "　" +$.random [
          """法隆寺"""
          """きみがいる"""
          """逃げちゃダメだ"""
      ])
      .$_______()
      .パスタ１( -> " " + $.random [
          """ものの哀れを"""
          """いつもそこには"""
          """逃げちゃダメだ"""
      ])
      .パスタ１( ->  """表現してみました。""")
  */


}).call(this);