// ----------------------------------------------------------------------------
// 別プロセス通信SHIORI SHIOLINK.DLL
//   The MIT License
//   http://sourceforge.jp/projects/opensource/wiki/licenses%2FMIT_license
// ----------------------------------------------------------------------------
#pragma once

class StdErrReadThread
{
private:
   static const int MAX_OSS_LENGTH = 1024*16;
   HANDLE hReadPipe;
   HANDLE hThread;
   DWORD  threadID;
   CRITICAL_SECTION ce;
   std::ostringstream oss;
   volatile int ossLegth;
   volatile bool isJoinWait;

public:
   StdErrReadThread(HANDLE hReadPipe);
   ~StdErrReadThread(void);
   void Kill();
   void Join();
   std::string GetText();
   DWORD MainLoop();

private:


};

// EOF