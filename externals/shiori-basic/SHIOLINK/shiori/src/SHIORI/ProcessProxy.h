// ----------------------------------------------------------------------------
// 別プロセス通信SHIORI SHIOLINK.DLL
//   The MIT License
//   http://sourceforge.jp/projects/opensource/wiki/licenses%2FMIT_license
// ----------------------------------------------------------------------------
#pragma once

#include "ShioriEXE.h"
#include "Log.h"

class ProcessProxy
{
public:
   enum CharMode {
      MODE_ANSI,
      MODE_UTF_8,
   };

private:
   HINSTANCE hinst;
   std::wstring loaddir;
   Poco::UUIDGenerator uuidGen;
   Poco::Timespan waitLimit;
   CharMode charMode;

   ShioriEXE* exe;
   Logger log;

   void Init(void);


public:
   ProcessProxy(const HINSTANCE hinst ,const std::string& loaddir);
   ~ProcessProxy(void);
   bool request(const std::string& request ,std::string& response);
};

//EOF