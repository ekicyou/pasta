/** \file src/xtal/xtal_stream.h
* \brief src/xtal/xtal_stream.h
*/

#ifndef XTAL_STREAM_H_INCLUDE_GUARD
#define XTAL_STREAM_H_INCLUDE_GUARD

#pragma once

namespace xtal{

/**
* \xbind lib::builtin
* \xinherit lib::builtin::Any
* \brief ストリーム
* 各メンバの中でエラーが起こった場合、xtalの例外が設定される
*/
class Stream : public Base{
public:

	virtual ~Stream(){}

	/**
	* \xbind
	* \brief 文字列strをストリームに流す
	*/
	void put_s(const StringPtr& str){
		put_s(str->data(), str->data_size());
	}

	/**
	* \brief 文字列strをストリームに流す
	*/
	void put_s(const char_t* str){
		put_s(str, string_data_size(str));
	}
	
	/**
	* \brief 文字列strをストリームに流す
	*/
	void put_s(const char_t* str, const char_t* end){
		put_s(str, end-str);
	}

	/*
	* \brief 文字列strをストリームに流す
	*/
	void put_s(const char_t* str, uint_t length);

	/**
	* \brief 文字列strをストリームに流す
	*/
	template<int N>
	void put_s(const LongLivedStringN<N>& str){
		put_s(str.str(), str.size());
	}

	/**
	* \brief 文字列strをストリームに流す
	*/
	void put_s(const LongLivedString& str){
		put_s(str.str(), str.size());
	}

	/**
	* \brief 文字列strをストリームに流す
	*/
	void put_s(const AnyPtr& str);
	
	/**
	* \xbind
	* \brief length文字分ストリームから取り出し、文字列として返す。
	*/
	StringPtr get_s(uint_t length){
		return on_get_s(length);
	}

	StringPtr get_ch();

	/**
	* \xbind
	* \brief ストリームからすべての文字を取り出し、文字列として返す
	*/
	StringPtr get_s_all(){
		return on_get_s_all();
	}

	uint_t read_charactors(AnyPtr* buffer, uint_t max){
		return on_read_charactors(buffer, max);
	}

	/**
	* \xbind
	* \brief valueを文字列化してプリントする
	*/
	void print(const AnyPtr& value);

	/**
	* \xbind
	* \brief valueを文字列化し、改行を加えてプリントする
	*/
	void println(const AnyPtr& value);

	/**
	* \xbind
	* \brief 
	*/
	void printf(const StringPtr& format_string, const ArgumentsPtr& args);

	uint_t write(const void* p, uint_t size){
		return on_write(p, size);
	}

	uint_t read(void* p, uint_t size){
		return on_read(p, size);
	}

	void read_strict(void* p, uint_t size);

	/**
	* \xbind
	* \brief ストリームの先頭からの位置を返す
	*/
	uint_t tell(){
		return on_tell();
	}

	/**
	* \xbind
	* \brief ストリームの先頭からoffsetの位置に移動する
	* ストリームの種類によってはシークできない場合もある。
	*/
	void seek(uint_t offset){
		on_seek(offset);
	}
	
	/**
	* \xbind
	* \brief ストリームで利用可能なバイト数を取得する
	* ストリームの種類によっては利用できない可能性がある
	*/
	uint_t available(){
		return on_available();
	}

	/**
	* \xbind
	* \brief ストリームをクローズする
	*/
	void close(){
		on_close();
	}

	/**
	* \xbind
	* \brief ストリームをフラッシュする
	*/
	void flush(){
		on_flush();
	}

	/**
	* \xbind
	* \brief ストリームからストリームにsizeバイト流し込む 
	*/
	uint_t pour(const StreamPtr& in_stream, uint_t size){
		return on_pour(in_stream, size);
	}

	/**
	* \xbind
	* \brief ストリームからストリームにすべて流し込む 
	*/
	uint_t pour_all(const StreamPtr& in_stream){
		return on_pour_all(in_stream);
	}

	/**
	* \xbind
	* \brief ストリームの全サイズを返す
	* ストリームの種類によっては、サイズを得ることは不可能である。
	*/
	virtual uint_t size(){
		return on_size();
	}

	/**
	* \xbind
	* \brief ストリームが終わっているか返す
	*/
	bool eos(){
		return on_eos();
	}

	void block_first(const VMachinePtr& vm);

	void block_next(const VMachinePtr& vm);

	void block_break(const VMachinePtr& vm);

