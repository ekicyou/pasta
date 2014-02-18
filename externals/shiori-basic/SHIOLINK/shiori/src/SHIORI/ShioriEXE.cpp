// ----------------------------------------------------------------------------
// 別プロセス通信SHIORI SHIOLINK.DLL
//   The MIT License
//   http://sourceforge.jp/projects/opensource/wiki/licenses%2FMIT_license
// ----------------------------------------------------------------------------
#include "stdafx.h"
#include "ShioriEXE.h"
#include "Log.h"


#define LOGGER_NAME     *log

#define R 0
#define W 1

static void DupPipe(HANDLE hProc , HANDLE srcPipe ,HANDLE &dist)
{
   DuplicateHandle(hProc, srcPipe, hProc, &dist, 0, TRUE, DUPLICATE_SAME_ACCESS);
   CloseHandle(srcPipe);
}

ShioriEXE::ShioriEXE(
   Logger& logger,
   const std::wstring& command,
   const bool viewConsole, const Poco::Timespan wt):
      waitTime(wt)
{
   log =&logger;
   WRITE_DEBUG("EXE: OPEN command [");
   WRITE_DEBUG(command);
   WRITE_DEBUG("]\r\n");
   errReadThread = NULL;

   // コマンドラインをバッファにコピー
   Poco::Buffer<WCHAR> cmdbuf(command.length()+1);
   std::copy(command.begin() ,command.end() ,cmdbuf.begin());
   cmdbuf[cmdbuf.size()-1] =0;

   // ロジック参考
   // http://www.k5.dion.ne.jp/~minyu/program/windows/pipe.html

   // SECURITY_ATTRIBUTES の設定(パイプを作るのに必要)
   SECURITY_ATTRIBUTES secAtt;
   secAtt.nLength              = sizeof(SECURITY_ATTRIBUTES);
   secAtt.lpSecurityDescriptor = NULL;
   secAtt.bInheritHandle       = FALSE;

   // パイプ作成 要は新しいハンドルを作成
   CreatePipe(&hWritePipe[R], // 読み込み用ハンドル
              &hWritePipe[W], // 書き込み用ハンドル
              &secAtt,        // ?
              0);             // バッファのサイズ

   CreatePipe(&hReadPipe[R],  // 読み込み用ハンドル
              &hReadPipe[W],  // 書き込み用ハンドル
              &secAtt,        // ?
              0);             // バッファのサイズ

   CreatePipe(&hErrPipe[R],  // 読み込み用ハンドル
              &hErrPipe[W],  // 書き込み用ハンドル
              &secAtt,        // ?
              0);             // バッファのサイズ

   /* 子プロセスの起動時の情報
    * これを設定しないと, コンソール系プロセスでは,
    * コンソールが立ち上がる。 */
   memset(&startInfo,0,sizeof(STARTUPINFO));
   startInfo.cb = sizeof(STARTUPINFO); // 構造体の大きさ;
   startInfo.dwFlags = STARTF_USEFILLATTRIBUTE |
                       STARTF_USECOUNTCHARS |
                       STARTF_USESTDHANDLES | /*ハンドルを継承するメンバ*/
                       STARTF_USESHOWWINDOW;  // どのメンバが有効か

   // ウィンドウ表示のパラメータ;
   if (viewConsole) startInfo.wShowWindow = SW_SHOWDEFAULT;
   else            startInfo.wShowWindow = SW_HIDE;

   // pipeの複製
   HANDLE hProc = GetCurrentProcess();
   DupPipe(hProc ,hWritePipe[R] ,startInfo.hStdInput);
   DupPipe(hProc ,hReadPipe[W]  ,startInfo.hStdOutput);
   DupPipe(hProc ,hErrPipe[W]  ,startInfo.hStdError);

   if (CreateProcessW(NULL ,cmdbuf.begin() ,NULL ,NULL ,TRUE,
                      0,NULL,NULL,&startInfo,&proInfo) != TRUE) {
      throw std::exception("NOT OPEN SHIORI EXE");
   }

   std::string cmd8;
   Poco::UnicodeConverter::toUTF8(command ,cmd8);
   LINE_INFO("EXE: RUN command [" ,cmd8 ,"]");
   // 不要なハンドルは閉じる
   CloseHandle(proInfo.hThread);

   // stderrの読み捨てスレッド
   errReadThread = new StdErrReadThread(hErrPipe[R]);
}


ShioriEXE::~ShioriEXE(void)
{
   LINE_DEBUG("EXE: unload start");
   if (proInfo.hProcess != NULL) {
      // 子プロセスが終るまで停止
      WaitForSingleObject(proInfo.hProcess,INFINITE);
      CloseHandle(proInfo.hProcess);
   }
   if(errReadThread != NULL) delete errReadThread;
   CloseHandle(hWritePipe[W]);
   CloseHandle(hReadPipe[R]);
   LINE_INFO("EXE: unload");
}

void ShioriEXE::Kill(void)
{
   if (proInfo.hProcess != NULL) {
      LINE_WARN("EXE: kill!!");
      TerminateProcess(proInfo.hProcess ,0);
   }
}

bool ShioriEXE::eof() const
{
   return false;
}

// stderrを読み捨てる
void ShioriEXE::ClearStderr()
{
   std::string errText = errReadThread->GetText();
   if(errText.length()==0) return;
   LINE_INFO("EXE: read stderr *---------------------------* start");
   LINE_INFO(errText.c_str());
   LINE_INFO("EXE: *---------------------------------------*   end");
   LINE_INFO("");
}


void ShioriEXE::WriteLine(const std::string& line)
{
   // 書き込み本体
   DWORD len;
   std::string buf = line + "\r\n";
   WriteFile(hWritePipe[W], buf.data(), buf.length() ,&len ,NULL);
   FlushFileBuffers(hWritePipe[W]);
}

std::string ShioriEXE::ReadLine()
{
   std::ostringstream buf;
   // 改行を発見するまでキューを読み込む
   while (true) {
      if (queue.empty()) {
         ReadQueue();
         continue;
      }
      char c = queue.front();
      queue.pop();
      if (c == '\n') break;
      if (c == '\r') continue;
      buf << c;
   }
   // 結果を返す
   std::string line =buf.str();
   return line;
}

void ShioriEXE::ReadQueue()
{
   char buf[1024];
   DWORD readlen;

   // まず１文字読み込む
   if (ReadFile(hReadPipe[R], buf ,1 ,&readlen ,NULL)!=TRUE) {
      throw std::ios_base::failure("READ PIPE ERROR(1)");
   }
   queue.push(buf[0]);

   // 残りのバイト数を確認する
   DWORD remain;
   if (PeekNamedPipe(hReadPipe[R] ,NULL ,0 ,NULL, &remain ,NULL)!=TRUE) {
      throw std::ios_base::failure("PeekNamedPipe ERROR");
   }

   // 残りの情報を読み込む
   while (remain > 0) {
      DWORD size = remain;
      if (size>sizeof(buf)) size = sizeof(buf);
      if (ReadFile(hReadPipe[R], buf ,size ,&readlen ,NULL)!=TRUE) {
         throw std::ios_base::failure("READ PIPE ERROR(1)");
      }
      for (DWORD i=0; i<readlen ;i++) queue.push(buf[i]);
      remain -= readlen;
   }
}

