/** \file src/xtal/xtal_filesystem.h
* \brief src/xtal/xtal_filesystem.h
*/

#ifndef XTAL_FILESYSTEM_H_INCLUDE_GUARD
#define XTAL_FILESYSTEM_H_INCLUDE_GUARD

#pragma once

namespace xtal{

class FilesystemLib;

/**
* \xbind lib::builtin
* \brief ファイルシステム
*/
namespace filesystem{

/**
* \xbind lib::builtin::filesystem
* \brief ファイルオープン
*/
StreamPtr open(const StringPtr& file_name, const StringPtr& flags);

/**
* \xbind lib::builtin::filesystem
* \brief ディレクトリかどうか
*/
bool is_directory(const StringPtr& path);

/**
* \xbind lib::builtin::filesystem
* \brief path以下のエントリを列挙するIteratorを返す
*/
AnyPtr entries(const StringPtr& path);

class Filesystem{};

class Entries : public Base{
public:

	Entries(const StringPtr& path){
		impl_ = filesystem_lib()->new_entries(path->c_str());
	}

	~Entries(){
		block_break();
		filesystem_lib()->delete_entries(impl_);
	}

	void block_next(const VMachinePtr& vm){
		if(const char_t* next = filesystem_lib()->next_entries(impl_)){
			vm->return_result(to_smartptr(this), xnew<String>(next));
		}
		else{
			vm->return_result(null, null);
		}
	}

	void block_break(){
		filesystem_lib()->break_entries(impl_);
	}

private:
	void* impl_;
};

}

}


#endif // XTAL_FILESYSTEM_H_INCLUDE_GUARD
