/** \file src/xtal/xtal_environment.h
* \brief src/xtal/xtal_environment.h
*/

#ifndef XTAL_ENVIRONMENT_H_INCLUDE_GUARD
#define XTAL_ENVIRONMENT_H_INCLUDE_GUARD

#pragma once

namespace xtal{

/// \name Xtal環境の初期化、破棄周り
//@{

/**
* \brief アロケータライブラリ
*/
class AllocatorLib{
public:
	virtual ~AllocatorLib(){}

	virtual void* malloc(std::size_t size){ return std::malloc(size); }
	virtual void free(void* p, std::size_t /*size*/){ std::free(p); }

	virtual void* malloc_align(std::size_t size, std::size_t alignment);
	virtual void free_align(void* p, std::size_t size, std::size_t alignment);

	virtual void out_of_memory(){}
};

/**
* \brief 文字コードライブラリ
*/
class ChCodeLib{
public:
	virtual ~ChCodeLib(){}

	virtual int_t ch_len(char_t /*lead*/){ return 1; }
	virtual int_t ch_len2(const char_t* str){ return ch_len(*str); }
	virtual int_t ch_inc(const char_t* data, int_t data_size, char_t* dest, int_t dest_size);
	virtual int_t ch_cmp(const char_t* a, int_t asize, const char_t* b, int_t bsize);
};

/**
* \brief スレッドライブラリ
*/
class ThreadLib{
public:
	virtual ~ThreadLib(){}

	virtual void* new_thread(){ return 0; }
	virtual void delete_thread(void* /*thread_object*/){}
	virtual void start_thread(void* /*thread_object*/, void (*callback)(void*), void* data){ callback(data); }
	virtual void join_thread(void* /*thread_object*/){}

	virtual void* new_mutex(){ return 0; }
	virtual void delete_mutex(void* /*mutex_object*/){}
	virtual void lock_mutex(void* /*mutex_object*/){}
	virtual void unlock_mutex(void* /*mutex_object*/){}

	virtual void* new_event(){ return 0; }
	virtual void delete_event(void* /*event_object*/){}
	virtual void wait_event(void* /*event_object*/){}
	virtual void signal_event(void* /*event_object*/){}

	virtual void yield(){}
	virtual void sleep(float_t /*sec*/){}
};

/**
* \brief 標準入出力ライブラリ
*/
class StdStreamLib{
public:
	virtual ~StdStreamLib(){}

	virtual void* new_stdin_stream(){ return 0; }
	virtual void delete_stdin_stream(void* /*stdin_stream_object*/){}
	virtual uint_t read_stdin_stream(void* /*stdin_stream_object*/, void* /*dest*/, uint_t /*size*/){ return 0; }

	virtual void* new_stdout_stream(){ return 0; }
	virtual void delete_stdout_stream(void* /*stdout_stream_object*/){}
	virtual uint_t write_stdout_stream(void* /*stdout_stream_object*/, const void* /*src*/, uint_t /*size*/){ return 0; }

	virtual void* new_stderr_stream(){ return 0; }
	virtual void delete_stderr_stream(void* /*stderr_stream_object*/){}
	virtual uint_t write_stderr_stream(void* /*stderr_stream_object*/, const void* /*src*/, uint_t /*size*/){ return 0; }
};

/**
* \brief ファイルシステムライブラリ
*/
class FilesystemLib{
public:
	virtual ~FilesystemLib(){}

	virtual bool is_directory(const char_t* /*path*/){ return false; }
	virtual uint_t mtime(const char_t*){ return 0; }

