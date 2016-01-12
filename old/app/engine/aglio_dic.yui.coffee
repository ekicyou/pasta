### UTF-8 ＠ぱすた＠ ###
#----------------------------------------------------------------------
### アーリオ：会話辞書管理 
管理単位：item
            name: トーク名
            talk: トーク本体
###
#----------------------------------------------------------------------
define ["jquery", "engine/jsutil"], () ->
    "use strict";

#---# --------------------------------------------------------------
    ### 属性チェック関数 ###
    isFunc  = (o) -> typeof(o) == "function"
    isArray = (o) -> Object::toString.call(o) == '[object Array]'

#---# --------------------------------------------------------------
    ### タグ領域の作成 ###
    createTagArea = (id) ->
        $("<div>")
            .attr( id: id )
            .css( display: "none" )
            .appendTo("body")
        $("div#"+id)[0]
        
    ### タグ領域の追加 ###
    addTagArea = (dic, tags, value) ->
        values = if isArray(value) then value else [value]
        for v in values
            el = $("<p/>")
            el.addClass tag for tag in tags
            el.attr {'data-value': v}
            el.appendTo dic
        return
        
#---# --------------------------------------------------------------
    ### スクラップ：ストーリーの１節 ###
    class Scrap
        constructor: (@tags, @callback) ->

#---# --------------------------------------------------------------
    ### 会話辞書登録管理 ###
    class AglioDic
#-------# ----------------------------------------------------------
        ### コンストラクタ ###
        constructor: () ->
            ### 記憶の糸 ###
            @scraps = []

            ### タグ領域 ###
            @scrapTags = createTagArea("scrapTags")
            @wordTags = createTagArea("wordTags")
        

#-------# ----------------------------------------------------------
        ### スクラップ登録 ###
        scrap: (tags, callback) ->
            scrap = @createScrap tags, callback
            @scraps.push scrap
            index = @scraps.length - 1
            scrap.index = index
            scrap.next = => @getScrap(index+1)
            addTagArea @scrapTags, tags, index
            return

        createScrap: (tags, callback) -> new Scrap(tags, callback)

        getScrap: (index) ->
            index = 0 if index >= @scraps.length 
            @scraps[index]

#-------# ----------------------------------------------------------
        ### 単語登録 ###
        word: (tags, values) ->
            addTagArea @wordTags, tags, values

#-------# ----------------------------------------------------------
        ### タグがすべて含まれたスクラップエレメントの検索 ###
        getScrapElement: (tags) ->
            selector = "div#scrapTags p"
            selector += "." + tag for tag in tags
            $(selector)

#-------# ----------------------------------------------------------
        ### スクラップ選択 ###
        selectScrap: (tags) ->
            $$ = @getScrapElement tags
            return undefined if $$.length == 0
            $i = Math.random() * $$.length | 0
            index = $$.eq($i).data("value")
            @scraps[index]

#---# --------------------------------------------------------------
    AglioDic.instance = new AglioDic
