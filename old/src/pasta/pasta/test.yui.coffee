### UTF-8 ＠ユニットテスト＠　起動 ###

define [
    "engine/jarvisutil"
    "test/jsutil"
    "test/talk1"
], (Run, jsutil, talk1) ->
#---# --------------------------------------------------------------
    ### 試験データ ###
#---# --------------------------------------------------------------
    $.holdReady false
    Run
        name: "AllTest"
        tearDown: ->
        setup: ->
        jsutil: jsutil
        talk1: talk1
