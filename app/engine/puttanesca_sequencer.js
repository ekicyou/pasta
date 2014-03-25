
/* UTF-8 ＠ぱすた＠
*/


(function() {
  "use strict";

  define(["engine/jsutil", "jquery", "modernizr"], function(jsutil, Napoletano) {
    /* プッタネスカ：トークアニメーション処理
    */

    /* スペースを削除する関数
    */

    var PuttanescaSequencer, SectionArea, SectionAreaItem, createTagArea, delSpace, endChars, endWait1Chars, endWait2Chars, endWait3Chars, endWait4Chars, getScrapArea, hangChars, isStartEndType, isWaitType, normalMS, scrapArea, sectionCloseWait, showCharWait, startChars, waitMSTable;
    delSpace = function(str) {
      return str;
    };
    /* 禁則文字列
    */

    startChars = "（［｛「『([{｢";
    endChars = "、。，．・？！゛゜ヽヾゝゞ々）］｝」』!),.:;?]}｡｣､･ﾞﾟ‥…";
    endWait1Chars = delSpace("、  ，                                  ,        ､       ");
    endWait2Chars = delSpace("        ・                                        ･  ‥…");
    endWait3Chars = delSpace("            ！                        !                  ");
    endWait4Chars = delSpace("  。  ．  ？                               .?  ｡         ");
    hangChars = "、。，．,.";
    normalMS = 120;
    waitMSTable = {
      a: normalMS * 1.0,
      b: normalMS * 3.0,
      c: normalMS * 1.5,
      d: normalMS * 3.5,
      e: normalMS * 4.5,
      period: normalMS * 5.0,
      section: normalMS * 5.0,
      talk: 20000
    };
    showCharWait = 300;
    sectionCloseWait = 400;
    /* 文字タイプの判定
    */

    isStartEndType = function(c) {
      var CHECK;
      CHECK = function(items) {
        return (items.indexOf(c)) > -1;
      };
      if (c === " ") {
        return "normal";
      } else if (CHECK(startChars)) {
        return "start";
      } else if (CHECK(endChars)) {
        if (CHECK(endWait1Chars)) {
          return "end1";
        } else if (CHECK(endWait2Chars)) {
          return "end2";
        } else if (CHECK(endWait3Chars)) {
          return "end3";
        } else if (CHECK(endWait4Chars)) {
          return "end4";
        } else {
          return "end";
        }
      }
    };
    /* ウェイトタイプ判定
    */

    isWaitType = function(c) {
      var tp;
      tp = isStartEndType(c);
      switch (tp) {
        case "end":
          return "b";
        case "end1":
          return "b";
        case "end2":
          return "c";
        case "end3":
          return "d";
        case "end4":
          return "e";
        default:
          return "a";
      }
    };
    /* タグ：タグ領域の作成
    */

    /* タグ領域の作成
    */

    createTagArea = function(id, parent) {
      $("<div>").attr({
        id: id
      }).appendTo(parent);
      return $("div#" + id)[0];
    };
    scrapArea = void 0;
    getScrapArea = function() {
      if (scrapArea != null) {
        return scrapArea;
      }
      scrapArea = $("div#scrapArea")[0];
      if (scrapArea != null) {
        return scrapArea;
      }
      scrapArea = createTagArea("scrapArea", "body");
      return scrapArea;
    };
    /* セクション領域のアイテム
    */

    /* 表示はクラス制御
        初期：  waiting
        再生中：playing
        再生終：played
        終了中：closeing
        終了：(終了時にはエレメントが削除)
    */

    SectionAreaItem = (function() {
      /* 定数/グローバル変数/メソッド
      */

      var SectionIndexCounter;

      SectionIndexCounter = 0;

      /* コンストラクタ
      */


      function SectionAreaItem(parent, areaClass, index) {
        this.parent = parent;
        this.areaClass = areaClass;
        this.index = index != null ? index : SectionIndexCounter++;
        this.id = "" + this.parent + "-" + this.index;
        console.log("[SectionAreaItem<" + this.id + ">::constructor]");
        this.area = $(createTagArea(this.id, this.parent));
        this.area.addClass(this.areaClass);
        this.area.addClass("waiting");
        this.isShow = true;
        this.hide();
      }

      /* DOMから自身を削除する
      */


      SectionAreaItem.prototype.remove = function() {
        this.area.remove();
        console.log("[SectionAreaItem<" + this.id + ">::remove]");
      };

      /* 表示
      */


      SectionAreaItem.prototype.show = function() {
        console.log("[SectionAreaItem<" + this.id + ">::show]");
        this.setIsShow(true);
      };

      /* 非表示
      */


      SectionAreaItem.prototype.hide = function() {
        console.log("[SectionAreaItem<" + this.id + ">::hide]");
        this.setIsShow(false);
      };

      /* クラスの置き換え
      */


      SectionAreaItem.prototype.setIsShow = function(isShow) {
        if (isShow === this.isShow) {
          return;
        }
        this.isShow = isShow;
        if (this.isShow) {
          this.area.removeClass("hidden");
        } else {
          this.area.addClass("hidden");
        }
      };

      /* 段落追加
      */


      SectionAreaItem.prototype.append = function(paragraph) {
        this.area.append(paragraph);
      };

      return SectionAreaItem;

    })();
    /* セクション領域の管理
    */

    SectionArea = (function() {
      /* コンストラクタ
      */

      function SectionArea(id) {
        this.id = id;
        this.resetNextAreaClass();
      }

      /* 次のセクション位置をリセットする
      */


      SectionArea.prototype.resetNextAreaClass = function() {
        return this.areaClassIndex = -1;
      };

      /* 次のセクション位置を取得する
      */


      SectionArea.prototype.getNextAreaClass = function() {
        this.areaClassIndex++;
        if (this.areaClassIndex >= areaItems.length) {
          this.areaClassIndex = 0;
        }
        return areaItems[this.areaClassIndex];
      };

      /* 追加
      */


      SectionArea.prototype.startSection = function(areaClass) {
        var section;
        if (areaClass == null) {
          areaClass = this.getNextAreaClass();
        }
        return section = new SectionAreaItem;
      };

      /* 段落追加
      */


      SectionArea.prototype.append = function(paragraph) {};

      return SectionArea;

    })();
    /* プッタネスカ：アニメーションシーケンサ
    */

    /* １回のアニメーションについて、開放まで管理
         発生する契約
         ・再生開始：@playDeferred
         ・再生完了：@playedDeferred
         ・終了開始：@closeingDeferred
         ・終了完了：@closedDeferred
    */

    PuttanescaSequencer = (function() {
      var sectionIndex, talkIndex;

      sectionIndex = 0;

      talkIndex = 0;

      /* コンストラクタ
      */


      function PuttanescaSequencer(startTime, sectionAreas) {
        var i, j, key, val, _i, _ref,
          _this = this;
        this.startTime = startTime != null ? startTime : window.Date.now();
        this.sectionAreas = sectionAreas != null ? sectionAreas : ["area1", "area2"];
        this.scrapClass = "scrap" + (++talkIndex);
        console.log("スクラップ追加", [this.scrapClass]);
        $("#" + this.scrapClass).remove();
        this.scrapArea = $("<div />");
        this.scrapArea.attr({
          id: this.scrapClass,
          "class": "scrap"
        });
        this.scrapArea.appendTo(getScrapArea());
        this.playDeferred = $.Deferred();
        this.playedDeferred = $.Deferred();
        this.closedDeferred = $.Deferred();
        this.closeingDeferred = $.Deferred();
        this.playDeferred.fail(function() {
          return _this.playedDeferred.reject();
        });
        this.closeingDeferred.always(function() {
          var closeCheck, closeCount, item, _i, _len, _ref;
          _this.playDeferred.reject();
          _this.playedDeferred.reject();
          closeCount = 1;
          closeCheck = function() {
            closeCount--;
            console.log("終了チェック", [closeCount]);
            if (closeCount > 0) {
              return;
            }
            _this.scrapArea.remove();
            console.log("終了！");
            return _this.closedDeferred.resolve();
          };
          _ref = _this.areaCloseAnimes;
          for (_i = 0, _len = _ref.length; _i < _len; _i++) {
            item = _ref[_i];
            closeCount++;
            item.promise().always(closeCheck);
            item.reject();
          }
          closeCount--;
          return closeCheck();
        });
        this.nextArea = {};
        for (i = _i = 0, _ref = this.sectionAreas.length; 0 <= _ref ? _i < _ref : _i > _ref; i = 0 <= _ref ? ++_i : --_i) {
          j = i + 1;
          if (j >= this.sectionAreas.length) {
            j = 0;
          }
          key = this.sectionAreas[i];
          val = this.sectionAreas[j];
          this.nextArea[key] = val;
        }
        this.p = void 0;
        this.area = void 0;
        this.areaClass = void 0;
        this.areaCloseAnimes = [];
      }

      /* ステータス
      */


      PuttanescaSequencer.prototype.state = function() {
        if (this.closedDeferred.state() !== "pending") {
          return "close";
        }
        if (this.closeingDeferred.state() !== "pending") {
          return "closeing";
        }
        if (this.playedDeferred.state() !== "pending") {
          return "played";
        }
        return "play";
      };

      /* トーク契約
      */


      PuttanescaSequencer.prototype.playedPromise = function() {
        return this.playedDeferred.promise();
      };

      PuttanescaSequencer.prototype.closedPromise = function() {
        return this.closedDeferred.promise();
      };

      /* 再生終了状態に早送り
      */


      PuttanescaSequencer.prototype.played = function(args) {
        return this.playDeferred.resolve(args);
      };

      /* 表示クローズ
      */


      PuttanescaSequencer.prototype.close = function(args) {
        return this.closeingDeferred.reject(args);
      };

      /* スクラップをひとつ閉じて、会話を確定する。
      */


      PuttanescaSequencer.prototype.closeScrap = function() {
        this.endSection();
        return this.calcWait(this.scrapArea);
      };

      /* 指定エリアのウェイト計算。
      */


      PuttanescaSequencer.prototype.calcWait = function(area) {
        var commit, setMS, time, waitEl, waitMS;
        if (area == null) {
          area = this.scrapArea;
        }
        /* ウエイト計算
        */

        waitEl = void 0;
        waitMS = 0;
        setMS = function(el, ms) {
          return el.attr({
            "data-wait-ms": ms
          });
        };
        commit = function() {
          if (!(waitEl != null)) {
            return;
          }
          setMS(waitEl, waitMS);
          waitEl = void 0;
          waitMS = 0;
        };
        $("span.waiting", area).each(function() {
          var el, ms, waitType;
          el = $(this);
          waitType = el.data("waitType");
          ms = waitMSTable[waitType];
          if (!(ms != null)) {
            ms = 0;
          }
          setMS(el, ms);
          return;
          if (waitType === "c") {
            commit();
            setMS(el, ms);
            return;
          }
          if (ms < waitMS) {
            commit();
          }
          setMS(el, normalMS);
          waitEl = el;
          waitMS = ms;
        });
        /* 0スタートの発動タイミングの確定＆表示を消す
        */

        time = 0;
        $("span.waiting", area).each(function() {
          var el, ms;
          el = $(this);
          el.textHide();
          ms = el.data("waitMs");
          el.attr("data-start-time", time);
          return time += ms;
        });
        return console.log("[会話", [this.scrapClass], "] 終了");
      };

      /* 実行
      */


      PuttanescaSequencer.prototype.run = function() {
        var THIS, count, items, lastIndex, lastTime, startTime;
        startTime = Date.now();
        console.log(isodate.format(new Date(startTime)), ": !!START!!");
        lastTime = 0;
        count = 0;
        THIS = this;
        items = $("span.waiting", this.scrapArea);
        lastIndex = items.length - 1;
        items.each(function() {
          var dfd, el, emote, func, index, lastFunc, text, time, timestamp;
          el = $(this);
          time = el.data("startTime");
          text = el.text();
          timestamp = startTime + time;
          index = count++;
          emote = el.data("emote");
          func = function() {
            return THIS.playedDeferred.resolve();
          };
          lastFunc = func;
          dfd = $.Deferred();
          dfd.done(function() {
            var _ref;
            if (THIS.state() !== "play") {
              return;
            }
            if ((_ref = $.emote) != null) {
              if (typeof _ref[emote] === "function") {
                _ref[emote]();
              }
            }
            el.textShow(showCharWait);
            if (index === lastIndex) {
              THIS.playedDeferred.resolve();
            }
          });
          THIS.playDeferred.done(function() {
            return dfd.resolve();
          });
          $.timestamp(timestamp, true).done(function() {
            return dfd.resolve();
          });
          lastTime = time;
          return true;
        });
        return this.playDeferred;
      };

      /* セクションの終わり
      */


      PuttanescaSequencer.prototype.endSection = function() {
        var p;
        if (!(this.area != null)) {
          return;
        }
        p = this.p;
        this.endPeriod();
        if (p != null) {
          this.addChar(p, "section");
        }
        console.log("[セクション", [this.areaID], "] 終了");
        this.area = void 0;
      };

      /* セクションの始まり
      */


      PuttanescaSequencer.prototype.section = function(areaClass) {
        var closeAnim, closeDfd, dfd, el,
          _this = this;
        this.areaClass = areaClass != null ? areaClass : this.nextSectionArea();
        this.endSection();
        console.log("[セクション", [this.areaClass], "] 開始");
        el = $("<div />");
        el.attr({
          id: "section" + (++sectionIndex),
          "class": "section " + this.areaClass + " waiting"
        });
        el.appendTo(this.scrapArea);
        this.area = el[0];
        closeDfd = $.Deferred();
        closeAnim = function() {
          var opts, param;
          param = {
            top: "-16px",
            opacity: 0.0
          };
          opts = {
            duration: sectionCloseWait,
            easing: "easeInQuad"
          };
          return el.animate(param, opts).promise().done(function() {
            el.remove();
            return closeDfd.reject();
          });
        };
        dfd = $.Deferred();
        dfd.always(closeAnim);
        dfd.promise = closeDfd.promise;
        this.areaCloseAnimes.push(dfd);
      };

      /* 段落の終わり
      */


      PuttanescaSequencer.prototype.nextSectionArea = function() {
        var next;
        next = this.nextArea[this.areaClass];
        if (!next) {
          return this.sectionAreas[0];
        }
        return next;
      };

      /* 段落の終わり
      */


      PuttanescaSequencer.prototype.endPeriod = function(actor) {
        if (!(this.p != null)) {
          return;
        }
        console.log("段落閉じる：", [this.p]);
        this.addChar(this.p, "period");
        this.p.appendTo(this.area);
        this.p = void 0;
      };

      /* 段落の始まり
      */


      PuttanescaSequencer.prototype.period = function(actor) {
        this.endPeriod();
        this.p = $("<p />").addClass(actor);
        console.log("段落：", [this.p]);
      };

      /* 表情変更
      */


      PuttanescaSequencer.prototype.emote = function(em) {
        var el;
        if (!(this.p != null)) {
          period(this.defaultActor);
        }
        console.log("表情:", [em]);
        el = $("<span />");
        el.attr({
          "data-emote": em,
          "class": "waiting"
        });
        el.appendTo(this.p);
      };

      /* 会話
      */


      PuttanescaSequencer.prototype.talk = function(text) {
        var c, _i, _len;
        if (!(this.p != null)) {
          period(this.defaultActor);
        }
        console.log("会話:", [text]);
        for (_i = 0, _len = text.length; _i < _len; _i++) {
          c = text[_i];
          this.addChar(this.p, isWaitType(c), c);
        }
      };

      PuttanescaSequencer.prototype.addChar = function(parent, wait, c) {
        var el;
        el = $("<span />");
        if (c != null) {
          el.text(c);
        }
        el.attr({
          "data-wait-type": wait,
          "class": "waiting"
        });
        return el.appendTo(parent);
      };

      /* 改行
      */


      PuttanescaSequencer.prototype.br = function() {
        if (!(this.p != null)) {
          period(this.defaultActor);
        }
        console.log("改行:");
        $("<br />").appendTo(this.p);
      };

      return PuttanescaSequencer;

    })();
    return PuttanescaSequencer;
  });

}).call(this);
