
/* UTF-8 ＠ぱすた＠
*/


/* 辞書ストア
*/


(function() {

  define(["engine/jsutil"], function() {
    "use strict";

    /* Array APIに対しキャッシュ付きの辞書操作を行います
    */

    var CacheStore;
    CacheStore = (function() {
      /* コンストラクタ
          @getItemsFunc 要素配列を全件取得するための関数
      */

      function CacheStore(getAllItemsFunc) {
        this.getAllItemsFunc = getAllItemsFunc;
        this.reset();
      }

      /* アイテムのクリア
      */


      CacheStore.prototype.reset = function() {
        this.allItems = void 0;
      };

      /* アイテムの全件取得
      */


      CacheStore.prototype.getAllItems = function() {
        if (!(this.allItems != null)) {
          this.allItems = this.getAllItemsFunc();
          this.cache = {};
        }
        return this.allItems;
      };

      /* キーの条件に対応した要素を取得。
          要素がキャッシュされていなければ関数を適用する
      */


      CacheStore.prototype.getOne = function(filter) {
        var rc;
        rc = this.cache[filter.key];
        if (!(rc != null)) {
          rc = filter.func(this.getAllItems());
          this.cache[filter.key] = rc;
        }
        return rc;
      };

      /* 複合条件の検索。すべての条件に一致するものを返す
      */


      CacheStore.prototype.get = function(filters) {
        var filter, items, rc, smallItems, smallKey, _i, _j, _len, _len1;
        smallKey = "";
        smallItems = this.getAllItems();
        for (_i = 0, _len = filters.length; _i < _len; _i++) {
          filter = filters[_i];
          items = this.getOne(filter);
          if (items.length <= smallItems.length) {
            smallKey = filter.key;
            smallItems = items;
          }
        }
        rc = smallItems;
        for (_j = 0, _len1 = filters.length; _j < _len1; _j++) {
          filter = filters[_j];
          if (smallKey !== filter.key) {
            rc = filter.func(rc);
          }
        }
        return rc;
      };

      return CacheStore;

    })();
    return CacheStore;
  });

}).call(this);
