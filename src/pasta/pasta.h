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

		xtal::CStdioStdStreamLib std_stream_lib;     // std*��C�̕W�����C�u�������g��
		xtal::WinThreadLib thread_lib;               // Windows�̃X���b�h���g��
		xtal::WinFilesystemLib filesystem_lib;       // Windows�̃t�@�C���V�X�e�����g��
		xtal::UTF8ChCodeLib ch_code_lib;             // UTF8���g��
		xtal::Setting setting;
		xtal::Environment* env;

	public:
		App(const HINSTANCE hinst, const std::string& loaddir);
		~App(void);
		bool request(const std::string& request, std::string& response);
	};

}