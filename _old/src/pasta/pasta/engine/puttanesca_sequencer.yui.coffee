### UTF-8 ＠ぱすた＠ ###
"use strict";

define [
    "engine/jsutil"
    "jquery"
    "modernizr"
    ], (jsutil, Napoletano) ->

#---# --------------------------------------------------------------
    ### プッタネスカ：トークアニメーション処理  ###
#---# --------------------------------------------------------------
    
    ### スペースを削除する関数 ###
    delSpace = (str) -> str
    
    
    ### 禁則文字列 ###
    startChars    = "（［｛「『([{｢"
    
    endChars      =          "、。，．・？！゛゜ヽヾゝゞ々）］｝」』!),.:;?]}｡｣､･ﾞﾟ‥…"
    endWait1Chars = delSpace "、  ，                                  ,        ､       "
    endWait2Chars = delSpace "        ・                                        ･  ‥…"
    endWait3Chars = delSpace "            ！                        !                  "
    endWait4Chars = delSpace "  。  ．  ？                               .?  ｡         "
    
    hangChars     = "、。，．,."

    normalMS = 120
    waitMSTable =
        a       : normalMS * 1.0 # 通常ウエイト
        b       : normalMS * 3.0 # 半濁点ウエイト
        c       : normalMS * 1.5 # 「‥‥」ウエイト
        d       : normalMS * 3.5 # 「！」ウエイト
        e       : normalMS * 4.5 # 濁点ウエイト
        period  : normalMS * 5.0 # 段落ウエイト
        section : normalMS * 5.0 # セクションウェイト
        talk    : 20000          # トーク区切りウェイト
    showCharWait     = 300       # 文字が表示されるまでの時間
    sectionCloseWait = 400       # セクションを消す時のウェイト
        
#---# --------------------------------------------------------------
    ### 文字タイプの判定 ###
#---# --------------------------------------------------------------
    isStartEndType = (c) ->
        CHECK = (items)-> (items.indexOf c) > -1 
        if c == " "                     then "normal"
        else if CHECK startChars        then "start"
        else if CHECK endChars
            if      CHECK endWait1Chars then "end1"
            else if CHECK endWait2Chars then "end2"
            else if CHECK endWait3Chars then "end3"
            else if CHECK endWait4Chars then "end4"
            else                             "end"
    
#---# --------------------------------------------------------------
    ### ウェイトタイプ判定 ###
#---# --------------------------------------------------------------
    isWaitType = (c) ->
        tp = isStartEndType c
        switch tp
            when "end"  then "b"
            when "end1" then "b"
            when "end2" then "c"
            when "end3" then "d"
            when "end4" then "e"
            else             "a"


#---# --------------------------------------------------------------
    ### タグ：タグ領域の作成 ###
#---# --------------------------------------------------------------
    ### タグ領域の作成 ###
    createTagArea = (id, parent) ->
        $("<div>")
            .attr( id: id )
            .appendTo(parent)
        $("div#"+id)[0]
        
#---# --------------------------------------------------------------
    scrapArea = undefined
    getScrapArea = ->
        return scrapArea if scrapArea?

        scrapArea = $("div#scrapArea")[0]
        return scrapArea if scrapArea?

        scrapArea = createTagArea "scrapArea", "body"
        return scrapArea


#---# --------------------------------------------------------------
    ### セクション領域のアイテム                                 ###
    ### 表示はクラス制御
        初期：  waiting
        再生中：playing
        再生終：played
        終了中：closeing
        終了：(終了時にはエレメントが削除)
    ###
#---# --------------------------------------------------------------
    class SectionAreaItem
#-------# ----------------------------------------------------------
        ### 定数/グローバル変数/メソッド ###
        SectionIndexCounter = 0

#-------# ----------------------------------------------------------
        ### コンストラクタ ###
        constructor: (@parent, @areaClass, @index = SectionIndexCounter++ ) ->
            @id = "#{ @parent }-#{ @index }"
            console.log "[SectionAreaItem<#{ @id }>::constructor]"
            @area = $(createTagArea @id, @parent)
            @area.addClass(@areaClass)
            @area.addClass("waiting")
            @isShow = true
            @hide()

#-------# ----------------------------------------------------------
        ### DOMから自身を削除する ###
        remove: () ->
            @area.remove()
            console.log "[SectionAreaItem<#{ @id }>::remove]"
            return

#-------# ----------------------------------------------------------
        ### 表示 ###
        show: () ->
            console.log "[SectionAreaItem<#{ @id }>::show]"
            @setIsShow true
            return

#-------# ----------------------------------------------------------
        ### 非表示 ###
        hide: () ->
            console.log "[SectionAreaItem<#{ @id }>::hide]"
            @setIsShow false
            return

