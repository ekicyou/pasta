define ["rx.jquery", "iso8601"], ->
    
    ### jQueryのReady発動を抑制。 ###
    $.holdReady true

    ### Array拡張：配列をマップします。 ###
    if not Array::map?
        Array::map = (func) ->
            for item in @ then func item
    
    ### Array拡張：配列をシャッフルします。 ###
    randomArray = (array, num) ->
        a = array
        t = []
        r = []
        l = a.length
        n = if num < l then num else l
        while n-- > 0
            i = Math.random() * l | 0
            r[n] = t[i] || a[i]
            --l
            t[i] = t[l] || a[l]
        return r

    if not Array::shuffle?
        Array::shuffle = -> randomArray(@, @length)
    
    ### Array拡張：配列を重複せずに無限ランダム抽出するイテレータ。 ###
    Array::random_iter_loop = ->
        index = 0
        a = []
        return next: =>
            index++
            if index >= a.length
                index = 0
                a = @shuffle()
            return a[index]
    
    ### Array拡張：配列からランダムに要素を一つ取り出す。 ###
    Array::random = ->  
        return @[ Math.random() * @length | 0] if @length > 0
        undefined

    ### コンソールの存在を騙す ###
    if not window.console?
        window.console =
            log: ()->


    ### 簡易ブラウザ判定 ###
    checkBrowser = ->
        userAgent = window.navigator.userAgent.toLowerCase()
        return 'opera' if userAgent.indexOf('opera') != -1
        if userAgent.indexOf('msie') != -1
            appVersion = window.navigator.appVersion.toLowerCase()
            return 'ie6'  if appVersion.indexOf("msie 6.") != -1
            return 'ie7'  if appVersion.indexOf("msie 7.") != -1
            return 'ie8'  if appVersion.indexOf("msie 8.") != -1
            return 'ie9'  if appVersion.indexOf("msie 9.") != -1
            return 'ie10' if appVersion.indexOf("msie 10.") != -1
            return 'ie11' if appVersion.indexOf("msie 11.") != -1
            return 'ie12' if appVersion.indexOf("msie 12.") != -1
            return 'ie13' if appVersion.indexOf("msie 13.") != -1
            return 'ie14' if appVersion.indexOf("msie 14.") != -1
            return 'ie15' if appVersion.indexOf("msie 15.") != -1
            return 'ie'
        
        return 'chrome' if userAgent.indexOf('chrome') != -1
        return 'safari' if userAgent.indexOf('safari') != -1
        return 'gecko'  if userAgent.indexOf('gecko') != -1
        return 'unnone_browser';

    isb = "is_" + checkBrowser()
    window[isb] = true
    window.is_webkit = true if window.is_chrome?
    window.is_webkit = true if window.is_safari?

    window.is_ie_version = 0
    window.is_ie_version = 6  if window.is_ie6?
    window.is_ie_version = 7  if window.is_ie7?
    window.is_ie_version = 8  if window.is_ie8?
    window.is_ie_version = 9  if window.is_ie9?
    window.is_ie_version = 10 if window.is_ie10?
    window.is_ie_version = 11 if window.is_ie11?
    window.is_ie_version = 12 if window.is_ie12?
    window.is_ie_version = 13 if window.is_ie13?
    window.is_ie_version = 14 if window.is_ie14?
    window.is_ie_version = 15 if window.is_ie15?

    html = $("html")
    html.addClass("webkit") if window.is_webkit
    html.addClass("ie6" ) if window.is_ie6
    html.addClass("ie7" ) if window.is_ie7
    html.addClass("ie8" ) if window.is_ie8
    html.addClass("ie9" ) if window.is_ie9
    html.addClass("ie10") if window.is_ie10
    html.addClass("ie11") if window.is_ie11
    html.addClass("ie12") if window.is_ie12
    html.addClass("ie13") if window.is_ie13
    html.addClass("ie14") if window.is_ie14
    html.addClass("ie15") if window.is_ie15

    html.addClass("ie8_" ) if window.is_ie_version >= 8
    html.addClass("ie9_" ) if window.is_ie_version >= 9

    ### ieかどうかでテキスト用のfade/show関数を差し替える ###
    if window.is_webkit then $.fn.extend
        textHide: ()   -> @hide()
        textShow: (ms) -> @show(ms)
    else                     $.fn.extend
        textHide: ()   -> @fadeTo(0  ,0.0)
        textShow: (ms) -> @fadeTo(ms ,1.0)

    ### 割り込み関数の差し替え ###
    if not window.setImmediate?
        window.setImmediate = (func, args) -> window.setTimeout(func, 0, args)
        window.clearImmediate = window.clearTimeout

    if not window.requestAnimationFrame?
        if window.msRequestAnimationFrame?
            window.requestAnimationFrame = window.msRequestAnimationFrame
            window.cancelAnimationFrame  = window.msCancelAnimationFrame
        if window.webkitRequestAnimationFrame?
            window.requestAnimationFrame = window.webkitRequestAnimationFrame
            window.cancelAnimationFrame  = window.webkitCancelAnimationFrame
        if window.mozRequestAnimationFrame?
            window.requestAnimationFrame = window.mozRequestAnimationFrame
            window.cancelAnimationFrame  = window.mozCancelAnimationFrame
    
    ### jQueryオブジェクトのプラグイン ###
    $.fn.extend
        rx$load    : (data)             -> @loadAsObservable    'load'              ,data
        rx$click   : (selector, data)   -> @onAsObservable      'click'   ,selector ,data
        rx$message : (selector, data)   -> @onAsObservable      'message' ,selector ,data
        rx$resize  : (selector, data)   -> @onAsObservable      'resize'  ,selector ,data
    
        rx$liveclick   : (data) -> @liveAsObservable 'click' , data


    ### jQueryのプラグイン ###
    $.extend
        ### immediateの契約を返す ###
        immediate : () ->
            dfd = $.Deferred()
            id = window.setImmediate (-> dfd.resolve(0))
            dfd.always -> window.clearImmediate id
            dfd.promise()
    
        ### timeoutの契約を返す ###
        timeout : (time) ->
            return $.immediate() if time is 0
            dfd = $.Deferred()
            id = setTimeout (-> dfd.resolve time), time
            dfd.always -> clearTimeout id
            dfd.promise()
    
        ### timestampの契約を返す 
                timestamp: 発動希望時刻
                isSleepWait: trueなら、間にスリープが入った場合に発動を遅らせる
        ###
        timestamp : (timestamp, isSleepWait = true) ->
            dfd = $.Deferred()
            startWait = sleepWait
            id = undefined
            dfd.always ->
                clearTimeout id if id?
            func = ->
                sleep = if isSleepWait then sleepWait - startWait else 0
                now = Date.now() - sleep
                span = timestamp - now
                if span <= 0
                    dfd.resolve
                        timestamp: timestamp
                        sleep    : sleep
                else
                    id = setTimeout func, span
                return
            func()
            dfd.promise()

    ### スリープ時間の計測 ###
    sleepWait = 0
    sleepCheckLastTimestamp = Date.now()
    sleepCheckTimeoutId = undefined
    sleepCheckFunc = ->
        now = Date.now() 
        span = now - sleepCheckLastTimestamp
        sleepWait += span - 200 if span > 600
        sleepCheckLastTimestamp = now
        clearTimeout sleepCheckTimeoutId
        sleepCheckTimeoutId = setTimeout sleepCheckFunc, 200
    sleepCheckFunc()


    ### jQuery:easeInQuadが使えないとき用 ###
    $.easing.easeInQuad ?= (p) -> p*p

    return