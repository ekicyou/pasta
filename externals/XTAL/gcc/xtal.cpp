
#include "../src/xtal/xtal.h"

#include "../src/xtal/xtal_lib/xtal_pthread.h"
#include "../src/xtal/xtal_lib/xtal_cstdiostream.h"
#include "../src/xtal/xtal_lib/xtal_posixfilesystem.h"
#include "../src/xtal/xtal_lib/xtal_chcode.h"
#include "../src/xtal/xtal_lib/xtal_errormessage_jp_utf8.h"

using namespace xtal;

#ifndef XTAL_NO_PARSER

static int kcode = 'u';

static void print_usage(){
	fprintf(stdout,
		"usage: xtal [options] [script [args]].\n"
		"Available options are:\n"
		"  -Ks      set kcode sjis\n"
		"  -Ku      set kcode utf-8\n"
		"  -Ke      set kcode euc\n"
		"  -v       show version information\n"
	);
}

static int init_by_argv(int argc, char**argv){
	int i;
	for(i=1; i<argc; i++){
		if(strlen(argv[i])<2){
			print_usage();
			break;
		}
		if(argv[i][0]!='-')
			break;

		switch(argv[i][1]){
		case 'v':
			fprintf(stdout, "xtal %d.%d.%d.%d\n", VERSION1, VERSION2, VERSION3, VERSION4);
			break;

		case 'K':
			if (strlen(argv[i])>2){
				kcode = argv[i][2];
				break;
			}

		default:
			print_usage();
			return 0;
		}
	}
	return i;
}

static void handle_argv(int i, int argc, char** argv){
	if(i<argc){
		ArrayPtr arg_list(xnew<Array>());
		const char *filename = argv[i++];
		for(i=1; i<argc; i++){
			arg_list->push_back(argv[i]);
		}
		builtin()->def("argv", arg_list);
		load(filename);
	}
}

int main(int argc, char** argv){
	CStdioStdStreamLib stream_lib;
	PThreadLib thread_lib;
	PosixFilesystemLib filesystem_lib;
	SJISChCodeLib sjis_ch_code_lib;
	UTF8ChCodeLib utf8_ch_code_lib;
	EUCChCodeLib euc_ch_code_lib;
	
	Setting setting;
	setting.thread_lib = &thread_lib;
	setting.std_stream_lib = &stream_lib;
	setting.filesystem_lib = &filesystem_lib;
	
	int filename_index = init_by_argv(argc, argv);
	if (filename_index == 0) {
		return 1;
	}
	switch(kcode){
	case 's': setting.ch_code_lib = &sjis_ch_code_lib; break;
	case 'u': setting.ch_code_lib = &utf8_ch_code_lib; break;
	case 'e': setting.ch_code_lib = &euc_ch_code_lib; break;
	}

	initialize(setting);

	XTAL_PROTECT{
		bind_error_message();

		handle_argv(filename_index, argc, argv);
		
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

#else

int main(int argc, char** argv){
	puts("xtal no parser");
	return 0;
}

#endif
