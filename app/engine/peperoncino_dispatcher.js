
/* UTF-8 ＠ぱすた＠
*/


(function() {
  "use strict";

  define(["engine/aglio_dic", "engine/olio_story_picker", "engine/jsutil", "jquery"], function(aglio_dic, Picker) {
    /* ペペロンチーノ：イベントトリガ
    */

    var PeperoncinoDispatcher, PeperoncinoEvents;
    PeperoncinoEvents = (function() {

      function PeperoncinoEvents(parent) {
        this.parent = parent;
        this.picker = this.parent.picker;
        this.chain = void 0;
      }

      PeperoncinoEvents.prototype.start = function(seq) {
        return this.parent.start(seq);
      };

      /* 会話チェーンの中断
      */


      PeperoncinoEvents.prototype.breakChain = function() {
        var _ref;
        if ((_ref = this.chain) != null) {
          if (typeof _ref.reject === "function") {
            _ref.reject();
          }
        }
        return this.chain = void 0;
      };

      /* 起動。初回起動の時はfirstBoot、それ以外はnormalBootに分岐
      */


      PeperoncinoEvents.prototype.起動 = function() {
        return this.一般会話();
      };

      /* 通常起動。
      */


      PeperoncinoEvents.prototype.通常起動 = function() {};

      /* 初回起動。
      */


      PeperoncinoEvents.prototype.初回起動 = function() {};

      /* 会話。時刻トークがあるときは分岐
      */


      PeperoncinoEvents.prototype.会話 = function() {};

      PeperoncinoEvents.prototype.一般会話 = function() {
        var seq, _ref, _ref1;
        console.log("[PeperoncinoDispatcher::events::一般会話]");
        seq = (_ref = this.chain) != null ? _ref.next() : void 0;
        if (!(seq != null)) {
          if ((_ref1 = this.chain) != null) {
            if (typeof _ref1.resolve === "function") {
              _ref1.resolve();
            }
          }
          this.chain = this.picker.nextStory(["会話"]);
          console.log(["次の会話", this.chain]);
          seq = this.chain.next();
        }
        return this.start(seq);
      };

      PeperoncinoEvents.prototype.時刻00分 = function() {};

      return PeperoncinoEvents;

    })();
    /* ペペロンチーノ：イベント処理エンジン
    */

    return PeperoncinoDispatcher = (function() {
      /* コンストラクタ
      */

      function PeperoncinoDispatcher(dic) {
        this.dic = dic != null ? dic : aglio_dic;
        console.log("[PeperoncinoDispatcher::constructor]");
        this.order = {};
        this.order.sys = {
          dispTime: 15 * 1000
        };
        this.picker = new Picker(this.dic, this.order);
        this.events = new PeperoncinoEvents(this);
        try {
          this.events.起動();
        } catch (ex) {
          console.log(ex);
        }
      }

      /* 新しく作ったトーク（シーケンサ）の再生処理
      */


      PeperoncinoDispatcher.prototype.start = function(seq) {
        var _ref;
        if ((_ref = this.nowSeq) != null) {
          if (typeof _ref.close === "function") {
            _ref.close();
          }
        }
        this.nowSeq = seq;
        seq.closeScrap();
        $.timeout(500).done(function() {
          return seq.run();
        });
        seq.playedPromise().always(function() {
          return console.log("再生終了");
        });
        seq.closedPromise().always(function() {
          return console.log("表示終了");
        });
        return seq;
      };

      /* ストーリーより、直接シーケンサを作成
      */


      PeperoncinoDispatcher.prototype.createSeq = function(tags, callback) {
        var story;
        story = this.dic.createScrap(tags, callback);
        return this.picker.createSeq(story);
      };

      /* 会話再生中なら、再生終了状態に早送り
      */


      PeperoncinoDispatcher.prototype.played = function(args) {
        var _ref;
        if ((_ref = this.nowSeq) != null) {
          if (typeof _ref.played === "function") {
            _ref.played(args);
          }
        }
      };

      /* 会話再生中なら、表示クローズ
      */


      PeperoncinoDispatcher.prototype.close = function(args) {
        var _ref;
        if (((_ref = this.nowSeq) != null ? _ref.close : void 0) != null) {
          this.nowSeq.close(args);
          this.nowSeq = void 0;
        }
      };

      /* 会話チェーンの中断
      */


      PeperoncinoDispatcher.prototype.breakChain = function() {
        return this.events.breakChain();
      };

      /* 定期会話イベント：次の定期発動タイミングを計算して発動予約
      */


      PeperoncinoDispatcher.prototype.setNextSpanAction = function(act) {
        var a, next, nextMS, nextSec, now, nowSec, ratio, spanNextMS,
          _this = this;
        now = new Date;
        nowSec = now / 1000;
        a = now / this.spanMS | 0;
        nextMS = (a + 1) * this.spanMS;
        nextSec = nextMS / 1000;
        spanNextMS = (nextSec - nowSec) * 1000;
        ratio = spanNextMS / this.spanMS;
        if (ratio < 0.8) {
          nextMS = (a + 2) * this.spanMS;
        }
        next = new Date(nextMS);
        if (typeof console !== "undefined" && console !== null) {
          console.log("now  :", now);
        }
        if (typeof console !== "undefined" && console !== null) {
          console.log("next :", next);
        }
        this.nextSpanDisposable.disposable = Rx.Observable.timer(next - now).subscribe(function(ev) {
          _this.nextSpanDisposable.disposable = void 0;
          if (typeof console !== "undefined" && console !== null) {
            console.log("今！ :", new Date);
          }
          return act(ev);
        });
      };

      /* 定期会話イベント：処理の呼び出し
      */


      PeperoncinoDispatcher.prototype.actNextSpan = function(ev) {
        if (typeof console !== "undefined" && console !== null) {
          console.log("[PeperoncinoDispatcher::actNextSpan] ", ev);
        }
      };

      /* タッチトベント
      */


      PeperoncinoDispatcher.prototype.touch = function(points) {
        var _ref, _ref1;
        console.log("タッチ検出！", [points]);
        if (((_ref = this.nowSeq) != null ? typeof _ref.state === "function" ? _ref.state() : void 0 : void 0) === "played") {
          this.close();
          this.events.起動();
        } else if (((_ref1 = this.nowSeq) != null ? typeof _ref1.state === "function" ? _ref1.state() : void 0 : void 0) === "play") {
          this.played();
        } else {
          this.events.起動();
        }
      };

      return PeperoncinoDispatcher;

    })();
  });

}).call(this);
