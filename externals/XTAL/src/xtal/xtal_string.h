/** \file src/xtal/xtal_string.h
* \brief src/xtal/xtal_string.h
*/

#ifndef XTAL_STRING_H_INCLUDE_GUARD
#define XTAL_STRING_H_INCLUDE_GUARD

#pragma once

namespace xtal{

class StringData;

/**
* \xbind lib::builtin
* \xinherit lib::builtin::Any lib::builtin::Iterable
* \brief 文字列のクラス
*/
class String : public Any{
public:

	/**
	* \xbind
	* \brief 空文字列を構築する
	*/
	String();

	/**
	* \brief NUL終端のC文字列から構築する
	* \param str NUL終端文字列
	*/
	String(const char_t* str);

	/**
	* \brief C文字列からsize分の長さを取り出し構築する
	*/
	String(const char_t* str, uint_t size);

	/**
	* \brief 二つのC文字列から構築する
	*/
	String(const char_t* str1, uint_t size1, const char_t* str2, uint_t size2);


	struct long_lived_t{};

	/**
	* \brief 寿命が長く、開放されないような文字列で構築する
	*/
	String(const char_t* str, uint_t size, long_lived_t){
		init_long_lived_string(str, size);
	}

	/**
	* \brief 寿命が長く、開放されないような文字列で構築する
	*/
	template<int N>
	String(const LongLivedStringN<N>& str){
		init_long_lived_string(str.str(), str.size());
	}

	/**
	* \brief 寿命が長く、開放されないような文字列で構築する
	*/
	String(const LongLivedString& str){
		init_long_lived_string(str.str(), str.size());
	}

public:
	
	String(const String& s);

protected:

	String(noinit_t){}

public:

	/**
	* \brief 0終端の文字列先頭のポインタを返す。
	* data()とdata_size()で代用可能ならそちらを使ってください。
	*/
	const char_t* c_str() const;

	/**
	* \brief 文字列先頭のポインタを返す。
	* これは0終端文字列が返されるとは限らない。
	*/
	const char_t* data() const;

	/**
	* \xbind
	* \brief データサイズを返す。
	* マルチバイト文字を考慮しない。
	*/
	uint_t data_size() const;

	/**
	* \xbind
	* \brief 文字列の長さを返す。
	* マルチバイト文字を考慮する。
	*/
	uint_t length() const;

	/*
	* \brief 浅いコピーを返す。
	*/
	const StringPtr& clone() const;

	/**
	* \xbind
	* \brief 一意化した文字列を返す。
	*/
	IDPtr intern() const;

	/**
	* \xbind
	* \brief 一意化されているか返す。
	*/
	bool is_interned() const;

	/**
	* \xbind
	* \brief 整数に変換した結果を返す。
	*/ 
	int_t to_i() const;
	
	/**
	* \xbind
	* \brief 浮動小数点数に変換した結果を返す。
	*/ 
	float_t to_f() const;
	
	/**
	* \xbind
	* \brief 自分自身を返す。
	*/
	const StringPtr& to_s() const;

	/**
	* \xbind
	* \brief 一文字づつの文字列を要素とするIteratorを返す。
	* \return 一文字づつの文字列を要素とするIterator
	*/
	AnyPtr each() const;

	/**
	* \xbind
	* \brief 連結する
	* \return 連結された新しいオブジェクト
	*/
	StringPtr cat(const StringPtr& v) const;

public:

	/**
	* \xbind
	* \brief 一文字の文字列かどうか
	*/
	bool is_ch() const;

	/**
	* \xbind
	* \brief 空の文字列かどうか
	*/
	bool is_empty() const;
	bool empty() const{ return is_empty(); }

	/**
	* \xbind
	* \brief 一文字の文字列の場合、そのasciiコードを返す
	*/
	int_t ascii() const;

	/**
	* \xbind
	* \brief 文字の範囲オブジェクトの範囲内かどうか
	*/
	bool op_in(const ChRangePtr& range) const;

public:

	/**
	* \xbind
	* \brief 範囲オブジェクトを生成する
	*/
	ChRangePtr op_range(const StringPtr& right, int_t kind) const;
	
	/**
	* \xbind
	* \brief 連結する
	*/
	StringPtr op_cat(const StringPtr& v) const;
	
	/**
	* \xbind
	* \brief 値が等しいか比べる
	*/
	bool op_eq(const StringPtr& v) const;

	/**
	* \xbind
	* \brief より小さいか比べる
	*/
	bool op_lt(const StringPtr& v) const;

public:

	/**
	* \xbind
	* \brief 文字列をスキャンする
	*/
	AnyPtr scan(const AnyPtr& pattern) const;

	/**
	* \xbind
	* \brief 文字列を分割し、その部分文字列を要素とするIteratorを返す
	*/
	AnyPtr split(const AnyPtr& pattern) const;

