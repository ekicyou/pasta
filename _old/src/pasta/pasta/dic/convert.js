(function() {
  var $, cheerio, decode, dom2text, encode, entities, fs, html2text, htmlConfig, texts, util, xmlConfig;

  util = require('util');

  fs = require('fs');

  entities = require('cheerio/node_modules/entities');

  cheerio = require('cheerio');

  console.log("漢字試験");

  texts = fs.readFileSync('evernote.enex', 'utf8');

  htmlConfig = {
    ignoreWhitespace: false,
    xmlMode: false,
    lowerCaseTags: false
  };

  xmlConfig = {
    ignoreWhitespace: false,
    xmlMode: true,
    lowerCaseTags: false
  };

  $ = cheerio.load(texts, xmlConfig);

  encode = function(str) {
    return entities.encode(str, 0);
  };

  decode = function(str) {
    return entities.decode(str, 2);
  };

  html2text = function(html) {
    var el, rc;
    el = cheerio.load(html, xmlConfig);
    rc = dom2text(el("en-note"));
    return rc.trimRight();
  };

  dom2text = function(elems) {
    var elem, i, ret, _i, _ref;
    if (!(elems != null)) {
      return "";
    }
    ret = "";
    for (i = _i = 0, _ref = elems.length; 0 <= _ref ? _i < _ref : _i > _ref; i = 0 <= _ref ? ++_i : --_i) {
      elem = elems[i];
      switch (elem.type) {
        case "text":
          ret += (decode(elem.data)).trimRight();
          break;
        case "comment":
          break;
        default:
          if (elem.children != null) {
            ret += dom2text(elem.children);
          }
      }
      if (elem.type === "tag") {
        switch (elem.name) {
          case "div":
          case "br":
            ret += "\r\n";
        }
      }
    }
    return ret;
  };

  console.log("DOMツリーを出力");

  $("note").each(function() {
    var content, created, tags, title, updated;
    title = $("title", this);
    created = $("created", this);
    updated = $("updated", this);
    tags = $("tag", this).map(function() {
      return $(this).text().trim();
    });
    content = $("content", this).html();
    console.log("=====================================");
    console.log("■", title.text(), tags);
    console.log(html2text(content));
    console.log();
    return console.log();
  });

  console.log("終了");

}).call(this);
