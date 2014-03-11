(function() {

  define(["rx.jquery", "iso8601"], function() {
    /* jQueryのReady発動を抑制。
    */

    var checkBrowser, html, isb, randomArray, sleepCheckFunc, sleepCheckLastTimestamp, sleepCheckTimeoutId, sleepWait, _base, _ref;
    $.holdReady(true);
    /* Array拡張：配列をマップします。
    */

    if (!(Array.prototype.map != null)) {
      Array.prototype.map = function(func) {
        var item, _i, _len, _results;
        _results = [];
        for (_i = 0, _len = this.length; _i < _len; _i++) {
          item = this[_i];
          _results.push(func(item));
        }
        return _results;
      };
    }
    /* Array拡張：配列をシャッフルします。
    */

    randomArray = function(array, num) {
      var a, i, l, n, r, t;
      a = array;
      t = [];
      r = [];
      l = a.length;
      n = num < l ? num : l;
      while (n-- > 0) {
        i = Math.random() * l | 0;
        r[n] = t[i] || a[i];
        --l;
        t[i] = t[l] || a[l];
      }
      return r;
    };
    if (!(Array.prototype.shuffle != null)) {
      Array.prototype.shuffle = function() {
        return randomArray(this, this.length);
      };
    }
    /* Array拡張：配列を重複せずに無限ランダム抽出するイテレータ。
    */

    Array.prototype.random_iter_loop = function() {
      var a, index,
        _this = this;
      index = 0;
      a = [];
      return {
        next: function() {
          index++;
          if (index >= a.length) {
            index = 0;
            a = _this.shuffle();
          }
          return a[index];
        }
      };
    };
    /* Array拡張：配列からランダムに要素を一つ取り出す。
    */

    Array.prototype.random = function() {
      if (this.length > 0) {
        return this[Math.random() * this.length | 0];
      }
      return void 0;
    };
    /* コンソールの存在を騙す
    */

    if (!(window.console != null)) {
      window.console = {
        log: function() {}
      };
    }
    /* 簡易ブラウザ判定
    */

    checkBrowser = function() {
      var appVersion, userAgent;
      userAgent = window.navigator.userAgent.toLowerCase();
      if (userAgent.indexOf('trident') !== -1) {
        return 'ie11';
      }
      if (userAgent.indexOf('opera') !== -1) {
        return 'opera';
      }
      if (userAgent.indexOf('msie') !== -1) {
        appVersion = window.navigator.appVersion.toLowerCase();
        if (appVersion.indexOf("msie 6.") !== -1) {
          return 'ie6';
        }
        if (appVersion.indexOf("msie 7.") !== -1) {
          return 'ie7';
        }
        if (appVersion.indexOf("msie 8.") !== -1) {
          return 'ie8';
        }
        if (appVersion.indexOf("msie 9.") !== -1) {
          return 'ie9';
        }
        if (appVersion.indexOf("msie 10.") !== -1) {
          return 'ie10';
        }
        if (appVersion.indexOf("msie 11.") !== -1) {
          return 'ie11';
        }
        if (appVersion.indexOf("msie 12.") !== -1) {
          return 'ie12';
        }
        if (appVersion.indexOf("msie 13.") !== -1) {
          return 'ie13';
        }
        if (appVersion.indexOf("msie 14.") !== -1) {
          return 'ie14';
        }
        if (appVersion.indexOf("msie 15.") !== -1) {
          return 'ie15';
        }
        return 'ie';
      }
      if (userAgent.indexOf('chrome') !== -1) {
        return 'chrome';
      }
      if (userAgent.indexOf('safari') !== -1) {
        return 'safari';
      }
      if (userAgent.indexOf('gecko') !== -1) {
        return 'gecko';
      }
      return 'unnone_browser';
    };
    isb = "is_" + checkBrowser();
    window[isb] = true;
    if (window.is_chrome != null) {
      window.is_webkit = true;
    }
    if (window.is_safari != null) {
      window.is_webkit = true;
    }
    window.is_ie_version = 0;
    if (window.is_ie6 != null) {
      window.is_ie_version = 6;
    }
    if (window.is_ie7 != null) {
      window.is_ie_version = 7;
    }
    if (window.is_ie8 != null) {
      window.is_ie_version = 8;
    }
    if (window.is_ie9 != null) {
      window.is_ie_version = 9;
    }
    if (window.is_ie10 != null) {
      window.is_ie_version = 10;
    }
    if (window.is_ie11 != null) {
      window.is_ie_version = 11;
    }
    if (window.is_ie12 != null) {
      window.is_ie_version = 12;
    }
    if (window.is_ie13 != null) {
      window.is_ie_version = 13;
    }
    if (window.is_ie14 != null) {
      window.is_ie_version = 14;
    }
    if (window.is_ie15 != null) {
      window.is_ie_version = 15;
    }
    html = $("html");
    if (window.is_webkit) {
      html.addClass("webkit");
    }
    if (window.is_ie6) {
      html.addClass("ie6");
    }
    if (window.is_ie7) {
      html.addClass("ie7");
    }
    if (window.is_ie8) {
      html.addClass("ie8");
    }
    if (window.is_ie9) {
      html.addClass("ie9");
    }
    if (window.is_ie10) {
      html.addClass("ie10");
    }
    if (window.is_ie11) {
      html.addClass("ie11");
    }
    if (window.is_ie12) {
      html.addClass("ie12");
    }
    if (window.is_ie13) {
      html.addClass("ie13");
    }
    if (window.is_ie14) {
      html.addClass("ie14");
    }
    if (window.is_ie15) {
      html.addClass("ie15");
    }
    if (window.is_ie_version >= 8) {
      html.addClass("ie8_");
    }
    if (window.is_ie_version >= 9) {
      html.addClass("ie9_");
    }
    /* ieかどうかでテキスト用のfade/show関数を差し替える
    */

    if (window.is_webkit) {
      $.fn.extend({
        textHide: function() {
          return this.hide();
        },
        textShow: function(ms) {
          return this.show(ms);
        }
      });
    } else {
      $.fn.extend({
        textHide: function() {
          return this.fadeTo(0, 0.0);
        },
        textShow: function(ms) {
          return this.fadeTo(ms, 1.0);
        }
      });
    }
    /* 割り込み関数の差し替え
    */

    if (!(window.setImmediate != null)) {
      window.setImmediate = function(func, args) {
        return window.setTimeout(func, 0, args);
      };
      window.clearImmediate = window.clearTimeout;
    }
    if (!(window.requestAnimationFrame != null)) {
      if (window.msRequestAnimationFrame != null) {
        window.requestAnimationFrame = window.msRequestAnimationFrame;
        window.cancelAnimationFrame = window.msCancelAnimationFrame;
      }
      if (window.webkitRequestAnimationFrame != null) {
        window.requestAnimationFrame = window.webkitRequestAnimationFrame;
        window.cancelAnimationFrame = window.webkitCancelAnimationFrame;
      }
      if (window.mozRequestAnimationFrame != null) {
        window.requestAnimationFrame = window.mozRequestAnimationFrame;
        window.cancelAnimationFrame = window.mozCancelAnimationFrame;
      }
    }
    /* jQueryオブジェクトのプラグイン
    */

    $.fn.extend({
      rx$load: function(data) {
        return this.loadAsObservable('load', data);
      },
      rx$click: function(selector, data) {
        return this.onAsObservable('click', selector, data);
      },
      rx$message: function(selector, data) {
        return this.onAsObservable('message', selector, data);
      },
      rx$resize: function(selector, data) {
        return this.onAsObservable('resize', selector, data);
      },
      rx$liveclick: function(data) {
        return this.liveAsObservable('click', data);
      }
    });
    /* jQueryのプラグイン
    */

    $.extend({
      /* immediateの契約を返す
      */

      immediate: function() {
        var dfd, id;
        dfd = $.Deferred();
        id = window.setImmediate((function() {
          return dfd.resolve(0);
        }));
        dfd.always(function() {
          return window.clearImmediate(id);
        });
        return dfd.promise();
      },
      /* timeoutの契約を返す
      */

      timeout: function(time) {
        var dfd, id;
        if (time === 0) {
          return $.immediate();
        }
        dfd = $.Deferred();
        id = setTimeout((function() {
          return dfd.resolve(time);
        }), time);
        dfd.always(function() {
          return clearTimeout(id);
        });
        return dfd.promise();
      },
      /* timestampの契約を返す
              timestamp: 発動希望時刻
              isSleepWait: trueなら、間にスリープが入った場合に発動を遅らせる
      */

      timestamp: function(timestamp, isSleepWait) {
        var dfd, func, id, startWait;
        if (isSleepWait == null) {
          isSleepWait = true;
        }
        dfd = $.Deferred();
        startWait = sleepWait;
        id = void 0;
        dfd.always(function() {
          if (id != null) {
            return clearTimeout(id);
          }
        });
        func = function() {
          var now, sleep, span;
          sleep = isSleepWait ? sleepWait - startWait : 0;
          now = Date.now() - sleep;
          span = timestamp - now;
          if (span <= 0) {
            dfd.resolve({
              timestamp: timestamp,
              sleep: sleep
            });
          } else {
            id = setTimeout(func, span);
          }
        };
        func();
        return dfd.promise();
      }
    });
    /* スリープ時間の計測
    */

    sleepWait = 0;
    sleepCheckLastTimestamp = Date.now();
    sleepCheckTimeoutId = void 0;
    sleepCheckFunc = function() {
      var now, span;
      now = Date.now();
      span = now - sleepCheckLastTimestamp;
      if (span > 600) {
        sleepWait += span - 200;
      }
      sleepCheckLastTimestamp = now;
      clearTimeout(sleepCheckTimeoutId);
      return sleepCheckTimeoutId = setTimeout(sleepCheckFunc, 200);
    };
    sleepCheckFunc();
    /* jQuery:easeInQuadが使えないとき用
    */

    if ((_ref = (_base = $.easing).easeInQuad) == null) {
      _base.easeInQuad = function(p) {
        return p * p;
      };
    }
  });

}).call(this);