	/**
	* \xbind
	* \brief 文字列を分割し、その部分文字列を要素とするIteratorを返す
	*/	
	bool match(const AnyPtr& pattern) const;

	/**
	* \xbind
	* \brief マッチする部分を置き換えた新しい文字列を返す
	*/	
	StringPtr gsub(const AnyPtr& pattern, const AnyPtr& fn) const;

	/**
	* \xbind
	* \brief 最初にマッチする部分を置き換えた新しい文字列を返す
	*/	
	StringPtr sub(const AnyPtr& pattern, const AnyPtr& fn) const;

public:

	void on_rawcall(const VMachinePtr& vm);

private:
	void init_string(const char_t* str, uint_t size);
	void init_long_lived_string(const char_t* str, uint_t size);
	void init_small_string(const char_t* str, uint_t size);
	StringData* new_string_data(uint_t size);

public:

	class iterator{
	public:
		
		iterator(const char_t* p)
			:p_(p){}
		
		char_t operator *() const{
			return *p_;
		}

		iterator& operator ++(){
			++p_;
			return *this;
		}

		iterator operator ++(int){
			iterator temp(*this);
			++p_;
			return temp;
		}

		friend bool operator ==(iterator a, iterator b){
			return a.p_ == b.p_;
		}

		friend bool operator !=(iterator a, iterator b){
			return a.p_ != b.p_;
		}

	private:
		const char_t* p_;
	};

	iterator begin() const;

	iterator end() const;
};

class StringData : public RefCountingBase{
	char_t* buf_;
	uint_t data_size_;
public:

	enum{
		TYPE = TYPE_STRING
	};

	StringData(uint_t size);

	~StringData();

	uint_t data_size(){ return data_size_; }

	char_t* buf(){ return buf_; }

private:
	XTAL_DISALLOW_COPY_AND_ASSIGN(StringData);
};


IDPtr intern(const char_t* str);
IDPtr intern(const char_t* str, String::long_lived_t);
IDPtr intern(const char_t* str, uint_t size);
IDPtr intern(const char_t* str, uint_t size, String::long_lived_t);
IDPtr intern(const LongLivedString& str);
IDPtr intern(const StringPtr& name);		

/**
* \brief Intern済みのString
*
* これはC++の型システムのために存在する。
* XtalにおいてIntern済みStringはString型である。
*/
class ID : public String{
public:
	/**
	* \brief NUL終端のC文字列から構築する
	*
	* \param str NULL終端文字列
	*/
	ID(const char_t* str)
		:String(*xtal::intern(str)){}

	/**
	* \brief C文字列からsize分の長さを取り出し構築する
	*
	*/
	ID(const char_t* str, uint_t size)
		:String(*xtal::intern(str, size)){}

	/**
	* \brief 寿命が長く、開放されないような文字列で構築する
	*
	*/
	ID(const char_t* str, uint_t size, String::long_lived_t)
		:String(*xtal::intern(str, size, String::long_lived_t())){}

	/**
	* \brief 寿命が長く、開放されないような文字列で構築する
	*
	*/
	template<int N>
	ID(const LongLivedStringN<N>& str)
		:String(*xtal::intern(str.str(), str.size(), String::long_lived_t())){}

	/**
	* \brief 寿命が長く、開放されないような文字列で構築する
	*
	*/
	ID(const LongLivedString& str)
		:String(*xtal::intern(str.str(), str.size(), String::long_lived_t())){}

	/**
	* \brief Stringから構築する
	*
	*/
	ID(const StringPtr& name)	
		:String(name->is_interned() ? *name : *xtal::intern(name)){}

public:

	struct intern_t{};

	ID(const char_t* str, uint_t size, intern_t)
		:String(noinit_t()){
		value_.init_interned_string(str, size);
	}

	struct small_intern_t{};

	ID(const char_t* str, uint_t size, small_intern_t)
		:String(noinit_t()){
		value_.init_small_string(str, size);
	}
};

inline bool operator ==(const IDPtr& a, const IDPtr& b){ return XTAL_detail_raweq(a, b)!=0; }
inline bool operator !=(const IDPtr& a, const IDPtr& b){ return XTAL_detail_raweq(a, b)==0; }

int_t edit_distance(const StringPtr& str1, const StringPtr& str2);

class ChRange : public Range{
public:

	ChRange(const StringPtr& left, const StringPtr& right)
		:Range(left, right, RANGE_CLOSED){}

public:

	StringPtr left(){ return unchecked_ptr_cast<String>(left_); }

	StringPtr right(){ return unchecked_ptr_cast<String>(right_); }

	AnyPtr each();
};

class ChRangeIter : public Base{
public:

	ChRangeIter(const ChRangePtr& range)
		:it_(range->left()), end_(range->right()){}

	void block_next(const VMachinePtr& vm);

private:
	StringPtr it_, end_;
};

}

#endif // XTAL_STRING_H_INCLUDE_GUARD
