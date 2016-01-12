### UTF-8 メッセージ確認用 ###

define [
    "jquery"
], () ->
#---# --------------------------------------------------------------
    ### 試験データ ###
#---# --------------------------------------------------------------
    $.holdReady false
    $ ->
        # メッセージの受信処理
        receiveMessage = (ev) ->
            $("#parse-message").text(ev.data)

        window.addEventListener("message", receiveMessage, false)
