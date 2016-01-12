
/* UTF-8 試験コード
*/


(function() {

  define(["engine/aglio_dic", "engine/olio_story_picker", "test/testTalk", "jquery"], function(dic, Picker) {
    return {
      test1: function() {
        var picker, scrap, seq, story;
        picker = new Picker(dic);
        story = picker.nextStory(["tag3"]);
        Assert.that(story, Is.not.empty());
        seq = story.next();
        Assert.that(seq, Is.not.empty());
        seq.closeScrap();
        seq.run();
        scrap = dic.selectScrap(["tag3", "tag2"]);
        return Assert.that(scrap, Is.empty());
      },
      deferredTest1: function() {
        var dfd, doneCount, failCount;
        dfd = $.Deferred();
        doneCount = 0;
        failCount = 0;
        dfd.done(function() {
          return doneCount++;
        });
        dfd.fail(function() {
          return failCount++;
        });
        dfd.resolve();
        dfd.resolve();
        dfd.reject();
        Assert.that(doneCount, Is.equalTo(1));
        return Assert.that(failCount, Is.equalTo(0));
      },
      deferredTest2: function() {
        var dfd, doneCount, failCount;
        dfd = $.Deferred();
        doneCount = 0;
        failCount = 0;
        dfd.done(function() {
          return doneCount++;
        });
        dfd.fail(function() {
          return failCount++;
        });
        dfd.reject();
        dfd.resolve();
        dfd.resolve();
        dfd.reject();
        Assert.that(doneCount, Is.equalTo(0));
        return Assert.that(failCount, Is.equalTo(1));
      }
    };
  });

}).call(this);
