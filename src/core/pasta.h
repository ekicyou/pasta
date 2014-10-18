#pragma once

#include <req_async.h>

namespace pasta{
	class Agent :public shiori::Agent{
	protected:
		void Run();

		void Load();
		void Notify();

	private:
		HINSTANCE hinst;
		std::wstring loaddir;
		int cp;
		duk_context *ctx;
	};
}