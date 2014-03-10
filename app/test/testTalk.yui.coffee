### UTF-8 　会話辞書１ ###
"use strict";

define [
    "engine/aglio_dic",
], (dic) ->
    story = ->
        title = "testStory1"
        dic.scrap [title, "tag1", "tag2"], (essence) ->
            @section()
            @period "pasta"
            @emote  "pasta.1"
            @talk   "pasta.talk1."
            @talk   @popWord ["wordKey1"]
            @talk   "pasta.talk2."
            @br()
            @talk   "pasta.talk3."

            @section()
            @period "solt"
            @emote  "solt.1"
            @talk   "solt.talk4."
        dic.scrap [title+"#1"], (essence) ->
            @section()
            @period "pasta"
            @emote  "pasta.2"
            @talk   "pasta.talk5."
            @close()
    story()


    dic.word ["wordKey1"], "word1"



    story = ->
        title = "testStory2"
        dic.scrap [title, "tag1", "tag3"], (essence) ->
            @section()
            @period "solt"
            @emote  "solt.1"
            @talk   "solt.talk61."
            @br()
            @talk   "solt.talk62."
            @period "pasta"
            @emote  "pasta.0"
            @talk   "今日はいい天気です、‥‥"
            @emote  "pasta.4"
            @talk   "あれ？曇ってきた‥‥"
            @emote  "pasta.1"
            @talk   "かあ。"
            @jump   ["tag4"]
    story()


    story = ->
        title = "testStory3"
        dic.scrap [title, "tag4", "tag1"] , (essence)->
            @section()
            @period "solt"
            @emote  "solt.e2"
            @talk   "solt.talk7"

        dic.scrap [title+"#1"], (essence) ->
            @section()
            @period "pasta"
            @emote  "pasta.e2"
            @talk   "pasta.talk8"
            @close()
    story()

