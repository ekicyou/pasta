
/* UTF-8 ＠ぱすた＠　シェルアニメーション
*/


(function() {

  define(["engine/jsutil"], function() {
    return $(window).ready(function() {
      var animate, change, lastSurface, nowSurface, select, wink;
      nowSurface = {
        x: "0",
        y: "0",
        z: "0"
      };
      lastSurface = {
        x: "0",
        y: "0",
        z: "0"
      };
      animate = void 0;
      change = function(kv) {
        var changed, k, text, v;
        changed = false;
        for (k in kv) {
          v = kv[k];
          if (lastSurface[k] !== v) {
            changed = true;
            lastSurface[k] = v;
          }
        }
        if (!changed) {
          return;
        }
        text = "x" + lastSurface["x"];
        text += " y" + lastSurface["y"];
        text += " z" + lastSurface["z"];
        $("div#surface").attr("class", text);
      };
      wink = function(time) {
        var addAnimate;
        if (time == null) {
          time = 1000 + 19000 * Math.random();
        }
        if (nowSurface.y === "3") {
          return;
        }
        console.log("まばたき", [nowSurface.y]);
        if (animate != null) {
          animate.reject();
          animate = void 0;
        }
        if (animate != null) {
          return;
        }
        animate = $.Deferred();
        animate.done(function() {
          change(nowSurface);
          if (nowSurface.y === "3") {
            return;
          }
          return wink();
        });
        animate.progress(function(kv) {
          if (animate.state() !== "pending") {
            return;
          }
          if (kv != null) {
            change(kv);
          }
          if (!(kv != null) || kv.y === nowSurface.y) {
            animate.resolve();
          }
        });
        addAnimate = function(kv, time) {
          var id;
          id = setTimeout((function() {
            return animate.notify(kv);
          }), time);
          return animate.always(function() {
            return clearTimeout(id);
          });
        };
        addAnimate({
          y: "3",
          z: "1"
        }, time);
        addAnimate({
          y: "6",
          z: nowSurface.z
        }, time += 100);
        return addAnimate(void 0, time += 70);
      };
      select = function(kv) {
        var k, v;
        change(kv);
        for (k in kv) {
          v = kv[k];
          nowSurface[k] = v;
        }
        console.log("サーフェスチェンジ→", [kv, nowSurface]);
        if ((kv != null ? kv.y : void 0) === "4") {
          return wink(0);
        } else {
          return wink(1000 + 2000 * Math.random());
        }
      };
      $.extend({
        emote: {
          "パスタ：ノーマル": function() {
            return select({
              x: "0",
              y: "0",
              z: "0"
            });
          },
          "パスタ：よそみ": function() {
            return select({
              x: "0",
              y: "1",
              z: "0"
            });
          },
          "パスタ：ちらっ": function() {
            return select({
              x: "0",
              y: "2",
              z: "0"
            });
          },
          "パスタ：めとじ": function() {
            return select({
              x: "0",
              y: "3",
              z: "1"
            });
          },
          "パスタ：うわのそら": function() {
            return select({
              x: "0",
              y: "4",
              z: "0"
            });
          },
          "パスタ：えっ？": function() {
            return select({
              x: "1",
              y: "5",
              z: "0"
            });
          },
          "パスタ：ジトー": function() {
            return select({
              x: "0",
              y: "6",
              z: "0"
            });
          },
          "パスタ：えへっ": function() {
            return select({
              x: "2",
              y: "0",
              z: "0"
            });
          },
          "パスタ：ほほえみ": function() {
            return select({
              x: "2",
              y: "3",
              z: "0"
            });
          },
          "パスタ：まばたき": function() {
            return wink(0);
          }
        }
      });
      window.emotePasta = select;
      select(0);
    });
  });

}).call(this);
