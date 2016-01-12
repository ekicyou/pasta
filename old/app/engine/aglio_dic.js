
/* UTF-8 ＠ぱすた＠
*/


/* アーリオ：会話辞書管理 
管理単位：item
            name: トーク名
            talk: トーク本体
*/


(function() {

  define(["jquery", "engine/jsutil"], function() {
    "use strict";

    /* 属性チェック関数
    */

    var AglioDic, Scrap, addTagArea, createTagArea, isArray, isFunc;
    isFunc = function(o) {
      return typeof o === "function";
    };
    isArray = function(o) {
      return Object.prototype.toString.call(o) === '[object Array]';
    };
    /* タグ領域の作成
    */

    createTagArea = function(id) {
      $("<div>").attr({
        id: id
      }).css({
        display: "none"
      }).appendTo("body");
      return $("div#" + id)[0];
    };
    /* タグ領域の追加
    */

    addTagArea = function(dic, tags, value) {
      var el, tag, v, values, _i, _j, _len, _len1;
      values = isArray(value) ? value : [value];
      for (_i = 0, _len = values.length; _i < _len; _i++) {
        v = values[_i];
        el = $("<p/>");
        for (_j = 0, _len1 = tags.length; _j < _len1; _j++) {
          tag = tags[_j];
          el.addClass(tag);
        }
        el.attr({
          'data-value': v
        });
        el.appendTo(dic);
      }
    };
    /* スクラップ：ストーリーの１節
    */

    Scrap = (function() {

      function Scrap(tags, callback) {
        this.tags = tags;
        this.callback = callback;
      }

      return Scrap;

    })();
    /* 会話辞書登録管理
    */

    AglioDic = (function() {
      /* コンストラクタ
      */

      function AglioDic() {
        /* 記憶の糸
        */
        this.scraps = [];
        /* タグ領域
        */

        this.scrapTags = createTagArea("scrapTags");
        this.wordTags = createTagArea("wordTags");
      }

      /* スクラップ登録
      */


      AglioDic.prototype.scrap = function(tags, callback) {
        var index, scrap,
          _this = this;
        scrap = this.createScrap(tags, callback);
        this.scraps.push(scrap);
        index = this.scraps.length - 1;
        scrap.index = index;
        scrap.next = function() {
          return _this.getScrap(index + 1);
        };
        addTagArea(this.scrapTags, tags, index);
      };

      AglioDic.prototype.createScrap = function(tags, callback) {
        return new Scrap(tags, callback);
      };

      AglioDic.prototype.getScrap = function(index) {
        if (index >= this.scraps.length) {
          index = 0;
        }
        return this.scraps[index];
      };

      /* 単語登録
      */


      AglioDic.prototype.word = function(tags, values) {
        return addTagArea(this.wordTags, tags, values);
      };

      /* タグがすべて含まれたスクラップエレメントの検索
      */


      AglioDic.prototype.getScrapElement = function(tags) {
        var selector, tag, _i, _len;
        selector = "div#scrapTags p";
        for (_i = 0, _len = tags.length; _i < _len; _i++) {
          tag = tags[_i];
          selector += "." + tag;
        }
        return $(selector);
      };

      /* スクラップ選択
      */


      AglioDic.prototype.selectScrap = function(tags) {
        var $$, $i, index;
        $$ = this.getScrapElement(tags);
        if ($$.length === 0) {
          return void 0;
        }
        $i = Math.random() * $$.length | 0;
        index = $$.eq($i).data("value");
        return this.scraps[index];
      };

      return AglioDic;

    })();
    return AglioDic.instance = new AglioDic;
  });

}).call(this);
