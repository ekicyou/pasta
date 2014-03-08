#
util        = require 'util'
fs          = require 'fs'
entities    = require 'cheerio/node_modules/entities'
cheerio     = require 'cheerio'
console.log "漢字試験"

# コンテンツの読み込み
texts   = fs.readFileSync('evernote.enex', 'utf8')
htmlConfig =
    ignoreWhitespace: false
    xmlMode: false
    lowerCaseTags: false
xmlConfig =
    ignoreWhitespace: false
    xmlMode: true
    lowerCaseTags: false
$ = cheerio.load texts, xmlConfig

#
encode = (str) -> entities.encode(str, 0)
decode = (str) -> entities.decode(str, 2)

# HTMLをプレインテキストに変換する
html2text = (html) ->
    el = cheerio.load html, xmlConfig
    rc = dom2text el "en-note"
    rc.trimRight()

dom2text = (elems) ->
    return "" if ! elems?
    ret = ""
    for i in [0...elems.length]
        elem = elems[i]
        switch elem.type
            when "text"     then ret += (decode elem.data).trimRight()
            when "comment"  then
            else
                if elem.children?
                    ret += dom2text elem.children 
        if elem.type == "tag"
            switch elem.name
                when "div","br" then ret += "\r\n"
            

    return ret





# DOMツリーを出力する
console.log "DOMツリーを出力"
$("note").each ->
    title   = $("title"  ,@)
    created = $("created",@)
    updated = $("updated",@)

    tags = $("tag",@).map -> $(@).text().trim() 

    content = $("content",@).html()

    console.log "====================================="
    console.log "■", title.text(), tags
    console.log html2text content
    console.log()
    console.log()

console.log "終了"




