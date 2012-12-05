
/* UTF-8 ＠ぱすた＠
*/


/* アーリオ：会話辞書管理 
管理単位：item
            name: トーク名
            talk: トーク本体
*/


(function() {
  var __slice = [].slice;

  define(["jquery"], function() {
    /* １節
    */

    var Aglio, Knot, addKnot, addQuantum;
    Knot = (function() {

      function Knot(type, args) {
        var key, value;
        this.type = type;
        if (!(args != null)) {
          return;
        }
        for (key in args) {
          value = args[key];
          this[key] = value;
        }
      }

      return Knot;

    })();
    /* １節の追加
    */

    addKnot = function(aglio, type, args) {
      var index, item;
      item = new Knot(type, args);
      aglio.yarn.push(item);
      index = aglio.yarn.length - 1;
      item.index = index;
      return item;
    };
    /* 量子記憶
    */

    addQuantum = function(aglio, knot) {
      var el, index, key, quantums, _i, _len;
      index = knot.index;
      quantums = knot.quantums;
      el = $("<p/>");
      for (_i = 0, _len = quantums.length; _i < _len; _i++) {
        key = quantums[_i];
        el.addClass(key);
      }
      el.attr({
        'data-index': index
      });
      el.appendTo(aglio.quantumElements);
      return el;
    };
    /* 会話辞書登録管理
    */

    Aglio = (function() {

      function Aglio() {
        if (typeof console !== "undefined" && console !== null) {
          console.log("[Aglio::constructor]");
        }
        /* 記憶の糸
        */

        this.yarn = [];
        /* 量子情報
        */

        $("<div>").attr({
          id: "quantumElements"
        }).css({
          display: "none"
        }).appendTo("body");
        this.quantumElements = $("div#quantumElements")[0];
      }

      /* 会話確定
      */


      Aglio.prototype.end = function() {
        addKnot(this, "end");
        return this;
      };

      /* 会話登録開始：量子状態リスト
      */


      Aglio.prototype.quantum = function() {
        var quantum;
        quantum = 1 <= arguments.length ? __slice.call(arguments, 0) : [];
        this.end();
        addQuantum(this, addKnot(this, "quantum", {
          quantums: quantum
        }));
        return this;
      };

      /* １文登録
      */


      Aglio.prototype.sentence = function(actor, emotion, speech) {
        addKnot(this, "sentence", {
          actor: actor,
          emotion: emotion,
          speech: speech
        });
        return this;
      };

      /* セッション区切り
      */


      Aglio.prototype.sepSection = function() {
        addKnot(this, "sepSection");
        return this;
      };

      Aglio.prototype.$_______ = function() {
        return this.sepSection();
      };

      /* トーク区切り
      */


      Aglio.prototype.sepTalk = function() {
        addKnot(this, "sepTalk");
        return this;
      };

      Aglio.prototype.$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$ = function() {
        return this.sepTalk();
      };

      /* エンタングル。ランダム分岐。
          指定された量子状態を取るいずれかの会話に分岐する
      */


      Aglio.prototype.entangle = function(quantumState) {
        addKnot(this, "entangle", {
          quantumState: quantumState
        });
        return this;
      };

      /* ランダム終了
      */


      Aglio.prototype.ifend = function(terms) {
        addKnot(this, "terms", {
          terms: terms
        });
        return this;
      };

      /* ヘルパー関数：ランダム選択
      */


      Aglio.prototype.random = function(items) {
        return items.random();
      };

      return Aglio;

    })();
    Aglio.instance = new Aglio;
    return Aglio;
  });

}).call(this);
