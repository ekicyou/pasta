### UTF-8 ＠ぱすた＠ ###
define ["scripts/jsutil","modernizr"], () ->

#---# --------------------------------------------------------------
    ### プッタネスカ：トークアニメーション処理  ###
#---# --------------------------------------------------------------
    
    ### スペースを削除する関数 ###
    delSpace = (str) -> str
    
    
    ### 禁則文字列 ###
    startChars    = "（［｛「『([{｢"
    
    endChars      =          "、。，．・？！゛゜ヽヾゝゞ々）］｝」』!),.:;?]}｡｣､･ﾞﾟ‥…"
    endWait1Chars = delSpace "、  ，                                  ,        ､       "
    endWait2Chars = delSpace "            ！                        !                  "
    endWait3Chars = delSpace "  。  ．・？                               .?  ｡  ･  ‥…"
    
    hangChars     = "、。，．,."
    
    noWaitMS   = 20     # １文字ウエイト
    endWait1MS = 50     # 半濁点ウエイト
    endWait2MS = 100    # 「！」ウエイト
    endWait3MS = 150    # 濁点ウエイト
    spanWaitMS = 200    # トーク区切りウエイト

    
#---# --------------------------------------------------------------
    ### プッタネスカ：文字タイプの判定 ###
#---# --------------------------------------------------------------
    isStartEndType = (c) ->
        CHECK = (items)-> (items.indexOf c) > -1 
        if c == " "                     then "normal"
        else if CHECK startChars        then "start"
        else if CHECK endChars
            if      CHECK endWait1Chars then "end1"
            else if CHECK endWait2Chars then "end2"
            else if CHECK endWait3Chars then "end3"
            else                             "end"
    
#---# --------------------------------------------------------------
    ### ウェイトタイプ判定 ###
#---# --------------------------------------------------------------
    isShowWaite = (c) ->
        tp = isStartEndType c
        switch tp
            when "end"  then [c,2.0]
            when "end1" then [c,2.0]
            when "end2" then [c,2.0]
            when "end3" then [c,3.0]
            else             [c,1.0]
    
#---# --------------------------------------------------------------
    ###
    ・startCharsだったとき：次の文字と合成する
    ・endCharsだったとき：直前の文字と合成する
    
    ###
    
    
    ### 文字列を文字配列に変換 ###
    strSplit = (str) -> for c in str then c
    
        
    
    
    ### 文字配列にウェイトを合成。文字・数字・・・と続く ###
    toWaitChars = (chars) ->
        rc = []
        wait = 0
        for c in chars
            # 文字タイプの判定
            startEnd = "normal"
            sel = undefined
            if c == " "                             then startEnd = "normal"
            else if startChars      .indexOf c > -1 then startEnd = "start"
            else if endChars        .indexOf c > -1
                startEnd = "end"
                if      endWait1Chars   .indexOf c > -1 then sel = "end1"
                else if endWait2Chars   .indexOf c > -1 then sel = "end2"
                else if endWait3Chars   .indexOf c > -1 then sel = "end3"
    
            # 
    
#---# --------------------------------------------------------------
    ### プッタネスカ：エレメントにトーク１行作成して、情報を返す ###
#---# --------------------------------------------------------------
    createTalkLine = (el, line) ->
        span = undefined
        for c in strSplit line
            span = $("<span />").appendTo el
            span.text c
            span.attr "data-w": isWaitType c
    
        # 最終データのウェイトは常に4
        span?.attr "data-w": 4
    
        return
    
#---# --------------------------------------------------------------
    ### プッタネスカ：エレメントにトーク行を作成して、情報を返す ###
#---# --------------------------------------------------------------
    createTalkLines = (el, classNames, lines...) ->
        p = $("<p />").appendTo el
        p.addClass classNames
        isFirst = true
        for line in lines
            if isFirst then isFirst = false
            else            $("<br />").appendTo p
            createTalkLine p, line
        return
    
#---# --------------------------------------------------------------
    ### プッタネスカ：遅延タスク作成 ###
#---# --------------------------------------------------------------
    delay = (ms) -> (Rx.Observable.timer)(ms)
        
    