	/**
	* \xbind
	* \brief オブジェクトを直列化してストリームに書き込む
	*
	* \param obj 直列化して保存したいオブジェクト
	*/
	void serialize(const AnyPtr& obj);

	/**
	* \xbind
	* \brief 直列化されたオブジェクトをストリームから読み出す
	*
	* \return 復元されたオブジェクト
	*/	
	AnyPtr deserialize();

	void skip_bom();

public:

	/**
	* \xbind
	* \brief 符号付整数8-bitをストリームに書き込む
	*/
	void put_i8(i8 v){
		put_u8(v);
	}

	/**
	* \xbind
	* \brief 符号付整数16-bitをストリームに書き込む
	*/
	void put_i16be(i16 v){
		put_u16be(v);
	}

	/**
	* \xbind
	* \brief 符号付整数16-bitをストリームに書き込む
	*/
	void put_i16le(i16 v){
		put_u16le(v);
	}

	/**
	* \xbind
	* \brief 符号付整数32-bitをストリームに書き込む
	*/
	void put_i32be(i32 v){
		put_u32be(v);
	}

	/**
	* \xbind
	* \brief 符号付整数32-bitをストリームに書き込む
	*/
	void put_i32le(i32 v){
		put_u32le(v);
	}

	/**
	* \xbind
	* \brief 符号付整数64-bitをストリームに書き込む
	*/
	void put_i64be(i64 v){
		put_u64be(v);
	}

	/**
	* \brief 符号付整数64-bitをストリームに書き込む
	*/
	void put_i64le(i64 v){
		put_u64le(v);
	}

	/**
	* \xbind
	* \brief 符号無整数8-bitをストリームに書き込む
	*/
	void put_u8(u8 v){
		write(&v, 1);
	}

	/**
	* \xbind
	* \brief 符号無整数16-bitをストリームに書き込む
	*/
	void put_u16be(u16 v);

	/**
	* \xbind
	* \brief 符号無整数16-bitをストリームに書き込む
	*/
	void put_u16le(u16 v);

	/**
	* \xbind
	* \brief 符号無整数32-bitをストリームに書き込む
	*/
	void put_u32be(u32 v);

	/**
	* \xbind
	* \brief 符号無整数32-bitをストリームに書き込む
	*/
	void put_u32le(u32 v);

	/**
	* \brief 符号無整数64-bitをストリームに書き込む
	*/
	void put_u64be(u64 v);

	/**
	* \brief 符号無整数64-bitをストリームに書き込む
	*/
	void put_u64le(u64 v);

	/**
	* \xbind
	* \brief 符号付整数8-bitをストリームから取り出す
	*/
	i8 get_i8(){
		return (i8)get_u8();
	}

	/**
	* \xbind
	* \brief 符号付整数16-bitをストリームから取り出す
	*/
	i16 get_i16be(){
		return (i16)get_u16be();
	}

	/**
	* \xbind
	* \brief 符号付整数16-bitをストリームから取り出す
	*/
	i16 get_i16le(){
		return (i16)get_u16le();
	}

	/**
	* \xbind
	* \brief 符号付整数32-bitをストリームから取り出す
	*/
	i32 get_i32be(){
		return (i32)get_u32be();
	}

	/**
	* \xbind
	* \brief 符号付整数32-bitをストリームから取り出す
	*/
	i32 get_i32le(){
		return (i32)get_u32le();
	}

	/**
	* \brief 符号付整数64-bitをストリームから取り出す
	*/
	i64 get_i64be(){
		return (i64)get_u64be();
	}

	/**
	* \brief 符号付整数64-bitをストリームから取り出す
	*/
	i64 get_i64le(){
		return (i64)get_u64le();
	}

	/**
	* \xbind
	* \brief 符号無整数8-bitをストリームから取り出す
	*/
	u8 get_u8();

	/**
	* \xbind
	* \brief 符号無整数16-bitをストリームから取り出す
	*/
	u16 get_u16be();

	/**
	* \xbind
	* \brief 符号無整数16-bitをストリームから取り出す
	*/
	u16 get_u16le();

	/**
	* \xbind
	* \brief 符号無整数32-bitをストリームから取り出す
	*/
	u32 get_u32be();

	/**
	* \xbind
	* \brief 符号無整数32-bitをストリームから取り出す
	*/
	u32 get_u32le();

	/**
	* \brief 符号無整数64-bitをストリームから取り出す
	*/
	u64 get_u64be();

	/**
	* \brief 符号無整数64-bitをストリームから取り出す
	*/
	u64 get_u64le();

