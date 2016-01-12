### UTF-8 ＠ぱすた＠　UnitTest便利関数 ###

define [
    "jarvis"
], () ->

#---# --------------------------------------------------------------
    ### 属性チェック関数 ###
#---# --------------------------------------------------------------
    isFunc = (o)-> typeof(o) == "function"
    isArray = (o) -> Object::toString.call(o) == '[object Array]'

#---# --------------------------------------------------------------
    ### json定義をJarvis用に変換 ###
    ### http://jarvis.tmont.com/ ###
#---# --------------------------------------------------------------
    toJarvisImpl = (testObj) ->
        rc = {}
        rc.name     = testObj.name      if testObj.name?
        rc.setup    = testObj.setup     if testObj.setup?
        rc.tearDown = testObj.tearDown  if testObj.tearDown?
        test = []
        for key, value of testObj
            switch key
                when "name"     then rc.name     = value
                when "setup"    then rc.setup    = value
                when "tearDown" then rc.tearDown = value
                else
                    if isFunc value
                        value.testName = key
                        test.push value
                    else
                        value.name = key
                        test.push toJarvis value
        rc.test = -> test
        rc

    toJarvis = (tree) ->
        toJarvisImpl tree

    if true
        rep = new Jarvis.Framework.Reporters.HtmlReporter
        rep.collapsedByDefault = true
    else
        rep = new Jarvis.Framework.Reporters.ConsoleReporter

    Run = (json) -> Jarvis.run (toJarvis json), rep
  
     