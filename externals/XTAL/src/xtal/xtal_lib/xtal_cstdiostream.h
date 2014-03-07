
#pragma once

#include <cstdio>

namespace xtal{

class CStdioStdStreamLib : public StdStreamLib{
public:
	virtual uint_t read_stdin_stream(void* stdin_stream_object, void* dest, uint_t size){
		return fread(dest, 1, size, stdin); 
	}

	virtual uint_t write_stdout_stream(void* stdout_stream_object, const void* src, uint_t size){ 
		return fwrite(src, 1, size, stdout); 
	}

	virtual uint_t write_stderr_stream(void* stderr_stream_object, const void* src, uint_t size){ 
		return fwrite(src, 1, size, stderr); 
	}
};

}
