### UTF-8 ＠ぱすた＠ ###
#----------------------------------------------------------------------
### 辞書ストア
###
#----------------------------------------------------------------------
define ["engine/jsutil"], () ->
    "use strict";


#---# --------------------------------------------------------------
    ### Array APIに対しキャッシュ付きの辞書操作を行います ###
    class CacheStore
#-------# ----------------------------------------------------------
        ### コンストラクタ
            @getItemsFunc 要素配列を全件取得するための関数
        ###
        constructor: (@getAllItemsFunc) ->
            @reset()

#-------# ----------------------------------------------------------
        ### アイテムのクリア ###
        reset: () ->
            @allItems = undefined
            return

#-------# ----------------------------------------------------------
        ### アイテムの全件取得 ###
        getAllItems: () ->
            if not @allItems?
                @allItems = @getAllItemsFunc()
                @cache = {}
            return @allItems

#-------# ----------------------------------------------------------
        ### キーの条件に対応した要素を取得。
            要素がキャッシュされていなければ関数を適用する
        ###
        getOne: (filter) ->
            rc = @cache[filter.key]
            if not rc?
                rc = filter.func @getAllItems()
                @cache[filter.key] = rc
            return rc

#-------# ----------------------------------------------------------
        ### 複合条件の検索。すべての条件に一致するものを返す ###
        get: (filters) ->
            # 最も要素数が少ないキーを取得
            smallKey   = ""
            smallItems =  @getAllItems()
            for filter in filters
                items = @getOne(filter)
                if items.length <= smallItems.length
                    smallKey   = filter.key
                    smallItems = items

            # 残りのキーで絞り込む
            rc = smallItems
            for filter in filters when smallKey != filter.key 
                rc = filter.func rc
            rc
       


#---# --------------------------------------------------------------
    CacheStore
