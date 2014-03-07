
#pragma once

#include <windows.h>
#include <stdio.h>
#include <tchar.h>

#include "xtal_cstdiostream.h"

#ifdef XTAL_USE_WCHAR
#define XTAL_TC(x) x##W
#else
#define XTAL_TC(x) x##A
#endif

namespace xtal{

class WinFindNextFile{
	HANDLE h_;
	XTAL_TC(WIN32_FIND_DATA) fd_;
	bool first_;
public:

	WinFindNextFile(const char_t* path){
		char_t path2[MAX_PATH];
		XTAL_SPRINTF(path2, MAX_PATH, XTAL_STRING("%s/*").str(), path);
		h_ = XTAL_TC(FindFirstFile)(path2, &fd_);
		first_ = true;
	}
	
	~WinFindNextFile(){
		stop();
	}
	
	const char_t* next(){
		if(h_==INVALID_HANDLE_VALUE){
			return 0;
		}

		if(first_){
			first_ = false;
			return fd_.cFileName;
		}
		else{	
			if(!XTAL_TC(FindNextFile)(h_, &fd_)){
				stop();
				return 0;
			}

			return fd_.cFileName;
		}
	}
	
	void stop(){
		if(h_!=INVALID_HANDLE_VALUE){
			FindClose(h_);
			h_ = INVALID_HANDLE_VALUE;
		}
	}
};

class WinFilesystemLib : public FilesystemLib{
public:

	virtual bool is_directory(const char_t* path){
		XTAL_TC(WIN32_FIND_DATA) fd;
		HANDLE h = XTAL_TC(FindFirstFile)(path, &fd);
		if(h==INVALID_HANDLE_VALUE){
			return false;
		}
		bool ret = (fd.dwFileAttributes&FILE_ATTRIBUTE_DIRECTORY)!=0;
		FindClose(h);
		return ret;
	}
	
	time_t FileTimeToUnixTime(const FILETIME& ft){
 		LONGLONG ll = ((LONGLONG)ft.dwHighDateTime << 32) + ft.dwLowDateTime;
		return (time_t)((ll - 116444736000000000) / 10000000);
	}
	
	virtual uint_t mtime(const char_t* path){ 
		XTAL_TC(WIN32_FIND_DATA) fd;
		HANDLE h = XTAL_TC(FindFirstFile)(path, &fd);
		if(h==INVALID_HANDLE_VALUE){
			return 0;
		}
		uint_t ret = FileTimeToUnixTime(fd.ftLastWriteTime);
		FindClose(h);
		return ret;
	}
	

	virtual void* new_file_stream(const char_t* path, const char_t* flags){
		return _tfopen(path, flags);
	}
	
	virtual void delete_file_stream(void* file_stream_object){
		fclose((FILE*)file_stream_object);
	}
	
	virtual uint_t read_file_stream(void* file_stream_object, void* dest, uint_t size){
		return fread(dest, 1, size, (FILE*)file_stream_object);
	}
	
	virtual uint_t write_file_stream(void* file_stream_object, const void* src, uint_t size){
		return fwrite(src, 1, size, (FILE*)file_stream_object);
	}
	
	virtual void seek_file_stream(void* file_stream_object, uint_t offset){
		fseek((FILE*)file_stream_object, offset, SEEK_SET);
	}
	
	virtual uint_t tell_file_stream(void* file_stream_object){
		return ftell((FILE*)file_stream_object);
	}
	
	virtual bool end_file_stream(void* file_stream_object){
		int ch = getc((FILE*)file_stream_object);
		if(feof((FILE*)file_stream_object)){
			return true;
		}
		ungetc(ch, (FILE*)file_stream_object);
		return false;
	}
	
	virtual uint_t size_file_stream(void* file_stream_object){
		uint_t pos = ftell((FILE*)file_stream_object);
		fseek((FILE*)file_stream_object, 0, SEEK_END);
		uint_t len = ftell((FILE*)file_stream_object);
		fseek((FILE*)file_stream_object, pos, SEEK_SET);
		return len;
	}

	virtual void flush_file_stream(void* file_stream_object){
		fflush((FILE*)file_stream_object);
	}

	virtual void* new_entries(const char_t* path){
		void* p = xmalloc(sizeof(WinFindNextFile));
		return new(p) WinFindNextFile(path);
	}

	virtual void delete_entries(void* entries_object){
		((WinFindNextFile*)entries_object)->~WinFindNextFile();
		xfree(entries_object, sizeof(WinFindNextFile));
	}

	virtual const char_t* next_entries(void* entries_object){
		return ((WinFindNextFile*)entries_object)->next();
	}

	virtual void break_entries(void* entries_object){
		((WinFindNextFile*)entries_object)->stop();
	}

};

}