#---# --------------------------------------------------------------
    ### プッタネスカ：トーク発動処理 ###
#---# --------------------------------------------------------------
    runTalkAndNext = (endFunc) ->
        
        # 一度消す
        $("div#mainArea").removeClass("show")
    
        # wait設定スクリプト
        ms = 0              # 表示する時刻
        oldMS = 0           # 前回の文字の表示時刻
        acts = []
        lastWaitType = -1
        baseWait = 120
        cd = new Rx.CompositeDisposable
        talkResult = "break"

        charTask = ->
            $$ = $(@)       
    
            # 追加ウェイト
            waitType = $$.data("w")
            nowWaitType =
                if lastWaitType < 0
                    lastWaitType = waitType
                    4
                else if waitType > 0
                    lastWaitType = waitType if waitType > lastWaitType
                    0
                else
                    rc = lastWaitType
                    lastWaitType = 0
                    rc
    
            addWait = switch nowWaitType
                when 1  then baseWait * 2
                when 2  then baseWait * 3
                when 3  then baseWait * 4
                when 4  then baseWait * 6
                else         0
            ms += addWait

            text = $$.text()
            lastWaitType = waitType
    
            # 文字の消去
            $$.fadeTo(0,0.001)
    
            # 文字の再表示タスク
            show = Rx.Disposable.create -> $$.fadeTo(300,1.0)
            cd.add show
                
            # 再表示の呼び出しタイミング登録
            timerTask = delay(ms)
                .subscribe -> show.dispose()
            cd.add timerTask
            $$.attr
                "data-span": ms - oldMS
                "data-ms": ms
            oldMS = ms
    
            # 次のウエイト
            ms += baseWait

        # １行単位の割り当てタスク
        lineTask = ->
        
        # 文字にウエイトタスクを適用
        $("#mainArea section").each ->
            $("p", @).each ->
                lastWaitType = -1
                $("span", @).each charTask
            ms += baseWait * 3

        # 終了時処理
        cd.add(end = Rx.Disposable.create -> endFunc talkResult)
        endTask = (Rx.Observable.timer)(ms)
            .subscribe ->
                talkResult = "end"
                cd.dispose()
        cd.add endTask
    
    
        # 表示
        cd.add (Rx.Observable.timer)(0)
            .subscribe ->
                $("div#mainArea").addClass("show")
    
        # 戻り値。disposeすると一気に終了する。
        cd
    
    
#---# --------------------------------------------------------------
    ### プッタネスカ：IEかどうかでimg/imageを切り替える  ###
#---# --------------------------------------------------------------
    $(document).ready ->
        if window.is_webkit then $("#actor div.webkit").css display: "block"
        else                     $("#actor div.ie")    .css display: "block"
            

    
#---# --------------------------------------------------------------
    ### プッタネスカ：初期化処理 ViewModelのバインドを行う ###
#---# --------------------------------------------------------------
    class Puttanesca
#---# --------------------------------------------------------------

        ### コンストラクタ ###
        constructor: () ->



        ### 試験用ロード処理 ###
        loadTest: () ->
            console?.log "[Puttanesca::loadTest]"
    
            doc = document
            text1 = $("#text1 div").empty()
            createTalkLines text1, "a2 t1", "今日も暑いですね。"
            createTalkLines text1, "a1 t1", "そーですねえ。"
            createTalkLines text1, "a2 t1", "汗かかない？一枚脱ぐ？"
    
            text2 = $("#text2 div").empty()
            createTalkLines text2, "a1 t1", "それって、逆に汗かきませんか？"
            createTalkLines text2, "a2 t1", "・・・・・・・・。"

            bb = undefined
            runTalk = ->
                bb = runTalkAndNext -> console?.log "トーク終了！"
            runTalk()
    
            #$("#actor div.hittest.おでこ")
            $("#actor")
                .rx$liveclick()
                .throttle(300)
                .subscribe (ev) ->
                    console?.log "おでこクリック！" ,ev.target.outerHTML
                    if bb.isDisposed() then runTalk()
                    else                    bb.dispose()
    
            return
    
    
    
#---# --------------------------------------------------------------
    
    Puttanesca