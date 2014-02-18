#pragma once

class Logger
{
public:
   /**
    * エラーレベル
    */
   enum vLevel {
      vDebug,                 ///< デバッグ
      vInfo,                  ///< インフォ
      vWarn,                  ///< ワーニング
      vError,                 ///< エラー
      vFatal,                 ///< フェイタル
      vNone,                  ///< 何も出さないときに指定する値
   };

private:
   vLevel level;
   std::wstring logpath;
   std::ofstream *os;
   void Open();

public:
   Logger();
   Logger(const std::wstring& path, const vLevel level);
   ~Logger();

   void SetLevel(const vLevel level);
   void SetPath (const std::wstring& path);

   bool IsOutput(const vLevel level) const;

   void WriteImpl(const std::string& text);
   void WriteImpl(const std::wstring& text);

   void Write    (const vLevel level, const std::string& text);
   void WriteLine(const vLevel level, const std::string& text);

   void Write    (const vLevel level, const std::string& t1 ,const std::string& t2);
   void WriteLine(const vLevel level, const std::string& t1 ,const std::string& t2);

   void Write    (const vLevel level, const std::string& t1 ,const std::string& t2 ,const std::string& t3);
   void WriteLine(const vLevel level, const std::string& t1 ,const std::string& t2 ,const std::string& t3);

   void Write    (const vLevel level, const std::wstring& text);
   void WriteLine(const vLevel level, const std::wstring& text);

   static char*  Logger::levelText(const vLevel level);
   static vLevel Logger::levelId  (const std::wstring& text);

};


#define CHECKED_WRITE(__level__, __text__)   \
   {                                            \
      if((LOGGER_NAME).IsOutput(__level__)){    \
         (LOGGER_NAME).WriteImpl(__text__);     \
      }                                         \
   }

#define WRITE_DEBUG(__text__) CHECKED_WRITE(Logger::vDebug ,__text__)
#define WRITE_INFO(__text__)  CHECKED_WRITE(Logger::vInfo  ,__text__)
#define WRITE_WARN(__text__)  CHECKED_WRITE(Logger::vWarn  ,__text__)
#define WRITE_ERROR(__text__) CHECKED_WRITE(Logger::vError ,__text__)
#define WRITE_FATAL(__text__) CHECKED_WRITE(Logger::vFatal ,__text__)

#define LINE_DEBUG(...) {(LOGGER_NAME).WriteLine(Logger::vDebug ,__VA_ARGS__);}
#define LINE_INFO(...)  {(LOGGER_NAME).WriteLine(Logger::vInfo  ,__VA_ARGS__);}
#define LINE_WARN(...)  {(LOGGER_NAME).WriteLine(Logger::vWarn  ,__VA_ARGS__);}
#define LINE_ERROR(...) {(LOGGER_NAME).WriteLine(Logger::vError ,__VA_ARGS__);}
#define LINE_FATAL(...) {(LOGGER_NAME).WriteLine(Logger::vFatal ,__VA_ARGS__);}


//EOF