#-------# ----------------------------------------------------------
        ### クラスの置き換え ###
        setIsShow: (isShow) ->
            return if isShow == @isShow
            @isShow = isShow
            if @isShow then @area.removeClass("hidden")
            else            @area.addClass   ("hidden")
            return

#-------# ----------------------------------------------------------
        ### 段落追加 ###
        append: (paragraph) ->
            @area.append paragraph
            return




#---# --------------------------------------------------------------
    ### セクション領域の管理                                     ###
#---# --------------------------------------------------------------
    class SectionArea
#-------# ----------------------------------------------------------
        ### コンストラクタ ###
        constructor: (@id) ->
            @resetNextAreaClass()
            
#-------# ----------------------------------------------------------
        ### 次のセクション位置をリセットする ###
        resetNextAreaClass: () ->
            @areaClassIndex = -1
            

#-------# ----------------------------------------------------------
        ### 次のセクション位置を取得する ###
        getNextAreaClass: () ->
            @areaClassIndex++
            @areaClassIndex = 0 if @areaClassIndex >= areaItems.length
            areaItems[@areaClassIndex]

#-------# ----------------------------------------------------------
        ### 追加 ###
        startSection: (areaClass = @getNextAreaClass()) ->
            section = new SectionAreaItem


#-------# ----------------------------------------------------------
        ### 段落追加 ###
        append: (paragraph) ->





#---# --------------------------------------------------------------
    ### プッタネスカ：アニメーションシーケンサ                   ###
    ### １回のアニメーションについて、開放まで管理
         発生する契約
         ・再生開始：@playDeferred
         ・再生完了：@playedDeferred
         ・終了開始：@closeingDeferred
         ・終了完了：@closedDeferred
    ###
#---# --------------------------------------------------------------
    class PuttanescaSequencer
#-------# ----------------------------------------------------------
        sectionIndex = 0
        talkIndex = 0

#-------# ----------------------------------------------------------
        ### コンストラクタ ###
        constructor: (@startTime = window.Date.now(), @sectionAreas=["area1","area2"]) ->
            @scrapClass = "scrap#{ ++talkIndex}"
            console.log "スクラップ追加",[@scrapClass]
            $("#"+@scrapClass).remove()

            @scrapArea = $("<div />")
            @scrapArea.attr
                id:    @scrapClass
                class: "scrap"
            @scrapArea.appendTo getScrapArea()

            # 契約
            @playDeferred = $.Deferred()
            @playedDeferred = $.Deferred()
            @closedDeferred = $.Deferred()
            @closeingDeferred = $.Deferred()

            # 再生開始を失敗したときに発生
            @playDeferred.fail =>
                @playedDeferred.reject()

            # 終了開始時に必ず発生
            @closeingDeferred.always =>
                @playDeferred.reject()
                @playedDeferred.reject()
                
                closeCount = 1
                closeCheck = =>
                    closeCount--
                    console.log "終了チェック", [closeCount]
                    return if closeCount > 0
                    @scrapArea.remove()
                    console.log "終了！"
                    @closedDeferred.resolve()

                for item in @areaCloseAnimes
                    closeCount++
                    item.promise().always closeCheck
                    item.reject()

                closeCount--
                closeCheck()
            
            # 次のエリア
            @nextArea = {}
            for i in [0...@sectionAreas.length]
                j = i+1
                j=0 if j>=@sectionAreas.length
                key = @sectionAreas[i]
                val = @sectionAreas[j]
                @nextArea[key]=val

            # その他初期値
            @p        = undefined
            @area     = undefined
            @areaClass= undefined
            @areaCloseAnimes    = []

#-------# ----------------------------------------------------------
        ### ステータス ###
        state: ->
            return "close"    if @closedDeferred.state()   isnt "pending"
            return "closeing" if @closeingDeferred.state() isnt "pending"
            return "played"   if @playedDeferred.state()   isnt "pending"
            return "play"


#-------# ----------------------------------------------------------
        ### トーク契約 ###
        playedPromise: -> @playedDeferred.promise()
        closedPromise: -> @closedDeferred.promise()

        ### 再生終了状態に早送り ###
        played : (args) -> @playDeferred .resolve args

        ### 表示クローズ ###
        close  : (args) -> @closeingDeferred.reject args


#-------# ----------------------------------------------------------
        ### スクラップをひとつ閉じて、会話を確定する。 ###
        closeScrap: () -> 
            @endSection()
            @calcWait @scrapArea