	virtual void* new_file_stream(const char_t* /*path*/, const char_t* /*flags*/){ return 0; }
	virtual void delete_file_stream(void* /*file_stream_object*/){}
	virtual uint_t read_file_stream(void* /*file_stream_object*/, void* /*dest*/, uint_t /*size*/){ return 0; }
	virtual uint_t write_file_stream(void* /*file_stream_object*/, const void* /*src*/, uint_t /*size*/){ return 0; }
	virtual void seek_file_stream(void* /*file_stream_object*/, uint_t /*pos*/){}
	virtual uint_t tell_file_stream(void* /*file_stream_object*/){ return 0; }
	virtual bool end_file_stream(void* /*file_stream_object*/){ return true; }
	virtual uint_t size_file_stream(void* /*file_stream_object*/){ return 0; }
	virtual void flush_file_stream(void* /*file_stream_object*/){}

	virtual void* new_entries(const char_t* /*path*/){ return 0; }
	virtual void delete_entries(void* /*entries_object*/){}
	virtual const char_t* next_entries(void* /*entries_object*/){ return 0; }
	virtual void break_entries(void* /*entries_object*/){}
};

/**
* \brief 使用ライブラリの指定のための構造体
*/
struct Setting{
	AllocatorLib* allocator_lib;
	ChCodeLib* ch_code_lib;
	ThreadLib* thread_lib;
	StdStreamLib* std_stream_lib;
	FilesystemLib* filesystem_lib;

	/**
	* \brief ほとんど何もしない動作を設定する。
	*/
	Setting();
};

/**
* \brief Xtal実行環境を作成し、初期化し、カレントに設定する。
*/
void initialize(const Setting& setting);

/**
* \brief カレントのXtal実行環境を破棄する。
*/
void uninitialize();

/**
* \brief 現在のカレントのXtal実行環境を取得する。
*/
Environment* environment();

/**
* \brief カレントのXtal実行環境を設定する。
*/
void set_environment(Environment* e);

//@}

ThreadLib* thread_lib();

StdStreamLib* std_stream_lib();

FilesystemLib* filesystem_lib();

/////////////////////////////////////////////////////

/// \name メモリ周り
//@{

/**
* \brief ユーザーが登録したメモリアロケート関数を使ってメモリ確保する。
*/
void* xmalloc(size_t size);

/**
* \brief ユーザーが登録したメモリデアロケート関数を使ってメモリ解放する。
*/
void xfree(void* p, size_t size);

/**
* \brief メモリ確保をスコープに閉じ込めるためのユーティリティクラス
*/
struct XMallocGuard{
	/**
	* \brief 確保せずに構築されるコンストラクタ
	*/
	XMallocGuard():p(0){}
	
	/**
	* \brief size分のメモリ確保するコンストラクタ
	*/
	explicit XMallocGuard(uint_t size):p(xmalloc(size)), sz(size){}

	/**
	* \brief メモリを解放する
	*/
	~XMallocGuard(){ if(p)xfree(p, sz); }
public:	

	/**
	* \brief メモリを確保する
	* それまでに確保されていたメモリは解放される
	*/
	void malloc(size_t size){ if(p)xfree(p, sz); p = xmalloc(size); sz = size; }

	/**
	* \brief メモリの先頭アドレスを返す
	*/
	void* get(){ return p; }

	/**
	* \brief メモリの先頭アドレスを返す
	*/
	void* release(){
		void* temp = p;
		p = 0;
		return temp; 
	}

	/**
	* \brief 確保されているメモリのサイズを返す
	*/
	uint_t size(){ return sz; }

private:
	void* p;
	uint_t sz;

