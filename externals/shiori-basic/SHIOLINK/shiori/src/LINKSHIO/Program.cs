using System;
using System.Collections.Generic;
using System.Text;
using System.Diagnostics;

namespace LINKSHIO
{


  class Program
  {
    enum ReadMode { SKIP, BUFFERING }

    static int Main(string[] args)
    {
      Console.InputEncoding = Encoding.UTF8;
      Console.OutputEncoding = Console.InputEncoding;

      try {
        List<string> linebuf = new List<string>();
        string loaddir = string.Empty;
        ReadMode mode = ReadMode.SKIP;
        while (true) {
          // １行読み込む
          string line = Console.In.ReadLine();
          if (line == null) return 0;
          line = line.TrimEnd();
          Debug.WriteLine(line);

          switch (mode) {
            case ReadMode.SKIP: // SKIPモードの処理
              if (line.Length < 3) continue;
              switch (line.Substring(0, 3)) {
                case "*L:": // loadコマンド
                  loaddir = line.Substring(3);
                  continue;
                case "*U:": // unloadコマンド
                  return 0;
                case "*S:": // 同期コマンド（次の行からSHIORI REQUEST）
                  Console.Out.WriteLine(line);
                  Console.Out.Flush();
                  mode = ReadMode.BUFFERING;
                  continue;
              }
              continue;

            case ReadMode.BUFFERING:  // バッファモードの処理
              if (line.Length != 0) {
                // 空行以外は情報を溜め込む
                linebuf.Add(line);
                continue;
              }
              // 空行のときはリクエスト確定
              Console.Out.WriteLine(request(linebuf));
              Console.Out.Flush();
              linebuf.Clear();
              mode = ReadMode.SKIP;
              continue;
          }
        }
      }
      catch { }
      return 0;
    }

    enum RequestType { GET, NOTIFY }



    static string request(List<string> lines)
    {
      try {
        // Header check
        if (lines.Count == 0) throw new ArgumentException("リクエストが存在しませんでした。");
        string head = lines[0];
        RequestType type;
        switch (head) {
          case "GET SHIORI/3.0": type = RequestType.GET; break;
          case "NOTIFY SHIORI/3.0": type = RequestType.NOTIFY; break;
          default: return response_bad_request("NOT SHIORI/3.0");
        }
        if (type == RequestType.NOTIFY) return response_no_content("");

        Dictionary<string, string> items = new Dictionary<string, string>();
        for (int i = 1; i < lines.Count; i++) {
          string line = lines[i];
          int sep = line.IndexOf(": ");
          if (sep < 1) continue;
          string k = line.Substring(0, sep);
          string v = line.Substring(sep + 2);
          items[k] = v;
        }
        string ID = string.Empty;
        if (!items.TryGetValue("ID", out ID)) return response_no_content("");
        switch (ID) {
          case "OnBoot":
          case "OnFirstBoot":
          case "OnGhostChanged":
            return response_ok("Value: " +
              @"\u\s[10]\h\s[0]さて、\w5このプログラムは単なる起動テストなのよ。\w9\w9" +
              @"\uだね。\w9\w9" +
              @"\h\s[6]\n\n[half]でもね、\w5ただ起動するだけじゃなくて、\w5私的にはこういうときこそ人生の潤いを目指すべきだと。\w9\w9" +
              @"\u\s[120]\n\n[half]‥‥深いね。\w9\w9" +
              @"\h\s[5]\n\n[half]やーでも面倒だから落ちるけどね。\w9\w9" +
              @"\u\s[16]\n\n[half]浅いね‥\w5‥\w5‥\w5。\w9\w9\w9\-" +
              "\r\n");
          case "version": return response_ok("Value: LINKSHIO1.0.0\r\n");
        }
        return response_no_content("");

      }
      catch (Exception ex) {
        return response_bad_request(ex.Message);
      }
    }

    static string response(string resid, string lines)
    {
      StringBuilder buf = new StringBuilder();
      buf.AppendLine("SHIORI/3.0 " + resid);
      buf.AppendLine("Charset: UTF-8");
      buf.AppendLine("Sender: LINKSHIO");
      buf.Append(lines);
      return buf.ToString();
    }

    static string response_ok(string lines) { return response("200 OK", lines); }
    static string response_no_content(string lines) { return response("204 No Content", lines); }
    static string response_bad_request(string resion)
    {
      return response("400 Bad Request", "X-LINKSHIO-Reason: " + resion + "\r\n");
    }


  }
}
