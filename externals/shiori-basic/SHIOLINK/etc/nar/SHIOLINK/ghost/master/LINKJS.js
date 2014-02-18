var loaddir = "";
var stdin  = WScript.StdIn;
var stdout = WScript.StdOut;
var stderr = WScript.StdErr;

//=============================================================
// リクエストを１つ切り出すまでデータを読み込む。
//=============================================================
function readRequest(){
  var request, line, ss;
  mode = 1;
  request        = new Object();
  request.method = null;
  request.item   = new Object();

  while (!stdin.AtEndOfStream){
    line = stdin.ReadLine();
    line = line.replace(/[ 　\t\r\n]*$/gim, "");
    stderr.WriteLine("get command:[" + line +"]");

    switch(mode){
    case 1: // *------- コマンドモード
      var cmd = line.substr(0,3);
      switch(cmd){
      case "*L:":
        loaddir = line.substr(3);
        continue;
      case "*U:":
        return null;
      case "*S:":
        // seq応答
        stdout.WriteLine(line);
        mode = 2;
      }
      continue;

    case 2: // *------- request method受信モード
      switch(line){
      case "GET SHIORI/3.0":    request.method = "GET"; break;
      case "NOTIFY SHIORI/3.0": request.method = "NOTIFY"; break;
      default:
        return request;
      }
      mode = 3;
      continue;

    case 3: // *------- request item受信モード
      if(line.length == 0) return request;
      ss = line.split(": ");
      request.item[ss[0]] = ss[1];
    }
    continue;

  }
  return null;
}


//=============================================================
// 応答関数色々
//=============================================================
function response(resid ,lines){
  var buf = "SHIORI/3.0 " + resid + "\r\n";
  buf = buf + "Charset: Shift_JIS" + "\r\n";
  buf = buf + "Sender: LINKJS" + "\r\n";
  buf = buf + lines;
  stderr.WriteLine(buf);// イジメのようにstderrに吐きまくる
  stdout.WriteLine(buf);
}

function response_bad_request(region){
  response(
    "400 Bad Request",
    "X-LINKJS-Reason: " + region + "\r\n");
}

function response_no_content(lines){
  response(
    "204 No Content",
    lines);
}

function response_ok(lines) {
  response(
    "200 OK",
    lines);
}


//=============================================================
// イベント登録
//=============================================================
var event = new Object();

event.version = function(request){
  response_ok("Value: LINKJS_1.2.0.0\r\n");
}

event.OnBoot = function(request){
  response_ok("Value: " +
    "\\u\\s[-1]\\h\\s[0]" +
    "‥\\w5‥\\w5‥\\w5‥\\w9\\w9\\n" +
    "端的に言えば、\\w5つながるという事は、" +
    "\\w9可能性が広がるということだ。\\n\\n[half]" +
    "\\w9\\w9‥\\w5‥\\w5‥\\w9\\w9ワタシの可能性は、\\w5諸君にかかっている。" +
    "\\w9\\w9\\w9\\n\\n[half]" +
    "‥\\w5‥\\w5‥\\w5‥\\w9\\n" +
    "今は来るべき未来に備え、" +
    "\\w5眠りにつくとしよう。" +
    "\\n\\w9\\w9‥\\w5‥\\w5君に、\\w5光あれ。" +
    "\\w9\\w9\\w9\\-" +
    "\r\n");
}
event.OnFirstBoot    = event.OnBoot;
event.OnGhostChanged = event.OnBoot;


//=============================================================
// メインループ
//=============================================================
while(true){
  var request = readRequest();
  if(request == null) WScript.Quit();
  switch(request.method){
  case null:
    response_bad_request("SHIORI 3.0ではありません");
    continue;
  case "GET":
    var func = event[request.item["ID"]];
    if(func != null){
      func(request);
      continue;
    }
  }
  response_no_content("");
}


