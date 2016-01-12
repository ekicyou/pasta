### UTF-8 ＠ぱすた＠　起動 ###

define [
    "animation"
    "dic/talk1"
    "engine/pasta"
    "engine/parsers/talk.min"
    "jquery-ui"
    "jquery.formtips"
], () ->
    $(window).ready ->
        dispatcher = window.pasta.dispatcher

        #------------------------------------------------------
        # 表情選択画面の処理
        #------------------------------------------------------
        sels = $("div.navi ul")
        if sels.length > 0
            sels.selectable
                filter: "li"
                selected: (e,ui) ->
                    el =$(ui.selected) 
                    surface = el.data("surface")
                    k = surface[0]
                    v = surface[1]
                    kv ={}
                    kv[k] = v
                    console.log "selected", [surface,kv,e,ui]
                    window.emotePasta kv
            sels.selectable "enable"


        #------------------------------------------------------
        # ストーリー生成
        #------------------------------------------------------
        keyTask =
            "おじさん"   : 'this.P("mister p1");'
            "パスタ"     : 'this.P("pasta p1");'
            "ノーマル"   : 'this.E("パスタ：ノーマル");'
            "よそみ"     : 'this.E("パスタ：よそみ");'
            "ちらっ"     : 'this.E("パスタ：ちらっ");'
            "めとじ"     : 'this.E("パスタ：めとじ");'
            "うわのそら" : 'this.E("パスタ：うわのそら");'
            "えっ？"     : 'this.E("パスタ：えっ？");'
            "ジトー"     : 'this.E("パスタ：ジトー");'
            "えへっ"     : 'this.E("パスタ：えへっ");'
            "ほほえみ"   : 'this.E("パスタ：ほほえみ");'
            "まばたき"   : 'this.E("パスタ：まばたき");'

        ConvertKeyword = (keyword) ->
            return keyTask[keyword] if keyTask[keyword]?
            jsonstr = JSON.stringify value
            return "this.W(#{jsonstr});"

        ConvertStory = (ast) ->
            rc = [
                'callback = function(essence) {'
            ]
            area = 0
            isOutputSection = true
            for line in ast
                if line.length < 2
                    isOutputSection = true
                    continue
                if isOutputSection
                    isOutputSection = false
                    area++
                    rc.push '  this.S("area' + area + '")'

                for value in line
                    tp = value[0]
                    value = value.substr 1
                    src = switch tp
                        when "B" then '  this.B();'
                        when "@" then '  ' + ConvertKeyword value
                        when "T" then '  this.T(' + (JSON.stringify value) + ');'
                    rc.push src if src?
            rc.push '  this.Z();'
            rc.push '}'
            return rc.join "\n"

        RunStory = (ast) ->
            callback = null
            evalstr = ConvertStory ast
            eval evalstr
            story = dispatcher.createSeq ["会話"], callback
            seq = story.next()
            dispatcher.breakChain()
            dispatcher.start seq
            return

        #------------------------------------------------------
        # パスタスクリプトの辞書チェック画面
        #------------------------------------------------------
        # パース実行
        $("#script_run").on "click", (ev) ->
            text = $("#script_input").attr("value")
            
            mes = $("#parse-message")
            try
                ast = window.pastaTalkParser.parse text
                RunStory ast
                mes.text "トーク開始！"
            catch ex
                mes.text ex.message
                throw ex
            return


        #------------------------------------------------------
        return
         

    $.holdReady false