	XTAL_DISALLOW_COPY_AND_ASSIGN(XMallocGuard);
};

#define XTAL_PROTECT if(const ::xtal::Protect& xtalprotect = setjmp(::xtal::protect().buf)!=0)

#define XTAL_OUT_OF_MEMORY else 

//@}

/**
* \internal
* \brief jmp_bufを閉じ込める構造体
*/
struct JmpBuf{
	jmp_buf buf;
};

JmpBuf& protect();

void reset_protect();

bool ignore_memory_assert();

struct Protect{
	bool pass;
	Protect(bool pass):pass(pass){}
	~Protect(){ reset_protect(); }
	operator bool() const{ return !pass; }
};

/////////////////////////////////////////////////////

/**
* \xbind lib::builtin
* \brief ガーベジコレクションを実行する
*
* さほど時間はかからないが、完全にゴミを解放できないガーベジコレクト関数
* 例えば循環参照はこれでは検知できない。
*/
void gc();

/**
* \xbind lib::builtin
* \brief 循環オブジェクトも解放するフルガーベジコレクションを実行する
*
* 時間はかかるが、現在ゴミとなっているものはなるべく全て解放するガーベジコレクト関数
* 循環参照も検知できる。
*/
void full_gc();

/**
* \xbind lib::builtin
* \brief ガーベジコレクションを無効化する
*
* gcやfull_gcの呼び出しを無効化する関数。
* 内部でこれが何回呼び出されたか記憶されており、呼び出した回数enable_gcを呼びないとガーベジコレクションは有効にならない
*/
void disable_gc();

/**
* \xbind lib::builtin
* \brief ガーベジコレクションを有効化する

* disable_gcが呼ばれた回数と同じだけ呼び出すとガーベジコレクションが有効になる
*/
void enable_gc();

void set_gc_stress(bool b);

uint_t alive_object_count();

AnyPtr alive_object(uint_t i);

/////////////////////////////////////////////////////

/**
* \internal
* \brief keyに対応するC++のクラスのクラスオブジェクトを返す。
*/
const ClassPtr& cpp_class(CppClassSymbolData* key);

/**
* \brief クラスTに対応するC++のクラスのクラスオブジェクトを返す。
*/
template<class T>
inline const ClassPtr& cpp_class(){
	return cpp_class(&CppClassSymbol<T>::value);
}

const ClassPtr& cpp_class_index(uint_t index);

void set_cpp_class(CppClassSymbolData* key, const ClassPtr& cls);

/////////////////////////////////////////////////////

/**
* \internal
* \brief T型のオブジェクトを環境から取り出す
*/
const AnyPtr& cpp_value(CppValueSymbolData* key);

/**
* \brief T型のオブジェクトを環境から取り出す
*/
template<class T>
const SmartPtr<T>& cpp_value(){
	return unchecked_ptr_cast<T>(cpp_value(&CppValueSymbol<T>::value));
}

/////////////////////////////////////////////////////

/**
* \internal
* \brief クラスのメンバを取り出す。
*/
//const AnyPtr& cache_member(Base* target_class, const IDPtr& primary_key, const AnyPtr& secondary_key, int_t& accessibility);

/**
* \internal
* \brief クラスの継承関係を調べる。
*/
//bool cache_is(const AnyPtr& target_class, const AnyPtr& klass);

/**
* \internal
* \brief クラスのコンストラクタがキャッシュされているから調べる。
*/
//bool cache_ctor(const AnyPtr& target_class, int_t kind);

/**
* \internal
* \brief キャッシュを消す。
*/
void clear_cache();

/**
* \internal
* \brief メンバーのキャッシュテーブルに登録されているデータを無効にする。
*/
void invalidate_cache_member();

/**
* \internal
* \brief 継承関係のキャッシュテーブルに登録されているデータを無効にする。
*/
void invalidate_cache_is();

/////////////////////////////////////////////////////

/**
* \internal
* \brief VMachineインスタンスをレンタルする。
*/
VMachinePtr vmachine_take_over();

/**
* \internal
* \brief VMachineインスタンスを返却する。
*/
void vmachine_take_back(const VMachinePtr& vm);

/**
* \internal
* \brief デフォルトで使用するVMachineインスタンスを設定する。
*/
VMachinePtr set_vmachine(const VMachinePtr& vm);

/////////////////////////////////////////////////////

/// \name ライブラリオブジェクト取得関数
// @{

/**
* \brief builtinオブジェクトを返す
*/
const ClassPtr& builtin();

/**
* \brief libオブジェクトを返す
*/
const LibPtr& lib();

/**
* \brief globalオブジェクトを返す
*/
const ClassPtr& global();

/**
* \brief stdinストリームオブジェクトを返す
*/
const StreamPtr& stdin_stream();

/**
* \brief stdoutストリームオブジェクトを返す
*/
const StreamPtr& stdout_stream();

/**
* \brief stderrストリームオブジェクトを返す
*/
const StreamPtr& stderr_stream();

/**
* \brief VMachinePtrオブジェクトを返す
*
* グローバルなVMachinePtrオブジェクトを返す。
* スレッド毎にこのグローバルVMachinePtrオブジェクトは存在する。
*/
const VMachinePtr& vmachine();

VMachine* vmachine2();

const VMachinePtr& setup_call();
const VMachinePtr& setup_call(int_t need_result_count);

// @}

/**
* \brief 文字列をインターン済み文字列に変換する
*/
IDPtr intern(const char_t* str);

/**
* \brief C文字列からsize分の長さを取り出しインターン済み文字列に変換する
*
*/
IDPtr intern(const char_t* str, uint_t size);

/**
* \brief C文字列からsize分の長さを取り出しインターン済み文字列に変換する
*
*/
IDPtr intern(const char_t* str, uint_t size, String::long_lived_t);

/**
* \brief C文字列リテラルからインターン済み文字列に変換する
*
*/
template<int N>
inline IDPtr intern(const LongLivedStringN<N>& str){
	return intern(str.str(), str.size(), String::long_lived_t());
}

/**
* \brief C文字列リテラルからインターン済み文字列に変換する
*
*/
IDPtr intern(const LongLivedString& str);

/**
* \brief Stringからインターン済み文字列に変換する
*
*/
IDPtr intern(const StringPtr& name);		

/**
* \brief 定義済みインターン済み文字列を取得する
*
*/
IDPtr fetch_defined_id(uint_t index);

/**
* \brief 環境をロックする
*/
void xlock();

/**
* \brief 環境をアンロックする
*/
void xunlock();

struct XUnlock{
	XUnlock(int){ xunlock(); }
	~XUnlock(){ xlock(); }
	operator bool() const{ return true; }
private:
	XUnlock(const XUnlock&);
	void operator =(const XUnlock&);
};

struct XLock{
	XLock(int){ xlock(); }
	~XLock(){ xunlock(); }
	operator bool() const{ return true; }
private:
	XLock(const XLock&);
	void operator =(const XLock&);
};

#ifdef XTAL_NO_THREAD
#	define XTAL_UNLOCK 
#	define XTAL_LOCK 
#	define XTAL_UNLOCK_DIRECT 
#	define XTAL_LOCK_DIRECT 
#else
#	define XTAL_UNLOCK if(const ::xtal::XUnlock& xunlock = (XTAL_UNUSED_VAR(xunlock), 0))
#	define XTAL_LOCK if(const ::xtal::XLock& xlock = (XTAL_UNUSED_VAR(xlock), 0))
#	define XTAL_UNLOCK_DIRECT ::xtal::xunlock()
#	define XTAL_LOCK_DIRECT ::xtal::xlock()
#endif


bool register_thread(Environment*);
void unregister_thread(Environment*);

bool register_thread();
void unregister_thread();

struct XInterpose{
	XInterpose(int){ registered_ = register_thread(); }
	~XInterpose(){ if(registered_)unregister_thread(); }
	operator bool() const{ return true; }
private:
	XInterpose(const XInterpose&);
	void operator =(const XInterpose&);
private:
	bool registered_;
};

#define XTAL_INTERPOSE if(const ::xtal::XInterpose& xinterpose = (XTAL_UNUSED_VAR(xinterpose), 0))

/**
* \brief テキストマップを返す
*/
const MapPtr& text_map();

/**
* \internal
* \brief 先頭バイトを見て、そのマルチバイト文字が何文字かを調べる。
*
* マイナスの値が返された場合、最低文字数を返す。
* -2の場合、最低2文字以上の文字で、本当の長さは2文字目を読まないと判断できない、という意味となる。
*/
int_t ch_len(char_t lead);

/**
* \internal
* \brief マルチバイト文字が何文字かを調べる。
*
* int_t ch_len(char_t lead)が呼ばれた後、マイナスの値を返した場合に続けて呼ぶ。
* ch_lenで-2の値を返した後は、strの先には最低2バイト分のデータを格納すること。
*/
int_t ch_len2(const char_t* str);

/**
* \internal
* \brief 一つ先の文字を返す
*
* 例えば a を渡した場合、b が返る
*/
int_t ch_inc(const char_t* data, int_t data_size, char_t* dest, int_t dest_size);

/**
* \internal
* \brief 文字の大小判定
*
* 負の値 a の文字の方がbの文字より小さい
* 0の値 等しい
* 正の値 bの文字の方がaの文字より小さい
*/
int_t ch_cmp(const char_t* a, uint_t asize, const char_t* b, uint_t bsize);


StreamPtr open(const StringPtr& file_name, const StringPtr& mode);

/// \name コンパイル系関数
//@{

/**
* \xbind lib::builtin
* \brief file_nameファイルをコンパイルする。
* この戻り値をserializeすると、バイトコード形式で保存される。
* \param file_name Xtalスクリプトが記述されたファイルの名前
* \return 実行できるCodeオブジェクト
*/
CodePtr compile_file(const StringPtr& file_name);

/**
* \xbind lib::builtin
* \brief sourceをコンパイルする。
* この戻り値をserializeすると、バイトコード形式で保存される。
* \param source Xtalスクリプトが記述されたStringかStream
* \return 実行できるCodeオブジェクト
*/
CodePtr compile(const AnyPtr& source, const StringPtr& source_name = empty_string);

/**
* \xbind lib::builtin
* \brief file_nameファイルをコンパイルして実行する。
* \param file_name Xtalスクリプトが記述されたファイルの名前
* \return スクリプト内でreturnされた値
*/
AnyPtr load(const StringPtr& file_name);

//@}

CodePtr source(const char_t* src, int_t size);
void exec_source(const char_t* src, int_t size);

CodePtr compiled_source(const void* src, int_t size);
void exec_compiled_source(const void* src, int_t size);

/**
* \brief requireの実行をhookする関数を登録する
* \param hook フックする関数
*/
void append_require_source_hook(const AnyPtr& hook);
inline void set_require_source_hook(const AnyPtr& hook){ append_require_source_hook(hook); }

/**
* \brief requireの実行をhookする関数を登録解除する
* \param hook 登録した関数
*/
void remove_require_source_hook(const AnyPtr& hook);

/**
* \xbind lib::builtin
* \brief nameスクリプトを要求し、Codeオブジェクトの状態で返す
* \param nameスクリプト名
* 例えば、test.xtalを読み込みたい場合、require_source("test")とする。test.xtalcがあればそちらを読み、なければtest.xtalを読もうとする
* ただし、set_require_source_hookでフックされた関数がある場合、
* \return nameスクリプトのCodeオブジェクト
*/
CodePtr require_source(const StringPtr& name);

/**
* \xbind lib::builtin
* \brief nameスクリプトを要求し、実行した結果を返す
* \param nameスクリプト名
* 例えば、test.xtalを読み込みたい場合、require("test")とする。test.xtalcがあればそちらを読み、なければtest.xtalを読もうとする
* \return nameスクリプトが返すオブジェクト。nameスクリプトが明示的に値を返さない場合、Codeオブジェクトが返る。
*/
AnyPtr require(const StringPtr& name);

#ifndef XTAL_NO_PARSER

CodePtr eval_compile(const AnyPtr& source);

#endif

}

#endif // XTAL_ENVIRONMENT_H_INCLUDE_GUARD
