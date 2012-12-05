
/* UTF-8 ＠ぱすた＠　UnitTest便利関数
*/


(function() {

  define(["jarvis"], function() {
    /* 属性チェック関数
    */

    var Run, isArray, isFunc, rep, toJarvis, toJarvisImpl;
    isFunc = function(o) {
      return typeof o === "function";
    };
    isArray = function(o) {
      return Object.prototype.toString.call(o) === '[object Array]';
    };
    /* json定義をJarvis用に変換
    */

    /* http://jarvis.tmont.com/
    */

    toJarvisImpl = function(testObj) {
      var key, rc, test, value;
      rc = {};
      if (testObj.name != null) {
        rc.name = testObj.name;
      }
      if (testObj.setup != null) {
        rc.setup = testObj.setup;
      }
      if (testObj.tearDown != null) {
        rc.tearDown = testObj.tearDown;
      }
      test = [];
      for (key in testObj) {
        value = testObj[key];
        switch (key) {
          case "name":
            rc.name = value;
            break;
          case "setup":
            rc.setup = value;
            break;
          case "tearDown":
            rc.tearDown = value;
            break;
          default:
            if (isFunc(value)) {
              value.testName = key;
              test.push(value);
            } else {
              value.name = key;
              test.push(toJarvis(value));
            }
        }
      }
      rc.test = function() {
        return test;
      };
      return rc;
    };
    toJarvis = function(tree) {
      return toJarvisImpl(tree);
    };
    if (true) {
      rep = new Jarvis.Framework.Reporters.HtmlReporter;
      rep.collapsedByDefault = true;
    } else {
      rep = new Jarvis.Framework.Reporters.ConsoleReporter;
    }
    return Run = function(json) {
      return Jarvis.run(toJarvis(json), rep);
    };
  });

}).call(this);
