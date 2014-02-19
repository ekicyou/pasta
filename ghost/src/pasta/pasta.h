#pragma once

#include <windows.h>
#include <string>

namespace pasta { 

	class App
	{
	public:
		enum CharMode {
			MODE_ANSI,
			MODE_UTF_8,
		};

	private:
		HINSTANCE hinst;
		std::wstring loaddir;
		CharMode charMode;

		void Init(void);


	public:
		App(const HINSTANCE hinst, const std::string& loaddir);
		~App(void);
		bool request(const std::string& request, std::string& response);
	};



}