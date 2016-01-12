
/* UTF-8 試験コード
*/


(function() {

  define(["engine/store"], function(CacheStore) {
    return {
      storeTest1: function() {
        var Filter, allItems, filter_a, filter_b, items_a, items_b, store;
        allItems = [
          {
            tags: ["a"],
            value: "item1"
          }, {
            tags: ["b"],
            value: "item2"
          }, {
            tags: ["c"],
            value: "item3"
          }, {
            tags: ["a", "c"],
            value: "item4"
          }, {
            tags: ["a", "b"],
            value: "item5"
          }
        ];
        Assert.that(allItems.length, Is.equalTo(5));
        store = new CacheStore(function() {
          return allItems;
        });
        Filter = (function() {

          function Filter(key) {
            this.key = key;
          }

          Filter.prototype.func = function(items) {
            var a, _i, _len, _results;
            _results = [];
            for (_i = 0, _len = items.length; _i < _len; _i++) {
              a = items[_i];
              if ((a.tags.indexOf(this.key)) >= 0) {
                _results.push(a);
              }
            }
            return _results;
          };

          return Filter;

        })();
        filter_a = new Filter("a");
        filter_b = new Filter("b");
        items_a = store.get([filter_a]);
        if (typeof console !== "undefined" && console !== null) {
          console.log(items_a);
        }
        Assert.that(items_a.length, Is.equalTo(3));
        Assert.that(items_a[0].value, Is.equalTo("item1"));
        Assert.that(items_a[1].value, Is.equalTo("item4"));
        Assert.that(items_a[2].value, Is.equalTo("item5"));
        items_b = store.get([filter_a, filter_b]);
        Assert.that(items_b.length, Is.equalTo(1));
        return Assert.that(items_b[0].value, Is.equalTo("item5"));
      }
    };
  });

}).call(this);
