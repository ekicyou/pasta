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

	public:
		App(const HINSTANCE hinst, const std::string& loaddir);
		~App(void);
		bool request(const std::string& request, std::string& response);
	};

}