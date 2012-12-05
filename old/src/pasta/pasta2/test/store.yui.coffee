### UTF-8 試験コード ###
define [
    "engine/store"
], (CacheStore) ->
    storeTest1: ->
        # ストアの作成
        allItems = [
            {tags: ["a"]    , value: "item1"}
            {tags: ["b"]    , value: "item2"}
            {tags: ["c"]    , value: "item3"}
            {tags: ["a","c"], value: "item4"}
            {tags: ["a","b"], value: "item5"}
        ]
        Assert.that allItems.length, Is.equalTo 5
        store = new CacheStore () -> allItems

        # フィルタの作成
        class Filter
            constructor: (@key) ->
            func: (items)-> a for a in items when (a.tags.indexOf @key) >= 0


        filter_a = new Filter "a"
        filter_b = new Filter "b"

        # フィルタの試験1
        items_a = store.get [filter_a]
        console?.log items_a
        Assert.that items_a.length, Is.equalTo 3
        Assert.that items_a[0].value, Is.equalTo "item1"
        Assert.that items_a[1].value, Is.equalTo "item4"
        Assert.that items_a[2].value, Is.equalTo "item5"


        # フィルタの試験2
        items_b = store.get [filter_a,filter_b]
        Assert.that items_b.length, Is.equalTo 1
        Assert.that items_b[0].value, Is.equalTo "item5"
