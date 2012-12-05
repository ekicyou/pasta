
/* UTF-8 ＠ぱすた＠
*/


/* オリオ：会話シーケンサ 
辞書より会話を構成する。呼び出される毎に会話の１節を返す。
状態はessenceに保持する
*/


(function() {

  define(["engine/aglio_dic", "engine/puttanesca_sequencer", "engine/jsutil"], function(aglio_dic, Sequencer) {
    "use strict";

    /* 属性チェック関数
    */

    var OlioStoryPicker, isArray, isFunc;
    isFunc = function(o) {
      return typeof o === "function";
    };
    isArray = function(o) {
      return Object.prototype.toString.call(o) === '[object Array]';
    };
    /* 会話スクリプタ
    */

    OlioStoryPicker = (function() {
      /* コンストラクタ
            @dic      : 辞書
            @essence  : 記憶
      */

      function OlioStoryPicker(dic, essence) {
        this.dic = dic != null ? dic : aglio_dic;
        this.essence = essence != null ? essence : {};
        this.nowStoryThis = void 0;
      }

      /* 会話選択とセクションの読み込み
              tags     : 会話引用タグ
              sequencer: 会話シーケンサ
      */


      OlioStoryPicker.prototype.nextStory = function(tags, sequencerConstructor) {
        var scrap;
        if (sequencerConstructor == null) {
          sequencerConstructor = Sequencer;
        }
        scrap = this.dic.selectScrap(tags);
        if (!scrap) {
          console.log("[OlioStoryPicker::nextStory] キーワードが見つかりませんでした。", tags);
          return void 0;
        }
        return this.createSeq(scrap, sequencerConstructor);
      };

      OlioStoryPicker.prototype.createSeq = function(scrap, sequencerConstructor) {
        var THIS,
          _this = this;
        if (sequencerConstructor == null) {
          sequencerConstructor = Sequencer;
        }
        THIS = $.Deferred();
        THIS.sequencerConstructor = sequencerConstructor;
        THIS.sequencer = void 0;
        THIS.essence = this.essence;
        THIS.scrap = scrap;
        THIS.next = function() {
          return _this.nextScrap(THIS);
        };
        /* 終了
        */

        THIS.isClose = false;
        THIS.Z = function() {
          return this.isClose = true;
        };
        THIS.close = function() {
          return this.Z();
        };
        /* １会話の区切り
        */

        THIS.isSeparate = false;
        THIS.Y = function() {
          return this.isSeparate = true;
        };
        THIS.separate = function() {
          return this.Y();
        };
        /* セクション区切り
        */

        THIS.S = function(area) {
          return THIS.sequencer.section(area);
        };
        THIS.section = function(area) {
          return this.S(area);
        };
        /* 段落区切り(p)
        */

        THIS.P = function(actor) {
          return THIS.sequencer.period(actor);
        };
        THIS.period = function(actor) {
          return this.P(actor);
        };
        /* 表情変更
        */

        THIS.E = function(em) {
          return THIS.sequencer.emote(em);
        };
        THIS.emote = function(em) {
          return this.E(em);
        };
        /* 改行
        */

        THIS.B = function() {
          return THIS.sequencer.br();
        };
        THIS.br = function() {
          return this.B();
        };
        /* テキスト
        */

        THIS.T = function(text) {
          return THIS.sequencer.talk(text);
        };
        THIS.talk = function(text) {
          return this.T(text);
        };
        /* ジャンプ予約
        */

        THIS.jumpTags = void 0;
        THIS.J = function(tags) {
          return THIS.jumpTags = tags;
        };
        THIS.jump = function(tags) {
          return this.J(tags);
        };
        /* 単語の取得
        */

        THIS.W = function(tags) {
          return _this.popWord(tags);
        };
        THIS.popWord = function(tags) {
          return this.W(tags);
        };
        /* スクラップの実行
            シーケンサに１会話分の指示を与える
        */

        THIS.callScrap = function() {
          THIS.isSeparate = false;
          while (THIS.scrap != null) {
            THIS.jumpTags = void 0;
            THIS.scrap.callback.call(THIS, THIS.essence);
            THIS.scrap = THIS.isClose ? void 0 : THIS.jumpTags != null ? _this.dic.selectScrap(THIS.jumpTags) : THIS.scrap.next();
            if (THIS.isSeparate === true) {
              return;
            }
          }
        };
        THIS.always(function() {
          var _ref;
          if ((_ref = THIS.sequencer) != null) {
            _ref.close();
          }
        });
        return this.nowStoryThis = THIS;
      };

      /* １会話分、スクラップを進める。進んだ場合は会話シーケンサを返す
      */


      OlioStoryPicker.prototype.nextScrap = function(THIS) {
        var _ref;
        if (THIS == null) {
          THIS = this.nowStoryThis;
        }
        if ((_ref = THIS.sequencer) != null) {
          if (typeof _ref.close === "function") {
            _ref.close();
          }
        }
        THIS.sequencer = void 0;
        if ((THIS != null ? THIS.scrap : void 0) != null) {
          THIS.sequencer = new THIS.sequencerConstructor();
          THIS.callScrap();
          return THIS.sequencer;
        } else {
          THIS.resolve();
          if (THIS === this.nowStoryThis) {
            this.nowStoryThis = void 0;
          }
          return void 0;
        }
      };

      /* ランダムにセクションジャンプ
      */


      OlioStoryPicker.prototype.jump = function(tags) {
        return console.log("ジャンプ:", tags);
      };

      /* ランダムに単語辞書から取得
      */


      OlioStoryPicker.prototype.popWord = function(tags) {
        return console.log("単語取得:", tags);
      };

      return OlioStoryPicker;

    })();
    return OlioStoryPicker;
  });

}).call(this);
