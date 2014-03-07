
#pragma once

#include <dirent.h>
#include <sys/types.h>
#include <sys/stat.h>
#include "xtal_cstdiostream.h"

namespace xtal{

class Dirent{
	DIR* dir_;
public:

	Dirent(const char_t* path){
		dir_ = opendir(path);
	}
	
	~Dirent(){
		stop();
	}
	
	const char_t* next(){
		if(!dir_){
			return 0;
		}
		
		dirent* dp = readdir(dir_);
		if(!dp){
			return 0;
		}
		return dp->d_name;
	}
	
	void stop(){
		if(dir_){
			closedir(dir_);
			dir_ = 0;
		}
	}
};

class PosixFilesystemLib : public FilesystemLib{
public:

	virtual bool is_directory(const char_t* path){
		struct stat sb;
		stat(path, &sb);
		return stat(path, &sb)!=-1 && (sb.st_mode & S_IFMT)==S_IFDIR;
	}
	
	virtual uint_t mtime(const char_t* path){ 
		struct stat sb;
		stat(path, &sb);
		return stat(path, &sb)!=-1 && sb.st_mtime;
	}

	virtual void* new_file_stream(const char_t* path, const char_t* flags){
		return fopen(path, flags);
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
		void* p = xmalloc(sizeof(Dirent));
		return new(p) Dirent(path);
	}

	virtual void delete_entries(void* entries_object){
		((Dirent*)entries_object)->~Dirent();
		xfree(entries_object, sizeof(Dirent));
	}

	virtual const char_t* next_entries(void* entries_object){
		return ((Dirent*)entries_object)->next();
	}

	virtual void break_entries(void* entries_object){
		((Dirent*)entries_object)->stop();
	}

};

}
