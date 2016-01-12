
/* UTF-8 ＠ぱすた＠
*/


(function() {
  var __slice = [].slice;

  define(["scripts/jsutil", "modernizr"], function() {
    /* プッタネスカ：トークアニメーション処理
    */

    /* スペースを削除する関数
    */

    var Puttanesca, createTalkLine, createTalkLines, delSpace, delay, endChars, endWait1Chars, endWait1MS, endWait2Chars, endWait2MS, endWait3Chars, endWait3MS, hangChars, isStartEndType, isWaitType, noWaitMS, runTalkAndNext, spanWaitMS, startChars, strSplit, toWaitChars;
    delSpace = function(str) {
      return str;
    };
    /* 禁則文字列
    */

    startChars = "（［｛「『([{｢";
    endChars = "、。，．・？！゛゜ヽヾゝゞ々）］｝」』!),.:;?]}｡｣､･ﾞﾟ‥…";
    endWait1Chars = delSpace("、  ，                                  ,        ､       ");
    endWait2Chars = delSpace("            ！                        !                 ");
    endWait3Chars = delSpace("  。  ．・？                               .?  ｡  ･  ‥…");
    hangChars = "、。，．,.";
    noWaitMS = 20;
    endWait1MS = 50;
    endWait2MS = 100;
    endWait3MS = 150;
    spanWaitMS = 200;
    /* プッタネスカ：文字タイプの判定
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
          return 1;
        case "end1":
          return 1;
        case "end2":
          return 2;
        case "end3":
          return 3;
        default:
          return 0;
      }
    };
    /*
        ・startCharsだったとき：次の文字と合成する
        ・endCharsだったとき：直前の文字と合成する
    */

    /* 文字列を文字配列に変換
    */

    strSplit = function(str) {
      var c, _i, _len, _results;
      _results = [];
      for (_i = 0, _len = str.length; _i < _len; _i++) {
        c = str[_i];
        _results.push(c);
      }
      return _results;
    };
    /* 文字配列にウェイトを合成。文字・数字・・・と続く
    */

    toWaitChars = function(chars) {
      var c, rc, sel, startEnd, wait, _i, _len, _results;
      rc = [];
      wait = 0;
      _results = [];
      for (_i = 0, _len = chars.length; _i < _len; _i++) {
        c = chars[_i];
        startEnd = "normal";
        sel = void 0;
        if (c === " ") {
          _results.push(startEnd = "normal");
        } else if (startChars.indexOf(c > -1)) {
          _results.push(startEnd = "start");
        } else if (endChars.indexOf(c > -1)) {
          startEnd = "end";
          if (endWait1Chars.indexOf(c > -1)) {
            _results.push(sel = "end1");
          } else if (endWait2Chars.indexOf(c > -1)) {
            _results.push(sel = "end2");
          } else if (endWait3Chars.indexOf(c > -1)) {
            _results.push(sel = "end3");
          } else {
            _results.push(void 0);
          }
        } else {
          _results.push(void 0);
        }
      }
      return _results;
    };
    /* プッタネスカ：エレメントにトーク１行作成して、情報を返す
    */

    createTalkLine = function(el, line) {
      var c, span, _i, _len, _ref;
      span = void 0;
      _ref = strSplit(line);
      for (_i = 0, _len = _ref.length; _i < _len; _i++) {
        c = _ref[_i];
        span = $("<span />").appendTo(el);
        span.text(c);
        span.attr({
          "data-w": isWaitType(c)
        });
      }
      if (span != null) {
        span.attr({
          "data-w": 4
        });
      }
    };
    /* プッタネスカ：エレメントにトーク行を作成して、情報を返す
    */

    createTalkLines = function() {
      var classNames, el, isFirst, line, lines, p, _i, _len;
      el = arguments[0], classNames = arguments[1], lines = 3 <= arguments.length ? __slice.call(arguments, 2) : [];
      p = $("<p />").appendTo(el);
      p.addClass(classNames);
      isFirst = true;
      for (_i = 0, _len = lines.length; _i < _len; _i++) {
        line = lines[_i];
        if (isFirst) {
          isFirst = false;
        } else {
          $("<br />").appendTo(p);
        }
        createTalkLine(p, line);
      }
    };
    /* プッタネスカ：遅延タスク作成
    */

    delay = function(ms) {
      return Rx.Observable.timer(ms);
    };
    /* プッタネスカ：トーク発動処理
    */

    runTalkAndNext = function(endFunc) {
      var acts, baseWait, cd, charTask, end, endTask, lastWaitType, lineTask, ms, oldMS, talkResult;
      $("div#mainArea").removeClass("show");
      ms = 0;
      oldMS = 0;
      acts = [];
      lastWaitType = -1;
      baseWait = 120;
      cd = new Rx.CompositeDisposable;
      talkResult = "break";
      charTask = function() {
        var $$, addWait, nowWaitType, rc, show, text, timerTask, waitType;
        $$ = $(this);
        waitType = $$.data("w");
        nowWaitType = lastWaitType < 0 ? (lastWaitType = waitType, 4) : waitType > 0 ? (waitType > lastWaitType ? lastWaitType = waitType : void 0, 0) : (rc = lastWaitType, lastWaitType = 0, rc);
        addWait = (function() {
          switch (nowWaitType) {
            case 1:
              return baseWait * 2;
            case 2:
              return baseWait * 3;
            case 3:
              return baseWait * 4;
            case 4:
              return baseWait * 6;
            default:
              return 0;
          }
        })();
        ms += addWait;
        text = $$.text();
        lastWaitType = waitType;
        $$.fadeTo(0, 0.001);
        show = Rx.Disposable.create(function() {
          return $$.fadeTo(300, 1.0);
        });
        cd.add(show);
        timerTask = delay(ms).subscribe(function() {
          return show.dispose();
        });
        cd.add(timerTask);
        $$.attr({
          "data-span": ms - oldMS,
          "data-ms": ms
        });
        oldMS = ms;
        return ms += baseWait;
      };
      lineTask = function() {};
      $("#mainArea section").each(function() {
        $("p", this).each(function() {
          lastWaitType = -1;
          return $("span", this).each(charTask);
        });
        return ms += baseWait * 3;
      });
      cd.add(end = Rx.Disposable.create(function() {
        return endFunc(talkResult);
      }));
      endTask = Rx.Observable.timer(ms).subscribe(function() {
        talkResult = "end";
        return cd.dispose();
      });
      cd.add(endTask);
      cd.add(Rx.Observable.timer(0).subscribe(function() {
        return $("div#mainArea").addClass("show");
      }));
      return cd;
    };
    /* プッタネスカ：IEかどうかでimg/imageを切り替える
    */

    $(document).ready(function() {
      if (window.is_webkit) {
        return $("#actor div.webkit").css({
          display: "block"
        });
      } else {
        return $("#actor div.ie").css({
          display: "block"
        });
      }
    });
    /* プッタネスカ：初期化処理 ViewModelのバインドを行う
    */

    Puttanesca = (function() {
      /* コンストラクタ
      */

      function Puttanesca() {}

      /* 試験用ロード処理
      */


      Puttanesca.prototype.loadTest = function() {
        var bb, doc, runTalk, text1, text2;
        if (typeof console !== "undefined" && console !== null) {
          console.log("[Puttanesca::loadTest]");
        }
        doc = document;
        text1 = $("#text1 div").empty();
        createTalkLines(text1, "a2 t1", "パスタさん、今日も暑いですね。");
        createTalkLines(text1, "a1 t1", "そーですねえ。");
        createTalkLines(text1, "a2 t1", "汗かかない？一枚脱ぐ？");
        text2 = $("#text2 div").empty();
        createTalkLines(text2, "a1 t1", "それって、逆に汗かきませんか？");
        createTalkLines(text2, "a2 t1", "・・・・・・・・。");
        bb = void 0;
        runTalk = function() {
          return bb = runTalkAndNext(function() {
            return typeof console !== "undefined" && console !== null ? console.log("トーク終了！") : void 0;
          });
        };
        runTalk();
        $("#actor").rx$liveclick().throttle(300).subscribe(function(ev) {
          if (typeof console !== "undefined" && console !== null) {
            console.log("おでこクリック！", ev.target.outerHTML);
          }
          if (bb.isDisposed()) {
            return runTalk();
          } else {
            return bb.dispose();
          }
        });
      };

      return Puttanesca;

    })();
    return Puttanesca;
  });

}).call(this);
