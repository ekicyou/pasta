
/* UTF-8 ＠ぱすた＠　起動
*/


(function() {

  define(["animation", "dic/talk1", "engine/pasta", "engine/parsers/talk.min", "jquery-ui", "jquery.formtips"], function() {
    $(window).ready(function() {
      var ConvertKeyword, ConvertStory, RunStory, dispatcher, keyTask, sels;
      dispatcher = window.pasta.dispatcher;
      sels = $("div.navi ul");
      if (sels.length > 0) {
        sels.selectable({
          filter: "li",
          selected: function(e, ui) {
            var el, k, kv, surface, v;
            el = $(ui.selected);
            surface = el.data("surface");
            k = surface[0];
            v = surface[1];
            kv = {};
            kv[k] = v;
            console.log("selected", [surface, kv, e, ui]);
            return window.emotePasta(kv);
          }
        });
        sels.selectable("enable");
      }
      keyTask = {
        "おじさん": 'this.P("mister p1");',
        "パスタ": 'this.P("pasta p1");',
        "ノーマル": 'this.E("パスタ：ノーマル");',
        "よそみ": 'this.E("パスタ：よそみ");',
        "ちらっ": 'this.E("パスタ：ちらっ");',
        "めとじ": 'this.E("パスタ：めとじ");',
        "うわのそら": 'this.E("パスタ：うわのそら");',
        "えっ？": 'this.E("パスタ：えっ？");',
        "ジトー": 'this.E("パスタ：ジトー");',
        "えへっ": 'this.E("パスタ：えへっ");',
        "ほほえみ": 'this.E("パスタ：ほほえみ");',
        "まばたき": 'this.E("パスタ：まばたき");'
      };
      ConvertKeyword = function(keyword) {
        var jsonstr;
        if (keyTask[keyword] != null) {
          return keyTask[keyword];
        }
        jsonstr = JSON.stringify(value);
        return "this.W(" + jsonstr + ");";
      };
      ConvertStory = function(ast) {
        var area, isOutputSection, line, rc, src, tp, value, _i, _j, _len, _len1;
        rc = ['callback = function(essence) {'];
        area = 0;
        isOutputSection = true;
        for (_i = 0, _len = ast.length; _i < _len; _i++) {
          line = ast[_i];
          if (line.length < 2) {
            isOutputSection = true;
            continue;
          }
          if (isOutputSection) {
            isOutputSection = false;
            area++;
            rc.push('  this.S("area' + area + '")');
          }
          for (_j = 0, _len1 = line.length; _j < _len1; _j++) {
            value = line[_j];
            tp = value[0];
            value = value.substr(1);
            src = (function() {
              switch (tp) {
                case "B":
                  return '  this.B();';
                case "@":
                  return '  ' + ConvertKeyword(value);
                case "T":
                  return '  this.T(' + (JSON.stringify(value)) + ');';
              }
            })();
            if (src != null) {
              rc.push(src);
            }
          }
        }
        rc.push('  this.Z();');
        rc.push('}');
        return rc.join("\n");
      };
      RunStory = function(ast) {
        var callback, evalstr, seq, story;
        callback = null;
        evalstr = ConvertStory(ast);
        eval(evalstr);
        story = dispatcher.createSeq(["会話"], callback);
        seq = story.next();
        dispatcher.breakChain();
        dispatcher.start(seq);
      };
      $("#script_run").on("click", function(ev) {
        var ast, mes, text;
        text = $("#script_input").attr("value");
        mes = $("#parse-message");
        try {
          ast = window.pastaTalkParser.parse(text);
          RunStory(ast);
          mes.text("トーク開始！");
        } catch (ex) {
          mes.text(ex.message);
          throw ex;
        }
      });
    });
    return $.holdReady(false);
  });

}).call(this);
