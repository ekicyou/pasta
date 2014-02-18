#include "StdAfx.h"
#include "Log.h"

Logger::Logger():
      logpath()
{
   this->level = vNone;
   os = NULL;
}


Logger::Logger(const std::wstring &path, const Logger::vLevel level):
      logpath(path)
{
   this->level = level;
   os = NULL;
}


Logger::~Logger()
{
   if (os!=NULL) {
      delete os;
      os = NULL;
   }
}


void Logger::SetLevel(const vLevel level)
{
   this->level = level;
}
void Logger::SetPath (const std::wstring& path)
{
   logpath = path;
}

bool Logger::IsOutput(const Logger::vLevel level) const
{
   if (logpath.empty()) return false;
   return level >= this->level;
}


void Logger::Open()
{
   if (logpath.empty()) return;
   os = new std::ofstream(
      logpath.c_str(),
      std::ios::out | std::ios::binary);
}

void Logger::WriteImpl(const std::string &text)
{
   if (os==NULL) Open();
   if (os==NULL) return;
   (*os) << text;
   os->flush();
}

void Logger::WriteImpl(const std::wstring &text)
{
   std::string t8;
   Poco::UnicodeConverter::toUTF8(text ,t8);
   WriteImpl(t8);
}

void Logger::Write(const Logger::vLevel level, const std::string &text)
{
   if (! IsOutput(level)) return;
   WriteImpl(text);
}
void Logger::Write(const Logger::vLevel level
                   , const std::string &t1
                   , const std::string &t2)
{
   if (! IsOutput(level)) return;
   WriteImpl(t1+t2);
}
void Logger::Write(const Logger::vLevel level
                   , const std::string &t1
                   , const std::string &t2
                   , const std::string &t3)
{
   if (! IsOutput(level)) return;
   WriteImpl(t1+t2+t3);
}


void Logger::WriteLine(const Logger::vLevel level, const std::string &text)
{
   if (! IsOutput(level)) return;
   WriteImpl(text+"\r\n");
}
void Logger::WriteLine(const Logger::vLevel level
                       , const std::string &t1
                       , const std::string &t2)
{
   if (! IsOutput(level)) return;
   WriteImpl(t1+t2+"\r\n");
}
void Logger::WriteLine(const Logger::vLevel level
                       , const std::string &t1
                       , const std::string &t2
                       , const std::string &t3)
{
   if (! IsOutput(level)) return;
   WriteImpl(t1+t2+t3+"\r\n");
}


void Logger::Write(const Logger::vLevel level, const std::wstring &text)
{
   if (! IsOutput(level)) return;
   std::string t8;
   Poco::UnicodeConverter::toUTF8(text ,t8);
   WriteImpl(t8);
}

void Logger::WriteLine(const Logger::vLevel level, const std::wstring &text)
{
   if (! IsOutput(level)) return;
   std::string t8;
   Poco::UnicodeConverter::toUTF8(text ,t8);
   WriteImpl(t8+"\r\n");
}

char* Logger::levelText(const Logger::vLevel level)
{
   switch (level) {
   case vDebug:
      return "debug";
   case vInfo:
      return "info";
   case vWarn:
      return "warn";
   case vError:
      return "error";
   case vFatal:
      return "fatal";
   case vNone:
      return "none";
   }
   return "";
};


Logger::vLevel Logger::levelId(const std::wstring& text)
{
   if (text==_T("debug")) return vDebug;
   if (text==_T("info"))  return vInfo;
   if (text==_T("warn"))  return vWarn;
   if (text==_T("error")) return vError;
   if (text==_T("fatal")) return vFatal;
   if (text==_T("none"))  return vNone;
   return vNone;
};

//EOF