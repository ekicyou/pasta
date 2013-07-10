
/* UTF-8 ＠ぱすた＠
*/


/* アーリオ：会話辞書管理 
管理単位：item
            name: トーク名
            talk: トーク本体
*/


(function() {
  var __slice = [].slice;

  define(["engine/aglio_dic"], function(dic) {
    /* 会話/イベント登録関数のショートカット
    */
    dic.prototype.会話 = function() {
      var arg;
      arg = 1 <= arguments.length ? __slice.call(arguments, 0) : [];
      return this.quantum.apply(this, ["会話"].concat(__slice.call(arg)));
    };
    dic.prototype.分岐 = function() {
      var arg;
      arg = 1 <= arguments.length ? __slice.call(arguments, 0) : [];
      return this.quantum.apply(this, ["分岐"].concat(__slice.call(arg)));
    };
    dic.prototype.イベント = function() {
      var arg;
      arg = 1 <= arguments.length ? __slice.call(arguments, 0) : [];
      return this.quantum.apply(this, ["イベント"].concat(__slice.call(arg)));
    };
    /* entangle関数のショートカット
    */

    dic.prototype.呼出 = function(arg) {
      return this.entangle("分岐 " + arg);
    };
    /* sentence関数のショートカット
    */

    dic.prototype.パスタ１ = function(arg) {
      return this.sentence("pasta", "p1", arg);
    };
    dic.prototype.パスタ２ = function(arg) {
      return this.sentence("pasta", "p2", arg);
    };
    dic.prototype.ソルト１ = function(arg) {
      return this.sentence("salt", "p1", arg);
    };
    dic.prototype.ソルト２ = function(arg) {
      return this.sentence("salt", "p2", arg);
    };
    /* 辞書インスタンスを返す
    */

    return dic;
  });

}).call(this);
