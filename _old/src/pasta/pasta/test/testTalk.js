
/* UTF-8 　会話辞書１
*/


(function() {
  "use strict";

  define(["engine/aglio_dic"], function(dic) {
    var story;
    story = function() {
      var title;
      title = "testStory1";
      dic.scrap([title, "tag1", "tag2"], function(essence) {
        this.section();
        this.period("pasta");
        this.emote("pasta.1");
        this.talk("pasta.talk1.");
        this.talk(this.popWord(["wordKey1"]));
        this.talk("pasta.talk2.");
        this.br();
        this.talk("pasta.talk3.");
        this.section();
        this.period("solt");
        this.emote("solt.1");
        return this.talk("solt.talk4.");
      });
      return dic.scrap([title + "#1"], function(essence) {
        this.section();
        this.period("pasta");
        this.emote("pasta.2");
        this.talk("pasta.talk5.");
        return this.close();
      });
    };
    story();
    dic.word(["wordKey1"], "word1");
    story = function() {
      var title;
      title = "testStory2";
      return dic.scrap([title, "tag1", "tag3"], function(essence) {
        this.section();
        this.period("solt");
        this.emote("solt.1");
        this.talk("solt.talk61.");
        this.br();
        this.talk("solt.talk62.");
        this.period("pasta");
        this.emote("pasta.0");
        this.talk("今日はいい天気です、‥‥");
        this.emote("pasta.4");
        this.talk("あれ？曇ってきた‥‥");
        this.emote("pasta.1");
        this.talk("かあ。");
        return this.jump(["tag4"]);
      });
    };
    story();
    story = function() {
      var title;
      title = "testStory3";
      dic.scrap([title, "tag4", "tag1"], function(essence) {
        this.section();
        this.period("solt");
        this.emote("solt.e2");
        return this.talk("solt.talk7");
      });
      return dic.scrap([title + "#1"], function(essence) {
        this.section();
        this.period("pasta");
        this.emote("pasta.e2");
        this.talk("pasta.talk8");
        return this.close();
      });
    };
    return story();
  });

}).call(this);