#-------# ----------------------------------------------------------
        ### 指定エリアのウェイト計算。 ###
        calcWait: (area = @scrapArea) ->
            ### ウエイト計算 ###
            waitEl = undefined
            waitMS = 0

            setMS = (el,ms) -> el.attr "data-wait-ms": ms

            commit = ->
                return if not waitEl?
                setMS waitEl, waitMS
                waitEl = undefined
                waitMS = 0
                return

            $("span.waiting", area).each ->
                el = $(@)
                waitType = el.data("waitType")
                ms = waitMSTable[waitType]
                ms = 0 if not ms?
                # ms=0の時は0wait設定を行なって無視
                setMS el, ms
                return

                # "c"の時はコミットして強制wait
                if waitType == "c"
                    commit()
                    setMS el, ms
                    return

                # msが前回より小さくなるならcommit
                if ms < waitMS then commit()

                # 通常処理
                setMS el, normalMS
                waitEl = el
                waitMS = ms
                return

            ### 0スタートの発動タイミングの確定＆表示を消す ###
            time = 0
            $("span.waiting", area).each ->
                el = $(@)
                el.textHide()
                ms = el.data "waitMs"
                el.attr "data-start-time", time
                time += ms
                

            console.log "[会話", [@scrapClass],"] 終了"

#-------# ----------------------------------------------------------
        ### 実行 ###
        run: () -> 
            startTime = Date.now()
            console.log isodate.format( new Date(startTime) ) ,": !!START!!"
            lastTime = 0
            count = 0

            THIS = @
            items = $("span.waiting", @scrapArea)
            lastIndex = items.length - 1

            items.each ->
                el = $(@)
                time = el.data "startTime"
                text = el.text()
                timestamp = startTime + time
                index = count++
                emote = el.data "emote"
                func = -> THIS.playedDeferred.resolve()
                lastFunc = func
                # 表示処理の契約
                dfd = $.Deferred()
                dfd.done ->
                    return if THIS.state() != "play"
                    $.emote?[emote]?()          # エモート登録があれば発動
                    el.textShow(showCharWait)   # 文字表示
                    THIS.playedDeferred.resolve() if index == lastIndex
                    return

                THIS.playDeferred.done -> dfd.resolve()
                $.timestamp(timestamp, true).done -> dfd.resolve()

                # 時刻更新
                lastTime = time
                return true
            return @playDeferred


#-------# ----------------------------------------------------------
        ### セクションの終わり ###
        endSection: () ->
            return if not @area? 
            p = @p
            @endPeriod()
            # 段落ウェイトの登録
            if p? then @addChar p, "section"

            console.log "[セクション", [@areaID],"] 終了"
            @area = undefined
            return
            
#-------# ----------------------------------------------------------
        ### セクションの始まり ###
        section: (@areaClass = @nextSectionArea()) ->
            # 掃除
            @endSection()           

            # 登録
            console.log "[セクション", [@areaClass],"] 開始"
            el = $("<div />")
            el.attr
                id:    "section#{ ++sectionIndex }"
                class: "section #{ @areaClass } waiting"
            el.appendTo @scrapArea
            @area = el[0]

            # セクションのクローズアニメ作成
            closeDfd = $.Deferred()

            closeAnim = =>
                param =
                    top: "-16px"
                    opacity: 0.0
                opts =
                    duration: sectionCloseWait
                    easing: "easeInQuad"
                el.animate(param,opts).promise().done ->
                    el.remove()
                    closeDfd.reject()

            dfd = $.Deferred()
            dfd.always closeAnim
            dfd.promise = closeDfd.promise

            @areaCloseAnimes.push dfd

            return
#-------# ----------------------------------------------------------
        ### 段落の終わり ###
        nextSectionArea: () ->
            next = @nextArea[@areaClass]
            return @sectionAreas[0] if not next
            return next
            
#-------# ----------------------------------------------------------
        ### 段落の終わり ###
        endPeriod: (actor) ->
            return if not @p?
            console.log "段落閉じる：", [@p]

            # 段落ウェイトの登録
            @addChar @p, "period"

            # 登録して次へ
            @p.appendTo @area
            @p = undefined
            return


#-------# ----------------------------------------------------------
        ### 段落の始まり ###
        period: (actor) ->
            @endPeriod()
            @p = $("<p />").addClass(actor)
            console.log "段落：", [@p]
            return
            
#-------# ----------------------------------------------------------
        ### 表情変更 ###
        emote: (em) ->
            if not @p? then period @defaultActor
            console.log "表情:", [em]
            el = $("<span />")
            el.attr
                "data-emote": em
                class: "waiting"
            el.appendTo @p
            return


#-------# ----------------------------------------------------------
        ### 会話 ###
        talk: (text) ->
            if not @p? then period @defaultActor
            console.log "会話:", [text]
            @addChar @p, (isWaitType c), c for c in text
            return

        addChar: (parent, wait, c) ->
            el = $("<span />")
            el.text c if c?
            el.attr
                "data-wait-type": wait
                class: "waiting"
            el.appendTo(parent)

#-------# ----------------------------------------------------------
        ### 改行 ###
        br: () ->
            if not @p? then period @defaultActor
            console.log "改行:"
            $("<br />").appendTo(@p)
            return

    
    
#---# --------------------------------------------------------------
    
    PuttanescaSequencer