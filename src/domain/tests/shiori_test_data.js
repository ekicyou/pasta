// shiori_test_data

(function (definition) {
    // CommonJS/RequireJS/<script>
    if (typeof exports === "object") module.exports = definition();
    else if (typeof define === "function" && define.amd) define(definition);
    else shiori_test_data = definition();
})(function () {
    // 実際の定義を行う関数
    'use strict';
    var mod = {};
    mod.data = [
 {
    req: "GET Version SHIORI/2.6\r\nCharset: UTF-8\r\nSender: SSP\r\n\r\n"
  , res: "SHIORI/3.0 400 Bad Request\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
 }
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nID: version\r\nSecurityLevel: local\r\nSender: SSP\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: pasta-0.07.01/Duktape10000\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: enable_debug\r\nReference0: 1\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnInitialize\r\nReference0:\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: ownerghostname\r\nReference0: 今井知菜\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: basewareversion\r\nReference0: 2.3.51\r\nReference1: SSP\r\nReference2: 2.3.51.3000\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: capability\r\nReference0: request.status\r\nReference1: request.securitylevel\r\nReference2: request.baseid\r\nReference3: response.marker\r\nReference4: response.errorlevel\r\nReference5: response.errordescription\r\nReference6: response.securitylevel\r\nReference7: response.requestcharset\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnNotifyOSInfo\r\nReference0: WindowsNT,6.3,Windows 8.1\r\nReference1: Intel,4335,0.6.12.3,bfcbfbff\r\nReference2: 16710720,2097024\r\nReference3: 3756\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnNotifyFontInfo\r\nReference0: Terminal\r\nReference1: System\r\nReference2: FixedSys\r\nReference3: Courier\r\nReference4: MS Serif\r\nReference5: MS Sans Serif\r\nReference6: Small Fonts\r\nReference7: DINPro-Black\r\nReference8: DINPro-Bold\r\nReference9: DINPro-Light\r\nReference10: DINPro-Medium\r\nReference11: DINPro-Regular\r\nReference12: HelveticaNeueLT Pro 55 Roman\r\nReference13: Trajan Pro\r\nReference14: Terminal Greek 737 (437G)\r\nReference15: 8514oem\r\nReference16: DJS 秀英明朝 StdN B\r\nReference17: DJS 秀英明朝 StdN L\r\nReference18: DJS 秀英明朝 StdN M\r\nReference19: DJS 秀英横太明朝 StdN B\r\nReference20: DJS 秀英横太明朝 StdN M\r\nReference21: Terminal Greek 869\r\nReference22: フォントポにほんご\r\nReference23: はんなり明朝\r\nReference24: HP PSG\r\nReference25: ＫＦひま字\r\nReference26: キネ丸ボールド\r\nReference27: Modern\r\nReference28: Roman\r\nReference29: Script\r\nReference30: Segoe\r\nReference31: Segoe SemiBold\r\nReference32: SWMacro\r\nReference33: Function Pro Book\r\nReference34: Function Pro Medium\r\nReference35: Marlett\r\nReference36: Utsaah\r\nReference37: Microsoft JhengHei Light\r\nReference38: Microsoft JhengHei UI Light\r\nReference39: Sitka Small\r\nReference40: Sitka Text\r\nReference41: Sitka Subheading\r\nReference42: Sitka Heading\r\nReference43: Sitka Display\r\nReference44: Sitka Banner\r\nReference45: Microsoft YaHei\r\nReference46: Microsoft YaHei UI\r\nReference47: 游ゴシック\r\nReference48: ＭＳ 明朝\r\nReference49: ＭＳ Ｐ明朝\r\nReference50: Kokila\r\nReference51: メイリオ\r\nReference52: Meiryo UI\r\nReference53: Microsoft YaHei Light\r\nReference54: Microsoft YaHei UI Light\r\nReference55: ＭＳ ゴシック\r\nReference56: MS UI Gothic\r\nReference57: ＭＳ Ｐゴシック\r\nReference58: 游明朝 Demibold\r\nReference59: Microsoft JhengHei\r\nReference60: Microsoft JhengHei UI\r\nReference61: 游明朝 Light\r\nReference62: 游ゴシック Light\r\nReference63: Aparajita\r\nReference64: 游明朝\r\nReference65: SWTOR Trajan\r\nReference66: Razer Text Regular\r\nReference67: Razer Header Light\r\nReference68: Razer Header Regular\r\nReference69: Razer Header Regular Oblique\r\nReference70: MT Extra\r\nReference71: HG行書体\r\nReference72: HGP行書体\r\nReference73: HGS行書体\r\nReference74: HGｺﾞｼｯｸM\r\nReference75: HGPｺﾞｼｯｸM\r\nReference76: HGSｺﾞｼｯｸM\r\nReference77: HG教科書体\r\nReference78: HGP教科書体\r\nReference79: HGS教科書体\r\nReference80: HG明朝B\r\nReference81: HGP明朝B\r\nReference82: HGS明朝B\r\nReference83: HG明朝E\r\nReference84: HGP明朝E\r\nReference85: HGS明朝E\r\nReference86: HG創英ﾌﾟﾚｾﾞﾝｽEB\r\nReference87: HGP創英ﾌﾟﾚｾﾞﾝｽEB\r\nReference88: HGS創英ﾌﾟﾚｾﾞﾝｽEB\r\nReference89: HG正楷書体-PRO\r\nReference90: HG創英角ﾎﾟｯﾌﾟ体\r\nReference91: HGP創英角ﾎﾟｯﾌﾟ体\r\nReference92: HGS創英角ﾎﾟｯﾌﾟ体\r\nReference93: HG創英角ｺﾞｼｯｸUB\r\nReference94: HGP創英角ｺﾞｼｯｸUB\r\nReference95: HGS創英角ｺﾞｼｯｸUB\r\nReference96: HG丸ｺﾞｼｯｸM-PRO\r\nReference97: HGｺﾞｼｯｸE\r\nReference98: HGPｺﾞｼｯｸE\r\nReference99: HGSｺﾞｼｯｸE\r\nReference100: Agency FB\r\nReference101: Aharoni\r\nReference102: Aldhabi\r\nReference103: Algerian\r\nReference104: Andalus\r\nReference105: Andy\r\nReference106: Angsana New\r\nReference107: AngsanaUPC\r\nReference108: Book Antiqua\r\nReference109: あんずもじ\r\nReference110: あんずもじ湛\r\nReference111: あんずもじ奏\r\nReference112: あんずもじ等幅\r\nReference113: あくあＰフォント\r\nReference114: Arabic Typesetting\r\nReference115: Arial\r\nReference116: Arial Unicode MS\r\nReference117: Arial Black\r\nReference118: Arial Rounded MT Bold\r\nReference119: 有澤太楷書\r\nReference120: 有澤太楷書P\r\nReference121: Baskerville Old Face\r\nReference122: Batang\r\nReference123: BatangChe\r\nReference124: Gungsuh\r\nReference125: GungsuhChe\r\nReference126: Bauhaus 93\r\nReference127: Bell MT\r\nReference128: Bernard MT Condensed\r\nReference129: 恋文ペン字\r\nReference130: 麗流隷書\r\nReference131: Bodoni MT\r\nReference132: Bodoni MT Black\r\nReference133: Bodoni MT Condensed\r\nReference134: Bodoni MT Poster Compressed\r\nReference135: Bookman Old Style\r\nReference136: Bradley Hand ITC\r\nReference137: Britannic Bold\r\nReference138: Berlin Sans FB\r\nReference139: Berlin Sans FB Demi\r\nReference140: Broadway\r\nReference141: Browallia New\r\nReference142: BrowalliaUPC\r\nReference143: Brush Script MT\r\nReference144: Bookshelf Symbol 7\r\nReference145: Buxton Sketch\r\nReference146: Calibri\r\nReference147: Calibri Light\r\nReference148: Californian FB\r\nReference149: Calisto MT\r\nReference150: Cambria\r\nReference151: Cambria Math\r\nReference152: Candara\r\nReference153: Castellar\r\nReference154: Century Schoolbook\r\nReference155: Centaur\r\nReference156: Century\r\nReference157: Chiller\r\nReference158: CHRISTINA\r\nReference159: Colonna MT\r\nReference160: Comic Sans MS\r\nReference161: Consolas\r\nReference162: Constantia\r\nReference163: Cooper Black\r\nReference164: Copperplate Gothic Bold\r\nReference165: Copperplate Gothic Light\r\nReference166: Corbel\r\nReference167: Cordia New\r\nReference168: CordiaUPC\r\nReference169: Courier New\r\nReference170: Curlz MT\r\nReference171: DaunPenh\r\nReference172: David\r\nReference173: Dejima\r\nReference174: DengXian\r\nReference175: ＤＦ特太ゴシック体\r\nReference176: ＤＨＰ特太ゴシック体\r\nReference177: ＤＦ行書体\r\nReference178: ＤＨＰ行書体\r\nReference179: ＤＦ平成ゴシック体W5\r\nReference180: ＤＨＰ平成ゴシックW5\r\nReference181: ＤＦ平成明朝体W3\r\nReference182: ＤＨＰ平成明朝体W3\r\nReference183: ＤＦ平成明朝体W7\r\nReference184: ＤＨＰ平成明朝体W7\r\nReference185: DokChampa\r\nReference186: Ebrima\r\nReference187: 江戸勘亭流\r\nReference188: 江戸勘亭流Ｐ\r\nReference189: Elephant\r\nReference190: Engravers MT\r\nReference191: Eras Bold ITC\r\nReference192: Eras Demi ITC\r\nReference193: Eras Light ITC\r\nReference194: Eras Medium ITC\r\nReference195: Estrangelo Edessa\r\nReference196: Euphemia\r\nReference197: 有澤行書\r\nReference198: 有澤楷書\r\nReference199: Felix Titling\r\nReference200: ふみゴシック\r\nReference201: 魚石行書\r\nReference202: 祥南行書体\r\nReference203: 祥南行書体P\r\nReference204: 正調祥南行書体\r\nReference205: 正調祥南行書体P\r\nReference206: FGW FONT\r\nReference207: Forte\r\nReference208: Franklin Gothic Book\r\nReference209: Franklin Gothic Demi\r\nReference210: Franklin Gothic Demi Cond\r\nReference211: Franklin Gothic Heavy\r\nReference212: Franklin Gothic Medium\r\nReference213: Franklin Gothic Medium Cond\r\nReference214: FrankRuehl\r\nReference215: Freestyle Script\r\nReference216: French Script MT\r\nReference217: Footlight MT Light\r\nReference218: 富士ポップ\r\nReference219: 富士ポップＰ\r\nReference220: Gabriola\r\nReference221: Gadugi\r\nReference222: Garamond\r\nReference223: Gautami\r\nReference224: Georgia\r\nReference225: Gigi\r\nReference226: Gill Sans MT\r\nReference227: Gill Sans MT Condensed\r\nReference228: Gill Sans Ultra Bold Condensed\r\nReference229: Gill Sans Ultra Bold\r\nReference230: Gisha\r\nReference231: Gloucester MT Extra Condensed\r\nReference232: Gill Sans MT Ext Condensed Bold\r\nReference233: Century Gothic\r\nReference234: Goudy Old Style\r\nReference235: Goudy Stout\r\nReference236: Gulim\r\nReference237: GulimChe\r\nReference238: Dotum\r\nReference239: DotumChe\r\nReference240: Harlow Solid Italic\r\nReference241: Harrington\r\nReference242: Haettenschweiler\r\nReference243: HG明朝L\r\nReference244: HGP明朝L\r\nReference245: HGS明朝L\r\nReference246: HG平成角ｺﾞｼｯｸ体W3\r\nReference247: HGP平成角ｺﾞｼｯｸ体W3\r\nReference248: HGS平成角ｺﾞｼｯｸ体W3\r\nReference249: HG平成角ｺﾞｼｯｸ体W5\r\nReference250: HGP平成角ｺﾞｼｯｸ体W5\r\nReference251: HGS平成角ｺﾞｼｯｸ体W5\r\nReference252: HG平成角ｺﾞｼｯｸ体W9\r\nReference253: HGP平成角ｺﾞｼｯｸ体W9\r\nReference254: HGS平成角ｺﾞｼｯｸ体W9\r\nReference255: HG平成明朝体W3\r\nReference256: HGP平成明朝体W3\r\nReference257: HGS平成明朝体W3\r\nReference258: HG平成明朝体W9\r\nReference259: HGP平成明朝体W9\r\nReference260: HGS平成明朝体W9\r\nReference261: Microsoft Himalaya\r\nReference262: High Tower Text\r\nReference263: ふい字\r\nReference264: ふい字Ｐ\r\nReference265: Impact\r\nReference266: Imprint MT Shadow\r\nReference267: Informal Roman\r\nReference268: Iskoola Pota\r\nReference269: Blackadder ITC\r\nReference270: Edwardian Script ITC\r\nReference271: Kristen ITC\r\nReference272: Javanese Text\r\nReference273: CenturyOldst\r\nReference274: Embassy JS\r\nReference275: Fraktur JS\r\nReference276: %CenturyOldst\r\nReference277: Gothic720\r\nReference278: ARマーカー体E\r\nReference279: AR Pマーカー体E\r\nReference280: AR丸ゴシック体E\r\nReference281: AR P丸ゴシック体E\r\nReference282: AR丸ゴシック体M\r\nReference283: AR P丸ゴシック体M\r\nReference284: ARゴシック体M\r\nReference285: AR Pゴシック体M\r\nReference286: ARゴシック体S\r\nReference287: AR Pゴシック体S\r\nReference288: AR悠々ゴシック体E\r\nReference289: AR P悠々ゴシック体E\r\nReference290: ARマッチ体B\r\nReference291: AR Pマッチ体B\r\nReference292: Jing Jing\r\nReference293: &CenturyOldst\r\nReference294: &Gothic720\r\nReference295: AR楷書体M\r\nReference296: AR P楷書体M\r\nReference297: AR勘亭流H\r\nReference298: AR P勘亭流H\r\nReference299: AR明朝体L\r\nReference300: AR P明朝体L\r\nReference301: AR明朝体U\r\nReference302: AR P明朝体U\r\nReference303: Jokerman\r\nReference304: ARペン楷書体L\r\nReference305: AR Pペン楷書体L\r\nReference306: AR浪漫明朝体U\r\nReference307: AR P浪漫明朝体U\r\nReference308: JustEditMark\r\nReference309: JustHalfMarkG\r\nReference310: JustHalfMark\r\nReference311: ＪＳ平成明朝体W3\r\nReference312: JustKanaMarkG\r\nReference313: JustKanaMark\r\nReference314: JustOubunMarkG\r\nReference315: JustOubunMark\r\nReference316: $ＪＳゴシック\r\nReference317: $ＪＳ明朝\r\nReference318: JustUnitMarkG\r\nReference319: JustUnitMark\r\nReference320: JustWabunMarkG\r\nReference321: JustWabunMark\r\nReference322: AR教科書体M\r\nReference323: AR P教科書体M\r\nReference324: Juice ITC\r\nReference325: AR顏眞楷書体H\r\nReference326: AR P顏眞楷書体H\r\nReference327: DFKai-SB\r\nReference328: Kalinga\r\nReference329: Kartika\r\nReference330: Khmer UI\r\nReference331: Kootenay\r\nReference332: Kunstler Script\r\nReference333: Lucida Sans Unicode\r\nReference334: Lao UI\r\nReference335: Latha\r\nReference336: Wide Latin\r\nReference337: Lucida Bright\r\nReference338: Lucida Calligraphy\r\nReference339: Leelawadee UI\r\nReference340: Leelawadee\r\nReference341: Leelawadee UI Semilight\r\nReference342: Lucida Fax\r\nReference343: Lucida Handwriting\r\nReference344: Lindsey\r\nReference345: Lucida Sans\r\nReference346: Lucida Sans Typewriter\r\nReference347: Lucida Console\r\nReference348: Levenim MT\r\nReference349: Magneto\r\nReference350: Maiandra GD\r\nReference351: Sakkal Majalla\r\nReference352: まきばフォント\r\nReference353: まきばフォント太\r\nReference354: まきばフォント太Ｐ\r\nReference355: まきばフォントＰ\r\nReference356: Malgun Gothic\r\nReference357: Mangal\r\nReference358: Matura MT Script Capitals\r\nReference359: Microsoft Sans Serif\r\nReference360: MigMix 1P\r\nReference361: Migu 1M\r\nReference362: Migu 1VS\r\nReference363: みかちゃん-ぷち\r\nReference364: みかちゃん-ぷちB\r\nReference365: みかちゃん\r\nReference366: みかちゃん-P\r\nReference367: みかちゃん-PB\r\nReference368: みかちゃん-PS\r\nReference369: MingLiU\r\nReference370: PMingLiU\r\nReference371: MingLiU_HKSCS\r\nReference372: MingLiU-ExtB\r\nReference373: PMingLiU-ExtB\r\nReference374: MingLiU_HKSCS-ExtB\r\nReference375: Miramonte\r\nReference376: 美咲ゴシック\r\nReference377: 美咲明朝\r\nReference378: Mistral\r\nReference379: Myanmar Text\r\nReference380: MoboGothic\r\nReference381: MoboExGothic\r\nReference382: Mobo90Gothic\r\nReference383: MoboEx90Gothic\r\nReference384: Modern No. 20\r\nReference385: MogaGothic\r\nReference386: MogaExGothic\r\nReference387: Moga90Gothic\r\nReference388: MogaEx90Gothic\r\nReference389: MogaMincho\r\nReference390: MogaExMincho\r\nReference391: Moga90Mincho\r\nReference392: MogaEx90Mincho\r\nReference393: Moire\r\nReference394: Moire ExtraBold\r\nReference395: Moire Light\r\nReference396: Mongolian Baiti\r\nReference397: MoolBoran\r\nReference398: Motorwerk\r\nReference399: Miriam\r\nReference400: Miriam Fixed\r\nReference401: Microsoft MHei\r\nReference402: Microsoft NeoGothic\r\nReference403: Microsoft Uighur\r\nReference404: Microsoft Yi Baiti\r\nReference405: Monotype Corsiva\r\nReference406: MV Boli\r\nReference407: Myriad Web Pro\r\nReference408: Myriad Web Pro Condensed\r\nReference409: MYSTICAL\r\nReference410: News Gothic\r\nReference411: Niagara Engraved\r\nReference412: Niagara Solid\r\nReference413: Nina\r\nReference414: Nirmala UI\r\nReference415: Nirmala UI Semilight\r\nReference416: Narkisim\r\nReference417: Microsoft New Tai Lue\r\nReference418: Nyala\r\nReference419: OCR A Extended\r\nReference420: OCRB\r\nReference421: おひさまフォント\r\nReference422: おひさまフォント太\r\nReference423: Old English Text MT\r\nReference424: Onyx\r\nReference425: Palatino Linotype\r\nReference426: Palace Script MT\r\nReference427: Papyrus\r\nReference428: Parchment\r\nReference429: Perpetua\r\nReference430: Pericles\r\nReference431: Pericles Light\r\nReference432: Perpetua Titling MT\r\nReference433: Pescadero\r\nReference434: Microsoft PhagsPa\r\nReference435: Plantagenet Cherokee\r\nReference436: Playbill\r\nReference437: Poor Richard\r\nReference438: Pristina\r\nReference439: Quartz MS\r\nReference440: Raavi\r\nReference441: Rage Italic\r\nReference442: Ravie\r\nReference443: MS Reference Sans Serif\r\nReference444: MS Reference Specialty\r\nReference445: Rockwell Condensed\r\nReference446: Rockwell\r\nReference447: Rockwell Extra Bold\r\nReference448: Rod\r\nReference449: Script MT Bold\r\nReference450: Segoe Condensed\r\nReference451: Segoe360\r\nReference452: Segoe Keycaps\r\nReference453: Segoe Marker\r\nReference454: Segoe Print\r\nReference455: Segoe Script\r\nReference456: Segoe UI\r\nReference457: Segoe UI Light\r\nReference458: Segoe UI Mono\r\nReference459: Segoe UI Semilight\r\nReference460: Segoe WP\r\nReference461: Segoe WP Black\r\nReference462: Segoe WP Light\r\nReference463: Segoe WP Semibold\r\nReference464: Segoe WP SemiLight\r\nReference465: Segoe UI Black\r\nReference466: Segoe UI Emoji\r\nReference467: Segoe UI Semibold\r\nReference468: Segoe UI Symbol\r\nReference469: Shonar Bangla\r\nReference470: Showcard Gothic\r\nReference471: Shruti\r\nReference472: FangSong\r\nReference473: SimHei\r\nReference474: KaiTi\r\nReference475: Simplified Arabic\r\nReference476: Simplified Arabic Fixed\r\nReference477: SimSun\r\nReference478: NSimSun\r\nReference479: SimSun-ExtB\r\nReference480: SketchFlow Print\r\nReference481: Snap ITC\r\nReference482: Stencil\r\nReference483: STROKE\r\nReference484: SWGamekeys MT\r\nReference485: 正調祥南行書体EX\r\nReference486: 正調祥南行書体EXP\r\nReference487: Sylfaen\r\nReference488: Symbol\r\nReference489: Tahoma\r\nReference490: Microsoft Tai Le\r\nReference491: たぬき油性マジック\r\nReference492: Tw Cen MT\r\nReference493: Tw Cen MT Condensed\r\nReference494: Tw Cen MT Condensed Extra Bold\r\nReference495: Tempus Sans ITC\r\nReference496: TGothic-GT01\r\nReference497: TPGothic-GT01\r\nReference498: TGothic-GT02\r\nReference499: TPGothic-GT02\r\nReference500: TGothic-GT03\r\nReference501: TPGothic-GT03\r\nReference502: TGothic-GT04\r\nReference503: TPGothic-GT04\r\nReference504: TGothic-GT05\r\nReference505: TPGothic-GT05\r\nReference506: TGothic-GT06\r\nReference507: TPGothic-GT06\r\nReference508: TGothic-GT07\r\nReference509: TPGothic-GT07\r\nReference510: TGothic-GT08\r\nReference511: TPGothic-GT08\r\nReference512: TGothic-GT09\r\nReference513: TPGothic-GT09\r\nReference514: TGothic-GT10\r\nReference515: TPGothic-GT10\r\nReference516: TGothic-GT11\r\nReference517: TPGothic-GT11\r\nReference518: TGothic-GT12\r\nReference519: TPGothic-GT12\r\nReference520: Times New Roman\r\nReference521: TKaisho-GT01\r\nReference522: TPKaisho-GT01\r\nReference523: TMincho-GT01\r\nReference524: TPMincho-GT01\r\nReference525: Traditional Arabic\r\nReference526: Trebuchet MS\r\nReference527: Tunga\r\nReference528: DilleniaUPC\r\nReference529: EucrosiaUPC\r\nReference530: FreesiaUPC\r\nReference531: IrisUPC\r\nReference532: JasmineUPC\r\nReference533: KodchiangUPC\r\nReference534: LilyUPC\r\nReference535: Urdu Typesetting\r\nReference536: Vani\r\nReference537: Verdana\r\nReference538: Vijaya\r\nReference539: Viner Hand ITC\r\nReference540: Vivaldi\r\nReference541: Vladimir Script\r\nReference542: Vrinda\r\nReference543: Webdings\r\nReference544: Wingdings\r\nReference545: Wingdings 2\r\nReference546: Wingdings 3\r\nReference547: YOzFont5x7dB\r\nReference548: YOzFont5x7dEL\r\nReference549: YOzFont5x7dL\r\nReference550: YOzFont5x7dM\r\nReference551: YOzFont5x7dR\r\nReference552: YOzFont14s\r\nReference553: YOzFontA\r\nReference554: YOzFontA90\r\nReference555: YOzFontAF\r\nReference556: YOzFontAF90\r\nReference557: YOzFontAP\r\nReference558: YOzFontAP90\r\nReference559: YOzFontC\r\nReference560: YOzFontC90\r\nReference561: YOzFontCF\r\nReference562: YOzFontCF90\r\nReference563: YOzFontE\r\nReference564: YOzFontEM\r\nReference565: YOzFontE90\r\nReference566: YOzFontEM90\r\nReference567: YOzFontEF\r\nReference568: YOzFontEMF\r\nReference569: YOzFontEF90\r\nReference570: YOzFontEMF90\r\nReference571: YOzFontN\r\nReference572: YOzFontNM\r\nReference573: YOzFontN90\r\nReference574: YOzFontNM90\r\nReference575: YOzFontNF\r\nReference576: YOzFontNMF\r\nReference577: YOzFontNF90\r\nReference578: YOzFontNMF90\r\nReference579: YOzFont\r\nReference580: YOzFont90\r\nReference581: YOzFontF\r\nReference582: YOzFontF90\r\nReference583: YOzFontP\r\nReference584: YOzFontP90\r\nReference585: YOzFontK\r\nReference586: YOzFontK90\r\nReference587: YOzFontKA\r\nReference588: YOzFontKA90\r\nReference589: YOzFontM90\r\nReference590: YOzFontOTW\r\nReference591: YOzFontOTWL\r\nReference592: YOzFontOTWD\r\nReference593: YOzFontOTW Light\r\nReference594: YOzFontOTWL Light\r\nReference595: YOzFontOTWD Light\r\nReference596: YOzFontEX\r\nReference597: YOzFontEXM\r\nReference598: YOzFontEXF\r\nReference599: YOzFontEX90\r\nReference600: YOzFontEXM90\r\nReference601: YOzFontEXF90\r\nReference602: YOzFontNX\r\nReference603: YOzFontNXM\r\nReference604: YOzFontNXF\r\nReference605: YOzFontNX90\r\nReference606: YOzFontNXM90\r\nReference607: YOzFontNXF90\r\nReference608: YOzFontX\r\nReference609: YOzFontXM\r\nReference610: YOzFontXF\r\nReference611: YOzFontX90\r\nReference612: YOzFontXM90\r\nReference613: YOzFontXF90\r\nReference614: Yu Gothic\r\nReference615: ZWAdobeF\r\nReference616: Arial Narrow\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnNotifySelfInfo\r\nReference0: いまいち萌えない娘\r\nReference1: 今井知菜\r\nReference2: へんしゅう\r\nReference3: いまいちむすめ\r\nReference4: D:/wintools/ssp/ghost/imamoe/shell/imaitimusume/\r\nReference5: いまいちなバルーン\r\nReference6: D:/wintools/ssp/balloon/ks_balloon/\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnNotifyBalloonInfo\r\nReference0: いまいちなバルーン\r\nReference1: D:/wintools/ssp/balloon/ks_balloon/\r\nReference2: 0:0,1,2,3 1:0,1\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnNotifyShellInfo\r\nReference0: いまいちむすめ\r\nReference1: D:/wintools/ssp/ghost/imamoe/shell/imaitimusume/\r\nReference2: 0,1,2,3,4,5,6,7,8,9,10,11,12,20,100,110,201,202,203,204,211,212,213,214,301,302,303,304,305,401,402,403,1000\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: useorigin1\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: 1\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: name\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: craftman\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: dot-station\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: craftmanw\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: どっとステーション\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnNotifyUserInfo\r\nReference0: えちょ\r\nReference1: えちょ\r\nReference2: 1971,04,21\r\nReference3: male\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nID: otherghostname\r\nSecurityLevel: local\r\nCharset: UTF-8\r\nSender: SSP\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnNetworkStatusChange\r\nReference0: online\r\nReference1: 192.168.1.208192.168.56.1\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: ghostpathlist\r\nReference0: D:\\wintools\\ssp\\ghost\\\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: balloonpathlist\r\nReference0: D:\\wintools\\ssp\\balloon\\\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: headlinepathlist\r\nReference0: D:\\wintools\\ssp\\headline\\\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: pluginpathlist\r\nReference0: D:\\wintools\\ssp\\plugin\\\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: calendarskinpathlist\r\nReference0: D:\\wintools\\ssp\\calendar\\skin\\\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: calendarpluginpathlist\r\nReference0: D:\\wintools\\ssp\\calendar\\plugin\\\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: menu.background.bitmap.filename\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: menu.foreground.bitmap.filename\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: menu.background.color.r\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: menu.background.color.g\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: menu.background.color.b\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: menu.foreground.color.r\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: menu.foreground.color.g\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: menu.foreground.color.b\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: username\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: <<todo.username>>\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: installedghostname\r\nReference0: Emily/Phase4.5\r\nReference1: いまいち萌えない娘\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: installedballoonname\r\nReference0: Balloon for Emily/P4\r\nReference1: SSPデフォルト+\r\nReference2: いまいちなバルーン\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: installedheadlinename\r\nReference0: GAME Watch\r\nReference1: Google ニュース 日本語版\r\nReference2: ITmedia +D Games\r\nReference3: ITMedia News\r\nReference4: SlashDot-JP\r\nReference5: ねとわくアンテナ\r\nReference6: ハァハァアンテナ\r\nReference7: 何かアンテナ\r\nReference8: 各種スレアンテナ\r\nReference9: 回収・無償修理等のお知らせ\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: sakura.recommendsites\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: <<todo.sakura$recommendsites>>\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: sakura.portalsites\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: <<todo.sakura$portalsites>>\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: kero.recommendsites\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: <<todo.kero$recommendsites>>\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: installedshellname\r\nReference0: Master\r\nReference1: Master2\r\nReference2: Master3\r\nReference3: いまいちむすめ\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: uniqueid\r\nReference0: ssp_fmo_header_000003b4_00030c9e\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nID: otherghostname\r\nSecurityLevel: local\r\nCharset: UTF-8\r\nSender: SSP\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: hwnd\r\nReference0: 1998381248108\r\nReference1: 1998642297072\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: installedshellname\r\nReference0: Master\r\nReference1: Master2\r\nReference2: Master3\r\nReference3: いまいちむすめ\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnDisplayChange\r\nReference0: 32\r\nReference1: 2560\r\nReference2: 1600\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnBatteryNotify\r\nReference0: -1\r\nReference1: -1\r\nReference2: online\r\nReference3: no_battery\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnNotifyDressupInfo\r\nReference0: 0眼鏡ボストン0\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnScheduleTodayNotify\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: installedplugin\r\nReference0: SwissArmyKnife8F8BCFB8-B27A-456f-9BA0-551484856DDC\r\nReference1: 共有変数プラグインABED14AF-F34B-4ff2-95B7-30ED37D5802D\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: rateofusegraph\r\nReference0: Emily/Phase4.5EmilyTeddy12571.428571install\r\nReference1: いまいち萌えない娘今井知菜へんしゅう21028.571429boot\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSurfaceChange\r\nReference0: 0\r\nReference1: 10\r\nReference2: 0,0,250,332\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSurfaceChange\r\nReference0: 0\r\nReference1: 10\r\nReference2: 1,10,200,243\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 1\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 1\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 1\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 2\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 3\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 4\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 5\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 6\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 7\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 8\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 9\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMinuteChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 9\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 10\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 11\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 12\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 13\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 14\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnBatteryNotify\r\nReference0: -1\r\nReference1: -1\r\nReference2: online\r\nReference3: no_battery\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: rateofusegraph\r\nReference0: Emily/Phase4.5EmilyTeddy12569.444444install\r\nReference1: いまいち萌えない娘今井知菜へんしゅう21130.555556boot\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 15\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 16\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 17\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 18\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 19\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSurfaceRestore\r\nReference0: 0\r\nReference1: 10\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: \\0\\![set,trayballoon,--title=新着情報1,--timeout=10000,--icon=info,新しい着せ替え対応シェルが追加されました。,詳しくはここをクリック]\\0\\s[0]\\1\\s[10]\\e\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nID: OnTranslate\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nReference0: \\0\\![set,trayballoon,--title=新着情報1,--timeout=10000,--icon=info,新しい着せ替え対応シェルが追加されました。,詳しくはここをクリック]\\0\\s[0]\\1\\s[10]\\e\r\nReference1:\r\nReference2: OnSurfaceRestore\r\nReference3: 010\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 20\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 21\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 22\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 23\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 24\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 25\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 26\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 27\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 28\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 29\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 30\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 31\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 32\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 33\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 34\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 35\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 36\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 37\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 38\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 39\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 40\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 41\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 42\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 43\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 44\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 1\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 1\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 2\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnTrayBalloonTimeout\r\nReference0: 新着情報1\r\nReference1: 新しい着せ替え対応シェルが追加されました。  詳しくはここをクリック\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 3\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 4\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 5\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 6\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 7\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 8\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 9\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 10\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 11\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseEnterAll\r\nReference0: 97\r\nReference1: 49\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseEnter\r\nReference0: 97\r\nReference1: 49\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseMove\r\nReference0: 97\r\nReference1: 49\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseMove\r\nReference0: 111\r\nReference1: 60\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseMove\r\nReference0: 116\r\nReference1: 62\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseMove\r\nReference0: 119\r\nReference1: 62\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseMove\r\nReference0: 120\r\nReference1: 62\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseMove\r\nReference0: 121\r\nReference1: 64\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseHover\r\nReference0: 122\r\nReference1: 64\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: tooltip\r\nReference0: 122\r\nReference1: 64\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseDown\r\nReference0: 122\r\nReference1: 64\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseUp\r\nReference0: 122\r\nReference1: 64\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseClick\r\nReference0: 122\r\nReference1: 64\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseDoubleClick\r\nBaseID: OnMouseDown\r\nReference0: 122\r\nReference1: 64\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: \\1\\s[10]\\0\\0\\s[6]…\\w5…\\w5女の子の頭ドつくなんて。\\w9\\w9\\1愛のムチかもしれんぞ？\\w9\\w9\\0\\n\\n[half]愛…\\w5…\\w5…\\w5…\\w9\\w9\\w9\\w9\\0\\s[1]\\n…\\w5…\\w5…\\w5…\\w5…\\w9\\w9\\1\\n\\n[half]やばい方向に目覚めるんじゃねえぞ？\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nID: OnTranslate\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nReference0: \\1\\s[10]\\0\\0\\s[6]…\\w5…\\w5女の子の頭ドつくなんて。\\w9\\w9\\1愛のムチかもしれんぞ？\\w9\\w9\\0\\n\\n[half]愛…\\w5…\\w5…\\w5…\\w9\\w9\\w9\\w9\\0\\s[1]\\n…\\w5…\\w5…\\w5…\\w5…\\w9\\w9\\1\\n\\n[half]やばい方向に目覚めるんじゃねえぞ？\r\nReference1:\r\nReference2: OnMouseDoubleClick\r\nReference3: 1226400head0mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSurfaceChange\r\nReference0: 6\r\nReference1: 10\r\nReference2: 0,6,250,332\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnMouseUp\r\nReference0: 122\r\nReference1: 64\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnMouseMove\r\nReference0: 122\r\nReference1: 63\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnMouseLeave\r\nReference0: 108\r\nReference1: 36\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnMouseLeaveAll\r\nReference0: 108\r\nReference1: 36\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnMouseEnterAll\r\nReference0: 19\r\nReference1: 193\r\nReference2: 0\r\nReference3: 1\r\nReference4:\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnMouseEnter\r\nReference0: 19\r\nReference1: 193\r\nReference2: 0\r\nReference3: 1\r\nReference4:\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnMouseMove\r\nReference0: 19\r\nReference1: 193\r\nReference2: 0\r\nReference3: 1\r\nReference4:\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnMouseLeave\r\nReference0: 2\r\nReference1: 169\r\nReference2: 0\r\nReference3: 1\r\nReference4:\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnMouseLeaveAll\r\nReference0: 2\r\nReference1: 169\r\nReference2: 0\r\nReference3: 1\r\nReference4:\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 1\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSurfaceChange\r\nReference0: 1\r\nReference1: 10\r\nReference2: 0,1,250,332\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnMinuteChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnBatteryNotify\r\nReference0: -1\r\nReference1: -1\r\nReference2: online\r\nReference3: no_battery\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: rateofusegraph\r\nReference0: Emily/Phase4.5EmilyTeddy12567.567568install\r\nReference1: いまいち萌えない娘今井知菜へんしゅう21232.432432boot\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 1\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 2\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 3\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 4\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 5\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 6\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 7\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 8\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 9\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 10\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 11\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 12\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 13\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 14\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 15\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseEnterAll\r\nReference0: 13\r\nReference1: 186\r\nReference2: 0\r\nReference3: 1\r\nReference4:\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseEnter\r\nReference0: 13\r\nReference1: 186\r\nReference2: 0\r\nReference3: 1\r\nReference4:\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseMove\r\nReference0: 13\r\nReference1: 186\r\nReference2: 0\r\nReference3: 1\r\nReference4:\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseLeave\r\nReference0: 19\r\nReference1: 206\r\nReference2: 0\r\nReference3: 1\r\nReference4:\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseLeaveAll\r\nReference0: 19\r\nReference1: 206\r\nReference2: 0\r\nReference3: 1\r\nReference4:\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseEnterAll\r\nReference0: 102\r\nReference1: 43\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseEnter\r\nReference0: 102\r\nReference1: 43\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseMove\r\nReference0: 102\r\nReference1: 43\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseMove\r\nReference0: 111\r\nReference1: 57\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseMove\r\nReference0: 113\r\nReference1: 61\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseMove\r\nReference0: 114\r\nReference1: 62\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseMove\r\nReference0: 120\r\nReference1: 68\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseDown\r\nReference0: 123\r\nReference1: 70\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 1\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseUp\r\nReference0: 123\r\nReference1: 70\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 1\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseClick\r\nReference0: 123\r\nReference1: 70\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 1\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: sakura.popupmenu.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: sakura.popupmenu.type\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: recommendrootbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: sakura.recommendbuttoncaption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: recommendrootbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: 神戸とか、新聞社とか\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: portalrootbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: sakura.portalbuttoncaption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: portalrootbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: ポータル\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: homeurl\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: http://ms.shillest.net/ghost/imamoe/\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: updatebutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: updatebuttoncaption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: sakura.updatebuttoncaption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: updatebutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: ネットワーク更新\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: readmebutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: readmebuttoncaption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: readmebutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: read me...\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: vanishbuttonvisible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: vanishbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: 1\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: vanishbuttoncaption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: vanishbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: やっぱ萌えない\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: aistatebutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: getaistate\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: aistatebutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: sakura.portalsites\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: 神戸新聞http://www.kobe-np.co.jp/いまっち公式サイトhttp://imamoe.jp/kobeimamoe/ひょうご・神戸 いまもえ.jphttp://www.imamoe.jp/-----SSP BUGTRAQhttp://ssp.shillest.net/整備班http://ms.shillest.net/YAYAhttp://code.google.com/p/yaya-shiori/里々/整備班カスタムhttp://code.google.com/p/satoriya-shiori/華和梨http://kawari.sourceforge.net/\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: headlinesenserootbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: headlinesenserootbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: pluginrootbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: pluginrootbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: biffbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: biffbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: shellscalerootbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: shellscalerootbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: utilityrootbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: utilityrootbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: calendarbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: calendarbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: messengerbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: messengerbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: sntpbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: sntpbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: ghostexplorerbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: ghostexplorerbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: scriptlogbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: scriptlogbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: addressbarbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: addressbarbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: openfilebutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: openfilebutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: openfolderbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: openfolderbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: updateplatformbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: updateplatformbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: purgeghostcachebutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: purgeghostcachebutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: updatefmobutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: updatefmobutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: reloadinfobutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: reloadinfobutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: switchreloadbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: switchreloadbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: leavepassivebutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: leavepassivebutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: switchreloadtempghostbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: switchreloadtempghostbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: switchmovetodefaultpositionbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: switchmovetodefaultpositionbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: resetballoonpositionbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: resetballoonpositionbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: closeballoonbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: closeballoonbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: duibutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: duibutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: configurationrootbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: configurationrootbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: configurationbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: configurationbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: configurationghostbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: configurationghostbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: charsetbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: charsetbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: switchproxybutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: switchproxybutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: switchtalkghostbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: switchtalkghostbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: switchcompatiblemodebutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: switchcompatiblemodebutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: ghostrootbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: ghostrootbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: callghostrootbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: callghostrootbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: shellrootbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: shellrootbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: dressuprootbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: dressuprootbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: balloonrootbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: balloonrootbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: historyrootbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: historyrootbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: ghosthistorybutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: ghosthistorybutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: callghosthistorybutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: callghosthistorybutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: balloonhistorybutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: balloonhistorybutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: headlinesensehistorybutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: headlinesensehistorybutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: pluginhistorybutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: pluginhistorybutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: inforootbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: inforootbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: firststaffbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: firststaffbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: readmebutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: readmebutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: read me...\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: helpbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: helpbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: rateofusebutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: rateofusebutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: systeminfobutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: systeminfobutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: hidebutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: hidebutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: closebutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: closebutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: quitbutton.visible\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: quitbutton.caption\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseLeave\r\nReference0: 123\r\nReference1: 70\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMouseLeaveAll\r\nReference0: 123\r\nReference1: 70\r\nReference2: 0\r\nReference3: 0\r\nReference4: head\r\nReference5: 0\r\nReference6: mouse\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSurfaceRestore\r\nReference0: 1\r\nReference1: 10\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: \\0\\s[0]\\1\\s[10]\\e\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nID: OnTranslate\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nReference0: \\0\\s[0]\\1\\s[10]\\e\r\nReference1:\r\nReference2: OnSurfaceRestore\r\nReference3: 110\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSurfaceChange\r\nReference0: 0\r\nReference1: 10\r\nReference2: 0,0,250,332\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 1\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 2\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 3\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 4\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 5\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 6\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 7\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 8\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 9\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 10\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 11\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 12\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 13\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 14\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 15\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 16\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 0\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 1\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMinuteChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 1\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 2\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 3\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 4\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 5\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 6\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnBatteryNotify\r\nReference0: -1\r\nReference1: -1\r\nReference2: online\r\nReference3: no_battery\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: rateofusegraph\r\nReference0: Emily/Phase4.5EmilyTeddy12565.789474install\r\nReference1: いまいち萌えない娘今井知菜へんしゅう21334.210526boot\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 7\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 8\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 9\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 10\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 11\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 12\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 13\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 14\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 15\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 16\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 17\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 18\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 19\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 20\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 21\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 22\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 23\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 24\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 25\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 26\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 27\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 28\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 29\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 30\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 31\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 32\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 33\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 34\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 35\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 36\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 37\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 38\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 39\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 40\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 41\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 42\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 43\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 44\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 45\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 46\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 47\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 48\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 49\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 50\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 51\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 52\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 53\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 54\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 55\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 56\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 57\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 58\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 59\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 60\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 61\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMinuteChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 61\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 62\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 63\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 64\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 65\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 66\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnBatteryNotify\r\nReference0: -1\r\nReference1: -1\r\nReference2: online\r\nReference3: no_battery\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: rateofusegraph\r\nReference0: Emily/Phase4.5EmilyTeddy12564.102564install\r\nReference1: いまいち萌えない娘今井知菜へんしゅう21435.897436boot\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 67\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 68\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 69\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 70\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 71\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 72\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 73\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 74\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 75\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 76\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 77\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 78\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 79\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 80\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 81\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 82\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 83\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 84\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 85\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 86\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 87\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 88\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 89\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 90\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 91\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 92\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 93\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 94\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 95\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 96\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 97\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 98\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 99\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 100\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 101\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 102\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 103\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 104\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 105\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 106\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 107\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 108\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 109\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 110\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 111\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 112\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: \\1\\s[10]\\0\\0\\s[7]このっ！\\w9このっ！！\\w9\\w9\\0\\s[4]\\nあー、\\w5またやられてもーたー…\\w5…\\w9\\w9\\1…\\w5…\\w5ゲームまであるんだもんなあ…\\w5…\\w9\\w9\\0\\s[20]\\n\\n[half]ボス、\\w5倒されへん…\\w5…\\w9\\w9\\1\\n\\n[half]へたくそ。\\w9あ、\\w5\\_a[http://imamoe.jp/kobeimamoe/]公式サイト\\_aでダウンロードできるぞ。\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nID: OnTranslate\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nReference0: \\1\\s[10]\\0\\0\\s[7]このっ！\\w9このっ！！\\w9\\w9\\0\\s[4]\\nあー、\\w5またやられてもーたー…\\w5…\\w9\\w9\\1…\\w5…\\w5ゲームまであるんだもんなあ…\\w5…\\w9\\w9\\0\\s[20]\\n\\n[half]ボス、\\w5倒されへん…\\w5…\\w9\\w9\\1\\n\\n[half]へたくそ。\\w9あ、\\w5\\_a[http://imamoe.jp/kobeimamoe/]公式サイト\\_aでダウンロードできるぞ。\r\nReference1:\r\nReference2: OnSecondChange\r\nReference3: 62001112\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSurfaceChange\r\nReference0: 7\r\nReference1: 10\r\nReference2: 0,7,250,332\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 113\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSurfaceChange\r\nReference0: 4\r\nReference1: 10\r\nReference2: 0,4,250,332\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 114\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 115\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 116\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 117\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 118\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSurfaceChange\r\nReference0: 20\r\nReference1: 10\r\nReference2: 0,20,250,332\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 119\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 120\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 121\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnMinuteChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 121\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 122\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 123\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 124\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 125\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 126\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnBatteryNotify\r\nReference0: -1\r\nReference1: -1\r\nReference2: online\r\nReference3: no_battery\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: rateofusegraph\r\nReference0: Emily/Phase4.5EmilyTeddy12562.500000install\r\nReference1: いまいち萌えない娘今井知菜へんしゅう21537.500000boot\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 127\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 128\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 129\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 130\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 131\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 132\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 133\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 134\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 135\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 136\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 137\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnBalloonTimeout\r\nReference0: \\1\\s[10]\\0\\0\\s[7]このっ！\\w9このっ！！\\w9\\w9\\0\\s[4]\\nあー、\\w5またやられてもーたー…\\w5…\\w9\\w9\\1…\\w5…\\w5ゲームまであるんだもんなあ…\\w5…\\w9\\w9\\0\\s[20]\\n\\n[half]ボス、\\w5倒されへん…\\w5…\\w9\\w9\\1\\n\\n[half]へたくそ。\\w9あ、\\w5\\_a[http://imamoe.jp/kobeimamoe/]公式サイト\\_aでダウンロードできるぞ。\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 138\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 139\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 140\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 141\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 142\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 143\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 144\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 145\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 146\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 147\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 148\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 149\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 150\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 151\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 152\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 153\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSurfaceRestore\r\nReference0: 20\r\nReference1: 10\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: \\0\\s[0]\\1\\s[10]\\e\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nID: OnTranslate\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nReference0: \\0\\s[0]\\1\\s[10]\\e\r\nReference1:\r\nReference2: OnSurfaceRestore\r\nReference3: 2010\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSurfaceChange\r\nReference0: 0\r\nReference1: 10\r\nReference2: 0,0,250,332\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 154\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 155\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 156\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 157\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 158\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 159\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 160\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 161\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 162\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 163\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 164\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 165\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 166\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 167\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 168\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 169\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 170\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 171\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 172\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 173\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 174\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 175\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 176\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 177\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 178\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 179\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 180\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 181\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMinuteChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 181\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 182\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 183\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 184\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 185\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 186\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnBatteryNotify\r\nReference0: -1\r\nReference1: -1\r\nReference2: online\r\nReference3: no_battery\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: rateofusegraph\r\nReference0: Emily/Phase4.5EmilyTeddy12560.975610install\r\nReference1: いまいち萌えない娘今井知菜へんしゅう21639.024390boot\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 187\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 188\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 189\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 190\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 191\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 192\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 193\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 194\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 195\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 196\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 197\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 198\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 199\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 200\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 201\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 202\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 203\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 204\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 205\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 206\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 207\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 208\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 209\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 210\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 211\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 212\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 213\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 214\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 215\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 216\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 217\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 218\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 219\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 220\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 221\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 222\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 223\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 224\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 225\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 226\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 227\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 228\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 229\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 230\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 231\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 232\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 233\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 234\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 235\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 236\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 237\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 238\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 239\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 240\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 241\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMinuteChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 241\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 242\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 243\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 244\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 245\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 246\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnBatteryNotify\r\nReference0: -1\r\nReference1: -1\r\nReference2: online\r\nReference3: no_battery\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: rateofusegraph\r\nReference0: Emily/Phase4.5EmilyTeddy12559.523810install\r\nReference1: いまいち萌えない娘今井知菜へんしゅう21740.476190boot\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 247\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 248\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 249\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 250\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 251\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 252\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 253\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 254\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 255\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 256\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 257\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 258\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 259\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 260\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 261\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 262\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 263\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 264\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 265\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 266\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 267\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 268\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 269\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 270\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 271\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 272\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 273\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 274\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 275\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 276\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 277\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 278\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 279\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 280\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 281\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 282\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 283\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 284\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 285\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 286\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 287\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 288\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 289\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 290\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 291\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 292\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: \\1\\s[10]\\0\\0\\s[6]…\\w5確かに私は『右の人』\\w9とか言われるかも知らん。\\w9\\w9\\0\\s[7]\\nせやけど、\\w5『ミギー』\\w9とか略さんといてや！\\w9\\w9\\1なんでだよ。\\w9\\w9\\0\\n\\n[half]寄生獣とちゃうねんで！\\w9\\w9\\1\\n\\n[half]そっちかよ。\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nID: OnTranslate\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nReference0: \\1\\s[10]\\0\\0\\s[6]…\\w5確かに私は『右の人』\\w9とか言われるかも知らん。\\w9\\w9\\0\\s[7]\\nせやけど、\\w5『ミギー』\\w9とか略さんといてや！\\w9\\w9\\1なんでだよ。\\w9\\w9\\0\\n\\n[half]寄生獣とちゃうねんで！\\w9\\w9\\1\\n\\n[half]そっちかよ。\r\nReference1:\r\nReference2: OnSecondChange\r\nReference3: 62001292\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSurfaceChange\r\nReference0: 6\r\nReference1: 10\r\nReference2: 0,6,250,332\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 293\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 294\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSurfaceChange\r\nReference0: 7\r\nReference1: 10\r\nReference2: 0,7,250,332\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 295\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 296\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 297\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 298\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 299\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 300\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 301\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMinuteChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 301\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 302\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 303\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 304\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 305\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 306\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnBatteryNotify\r\nReference0: -1\r\nReference1: -1\r\nReference2: online\r\nReference3: no_battery\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: rateofusegraph\r\nReference0: Emily/Phase4.5EmilyTeddy12558.139535install\r\nReference1: いまいち萌えない娘今井知菜へんしゅう21841.860465boot\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 307\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 308\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 309\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 310\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 311\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 312\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 313\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 314\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 315\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnBalloonTimeout\r\nReference0: \\1\\s[10]\\0\\0\\s[6]…\\w5確かに私は『右の人』\\w9とか言われるかも知らん。\\w9\\w9\\0\\s[7]\\nせやけど、\\w5『ミギー』\\w9とか略さんといてや！\\w9\\w9\\1なんでだよ。\\w9\\w9\\0\\n\\n[half]寄生獣とちゃうねんで！\\w9\\w9\\1\\n\\n[half]そっちかよ。\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 316\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 317\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 318\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 319\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 320\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 321\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 322\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 323\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 324\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 325\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 326\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 327\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 328\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 329\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 330\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 331\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSurfaceRestore\r\nReference0: 7\r\nReference1: 10\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: \\0\\s[0]\\1\\s[10]\\e\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nID: OnTranslate\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nReference0: \\0\\s[0]\\1\\s[10]\\e\r\nReference1:\r\nReference2: OnSurfaceRestore\r\nReference3: 710\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSurfaceChange\r\nReference0: 0\r\nReference1: 10\r\nReference2: 0,0,250,332\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 332\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 333\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 334\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 335\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 336\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 337\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 338\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 339\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 340\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 341\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 342\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 343\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 344\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 345\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 346\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 347\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 348\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 349\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 350\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 351\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 352\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 353\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 354\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 355\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 356\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 357\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 358\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 359\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 360\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 361\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMinuteChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 361\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 362\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 363\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 364\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 365\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 366\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnBatteryNotify\r\nReference0: -1\r\nReference1: -1\r\nReference2: online\r\nReference3: no_battery\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: rateofusegraph\r\nReference0: Emily/Phase4.5EmilyTeddy12556.818182install\r\nReference1: いまいち萌えない娘今井知菜へんしゅう21943.181818boot\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 367\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 368\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 369\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 370\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 371\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 372\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 373\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 374\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 375\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 376\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 377\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 378\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 379\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 380\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 381\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 382\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 383\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 384\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 385\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 386\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 387\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 388\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 389\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 390\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 391\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 392\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 393\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 394\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 395\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 396\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 397\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 398\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 399\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 400\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 401\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 402\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 403\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 404\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 405\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 406\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 407\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 408\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 409\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 410\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 411\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 412\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 413\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 414\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 415\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 416\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 417\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 418\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 419\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 420\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 421\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMinuteChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 421\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 422\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 423\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 424\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 425\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 426\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnBatteryNotify\r\nReference0: -1\r\nReference1: -1\r\nReference2: online\r\nReference3: no_battery\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: rateofusegraph\r\nReference0: Emily/Phase4.5EmilyTeddy12555.555556install\r\nReference1: いまいち萌えない娘今井知菜へんしゅう22044.444444boot\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 427\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 428\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 429\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 430\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 431\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 432\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 433\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 434\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 435\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 436\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 437\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 438\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 439\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 440\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 441\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 442\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 443\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 444\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 445\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 446\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 447\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 448\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 449\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 450\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 451\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 452\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 453\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 454\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 455\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 456\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 457\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 458\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 459\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 460\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 461\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 462\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 463\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 464\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 465\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 466\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 467\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 468\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 469\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 470\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 471\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 472\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: \\1\\s[10]\\0\\0\\s[6]いまいち萌えない娘として売り出してる私が、\\w9\\n一部の方々を萌えさせてしまうと言う、\\w9\\nとんでもない事態を引き起こしてしまいました。\\w9\\w9\\1\\_a[http://imamoe.jp/kobeimamoe/?page_id=346]エイプリルフール\\_aネタをいつまで引っ張るんだよ。\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nID: OnTranslate\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nReference0: \\1\\s[10]\\0\\0\\s[6]いまいち萌えない娘として売り出してる私が、\\w9\\n一部の方々を萌えさせてしまうと言う、\\w9\\nとんでもない事態を引き起こしてしまいました。\\w9\\w9\\1\\_a[http://imamoe.jp/kobeimamoe/?page_id=346]エイプリルフール\\_aネタをいつまで引っ張るんだよ。\r\nReference1:\r\nReference2: OnSecondChange\r\nReference3: 62001472\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSurfaceChange\r\nReference0: 6\r\nReference1: 10\r\nReference2: 0,6,250,332\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 473\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 474\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 475\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 476\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 477\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 478\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 479\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 480\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 481\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMinuteChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 481\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 482\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 483\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 484\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 485\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 486\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnBatteryNotify\r\nReference0: -1\r\nReference1: -1\r\nReference2: online\r\nReference3: no_battery\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: rateofusegraph\r\nReference0: Emily/Phase4.5EmilyTeddy12554.347826install\r\nReference1: いまいち萌えない娘今井知菜へんしゅう22145.652174boot\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 487\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 488\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 489\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 490\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 491\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 492\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 493\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnBalloonTimeout\r\nReference0: \\1\\s[10]\\0\\0\\s[6]いまいち萌えない娘として売り出してる私が、\\w9\\n一部の方々を萌えさせてしまうと言う、\\w9\\nとんでもない事態を引き起こしてしまいました。\\w9\\w9\\1\\_a[http://imamoe.jp/kobeimamoe/?page_id=346]エイプリルフール\\_aネタをいつまで引っ張るんだよ。\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 494\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 495\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 496\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 497\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 498\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 499\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 500\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 501\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 502\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 503\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 504\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 505\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 506\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 507\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 508\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 509\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSurfaceRestore\r\nReference0: 6\r\nReference1: 10\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: \\0\\s[0]\\1\\s[10]\\e\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nID: OnTranslate\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nReference0: \\0\\s[0]\\1\\s[10]\\e\r\nReference1:\r\nReference2: OnSurfaceRestore\r\nReference3: 610\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSurfaceChange\r\nReference0: 0\r\nReference1: 10\r\nReference2: 0,0,250,332\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 510\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 511\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 512\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 513\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 514\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 515\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 516\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 517\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 518\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 519\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 520\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 521\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 522\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 523\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 524\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 525\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 526\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 527\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 528\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 529\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 530\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 531\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 532\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 533\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 534\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 535\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 536\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 537\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 538\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 539\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 540\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 541\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMinuteChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 541\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 542\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 543\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 544\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 545\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 546\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnBatteryNotify\r\nReference0: -1\r\nReference1: -1\r\nReference2: online\r\nReference3: no_battery\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: rateofusegraph\r\nReference0: Emily/Phase4.5EmilyTeddy12553.191489install\r\nReference1: いまいち萌えない娘今井知菜へんしゅう22246.808511boot\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 547\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 548\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 549\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 550\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 551\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 552\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 553\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 554\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 555\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 556\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 557\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 558\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 559\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 560\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 561\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 562\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 563\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 564\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 565\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 566\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 567\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 568\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 569\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 570\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 571\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 572\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 573\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 574\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 575\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 576\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 577\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 578\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 579\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 580\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 581\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 582\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 583\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 584\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 585\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 586\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 587\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 588\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 589\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 590\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 591\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 592\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 593\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 594\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 595\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 596\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 597\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 598\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 599\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 600\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 601\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnMinuteChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 601\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 602\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 603\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 604\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 605\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 606\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnBatteryNotify\r\nReference0: -1\r\nReference1: -1\r\nReference2: online\r\nReference3: no_battery\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: rateofusegraph\r\nReference0: Emily/Phase4.5EmilyTeddy12552.083333install\r\nReference1: いまいち萌えない娘今井知菜へんしゅう22347.916667boot\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 607\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 608\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 609\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 610\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 611\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 612\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 613\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 614\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 615\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 616\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 617\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 618\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 619\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 620\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 621\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 622\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 623\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 624\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 625\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 626\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 627\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 628\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 629\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 630\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 631\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 632\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 633\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 634\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 635\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 636\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 637\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 638\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 639\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 640\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 641\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 642\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 643\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 644\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 645\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 646\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 647\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 648\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 649\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 650\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 651\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 652\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: \\1\\s[10]\\0\\0\\s[5]へんさん、\\w5いんたーなしょなるやで。\\w9\\w9\\1何がだよ。\\w9\\w9\\0\\n\\n[half]海外の掲示板とかでな。\\w9\\n『Imaichi-tan』\\w9て呼ばれとるねんて。\\w9\\w9\\1\\n\\n[half]…\\w5よかったな。\\w9\\nジャパニーズHENTAIの仲間入りだ。\\w9\\w9\\0\\s[7]\\n\\n[half]キャラクターて言うて！\\w9\\nめっちゃ聞いたとこ悪いわ！\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nID: OnTranslate\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nReference0: \\1\\s[10]\\0\\0\\s[5]へんさん、\\w5いんたーなしょなるやで。\\w9\\w9\\1何がだよ。\\w9\\w9\\0\\n\\n[half]海外の掲示板とかでな。\\w9\\n『Imaichi-tan』\\w9て呼ばれとるねんて。\\w9\\w9\\1\\n\\n[half]…\\w5よかったな。\\w9\\nジャパニーズHENTAIの仲間入りだ。\\w9\\w9\\0\\s[7]\\n\\n[half]キャラクターて言うて！\\w9\\nめっちゃ聞いたとこ悪いわ！\r\nReference1:\r\nReference2: OnSecondChange\r\nReference3: 62001652\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSurfaceChange\r\nReference0: 5\r\nReference1: 10\r\nReference2: 0,5,250,332\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 653\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 654\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 655\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 656\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 657\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 658\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 659\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 660\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 661\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnMinuteChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 661\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSurfaceChange\r\nReference0: 7\r\nReference1: 10\r\nReference2: 0,7,250,332\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 662\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nStatus: talking\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 0\r\nReference4: 663\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 664\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 665\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 666\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnBatteryNotify\r\nReference0: -1\r\nReference1: -1\r\nReference2: online\r\nReference3: no_battery\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "NOTIFY SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: rateofusegraph\r\nReference0: Emily/Phase4.5EmilyTeddy12551.020408install\r\nReference1: いまいち萌えない娘今井知菜へんしゅう22448.979592boot\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 667\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 668\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 669\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 670\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 671\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 672\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 673\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 674\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 675\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 676\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 677\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 678\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnBalloonTimeout\r\nReference0: \\1\\s[10]\\0\\0\\s[5]へんさん、\\w5いんたーなしょなるやで。\\w9\\w9\\1何がだよ。\\w9\\w9\\0\\n\\n[half]海外の掲示板とかでな。\\w9\\n『Imaichi-tan』\\w9て呼ばれとるねんて。\\w9\\w9\\1\\n\\n[half]…\\w5よかったな。\\w9\\nジャパニーズHENTAIの仲間入りだ。\\w9\\w9\\0\\s[7]\\n\\n[half]キャラクターて言うて！\\w9\\nめっちゃ聞いたとこ悪いわ！\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 679\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 680\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 681\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 682\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 683\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 684\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 685\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 686\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 687\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 688\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 689\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 690\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 691\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 692\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 693\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSecondChange\r\nReference0: 62\r\nReference1: 0\r\nReference2: 0\r\nReference3: 1\r\nReference4: 694\r\n\r\n"
  , res: "SHIORI/3.0 204 No Content\r\nCharset: UTF-8\r\nSender: pasta\r\n\r\n"
}
, {
    req: "GET SHIORI/3.0\r\nCharset: UTF-8\r\nSender: SSP\r\nSecurityLevel: local\r\nID: OnSurfaceRestore\r\nReference0: 7\r\nReference1: 10\r\n\r\n"
  , res: "SHIORI/3.0 200 OK\r\nCharset: UTF-8\r\nSender: pasta\r\nValue: \\0\\s[0]\\1\\s[10]\\e\r\n\r\n"
}
    ];

    //---------------------------------------------------------
    // モジュールのエクスポート
    return mod;
});