### UTF-8 ＠ぱすた＠ ###
"use strict";

define [
    "engine/aglio_dic"          # 会話辞書
    "engine/olio_story_picker"  # 会話作成エンジン
    "engine/jsutil"    
    "jquery"
], (aglio_dic, Picker) ->

#---# --------------------------------------------------------------
    ### ペペロンチーノ：イベントトリガ ###
#---# --------------------------------------------------------------
    class PeperoncinoEvents
#-------# ----------------------------------------------------------
        constructor: (@parent) ->
            @picker = @parent.picker
            @chain = undefined

#-------# ----------------------------------------------------------
        start: (seq) -> @parent.start seq


#-------# ----------------------------------------------------------
        ### 会話チェーンの中断 ###
        breakChain: () ->
            @chain?.reject?()
            @chain = undefined

#-------# ----------------------------------------------------------
        ### 起動。初回起動の時はfirstBoot、それ以外はnormalBootに分岐 ###
        起動: ->
            @一般会話()


        ### 通常起動。 ###
        通常起動: ->


        ### 初回起動。 ###
        初回起動: ->


#-------# ----------------------------------------------------------
        ### 会話。時刻トークがあるときは分岐 ###
        会話: ->


        一般会話: ->
            console.log "[PeperoncinoDispatcher::events::一般会話]"
            seq = @chain?.next()
            if not seq?
                @chain?.resolve?()
                @chain = @picker.nextStory ["会話"]
                console.log ["次の会話" , @chain]
                seq = @chain.next()
            @start seq


        時刻00分: ->




#---# --------------------------------------------------------------
    ### ペペロンチーノ：イベント処理エンジン ###
#---# --------------------------------------------------------------
    class PeperoncinoDispatcher
        ### コンストラクタ ###
        constructor: (@dic = aglio_dic) ->
            console.log "[PeperoncinoDispatcher::constructor]"
            # 状態管理変数
            @order = {}
            @order.sys =
                dispTime: 15*1000   # メッセージ表示後の最低表示期間
                talk15min: 20       # １５分間にしゃべる回数
            
            # マスタートークジェネレータ
            @picker = new Picker @dic, @order
            
            # イベントトリガ
            @events = new PeperoncinoEvents(@)
    
            # 起動イベントの発動
            try
                @events.起動()
            catch ex
                console.log ex

            # 定期会話イベント：初期化
            # TODO: 定期イベントはrx使わないように変更する
            # @nextSpanDisposable = new Rx.SerialDisposable
            # @spanMS = 10 * 1000
            # @setNextSpanAction @actNextSpan


#-------# ----------------------------------------------------------
        ### 新しく作ったトーク（シーケンサ）の再生処理 ###
        start: (seq) ->
            @nowSeq?.close?()
            @nowSeq = seq

            seq.closeScrap()
            $.timeout(500).done -> seq.run()
            #表示終了時の処理
            seq.playedPromise().always ->
                console.log "再生終了"

            # 表示終了時の処理
            seq.closedPromise().always ->
                console.log "表示終了"

            return seq
        
#-------# ----------------------------------------------------------
        ### ストーリーより、直接シーケンサを作成 ###
        createSeq: (tags, callback) ->
            story = @dic.createScrap tags, callback
            return @picker.createSeq story
        
#-------# ----------------------------------------------------------
        ### 会話再生中なら、再生終了状態に早送り ###
        played : (args) ->
            @nowSeq?.played? args
            return

        ### 会話再生中なら、表示クローズ ###
        close  : (args) ->
            if @nowSeq?.close?
                @nowSeq.close args
                @nowSeq = undefined
            return

#-------# ----------------------------------------------------------
        ### 会話チェーンの中断 ###
        breakChain : -> @events.breakChain()

#-------# ----------------------------------------------------------
        ### 定期会話イベント：次の定期発動タイミングを計算して発動予約 ###
        setNextSpanAction: (act) ->
            now = new Date
            nowSec = now / 1000
    
            # 発動時刻を計算
            a = now / @spanMS | 0
            nextMS = (a + 1) * @spanMS
    
            # 時間差が0.8以下なら次に送る
            nextSec = nextMS / 1000
            spanNextMS = (nextSec - nowSec) * 1000
            ratio = spanNextMS / @spanMS
            nextMS = (a + 2) * @spanMS if ratio < 0.8
    
            # 発動時刻の確定
            next = new Date nextMS
    
            # 発動予約
            console?.log "now  :" ,now
            console?.log "next :" ,next
            @nextSpanDisposable.disposable = Rx.Observable
                .timer(next - now)
                .subscribe (ev) =>
                    @nextSpanDisposable.disposable = undefined
                    console?.log "今！ :" ,new Date
                    act(ev)
    
            return
    
    
    
        ### 定期会話イベント：処理の呼び出し ###
        actNextSpan: (ev) ->
            console?.log "[PeperoncinoDispatcher::actNextSpan] ", ev
    
            return
    
#-------# ----------------------------------------------------------
        ### タッチトベント ###
        touch: (points) ->
            console.log "タッチ検出！", [points]

            if @nowSeq?.state?() == "played"
                @close()
                @events.起動()
            else if @nowSeq?.state?() == "play"
                @played()
            else
                @events.起動()
            return


#---# --------------------------------------------------------------
