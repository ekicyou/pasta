
/* UTF-8 試験コード
*/


(function() {

  define([], function() {
    return {
      ArrayMapTest: function() {
        var a, map;
        a = [1, 2, 3];
        map = a.map(function(x) {
          return x * 2;
        });
        return Assert.that(map, Is.equalTo([2, 4, 6]));
      }
    };
  });

}).call(this);
