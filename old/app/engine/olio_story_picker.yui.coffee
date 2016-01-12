### UTF-8 ＠ぱすた＠ ###
#----------------------------------------------------------------------
### オリオ：会話シーケンサ 
辞書より会話を構成する。呼び出される毎に会話の１節を返す。
状態はessenceに保持する

###
#----------------------------------------------------------------------
define [
    "engine/aglio_dic"
    "engine/puttanesca_sequencer"
    "engine/jsutil"
], (aglio_dic, Sequencer) ->
    "use strict";

#---# --------------------------------------------------------------
    ### 属性チェック関数 ###
    isFunc  = (o) -> typeof(o) == "function"
    isArray = (o) -> Object::toString.call(o) == '[object Array]'


#---# --------------------------------------------------------------
    ### 会話スクリプタ ###
#---# --------------------------------------------------------------
    class OlioStoryPicker
#-------# ----------------------------------------------------------
        ### コンストラクタ
              @dic      : 辞書
              @essence  : 記憶
        ###
        constructor: (@dic = aglio_dic, @essence = {}) ->
            @nowStoryThis = undefined


#-------# ----------------------------------------------------------
        ### 会話選択とセクションの読み込み
                tags     : 会話引用タグ
                sequencer: 会話シーケンサ
        ###
        nextStory: (tags, sequencerConstructor = Sequencer) ->
            # 辞書からストーリーの取得
            scrap = @dic.selectScrap tags
            if not scrap
                console.log "[OlioStoryPicker::nextStory] キーワードが見つかりませんでした。", tags
                return undefined
            @createSeq scrap, sequencerConstructor

        createSeq: (scrap, sequencerConstructor = Sequencer) ->
            # thisの作成と関数呼び出し
            THIS = $.Deferred()
            THIS.sequencerConstructor = sequencerConstructor
            THIS.sequencer  = undefined
            THIS.essence    = @essence
            THIS.scrap      = scrap
            THIS.next       = => @nextScrap THIS
            #---

            ### 終了 ###
            THIS.isClose = false
            THIS.Z          = ()        -> @isClose = true
            THIS.close      = ()        -> @Z()

            ### １会話の区切り ###
            THIS.isSeparate = false
            THIS.Y          = ()        -> @isSeparate = true
            THIS.separate   = ()        -> @Y()

            ### セクション区切り ###
            THIS.S          = (area)    => THIS.sequencer.section area
            THIS.section    = (area)    -> @S(area)

            ### 段落区切り(p) ###
            THIS.P          = (actor)   => THIS.sequencer.period  actor
            THIS.period     = (actor)   -> @P(actor)

            ### 表情変更 ###
            THIS.E          = (em)      => THIS.sequencer.emote   em
            THIS.emote      = (em)      -> @E(em)

            ### 改行 ###
            THIS.B          = ()        => THIS.sequencer.br()
            THIS.br         = ()        -> @B()     

            ### テキスト ###
            THIS.T          = (text)    => THIS.sequencer.talk    text
            THIS.talk       = (text)    -> @T(text)

            ### ジャンプ予約 ###
            THIS.jumpTags = undefined
            THIS.J          = (tags)    -> THIS.jumpTags = tags
            THIS.jump       = (tags)    -> @J(tags)

            ### 単語の取得 ###
            THIS.W          = (tags)    => @popWord tags
            THIS.popWord    = (tags)    -> @W(tags)
            #---

            ### スクラップの実行
                シーケンサに１会話分の指示を与える
            ###
            THIS.callScrap  = =>
                THIS.isSeparate = false
                while THIS.scrap?
                    THIS.jumpTags = undefined
                    # 実行
                    THIS.scrap.callback.call THIS, THIS.essence
                    # 次のスクラップの取得
                    THIS.scrap =
                        if THIS.isClose        then undefined
                        else if THIS.jumpTags? then @dic.selectScrap THIS.jumpTags
                        else                        THIS.scrap.next()
                    # 中断か？
                    return if THIS.isSeparate == true
                return


            THIS.always ->
                THIS.sequencer?.close()
                return

            @nowStoryThis = THIS

            
#-------# ----------------------------------------------------------
        ### １会話分、スクラップを進める。進んだ場合は会話シーケンサを返す ###
        nextScrap: (THIS = @nowStoryThis) ->
            THIS.sequencer?.close?()
            THIS.sequencer = undefined
            if THIS?.scrap?
                THIS.sequencer = new THIS.sequencerConstructor()
                THIS.callScrap()
                return THIS.sequencer
            else
                THIS.resolve()
                @nowStoryThis = undefined if THIS == @nowStoryThis
                return undefined

#-------# ----------------------------------------------------------
        ### ランダムにセクションジャンプ ###
        jump: (tags) ->
            console.log "ジャンプ:", tags

#-------# ----------------------------------------------------------
        ### ランダムに単語辞書から取得 ###
        popWord: (tags) ->
            console.log "単語取得:", tags

#---# --------------------------------------------------------------
    OlioStoryPicker