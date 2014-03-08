
#include "../src/xtal/xtal.h"
#include "../src/xtal/xtal_macro.h"

#include "../src/xtal/xtal_lib/xtal_winthread.h"
#include "../src/xtal/xtal_lib/xtal_cstdiostream.h"
#include "../src/xtal/xtal_lib/xtal_winfilesystem.h"
#include "../src/xtal/xtal_lib/xtal_chcode.h"
#include "../src/xtal/xtal_lib/xtal_errormessage.h"

#include "../src/xtal/xtal_codebuilder.h"
	
using namespace xtal;

class InteractiveStream : public Stream{
public:

	InteractiveStream(){
		line_ = 1;
		continue_stmt_ = false;
		continue_line_ = false;
	}
	
	virtual uint_t on_read_charactors(AnyPtr* buffer, uint_t max){
		if(continue_line_){
			return read_line(buffer, max);
		}
		else{
			if(continue_stmt_){
				stdout_stream()->put_s(format(XTAL_STRING("ix:%03d>    "))->call(line_));
			}
			else{
				stdout_stream()->put_s(format(XTAL_STRING("ix:%03d>"))->call(line_));
			}

			continue_stmt_ = true;
		
			return read_line(buffer, max);
		}
	}

	int_t read_line(AnyPtr* buffer, int_t max){
		continue_line_ = true;

		for(int_t i=0; i<max; ++i){
			int_t ret = stdin_stream()->read_charactors(buffer + i, 1);
			if(ret==0){
				return i;
			}

			if(raweq(buffer[i], intern(XTAL_STRING("\n")))){
				line_++;
				continue_line_ = false;
				return i+1;
			}
		}

		return max;
	}

	void end_stmt(){
		continue_stmt_ = false;
	}

private:
	int_t line_;
	bool continue_stmt_;
	bool continue_line_;
};

XTAL_PREBIND(InteractiveStream){
	it->inherit(cpp_class<Stream>());
}

void interactive_compile_loop(const VMachinePtr& vm){
	SmartPtr<InteractiveStream> stream = unchecked_ptr_cast<InteractiveStream>(vm->arg(0));
	SmartPtr<xpeg::Executor> exec = unchecked_ptr_cast<xpeg::Executor>(vm->arg(1));

	CodeBuilder cb;
	CodePtr code = cb.eval_compile(exec);

	vm->eval(code, 1);
	
	XTAL_CATCH_EXCEPT(e){
		exec->skip();
		exec->clear_errors();
		stdout_stream()->println(e);
	}

	stream->end_stmt();
	vm->return_result();
}

void interactive_compile(){
	if(CodePtr code = Xsrc((
		quitvalue: false;
		fun quit(){ quitvalue = true; }

		while(!quitvalue){ 
			interactive_compile_loop(arg[0], arg[1]); 
		}
	))){
		code->def_fun(XTAL_STRING("interactive_compile_loop"), &interactive_compile_loop);
		StreamPtr stream = xnew<InteractiveStream>();
		xpeg::ExecutorPtr exec = xnew<xpeg::StreamExecutor>(stream, XTAL_STRING("ix"));
		code->call(stream, exec);
	}
}

int main(int argc, char** argv){

	CStdioStdStreamLib cstd_std_stream_lib;
	WinThreadLib win_thread_lib;
	WinFilesystemLib win_filesystem_lib;
	SJISChCodeLib sjis_ch_code_lib;

	Setting setting;
	setting.thread_lib = &win_thread_lib;
	setting.std_stream_lib = &cstd_std_stream_lib;
	setting.filesystem_lib = &win_filesystem_lib;
	setting.ch_code_lib = &sjis_ch_code_lib;

	initialize(setting);

	XTAL_PROTECT{
		bind_error_message();

		interactive_compile();

		XTAL_CATCH_EXCEPT(e){
			stderr_stream()->println(e);
		}
	}
	XTAL_OUT_OF_MEMORY{
		puts("out of memory");
	}
		
	uninitialize();

	return 0;
}
