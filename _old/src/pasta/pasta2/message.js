
/* UTF-8 メッセージ確認用
*/


(function() {

  define(["jquery"], function() {
    /* 試験データ
    */
    $.holdReady(false);
    return $(function() {
      var receiveMessage;
      receiveMessage = function(ev) {
        return $("#parse-message").text(ev.data);
      };
      return window.addEventListener("message", receiveMessage, false);
    });
  });

}).call(this);