	/**
	* \xbind
	* \brief 浮動小数点数32-bitをストリームに書き込む
	*/
	void put_f32be(f32 v){
		union{ u32 u; f32 f; } u;
		u.f = v;
		put_u32be(u.u);
	}

	/**
	* \xbind
	* \brief 浮動小数点数32-bitをストリームに書き込む
	*/
	void put_f32le(f32 v){
		union{ u32 u; f32 f; } u;
		u.f = v;
		put_u32le(u.u);
	}

	/**
	* \brief 浮動小数点数64-bitをストリームに書き込む
	*/
	void put_f64be(f64 v){
		union{ u64 u; f64 f; } u;
		u.f = v;
		put_u64be(u.u);
	}

	/**
	* \brief 浮動小数点数64-bitをストリームに書き込む
	*/
	void put_f64le(f64 v){
		union{ u64 u; f64 f; } u;
		u.f = v;
		put_u64le(u.u);
	}

	/**
	* \xbind
	* \brief 浮動小数点数32-bitをストリームから取り出す
	*/
	f32 get_f32be(){
		union{ u32 u; f32 f; } u;
		u.u = get_u32be();
		return u.f;
	}

	/**
	* \xbind
	* \brief 浮動小数点数32-bitをストリームから取り出す
	*/
	f32 get_f32le(){
		union{ u32 u; f32 f; } u;
		u.u = get_u32le();
		return u.f;
	}

	/**
	* \brief 浮動小数点数64-bitをストリームから取り出す
	*/
	f64 get_f64be(){
		union{ u64 u; f64 f; } u;
		u.u = get_u64be();
		return u.f;
	}

	/**
	* \brief 浮動小数点数64-bitをストリームから取り出す
	*/
	f64 get_f64le(){
		union{ u64 u; f64 f; } u;
		u.u = get_u64le();
		return u.f;
	}

	void put_ch_code_be(char_t ch){
		put_t_be<SelectType<sizeof(char_t)>::uint_t>(ch);
	}

	char_t get_ch_code_be(){
		return get_t_be<SelectType<sizeof(char_t)>::uint_t>();
	}

	void put_ch_code_le(char_t ch){
		put_t_le<SelectType<sizeof(char_t)>::uint_t>(ch);
	}

	char_t get_ch_code_le(){
		return get_t_le<SelectType<sizeof(char_t)>::uint_t>();
	}

public:

	template<class T>
	T get_t_be(){ return get_t_be((T*)0); }

	template<class T>
	void put_t_be(const T& v){ return put_t_be(v, (T*)0); }

	template<class T>
	T get_t_le(){ return get_t_le((T*)0); }

	template<class T>
	void put_t_le(const T& v){ return put_t_le(v, (T*)0); }


protected:
	/**
	* \xbind
	* \brief length文字分ストリームから取り出し、文字列として返す。
	*/
	virtual StringPtr on_get_s(uint_t length);

	/**
	* \xbind
	* \brief ストリームからすべての文字を取り出し、文字列として返す
	*/
	virtual StringPtr on_get_s_all();

	virtual uint_t on_read_charactors(AnyPtr* buffer, uint_t max);

	virtual uint_t on_write(const void* p, uint_t size);

	virtual uint_t on_read(void* p, uint_t size);

	/**
	* \xbind
	* \brief ストリームの先頭からの位置を返す
	*/
	virtual uint_t on_tell();

	/**
	* \xbind
	* \brief ストリームの先頭からoffsetの位置に移動する
	* ストリームの種類によってはシークできない場合もある。
	*/
	virtual void on_seek(uint_t offset);
	
	/**
	* \xbind
	* \brief ストリームで利用可能なバイト数を取得する
	* ストリームの種類によっては利用できない可能性がある
	*/
	virtual uint_t on_available();

	/**
	* \xbind
	* \brief ストリームをクローズする
	*/
	virtual void on_close(){}

	/**
	* \xbind
	* \brief ストリームをフラッシュする
	*/
	virtual void on_flush(){}

	/**
	* \xbind
	* \brief ストリームからストリームにsizeバイト流し込む 
	*/
	virtual uint_t on_pour(const StreamPtr& in_stream, uint_t size);

	/**
	* \xbind
	* \brief ストリームからストリームにすべて流し込む 
	*/
	virtual uint_t on_pour_all(const StreamPtr& in_stream);

	/**
	* \xbind
	* \brief ストリームの全サイズを返す
	* ストリームの種類によっては、サイズを得ることは不可能である。
	*/
	virtual uint_t on_size();

