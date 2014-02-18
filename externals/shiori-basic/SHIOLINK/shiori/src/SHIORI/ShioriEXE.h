// ----------------------------------------------------------------------------
// 別プロセス通信SHIORI SHIOLINK.DLL
//   The MIT License
//   http://sourceforge.jp/projects/opensource/wiki/licenses%2FMIT_license
// ----------------------------------------------------------------------------
#pragma once
#include "StdErrReadThread.h"

class Logger;

class ShioriEXE
{
private:
   HANDLE hReadPipe[2];
   HANDLE hErrPipe[2];
   HANDLE hWritePipe[2];

   STARTUPINFO startInfo;
   PROCESS_INFORMATION proInfo;

   Poco::Timespan waitTime;
   std::queue<char> queue;

   Logger* log;

   StdErrReadThread* errReadThread;
   

public:
   ShioriEXE(
      Logger& logger,
      const std::wstring& command,
      const bool viewConsole, const Poco::Timespan waitTime);
   ~ShioriEXE(void);
   void Kill();

   std::string ReadLine();
   void WriteLine(const std::string& line);
   bool eof() const;
   void ClearStderr();

private:
   void ReadQueue();
};
