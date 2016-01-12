
/* UTF-8 ＠ぱすた＠　起動
*/


(function() {

  define(["engine/jsutil", "engine/peperoncino_dispatcher"], function(jsutil, EventDispatcher) {
    /* 起動イベント発動
    */
    if (typeof console !== "undefined" && console !== null) {
      console.log("[pasta::ready] 予約");
    }
    return $(window).ready(function() {
      var dispatcher, pasta;
      if (typeof console !== "undefined" && console !== null) {
        console.log("[pasta::ready] 実行");
      }
      pasta = {};
      dispatcher = new EventDispatcher();
      pasta.dispatcher = dispatcher;
      window.pasta = pasta;
      $("#mainArea").on("click", ".hittest", function(args) {
        return dispatcher.touch(args.target.className);
      });
    });
  });

}).call(this);
