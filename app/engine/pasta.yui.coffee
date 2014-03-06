### UTF-8 ＠ぱすた＠　起動 ###
define [
    "engine/jsutil"
    "engine/peperoncino_dispatcher"
], (jsutil, EventDispatcher ) ->

#---# --------------------------------------------------------------
    ### 起動イベント発動 ###
#---# --------------------------------------------------------------
    console?.log "[pasta::ready] 予約"
    $(window).ready ->
        # 初期化
        console?.log "[pasta::ready] 実行"
        pasta = {}
        dispatcher = new EventDispatcher()
        pasta.dispatcher = dispatcher
        window.pasta = pasta

        # クリックイベント
        $("#mainArea").on "click", ".hittest", (args) ->
            dispatcher.touch args.target.className

        return
