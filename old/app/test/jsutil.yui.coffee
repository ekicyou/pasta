### UTF-8 試験コード ###

define [], ->
    ArrayMapTest: ->
        a = [1,2,3]
        map = a.map (x)-> x*2
        Assert.that map, Is.equalTo [2,4,6]
