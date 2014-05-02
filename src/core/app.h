#pragma once

#include <windows.h>
#include <string>


namespace pasta{

	enum CharMode {
		MODE_ANSI,
		MODE_UTF_8,
	};

	class App
	{
	public:
		std::string lastErrorMessage;

	private:
		HINSTANCE hinst;
		std::wstring loaddir;
		CharMode charMode;

		duk_context *ctx;


	public:
		App(const HINSTANCE hinst, const std::string& loaddir);
		~App(void);
		bool request(const std::string& request, std::string& response);
	};

}