	/**
	* \xbind
	* \brief ストリームが終わっているか返す
	*/
	virtual bool on_eos(){ return false; }

private:

	i8 get_t_be(i8*){ return get_i8(); }
	i16 get_t_be(i16*){ return get_i16be(); }
	i32 get_t_be(i32*){ return get_i32be(); }
	i64 get_t_be(i64*){ return get_i64be(); }
	u8 get_t_be(u8*){ return get_u8(); }
	u16 get_t_be(u16*){ return get_u16be(); }
	u32 get_t_be(u32*){ return get_u32be(); }
	u64 get_t_be(u64*){ return get_u64be(); }
	f32 get_t_be(f32*){ return get_f32be(); }
	f64 get_t_be(f64*){ return get_f64be(); }

	void put_t_be(i8 v, i8*){ put_i8(v); }
	void put_t_be(i16 v, i16*){ put_i16be(v); }
	void put_t_be(i32 v, i32*){ put_i32be(v); }
	void put_t_be(i64 v, i64*){ put_i64be(v); }
	void put_t_be(u8 v, u8*){ put_u8(v); }
	void put_t_be(u16 v, u16*){ put_u16be(v); }
	void put_t_be(u32 v, u32*){ put_u32be(v); }
	void put_t_be(u64 v, u64*){ put_u64be(v); }
	void put_t_be(f32 v, f32*){ put_f32be(v); }
	void put_t_be(f64 v, f64*){ put_f64be(v); }

	i8 get_t_le(i8*){ return get_i8(); }
	i16 get_t_le(i16*){ return get_i16le(); }
	i32 get_t_le(i32*){ return get_i32le(); }
	i64 get_t_le(i64*){ return get_i64le(); }
	u8 get_t_le(u8*){ return get_u8(); }
	u16 get_t_le(u16*){ return get_u16le(); }
	u32 get_t_le(u32*){ return get_u32le(); }
	u64 get_t_le(u64*){ return get_u64le(); }
	f32 get_t_le(f32*){ return get_f32le(); }
	f64 get_t_le(f64*){ return get_f64le(); }

	void put_t_le(i8 v, i8*){ put_i8(v); }
	void put_t_le(i16 v, i16*){ put_i16le(v); }
	void put_t_le(i32 v, i32*){ put_i32le(v); }
	void put_t_le(i64 v, i64*){ put_i64le(v); }
	void put_t_le(u8 v, u8*){ put_u8(v); }
	void put_t_le(u16 v, u16*){ put_u16le(v); }
	void put_t_le(u32 v, u32*){ put_u32le(v); }
	void put_t_le(u64 v, u64*){ put_u64le(v); }
	void put_t_le(f32 v, f32*){ put_f32le(v); }
	void put_t_le(f64 v, f64*){ put_f64le(v); }
};

/**
* \brief ポインタ読み取りストリーム
* このストリームは読み取り専用である
*/
class PointerStream : public Stream{
public:

	PointerStream(const void* data = 0, uint_t size = 0);

	const void* data(){
		return data_;
	}

protected:
	virtual uint_t on_tell();

	virtual uint_t on_read(void* p, uint_t size);

	virtual void on_seek(uint_t offset);

	virtual uint_t on_available(){
		return size() - tell();
	}

	virtual void on_close(){}

	virtual bool on_eos();

	virtual StringPtr on_get_s(uint_t length);

	virtual StringPtr on_get_s_all();

	virtual uint_t on_size(){
		return size_;
	}
	
protected:

	const u8* data_;
	uint_t size_;
	uint_t pos_;
};

/**
* \xbind lib::builtin
* \xinherit lib::builtin::Stream
* \brief メモリーストリーム
*/
class MemoryStream : public PointerStream{
public:

	MemoryStream();
	
	MemoryStream(const void* data, uint_t data_size);

	virtual ~MemoryStream();

	void clear();

	void resize(uint_t size);

	const StringPtr& to_s();

protected:
	virtual uint_t on_write(const void* p, uint_t size);

	virtual uint_t on_pour(const StreamPtr& in_stream, uint_t size);

	virtual uint_t on_pour_all(const StreamPtr& in_stream);

protected:
	StringPtr str_;
	void resize_data(uint_t size);
	uint_t capa_;
};

/**
* \xbind lib::builtin
* \xinherit lib::builtin::Stream
* \brief 文字列ストリーム
* 文字列を読み取るためのストリーム。書き込み不可。
*/
class StringStream : public PointerStream{
public:

