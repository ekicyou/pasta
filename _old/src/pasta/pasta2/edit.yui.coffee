### UTF-8 ＠辞書編集＆試験＠　起動 ###

define [
    "jquery"
], () ->
#---# --------------------------------------------------------------
    ### 試験データ ###
#---# --------------------------------------------------------------
    $.holdReady false
    $ ->
        # actor窓
        actorFrame = $("#actor iframe")[0]
        actorWindow = actorFrame.contentWindow
        actorFrame.src = "message.html"

        # メッセージの受信処理
        receiveMessage = (ev) ->
            actorWindow.postMessage(ev.data, "*")

        window.addEventListener("message", receiveMessage, false)

