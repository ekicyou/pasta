### UTF-8 ＠ぱすた＠ ###
#----------------------------------------------------------------------
### オリオ：会話シーケンサ 
辞書より会話を構成する。呼び出される毎に会話の１節を返す。
状態はessenceに保持する

###
#----------------------------------------------------------------------
define ["scripts/aglio", "scripts/jsutil"], (Aglio) ->

#---# --------------------------------------------------------------
    ### オブジェクトが関数ならtrue ###
#---# --------------------------------------------------------------
    isFunc = (obj)-> typeof(obj) == "function"


#---# --------------------------------------------------------------
    ### 会話シーケンサ ###
#---# --------------------------------------------------------------
    class Olio
        ### コンストラクタ
              @aglio     : 辞書
              @stateFunc : 会話開始時に無条件指定される状態指定関数
              @essence   : 遷移を保持するオブジェクト
        ###
        constructor: (@aglio, @stateFunc, @essence) ->
            @yarn   = @aglio.yarn
            @quantum= @aglio.quantum
            @essence = {} if not @essence?
            @essence.index     = -1
            @essence.state          = {} if not @essence.state?
            @essence.state.end      = {} if not @essence.state.end?
            @essence.state.entangle = {} if not @essence.state.entangle?

#-------# ----------------------------------------------------------
        ### 次の１節を取得します。 ###
        next: ->
            # 次の行を決定
            lastIndex = @essence.index
            last =
                if lastIndex >= @yarn.length then null
                else if lastIndex < 0        then null
                else @yarn[lastIndex]
            type = last?.type
            index = switch type
                when "entangle" then @entangle @essence.state.entangle, last.quantumState
                when "end"      then @entangle @essence.state.end,      @stateFunc
            index = lastIndex + 1 if not index?
            index = 0 if index >= @yarn.length

            # 値を保存して返す
            @essence.index = index
            @essence.knot  = @yarn[index]

#-------# ----------------------------------------------------------
        ### 量子ジャンプ。 ###
        entangle: (st, quantumState)->
            key =
                if isFunc(quantumState) then quantumState(@essence)
                else quantumState
            items = @quantum[key]
            return undefined if not items?
            item = items.random()
            st.item = item
            item.index


#---# --------------------------------------------------------------
    Olio