	StringStream(const StringPtr& str);
		
	const StringPtr& to_s(){
		return str_;
	}

public:
	void on_visit_members(Visitor& m){
		PointerStream::on_visit_members(m);
		m & str_;
	}

private:
	StringPtr str_;
};

class CompressEncoder : public Stream{
public:

	CompressEncoder(const StreamPtr& stream);

	virtual ~CompressEncoder();

protected:
	virtual uint_t on_write(const void* p, uint_t size);

	virtual void on_close();

private:

	void destroy();

private:
	void* impl_;
};

class CompressDecoder : public Stream{
public:

	CompressDecoder(const StreamPtr& stream);

	virtual ~CompressDecoder();

protected:
	virtual uint_t on_read(void* p, uint_t size);

	virtual void on_close();

private:

	void destroy();

private:
	void* impl_;
};

/**
* \xbind lib::builtin
* \xinherit lib::builtin::Stream
* \brief ファイルストリーム
*/
class FileStream : public Stream{
public:

	FileStream(){
		impl_ = 0;
	}

	/**
	* \brief ファイル開きをファイルストリームを構築する
	*/
	FileStream(const StringPtr& path, const StringPtr& flags){
		impl_ = 0;
		open(path, flags);
	}

	virtual ~FileStream(){
		if(impl_){
			filesystem_lib()->delete_file_stream(impl_);
		}
	}

	void open(const StringPtr& path, const StringPtr& flags);

	bool is_open(){
		return impl_!=0;
	}

protected:
	virtual void on_close(){
		if(impl_){
			filesystem_lib()->delete_file_stream(impl_);
			impl_ = 0;
		}
	}

	virtual uint_t on_available(){
		return size() - tell();
	}

	virtual uint_t on_read(void* dest, uint_t size){
		if(impl_){
			return filesystem_lib()->read_file_stream(impl_, dest, size);
		}
		return 0;
	}

	virtual uint_t on_write(const void* src, uint_t size){
		if(impl_){
			return filesystem_lib()->write_file_stream(impl_, src, size);
		}
		return 0;
	}

	virtual void on_seek(uint_t offset){
		if(impl_){
			filesystem_lib()->seek_file_stream(impl_, offset);
		}
	}

	virtual uint_t on_tell(){
		if(impl_){
			return filesystem_lib()->tell_file_stream(impl_);
		}
		return 0;
	}

	virtual bool on_eos(){
		if(impl_){
			return filesystem_lib()->end_file_stream(impl_);
		}
		return true;
	}

	virtual uint_t on_size(){
		if(impl_){
			return filesystem_lib()->size_file_stream(impl_);
		}
		return 0;
	}

	virtual void on_flush(){
		if(impl_){
			filesystem_lib()->flush_file_stream(impl_);
		}
	}

private:
	void* impl_;
};

class StdinStream : public Stream{
public:
	StdinStream(){
		impl_ = std_stream_lib()->new_stdin_stream();
	}

	virtual ~StdinStream(){
		std_stream_lib()->delete_stdin_stream(impl_);
	}

protected:
	virtual uint_t on_read(void* p, uint_t size){
		return std_stream_lib()->read_stdin_stream(impl_, p, size);
	}

	virtual uint_t on_read_charactors(AnyPtr* buffer, uint_t max){
		return Stream::on_read_charactors(buffer, max);
	}

private:
	void* impl_;
};

class StdoutStream : public Stream{
public:
	StdoutStream(){
		impl_ = std_stream_lib()->new_stdout_stream();
	}

	virtual ~StdoutStream(){
		std_stream_lib()->delete_stdout_stream(impl_);
	}

protected:
	virtual uint_t on_write(const void* p, uint_t size){
		return std_stream_lib()->write_stdout_stream(impl_, p, size);
	}

private:
	void* impl_;
};

class StderrStream : public Stream{
public:
	StderrStream(){
		impl_ = std_stream_lib()->new_stderr_stream();
	}

	virtual ~StderrStream(){
		std_stream_lib()->delete_stderr_stream(impl_);
	}

protected:
	virtual uint_t on_write(const void* p, uint_t size){
		return std_stream_lib()->write_stderr_stream(impl_, p, size);
	}

private:
	void* impl_;
};


template<>
struct XNew<MemoryStream> : public XXNew<MemoryStream>{
	XNew();
	XNew(const void* data, uint_t data_size);
};

}

#endif // XTAL_STREAM_H_INCLUDE_GUARD
