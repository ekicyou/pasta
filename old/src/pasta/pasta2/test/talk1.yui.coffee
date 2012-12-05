### UTF-8 試験コード ###
define [
    "engine/aglio_dic"
    "engine/olio_story_picker"
    "test/testTalk"
    "jquery"
], (dic, Picker) ->
    test1: ->
        picker = new Picker dic           # トーク切り出しエンジン
        story = picker.nextStory ["tag3"]
        Assert.that story , Is.not.empty()

        seq = story.next()
        Assert.that seq , Is.not.empty()
        seq.closeScrap()
        seq.run()

        scrap = dic.selectScrap ["tag3", "tag2"]
        Assert.that scrap, Is.empty()

    deferredTest1: ->
        dfd = $.Deferred()
        doneCount = 0
        failCount = 0
        dfd.done -> doneCount++
        dfd.fail -> failCount++

        dfd.resolve()
        dfd.resolve()
        dfd.reject()

        Assert.that doneCount, Is.equalTo 1
        Assert.that failCount, Is.equalTo 0


    deferredTest2: ->
        dfd = $.Deferred()
        doneCount = 0
        failCount = 0
        dfd.done -> doneCount++
        dfd.fail -> failCount++

        dfd.reject()
        dfd.resolve()
        dfd.resolve()
        dfd.reject()

        Assert.that doneCount, Is.equalTo 0
        Assert.that failCount, Is.equalTo 1



        

