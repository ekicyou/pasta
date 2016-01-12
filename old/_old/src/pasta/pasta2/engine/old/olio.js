
/* UTF-8 ＠ぱすた＠
*/


/* オリオ：会話シーケンサ 
辞書より会話を構成する。呼び出される毎に会話の１節を返す。
状態はessenceに保持する
*/


(function() {

  define(["scripts/aglio", "scripts/jsutil"], function(Aglio) {
    /* オブジェクトが関数ならtrue
    */

    var Olio, isFunc;
    isFunc = function(obj) {
      return typeof obj === "function";
    };
    /* 会話シーケンサ
    */

    Olio = (function() {
      /* コンストラクタ
            @aglio     : 辞書
            @stateFunc : 会話開始時に無条件指定される状態指定関数
            @essence   : 遷移を保持するオブジェクト
      */

      function Olio(aglio, stateFunc, essence) {
        this.aglio = aglio;
        this.stateFunc = stateFunc;
        this.essence = essence;
        this.yarn = this.aglio.yarn;
        this.quantum = this.aglio.quantum;
        if (!(this.essence != null)) {
          this.essence = {};
        }
        this.essence.index = -1;
        if (!(this.essence.state != null)) {
          this.essence.state = {};
        }
        if (!(this.essence.state.end != null)) {
          this.essence.state.end = {};
        }
        if (!(this.essence.state.entangle != null)) {
          this.essence.state.entangle = {};
        }
      }

      /* 次の１節を取得します。
      */


      Olio.prototype.next = function() {
        var index, last, lastIndex, type;
        lastIndex = this.essence.index;
        last = lastIndex >= this.yarn.length ? null : lastIndex < 0 ? null : this.yarn[lastIndex];
        type = last != null ? last.type : void 0;
        index = (function() {
          switch (type) {
            case "entangle":
              return this.entangle(this.essence.state.entangle, last.quantumState);
            case "end":
              return this.entangle(this.essence.state.end, this.stateFunc);
          }
        }).call(this);
        if (!(index != null)) {
          index = lastIndex + 1;
        }
        if (index >= this.yarn.length) {
          index = 0;
        }
        this.essence.index = index;
        return this.essence.knot = this.yarn[index];
      };

      /* 量子ジャンプ。
      */


      Olio.prototype.entangle = function(st, quantumState) {
        var item, items, key;
        key = isFunc(quantumState) ? quantumState(this.essence) : quantumState;
        items = this.quantum[key];
        if (!(items != null)) {
          return void 0;
        }
        item = items.random();
        st.item = item;
        return item.index;
      };

      return Olio;

    })();
    return Olio;
  });

}).call(this);
