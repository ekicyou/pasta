
/* UTF-8 ＠ユニットテスト＠　起動
*/


(function() {

  define(["engine/jarvisutil", "test/jsutil", "test/talk1", "test/store"], function(Run, jsutil, talk1, store) {
    /* 試験データ
    */
    $.holdReady(false);
    return Run({
      name: "AllTest",
      tearDown: function() {},
      setup: function() {},
      jsutil: jsutil,
      talk1: talk1,
      store: store
    });
  });

}).call(this);
