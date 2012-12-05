### UTF-8 ＠ぱすた＠ ###
#----------------------------------------------------------------------
### アーリオ：会話辞書管理 
管理単位：item
            name: トーク名
            talk: トーク本体
###
#----------------------------------------------------------------------
define ["jquery"], () ->

#---# --------------------------------------------------------------
    ### １節 ###
    class Knot
        constructor: (@type, args) ->
            return if not args?
            for key, value of args
                @[key] = value
            

#---# --------------------------------------------------------------
    ### １節の追加 ###
    addKnot = (aglio, type, args) ->
        item = new Knot(type, args)
        aglio.yarn.push item
        index = aglio.yarn.length - 1
        item.index = index
        item


#---# --------------------------------------------------------------
    ### 量子記憶 ###
    addQuantum = (aglio, knot) ->
        index    = knot.index
        quantums = knot.quantums

        el = $("<p/>")
        el.addClass key for key in quantums
        el.attr
            'data-index': index
        el.appendTo aglio.quantumElements
        el
        


#---# --------------------------------------------------------------
    ### 会話辞書登録管理 ###
    class Aglio
#---# --------------------------------------------------------------
        constructor: () ->
            console?.log "[Aglio::constructor]"
            ### 記憶の糸 ###
            @yarn       = []

            ### 量子情報 ###
            $("<div>")
                .attr( id: "quantumElements" )
                .css( display: "none" )
                .appendTo("body")
            @quantumElements = $("div#quantumElements")[0]

    
        ### 会話確定 ###
        end: ->
            addKnot(@, "end")
            @
            

        ### 会話登録開始：量子状態リスト ###
        quantum: (quantum...) ->
            @end()
            addQuantum @, addKnot(@, "quantum", quantums: quantum)
            @
  
  
        ### １文登録 ###
        sentence: (actor, emotion, speech) ->
            addKnot(@, "sentence",
                actor   :actor
                emotion :emotion
                speech  :speech
            )
            @

        ### セッション区切り ###
        sepSection: ->
            addKnot(@, "sepSection")
            @
        $_______: -> @sepSection()


        ### トーク区切り ###
        sepTalk: ->
            addKnot(@, "sepTalk")
            @
        $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$: -> @sepTalk()


        ### エンタングル。ランダム分岐。
            指定された量子状態を取るいずれかの会話に分岐する
        ###
        entangle: (quantumState) ->
            addKnot(@, "entangle",
                quantumState: quantumState
            )
            @

            
        ### ランダム終了 ###
        ifend: (terms) ->
            addKnot(@, "terms",
                terms: terms
            )
            @

        ### ヘルパー関数：ランダム選択 ###
        random: (items) -> items.random()
 
#---# --------------------------------------------------------------
    Aglio.instance = new Aglio
    Aglio
