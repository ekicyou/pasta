#pragma once

#include <windows.h>
#include <string>
#include "util.h"


namespace pasta{

	enum CharMode {
		MODE_ANSI,
		MODE_UTF_8,
	};

	class App
	{
	public:

	private:
		HINSTANCE hinst;
		std::wstring loaddir;
		CharMode charMode;

		xtal::CStdioStdStreamLib std_stream_lib;     // std*はCの標準ライブラリを使う
		xtal::WinThreadLib thread_lib;               // Windowsのスレッドを使う
		xtal::WinFilesystemLib filesystem_lib;       // Windowsのファイルシステムを使う
		xtal::UTF8ChCodeLib ch_code_lib;             // UTF8を使う
		xtal::Setting setting;
		xtal::Environment* env;

	public:
		App(const HINSTANCE hinst, const std::string& loaddir);
		~App(void);
		bool request(const std::string& request, std::string& response);
	};

}