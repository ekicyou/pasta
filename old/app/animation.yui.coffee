### UTF-8 ＠ぱすた＠　シェルアニメーション ###

define [
    "engine/jsutil"
], () ->
    $(window).ready ->
        # 管理変数
        nowSurface = 
            x:"0"
            y:"0"
            z:"0"
        lastSurface =
            x:"0"
            y:"0"
            z:"0"
        animate = undefined

        # 切り替え関数
        change = (kv) ->
            changed = false
            for k,v of kv
                if lastSurface[k] != v
                    changed = true
                    lastSurface[k] = v
            return if !changed

            text  =  "x" + lastSurface["x"]
            text += " y" + lastSurface["y"]
            text += " z" + lastSurface["z"]
            $("div#surface").attr "class", text
            return

        # まばたき関数
        wink = (time = 1000 + 19000 * Math.random())->
            return if nowSurface.y == "3"  # 目閉じの時は何もしない
            console.log "まばたき", [nowSurface.y]
            if animate?
                animate.reject()
                animate = undefined
            return if animate?
            
            # animation登録
            animate = $.Deferred()

            # 終わったら元に戻す
            animate.done ->
                change nowSurface
                return if nowSurface.y == "3"
                wink()      # 次のウィンクを予約
            
            # 時間経過の通知で画像を差し替える
            animate.progress (kv) ->
                return if animate.state() != "pending"
                change kv if kv?
                if not kv? or kv.y == nowSurface.y 
                    animate.resolve()
                    return

            # まばたきの時間経過通知の作成
            addAnimate = (kv, time) ->
                id = setTimeout (-> animate.notify kv), time
                animate.always -> clearTimeout id
            addAnimate {y:"3",z:"1"         }, time           # 目閉じ
            addAnimate {y:"6",z:nowSurface.z}, time += 100    # 半目
            addAnimate undefined             , time += 70     # 元に戻す


        # 選択関数
        select = (kv) ->
            change kv
            nowSurface[k] = v for k,v of kv
            console.log "サーフェスチェンジ→",[kv,nowSurface]
            if kv?.y == "4" then wink 0
            else                 wink(1000 + 2000 * Math.random())

        # 公開関数
        $.extend
            emote:
                "パスタ：ノーマル"   : -> select {x:"0", y:"0", z:"0",}
                "パスタ：よそみ"     : -> select {x:"0", y:"1", z:"0",}
                "パスタ：ちらっ"     : -> select {x:"0", y:"2", z:"0",}
                "パスタ：めとじ"     : -> select {x:"0", y:"3", z:"1",}
                "パスタ：うわのそら" : -> select {x:"0", y:"4", z:"0",}
                "パスタ：えっ？"     : -> select {x:"1", y:"5", z:"0",}
                "パスタ：ジトー"     : -> select {x:"0", y:"6", z:"0",}
                "パスタ：えへっ"     : -> select {x:"2", y:"0", z:"0",}
                "パスタ：ほほえみ"   : -> select {x:"2", y:"3", z:"0",}
                "パスタ：まばたき"   : -> wink   0

        window.emotePasta = select
        select 0

        return
