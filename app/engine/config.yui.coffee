﻿#----------------------------------------------------------------------
### require.config設定  ###
#----------------------------------------------------------------------

require.config
    shim:   # 外部モジュールの依存関係
        "jquery.signalR"    :["jquery"]
        "jquery-ui"         :["jquery"]
        "jquery.metro"      :["jquery"]
        "jquery.formtips"   :["jquery"]
        "rx.jquery"         :["jquery", "rx"]
        "rx.time"           :["rx"]
        "globalize.ja-JP"   :["globalize"]
        "SVGPan"            :["jsutil.modules"]
        "innerSVG"          :["jsutil.modules"]
        "jquery.svg"        :["jquery"]
        "jquery.svganim"    :["jquery.svg"]
        "jquery.svgdom"     :["jquery.svg"]
        "jquery.svgfilter"  :["jquery.svg"]
        "jquery.svggraph"   :["jquery.svg"]
        "jquery.svgplot"    :["jquery.svg"]

    paths:  # モジュール名と実際のパスの関係
        "jquery"            :"Scripts/jquery-1.8.2.min"
        "jquery.signalR"    :"Scripts/jquery.signalR-0.5.1.min"
        "jquery-ui"         :"Scripts/jquery-ui-1.9.0.min"
        "jquery.metro"      :"Scripts/jquery.metro"
        "jquery.formtips"   :"Scripts/jquery.formtips.1.2.5"
        "rx"                :"Scripts/rx.min"
        "rx.jquery"         :"Scripts/rx.jquery.min"
        "rx.time"           :"Scripts/rx.time.min"
        "globalize"         :"Scripts/jquery.globalize/globalize"
        "globalize.ja-JP"   :"Scripts/jquery.globalize/cultures/globalize.culture.ja-JP"
        "knockout"          :"Scripts/knockout-2.1.0"
        "knockout.mapping"  :"Scripts/knockout.mapping-latest"
        "modernizr"         :"Scripts/modernizr-2.6.2"
        "iso8601"           :"Scripts/iso8601.min"
        "tilt"              :"Scripts/tilt"
        "raphael"           :"Scripts/raphael-min"
        "SVGPan"            :"Scripts/SVGPan"
        "innerSVG"          :"Scripts/innerSVG"
        "jsutil.modules"    :"Scripts/jsutil.modules"
        "jsutil.rx"         :"Scripts/jsutil.rx"
        "jquery.svg"        :"Scripts/jquery.svg.pack"
        "jquery.svganim"    :"Scripts/jquery.svganim.pack"
        "jquery.svgdom"     :"Scripts/jquery.svgdom.pack"
        "jquery.svgfilter"  :"Scripts/jquery.svgfilter.pack"
        "jquery.svggraph"   :"Scripts/jquery.svggraph.pack"
        "jquery.svgplot"    :"Scripts/jquery.svgplot.pack"
        "jarvis"            :"Scripts/jarvis-browser-min"
        "peg"               :"Scripts/peg-0.7.0.min"