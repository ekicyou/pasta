/** \file src/xtal/xtal_lib.h
* \brief src/xtal/xtal_lib.h
*/

#ifndef XTAL_LIB_H_INCLUDE_GUARD
#define XTAL_LIB_H_INCLUDE_GUARD

#pragma once

namespace xtal{

/**
* \xbind lib::builtin
* \brief メンバにアクセスすると、名前に対応したライブラリファイルを読み込む機能をもったクラスの一種
*/
class AutoLoader : public Class{
public:

	AutoLoader();
	
	const AnyPtr& on_rawmember(const IDPtr& primary_key, const AnyPtr& secondary_key, bool inherited_too, int_t& accessibility, bool& nocache);

	/**
	* \xbind
	* \brief ロードパスを追加する
	*/
	void append_load_path(const StringPtr& path){
		load_path_list_->push_back(path);
	}

	void on_visit_members(Visitor& m){
		Class::on_visit_members(m);
		m & load_path_list_ & path_map_;
	}

private:
	ArrayPtr load_path_list_;
	MapPtr path_map_;
};

/**
* \xbind lib::builtin
* \brief ライブラリの登録などに使うクラスの一種
*/
class Lib : public AutoLoader{
public:
	Lib();
};

class Global : public Class{
public:
	void on_def(const IDPtr& primary_key, const AnyPtr& value, const AnyPtr& secondary_key, int_t accessibility);
};

}


#endif // XTAL_LIB_H_INCLUDE_GUARD
