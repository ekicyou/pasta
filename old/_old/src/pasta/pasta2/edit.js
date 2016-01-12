
/* UTF-8 ＠辞書編集＆試験＠　起動
*/


(function() {

  define(["jquery"], function() {
    /* 試験データ
    */
    $.holdReady(false);
    return $(function() {
      var actorFrame, actorWindow, receiveMessage;
      actorFrame = $("#actor iframe")[0];
      actorWindow = actorFrame.contentWindow;
      actorFrame.src = "message.html";
      receiveMessage = function(ev) {
        return actorWindow.postMessage(ev.data, "*");
      };
      return window.addEventListener("message", receiveMessage, false);
    });
  });

}).call(this);
