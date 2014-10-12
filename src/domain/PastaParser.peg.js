﻿module.exports = (function () {
    /*
     * Generated by PEG.js 0.8.0.
     *
     * http://pegjs.majda.cz/
     */

    function peg$subclass(child, parent) {
        function ctor() { this.constructor = child; }
        ctor.prototype = parent.prototype;
        child.prototype = new ctor();
    }

    function SyntaxError(message, expected, found, offset, line, column) {
        this.message = message;
        this.expected = expected;
        this.found = found;
        this.offset = offset;
        this.line = line;
        this.column = column;

        this.name = "SyntaxError";
    }

    peg$subclass(SyntaxError, Error);

    function parse(input) {
        var options = arguments.length > 1 ? arguments[1] : {},

            peg$FAILED = {},

            peg$startRuleIndices = { start: 0 },
            peg$startRuleIndex = 0,

            peg$consts = [
              [],
              peg$FAILED,
              void 0,
              "```",
              { type: "literal", value: "```", description: "\"```\"" },
              null,
              /^[:\uFF1A]/,
              { type: "class", value: "[:\\uFF1A]", description: "[:\\uFF1A]" },
              /^[\u300C\uFF62]/,
              { type: "class", value: "[\\u300C\\uFF62]", description: "[\\u300C\\uFF62]" },
              /^[\uFF63\u300D]/,
              { type: "class", value: "[\\uFF63\\u300D]", description: "[\\uFF63\\u300D]" },
              { type: "any", description: "any character" },
              /^[,\u3001]/,
              { type: "class", value: "[,\\u3001]", description: "[,\\u3001]" },
              /^[@\uFF20]/,
              { type: "class", value: "[@\\uFF20]", description: "[@\\uFF20]" },
              function (text) { return text; },
              /^[a-zA-Z$_]/,
              { type: "class", value: "[a-zA-Z$_]", description: "[a-zA-Z$_]" },
              /^[\x80-\uD7FF\uE000-\uFFFD]/,
              { type: "class", value: "[\\x80-\\uD7FF\\uE000-\\uFFFD]", description: "[\\x80-\\uD7FF\\uE000-\\uFFFD]" },
              /^[\uD800-\uDBFF]/,
              { type: "class", value: "[\\uD800-\\uDBFF]", description: "[\\uD800-\\uDBFF]" },
              /^[\uDC00-\uDFFF]/,
              { type: "class", value: "[\\uDC00-\\uDFFF]", description: "[\\uDC00-\\uDFFF]" },
              /^[0-9]/,
              { type: "class", value: "[0-9]", description: "[0-9]" },
              /^[\uFF65\u30FB\u2025\u2026]/,
              { type: "class", value: "[\\uFF65\\u30FB\\u2025\\u2026]", description: "[\\uFF65\\u30FB\\u2025\\u2026]" },
              /^[\-\uFF0D\uFF70\u30FC]/,
              { type: "class", value: "[\\-\\uFF0D\\uFF70\\u30FC]", description: "[\\-\\uFF0D\\uFF70\\u30FC]" },
              /^[\u3001\uFF0C\uFF0E,.\uFF64:;\uFF09)\u309B\u309C\u30FD\u30FE\u309D\u309E\u3005\uFF9E\uFF9F]/,
              { type: "class", value: "[\\u3001\\uFF0C\\uFF0E,.\\uFF64:;\\uFF09)\\u309B\\u309C\\u30FD\\u30FE\\u309D\\u309E\\u3005\\uFF9E\\uFF9F]", description: "[\\u3001\\uFF0C\\uFF0E,.\\uFF64:;\\uFF09)\\u309B\\u309C\\u30FD\\u30FE\\u309D\\u309E\\u3005\\uFF9E\\uFF9F]" },
              /^[\uFF1F\uFF01!?]/,
              { type: "class", value: "[\\uFF1F\\uFF01!?]", description: "[\\uFF1F\\uFF01!?]" },
              /^[\u3002\uFF3D\uFF5D\u300D\u300F\]}\uFF61\uFF63]/,
              { type: "class", value: "[\\u3002\\uFF3D\\uFF5D\\u300D\\u300F\\]}\\uFF61\\uFF63]", description: "[\\u3002\\uFF3D\\uFF5D\\u300D\\u300F\\]}\\uFF61\\uFF63]" },
              /^[\u3001\u3002\uFF0C\uFF0E,.]/,
              { type: "class", value: "[\\u3001\\u3002\\uFF0C\\uFF0E,.]", description: "[\\u3001\\u3002\\uFF0C\\uFF0E,.]" },
              /^[\uFF08\uFF3B\uFF5B\u300C\u300E([{\uFF62]/,
              { type: "class", value: "[\\uFF08\\uFF3B\\uFF5B\\u300C\\u300E([{\\uFF62]", description: "[\\uFF08\\uFF3B\\uFF5B\\u300C\\u300E([{\\uFF62]" },
              /^[ \t\xA0\u1680\u180E\u2000-\u200A\u202F\u205F\u3000]/,
              { type: "class", value: "[ \\t\\xA0\\u1680\\u180E\\u2000-\\u200A\\u202F\\u205F\\u3000]", description: "[ \\t\\xA0\\u1680\\u180E\\u2000-\\u200A\\u202F\\u205F\\u3000]" },
              /^[\u3005\u3007\u303B\u3400-\u9FFF\uF900-\uFAFF]/,
              { type: "class", value: "[\\u3005\\u3007\\u303B\\u3400-\\u9FFF\\uF900-\\uFAFF]", description: "[\\u3005\\u3007\\u303B\\u3400-\\u9FFF\\uF900-\\uFAFF]" },
              /^[\uD840-\uD87F]/,
              { type: "class", value: "[\\uD840-\\uD87F]", description: "[\\uD840-\\uD87F]" },
              /^[0-9a-fA-F]/,
              { type: "class", value: "[0-9a-fA-F]", description: "[0-9a-fA-F]" },
              "\\",
              { type: "literal", value: "\\", description: "\"\\\\\"" },
              "r",
              { type: "literal", value: "r", description: "\"r\"" },
              "n",
              { type: "literal", value: "n", description: "\"n\"" },
              "t",
              { type: "literal", value: "t", description: "\"t\"" },
              "u",
              { type: "literal", value: "u", description: "\"u\"" },
              "\r",
              { type: "literal", value: "\r", description: "\"\\r\"" },
              "\n",
              { type: "literal", value: "\n", description: "\"\\n\"" },
              "//",
              { type: "literal", value: "//", description: "\"//\"" },
              /^[#\uFF03]/,
              { type: "class", value: "[#\\uFF03]", description: "[#\\uFF03]" },
              "/*",
              { type: "literal", value: "/*", description: "\"/*\"" },
              "*/",
              { type: "literal", value: "*/", description: "\"*/\"" },
              function (rc) { return rc; },
              "break",
              { type: "literal", value: "break", description: "\"break\"" },
              "case",
              { type: "literal", value: "case", description: "\"case\"" },
              "catch",
              { type: "literal", value: "catch", description: "\"catch\"" },
              "continue",
              { type: "literal", value: "continue", description: "\"continue\"" },
              "debugger",
              { type: "literal", value: "debugger", description: "\"debugger\"" },
              "default",
              { type: "literal", value: "default", description: "\"default\"" },
              "delete",
              { type: "literal", value: "delete", description: "\"delete\"" },
              "do",
              { type: "literal", value: "do", description: "\"do\"" },
              "else",
              { type: "literal", value: "else", description: "\"else\"" },
              "finally",
              { type: "literal", value: "finally", description: "\"finally\"" },
              "for",
              { type: "literal", value: "for", description: "\"for\"" },
              "function",
              { type: "literal", value: "function", description: "\"function\"" },
              "if",
              { type: "literal", value: "if", description: "\"if\"" },
              "in",
              { type: "literal", value: "in", description: "\"in\"" },
              "instanceof",
              { type: "literal", value: "instanceof", description: "\"instanceof\"" },
              "new",
              { type: "literal", value: "new", description: "\"new\"" },
              "return",
              { type: "literal", value: "return", description: "\"return\"" },
              "switch",
              { type: "literal", value: "switch", description: "\"switch\"" },
              "this",
              { type: "literal", value: "this", description: "\"this\"" },
              "throw",
              { type: "literal", value: "throw", description: "\"throw\"" },
              "try",
              { type: "literal", value: "try", description: "\"try\"" },
              "typeof",
              { type: "literal", value: "typeof", description: "\"typeof\"" },
              "var",
              { type: "literal", value: "var", description: "\"var\"" },
              "void",
              { type: "literal", value: "void", description: "\"void\"" },
              "while",
              { type: "literal", value: "while", description: "\"while\"" },
              "with",
              { type: "literal", value: "with", description: "\"with\"" },
              "const",
              { type: "literal", value: "const", description: "\"const\"" },
              "enum",
              { type: "literal", value: "enum", description: "\"enum\"" },
              "export",
              { type: "literal", value: "export", description: "\"export\"" },
              "extends",
              { type: "literal", value: "extends", description: "\"extends\"" },
              "import",
              { type: "literal", value: "import", description: "\"import\"" },
              "super",
              { type: "literal", value: "super", description: "\"super\"" },
              "abstract",
              { type: "literal", value: "abstract", description: "\"abstract\"" },
              "boolean",
              { type: "literal", value: "boolean", description: "\"boolean\"" },
              "byte",
              { type: "literal", value: "byte", description: "\"byte\"" },
              "char",
              { type: "literal", value: "char", description: "\"char\"" },
              "class",
              { type: "literal", value: "class", description: "\"class\"" },
              "double",
              { type: "literal", value: "double", description: "\"double\"" },
              "final",
              { type: "literal", value: "final", description: "\"final\"" },
              "float",
              { type: "literal", value: "float", description: "\"float\"" },
              "goto",
              { type: "literal", value: "goto", description: "\"goto\"" },
              "implements",
              { type: "literal", value: "implements", description: "\"implements\"" },
              "int",
              { type: "literal", value: "int", description: "\"int\"" },
              "interface",
              { type: "literal", value: "interface", description: "\"interface\"" },
              "long",
              { type: "literal", value: "long", description: "\"long\"" },
              "native",
              { type: "literal", value: "native", description: "\"native\"" },
              "package",
              { type: "literal", value: "package", description: "\"package\"" },
              "private",
              { type: "literal", value: "private", description: "\"private\"" },
              "protected",
              { type: "literal", value: "protected", description: "\"protected\"" },
              "public",
              { type: "literal", value: "public", description: "\"public\"" },
              "short",
              { type: "literal", value: "short", description: "\"short\"" },
              "static",
              { type: "literal", value: "static", description: "\"static\"" },
              "synchronized",
              { type: "literal", value: "synchronized", description: "\"synchronized\"" },
              "throws",
              { type: "literal", value: "throws", description: "\"throws\"" },
              "transient",
              { type: "literal", value: "transient", description: "\"transient\"" },
              "volatile",
              { type: "literal", value: "volatile", description: "\"volatile\"" },
              "let",
              { type: "literal", value: "let", description: "\"let\"" },
              "yield",
              { type: "literal", value: "yield", description: "\"yield\"" }
            ],

            peg$bytecode = [
              peg$decode("  7!+&$,#&7!\"\"\" !"),
              peg$decode("  7[+&$,#&7[\"\"\" !*) \"7\"*# \"7%"),
              peg$decode("!7$+\x85$  !!87#9*$$\"\" \"\"# !+-$7Q+#%'\"%$\"# !\"# !,F&!!87#9*$$\"\" \"\"# !+-$7Q+#%'\"%$\"# !\"# !\"+-%7$+#%'#%$## !$\"# !\"# !"),
              peg$decode(".#\"\"2#3$"),
              peg$decode("!7#+-$7Q+#%'\"%$\"# !\"# !"),
              peg$decode("!7'+5$  7&,#&7&\"+#%'\"%$\"# !\"# !"),
              peg$decode("7\"*) \"7-*# \"7R"),
              peg$decode("!7(+W$7(+M%7)*# \" %+=%70*# \" %+-%7R+#%'%%$%# !$$# !$## !$\"# !\"# !"),
              peg$decode("0&\"\"1!3'"),
              peg$decode("!7*+7$7,+-%7++#%'#%$## !$\"# !\"# !"),
              peg$decode("0(\"\"1!3)"),
              peg$decode("0*\"\"1!3+"),
              peg$decode("  !!87+*# \"7P9*$$\"\" \"\"# !+2$-\"\"1!3,+#%'\"%$\"# !\"# !,Q&!!87+*# \"7P9*$$\"\" \"\"# !+2$-\"\"1!3,+#%'\"%$\"# !\"# !\""),
              peg$decode("!!87#9*$$\"\" \"\"# !+L$  7.+&$,#&7.\"\"\" !+3%7P*# \" %+#%'#%$## !$\"# !\"# !"),
              peg$decode("78*5 \"74*/ \"7O*) \"7<*# \"7/"),
              peg$decode("  !!87P9*$$\"\" \"\"# !+2$-\"\"1!3,+#%'\"%$\"# !\"# !+N$,K&!!87P9*$$\"\" \"\"# !+2$-\"\"1!3,+#%'\"%$\"# !\"# !\"\"\" !"),
              peg$decode("!72+5$  73,#&73\"+#%'\"%$\"# !\"# !"),
              peg$decode("0-\"\"1!3."),
              peg$decode("!  7E,#&7E\"+-$79+#%'\"%$\"# !\"# !"),
              peg$decode("!  7E,#&7E\"+I$71+?%  7E,#&7E\"+-%79+#%'$%$$# !$## !$\"# !\"# !"),
              peg$decode("!75+\\$!76+5$  77,#&77\"+#%'\"%$\"# !\"# !+5%  7E,#&7E\"+#%'#%$## !$\"# !\"# !"),
              peg$decode("0/\"\"1!30"),
              peg$decode("!!8759*$$\"\" \"\"# !+-$7:+#%'\"%$\"# !\"# !"),
              peg$decode("!!8759*$$\"\" \"\"# !+-$7;+#%'\"%$\"# !\"# !"),
              peg$decode("!75+-$75+#%'\"%$\"# !\"# !"),
              peg$decode("!!!87\\9*$$\"\" \"\"# !+?$7:+5%  7;,#&7;\"+#%'#%$## !$\"# !\"# !+' 4!61!! %"),
              peg$decode("7N* \"02\"\"1!33*s \"!!87E9*$$\"\" \"\"# !+3$04\"\"1!35+#%'\"%$\"# !\"# !*D \"!06\"\"1!37+3$08\"\"1!39+#%'\"%$\"# !\"# !"),
              peg$decode("0:\"\"1!3;*# \"7:"),
              peg$decode("7=*2 \"  7>+&$,#&7>\"\"\" !"),
              peg$decode("0<\"\"1!3="),
              peg$decode("7?*/ \"7@*) \"7A*# \"7B"),
              peg$decode("7E*) \"0>\"\"1!3?"),
              peg$decode("0@\"\"1!3A"),
              peg$decode("0B\"\"1!3C"),
              peg$decode("0D\"\"1!3E"),
              peg$decode("0F\"\"1!3G"),
              peg$decode("0H\"\"1!3I"),
              peg$decode("0J\"\"1!3K"),
              peg$decode("0L\"\"1!3M*D \"!0N\"\"1!3O+3$08\"\"1!39+#%'\"%$\"# !\"# !"),
              peg$decode("0:\"\"1!3;"),
              peg$decode("0P\"\"1!3Q"),
              peg$decode(".R\"\"2R3S"),
              peg$decode("!7I+3$.T\"\"2T3U+#%'\"%$\"# !\"# !"),
              peg$decode("!7I+3$.V\"\"2V3W+#%'\"%$\"# !\"# !"),
              peg$decode("!7I+3$.X\"\"2X3Y+#%'\"%$\"# !\"# !"),
              peg$decode("!7I+2$-\"\"1!3,+#%'\"%$\"# !\"# !"),
              peg$decode("!7I+f$.Z\"\"2Z3[+V%!7H+A$7H+7%7H+-%7H+#%'$%$$# !$## !$\"# !\"# !+#%'#%$## !$\"# !\"# !"),
              peg$decode("7J*5 \"7K*/ \"7L*) \"7N*# \"7M"),
              peg$decode("!.\\\"\"2\\3]*# \" %+3$.^\"\"2^3_+#%'\"%$\"# !\"# !*) \".\\\"\"2\\3]"),
              peg$decode("!!!87P9*$$\"\" \"\"# !+2$-\"\"1!3,+#%'\"%$\"# !\"# !+-$7P+#%'\"%$\"# !\"# !"),
              peg$decode("!  7E,#&7E\"+-$7P+#%'\"%$\"# !\"# !"),
              peg$decode("!7T+-$7Q+#%'\"%$\"# !\"# !"),
              peg$decode(".`\"\"2`3a*) \"0b\"\"1!3c"),
              peg$decode("!7V+\x8F$  !!87W9*$$\"\" \"\"# !+2$-\"\"1!3,+#%'\"%$\"# !\"# !,K&!!87W9*$$\"\" \"\"# !+2$-\"\"1!3,+#%'\"%$\"# !\"# !\"+-%7W+#%'#%$## !$\"# !\"# !"),
              peg$decode(".d\"\"2d3e"),
              peg$decode(".f\"\"2f3g"),
              peg$decode("7S*# \"7U"),
              peg$decode("7X*) \"7E*# \"7P"),
              peg$decode("!  7Y,#&7Y\"+' 4!6h!! %"),
              peg$decode("7S*@ \"!  7U,#&7U\"+-$7R+#%'\"%$\"# !\"# !"),
              peg$decode(".i\"\"2i3j*\u0605 \".k\"\"2k3l*\u05F9 \".m\"\"2m3n*\u05ED \".o\"\"2o3p*\u05E1 \".q\"\"2q3r*\u05D5 \".s\"\"2s3t*\u05C9 \".u\"\"2u3v*\u05BD \".w\"\"2w3x*\u05B1 \".y\"\"2y3z*\u05A5 \".{\"\"2{3|*\u0599 \".}\"\"2}3~*\u058D \".\"\"23\x80*\u0581 \".\x81\"\"2\x813\x82*\u0575 \".\x83\"\"2\x833\x84*\u0569 \".\x85\"\"2\x853\x86*\u055D \".\x87\"\"2\x873\x88*\u0551 \".\x89\"\"2\x893\x8A*\u0545 \".\x8B\"\"2\x8B3\x8C*\u0539 \".\x8D\"\"2\x8D3\x8E*\u052D \".\x8F\"\"2\x8F3\x90*\u0521 \".\x91\"\"2\x913\x92*\u0515 \".\x93\"\"2\x933\x94*\u0509 \".\x95\"\"2\x953\x96*\u04FD \".\x97\"\"2\x973\x98*\u04F1 \".\x99\"\"2\x993\x9A*\u04E5 \".\x9B\"\"2\x9B3\x9C*\u04D9 \".k\"\"2k3l*\u04CD \".m\"\"2m3n*\u04C1 \".\x9D\"\"2\x9D3\x9E*\u04B5 \".q\"\"2q3r*\u04A9 \".s\"\"2s3t*\u049D \".w\"\"2w3x*\u0491 \".\x9F\"\"2\x9F3\xA0*\u0485 \".\xA1\"\"2\xA13\xA2*\u0479 \".\xA3\"\"2\xA33\xA4*\u046D \".{\"\"2{3|*\u0461 \".\xA5\"\"2\xA53\xA6*\u0455 \".\xA7\"\"2\xA73\xA8*\u0449 \".\x8B\"\"2\x8B3\x8C*\u043D \".\x8F\"\"2\x8F3\x90*\u0431 \".\x91\"\"2\x913\x92*\u0425 \".\xA9\"\"2\xA93\xAA*\u0419 \".\xAB\"\"2\xAB3\xAC*\u040D \".\xAD\"\"2\xAD3\xAE*\u0401 \".k\"\"2k3l*\u03F5 \".m\"\"2m3n*\u03E9 \".\xAF\"\"2\xAF3\xB0*\u03DD \".\xB1\"\"2\xB13\xB2*\u03D1 \".\x9D\"\"2\x9D3\x9E*\u03C5 \".q\"\"2q3r*\u03B9 \".s\"\"2s3t*\u03AD \".w\"\"2w3x*\u03A1 \".\xB3\"\"2\xB33\xB4*\u0395 \".\x9F\"\"2\x9F3\xA0*\u0389 \".\xA1\"\"2\xA13\xA2*\u037D \".\xA3\"\"2\xA33\xA4*\u0371 \".\xB5\"\"2\xB53\xB6*\u0365 \".{\"\"2{3|*\u0359 \".\xB7\"\"2\xB73\xB8*\u034D \".\xB9\"\"2\xB93\xBA*\u0341 \".\xBB\"\"2\xBB3\xBC*\u0335 \".\xA5\"\"2\xA53\xA6*\u0329 \".\x85\"\"2\x853\x86*\u031D \".\xBD\"\"2\xBD3\xBE*\u0311 \".\xBF\"\"2\xBF3\xC0*\u0305 \".\xC1\"\"2\xC13\xC2*\u02F9 \".\xC3\"\"2\xC33\xC4*\u02ED \".\xC5\"\"2\xC53\xC6*\u02E1 \".\xC7\"\"2\xC73\xC8*\u02D5 \".\xC9\"\"2\xC93\xCA*\u02C9 \".\xCB\"\"2\xCB3\xCC*\u02BD \".\xCD\"\"2\xCD3\xCE*\u02B1 \".\xCF\"\"2\xCF3\xD0*\u02A5 \".\xA7\"\"2\xA73\xA8*\u0299 \".\x8B\"\"2\x8B3\x8C*\u028D \".\xD1\"\"2\xD13\xD2*\u0281 \".\x8F\"\"2\x8F3\x90*\u0275 \".\xD3\"\"2\xD33\xD4*\u0269 \".\xD5\"\"2\xD53\xD6*\u025D \".\x91\"\"2\x913\x92*\u0251 \".\xD7\"\"2\xD73\xD8*\u0245 \".\xA9\"\"2\xA93\xAA*\u0239 \".\xAB\"\"2\xAB3\xAC*\u022D \".\xAD\"\"2\xAD3\xAE*\u0221 \".\xAF\"\"2\xAF3\xB0*\u0215 \".\xB1\"\"2\xB13\xB2*\u0209 \".\x9D\"\"2\x9D3\x9E*\u01FD \".q\"\"2q3r*\u01F1 \".\xB3\"\"2\xB33\xB4*\u01E5 \".\x9F\"\"2\x9F3\xA0*\u01D9 \".\xA1\"\"2\xA13\xA2*\u01CD \".\xA3\"\"2\xA33\xA4*\u01C1 \".\xB5\"\"2\xB53\xB6*\u01B5 \".\xB7\"\"2\xB73\xB8*\u01A9 \".\xB9\"\"2\xB93\xBA*\u019D \".\xBB\"\"2\xBB3\xBC*\u0191 \".\xA5\"\"2\xA53\xA6*\u0185 \".\xBD\"\"2\xBD3\xBE*\u0179 \".\xBF\"\"2\xBF3\xC0*\u016D \".\xC1\"\"2\xC13\xC2*\u0161 \".\xC3\"\"2\xC33\xC4*\u0155 \".\xC5\"\"2\xC53\xC6*\u0149 \".\xC7\"\"2\xC73\xC8*\u013D \".\xC9\"\"2\xC93\xCA*\u0131 \".\xCB\"\"2\xCB3\xCC*\u0125 \".\xCD\"\"2\xCD3\xCE*\u0119 \".\xCF\"\"2\xCF3\xD0*\u010D \".\xA7\"\"2\xA73\xA8*\u0101 \".\xD1\"\"2\xD13\xD2*\xF5 \".\xD3\"\"2\xD33\xD4*\xE9 \".\xD5\"\"2\xD53\xD6*\xDD \".\xD7\"\"2\xD73\xD8*\xD1 \".\xB1\"\"2\xB13\xB2*\xC5 \".\x9F\"\"2\x9F3\xA0*\xB9 \".\xA1\"\"2\xA13\xA2*\xAD \".\xA3\"\"2\xA33\xA4*\xA1 \".\xA5\"\"2\xA53\xA6*\x95 \".\xA7\"\"2\xA73\xA8*\x89 \".\xBB\"\"2\xBB3\xBC*} \".\xBF\"\"2\xBF3\xC0*q \".\xD9\"\"2\xD93\xDA*e \".\xC5\"\"2\xC53\xC6*Y \".\xC7\"\"2\xC73\xC8*M \".\xC9\"\"2\xC93\xCA*A \".\xCB\"\"2\xCB3\xCC*5 \".\xCF\"\"2\xCF3\xD0*) \".\xDB\"\"2\xDB3\xDC")
            ],

            peg$currPos = 0,
            peg$reportedPos = 0,
            peg$cachedPos = 0,
            peg$cachedPosDetails = { line: 1, column: 1, seenCR: false },
            peg$maxFailPos = 0,
            peg$maxFailExpected = [],
            peg$silentFails = 0,

            peg$result;

        if ("startRule" in options) {
            if (!(options.startRule in peg$startRuleIndices)) {
                throw new Error("Can't start parsing from rule \"" + options.startRule + "\".");
            }

            peg$startRuleIndex = peg$startRuleIndices[options.startRule];
        }

        function text() {
            return input.substring(peg$reportedPos, peg$currPos);
        }

        function offset() {
            return peg$reportedPos;
        }

        function line() {
            return peg$computePosDetails(peg$reportedPos).line;
        }

        function column() {
            return peg$computePosDetails(peg$reportedPos).column;
        }

        function expected(description) {
            throw peg$buildException(
              null,
              [{ type: "other", description: description }],
              peg$reportedPos
            );
        }

        function error(message) {
            throw peg$buildException(message, null, peg$reportedPos);
        }

        function peg$computePosDetails(pos) {
            function advance(details, startPos, endPos) {
                var p, ch;

                for (p = startPos; p < endPos; p++) {
                    ch = input.charAt(p);
                    if (ch === "\n") {
                        if (!details.seenCR) { details.line++; }
                        details.column = 1;
                        details.seenCR = false;
                    } else if (ch === "\r" || ch === "\u2028" || ch === "\u2029") {
                        details.line++;
                        details.column = 1;
                        details.seenCR = true;
                    } else {
                        details.column++;
                        details.seenCR = false;
                    }
                }
            }

            if (peg$cachedPos !== pos) {
                if (peg$cachedPos > pos) {
                    peg$cachedPos = 0;
                    peg$cachedPosDetails = { line: 1, column: 1, seenCR: false };
                }
                advance(peg$cachedPosDetails, peg$cachedPos, pos);
                peg$cachedPos = pos;
            }

            return peg$cachedPosDetails;
        }

        function peg$fail(expected) {
            if (peg$currPos < peg$maxFailPos) { return; }

            if (peg$currPos > peg$maxFailPos) {
                peg$maxFailPos = peg$currPos;
                peg$maxFailExpected = [];
            }

            peg$maxFailExpected.push(expected);
        }

        function peg$buildException(message, expected, pos) {
            function cleanupExpected(expected) {
                var i = 1;

                expected.sort(function (a, b) {
                    if (a.description < b.description) {
                        return -1;
                    } else if (a.description > b.description) {
                        return 1;
                    } else {
                        return 0;
                    }
                });

                while (i < expected.length) {
                    if (expected[i - 1] === expected[i]) {
                        expected.splice(i, 1);
                    } else {
                        i++;
                    }
                }
            }

            function buildMessage(expected, found) {
                function stringEscape(s) {
                    function hex(ch) { return ch.charCodeAt(0).toString(16).toUpperCase(); }

                    return s
                      .replace(/\\/g, '\\\\')
                      .replace(/"/g, '\\"')
                      .replace(/\x08/g, '\\b')
                      .replace(/\t/g, '\\t')
                      .replace(/\n/g, '\\n')
                      .replace(/\f/g, '\\f')
                      .replace(/\r/g, '\\r')
                      .replace(/[\x00-\x07\x0B\x0E\x0F]/g, function (ch) { return '\\x0' + hex(ch); })
                      .replace(/[\x10-\x1F\x80-\xFF]/g, function (ch) { return '\\x' + hex(ch); })
                      .replace(/[\u0180-\u0FFF]/g, function (ch) { return '\\u0' + hex(ch); })
                      .replace(/[\u1080-\uFFFF]/g, function (ch) { return '\\u' + hex(ch); });
                }

                var expectedDescs = new Array(expected.length),
                    expectedDesc, foundDesc, i;

                for (i = 0; i < expected.length; i++) {
                    expectedDescs[i] = expected[i].description;
                }

                expectedDesc = expected.length > 1
                  ? expectedDescs.slice(0, -1).join(", ")
                      + " or "
                      + expectedDescs[expected.length - 1]
                  : expectedDescs[0];

                foundDesc = found ? "\"" + stringEscape(found) + "\"" : "end of input";

                return "Expected " + expectedDesc + " but " + foundDesc + " found.";
            }

            var posDetails = peg$computePosDetails(pos),
                found = pos < input.length ? input.charAt(pos) : null;

            if (expected !== null) {
                cleanupExpected(expected);
            }

            return new SyntaxError(
              message !== null ? message : buildMessage(expected, found),
              expected,
              found,
              pos,
              posDetails.line,
              posDetails.column
            );
        }

        function peg$decode(s) {
            var bc = new Array(s.length), i;

            for (i = 0; i < s.length; i++) {
                bc[i] = s.charCodeAt(i) - 32;
            }

            return bc;
        }

        function peg$parseRule(index) {
            var bc = peg$bytecode[index],
                ip = 0,
                ips = [],
                end = bc.length,
                ends = [],
                stack = [],
                params, i;

            function protect(object) {
                return Object.prototype.toString.apply(object) === "[object Array]" ? [] : object;
            }

            while (true) {
                while (ip < end) {
                    switch (bc[ip]) {
                        case 0:
                            stack.push(protect(peg$consts[bc[ip + 1]]));
                            ip += 2;
                            break;

                        case 1:
                            stack.push(peg$currPos);
                            ip++;
                            break;

                        case 2:
                            stack.pop();
                            ip++;
                            break;

                        case 3:
                            peg$currPos = stack.pop();
                            ip++;
                            break;

                        case 4:
                            stack.length -= bc[ip + 1];
                            ip += 2;
                            break;

                        case 5:
                            stack.splice(-2, 1);
                            ip++;
                            break;

                        case 6:
                            stack[stack.length - 2].push(stack.pop());
                            ip++;
                            break;

                        case 7:
                            stack.push(stack.splice(stack.length - bc[ip + 1], bc[ip + 1]));
                            ip += 2;
                            break;

                        case 8:
                            stack.pop();
                            stack.push(input.substring(stack[stack.length - 1], peg$currPos));
                            ip++;
                            break;

                        case 9:
                            ends.push(end);
                            ips.push(ip + 3 + bc[ip + 1] + bc[ip + 2]);

                            if (stack[stack.length - 1]) {
                                end = ip + 3 + bc[ip + 1];
                                ip += 3;
                            } else {
                                end = ip + 3 + bc[ip + 1] + bc[ip + 2];
                                ip += 3 + bc[ip + 1];
                            }

                            break;

                        case 10:
                            ends.push(end);
                            ips.push(ip + 3 + bc[ip + 1] + bc[ip + 2]);

                            if (stack[stack.length - 1] === peg$FAILED) {
                                end = ip + 3 + bc[ip + 1];
                                ip += 3;
                            } else {
                                end = ip + 3 + bc[ip + 1] + bc[ip + 2];
                                ip += 3 + bc[ip + 1];
                            }

                            break;

                        case 11:
                            ends.push(end);
                            ips.push(ip + 3 + bc[ip + 1] + bc[ip + 2]);

                            if (stack[stack.length - 1] !== peg$FAILED) {
                                end = ip + 3 + bc[ip + 1];
                                ip += 3;
                            } else {
                                end = ip + 3 + bc[ip + 1] + bc[ip + 2];
                                ip += 3 + bc[ip + 1];
                            }

                            break;

                        case 12:
                            if (stack[stack.length - 1] !== peg$FAILED) {
                                ends.push(end);
                                ips.push(ip);

                                end = ip + 2 + bc[ip + 1];
                                ip += 2;
                            } else {
                                ip += 2 + bc[ip + 1];
                            }

                            break;

                        case 13:
                            ends.push(end);
                            ips.push(ip + 3 + bc[ip + 1] + bc[ip + 2]);

                            if (input.length > peg$currPos) {
                                end = ip + 3 + bc[ip + 1];
                                ip += 3;
                            } else {
                                end = ip + 3 + bc[ip + 1] + bc[ip + 2];
                                ip += 3 + bc[ip + 1];
                            }

                            break;

                        case 14:
                            ends.push(end);
                            ips.push(ip + 4 + bc[ip + 2] + bc[ip + 3]);

                            if (input.substr(peg$currPos, peg$consts[bc[ip + 1]].length) === peg$consts[bc[ip + 1]]) {
                                end = ip + 4 + bc[ip + 2];
                                ip += 4;
                            } else {
                                end = ip + 4 + bc[ip + 2] + bc[ip + 3];
                                ip += 4 + bc[ip + 2];
                            }

                            break;

                        case 15:
                            ends.push(end);
                            ips.push(ip + 4 + bc[ip + 2] + bc[ip + 3]);

                            if (input.substr(peg$currPos, peg$consts[bc[ip + 1]].length).toLowerCase() === peg$consts[bc[ip + 1]]) {
                                end = ip + 4 + bc[ip + 2];
                                ip += 4;
                            } else {
                                end = ip + 4 + bc[ip + 2] + bc[ip + 3];
                                ip += 4 + bc[ip + 2];
                            }

                            break;

                        case 16:
                            ends.push(end);
                            ips.push(ip + 4 + bc[ip + 2] + bc[ip + 3]);

                            if (peg$consts[bc[ip + 1]].test(input.charAt(peg$currPos))) {
                                end = ip + 4 + bc[ip + 2];
                                ip += 4;
                            } else {
                                end = ip + 4 + bc[ip + 2] + bc[ip + 3];
                                ip += 4 + bc[ip + 2];
                            }

                            break;

                        case 17:
                            stack.push(input.substr(peg$currPos, bc[ip + 1]));
                            peg$currPos += bc[ip + 1];
                            ip += 2;
                            break;

                        case 18:
                            stack.push(peg$consts[bc[ip + 1]]);
                            peg$currPos += peg$consts[bc[ip + 1]].length;
                            ip += 2;
                            break;

                        case 19:
                            stack.push(peg$FAILED);
                            if (peg$silentFails === 0) {
                                peg$fail(peg$consts[bc[ip + 1]]);
                            }
                            ip += 2;
                            break;

                        case 20:
                            peg$reportedPos = stack[stack.length - 1 - bc[ip + 1]];
                            ip += 2;
                            break;

                        case 21:
                            peg$reportedPos = peg$currPos;
                            ip++;
                            break;

                        case 22:
                            params = bc.slice(ip + 4, ip + 4 + bc[ip + 3]);
                            for (i = 0; i < bc[ip + 3]; i++) {
                                params[i] = stack[stack.length - 1 - params[i]];
                            }

                            stack.splice(
                              stack.length - bc[ip + 2],
                              bc[ip + 2],
                              peg$consts[bc[ip + 1]].apply(null, params)
                            );

                            ip += 4 + bc[ip + 3];
                            break;

                        case 23:
                            stack.push(peg$parseRule(bc[ip + 1]));
                            ip += 2;
                            break;

                        case 24:
                            peg$silentFails++;
                            ip++;
                            break;

                        case 25:
                            peg$silentFails--;
                            ip++;
                            break;

                        default:
                            throw new Error("Invalid opcode: " + bc[ip] + ".");
                    }
                }

                if (ends.length > 0) {
                    end = ends.pop();
                    ip = ips.pop();
                } else {
                    break;
                }
            }

            return stack[0];
        }


        function isSpace(c) {

        }
        function 漢字の変数(c) {
        }


        peg$result = peg$parseRule(peg$startRuleIndex);

        if (peg$result !== peg$FAILED && peg$currPos === input.length) {
            return peg$result;
        } else {
            if (peg$result !== peg$FAILED && peg$currPos < input.length) {
                peg$fail({ type: "end", description: "end of input" });
            }

            throw peg$buildException(null, peg$maxFailExpected, peg$maxFailPos);
        }
    }

    return {
        SyntaxError: SyntaxError,
        parse: parse
    };
})();