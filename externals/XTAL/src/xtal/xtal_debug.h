/** \file src/xtal/xtal_debug.h
* \brief src/xtal/xtal_debug.h
*/

#ifndef XTAL_DEBUG_H_INCLUDE_GUARD
#define XTAL_DEBUG_H_INCLUDE_GUARD

#pragma once

namespace xtal{

/**
* \xbind lib::builtin
* \brief デバッグ
*/
namespace debug{

class CallerInfo : public Base{
public:

	/**
	* \xbind
	* \brief 呼び出し場所の行数を返す
	*/
	int_t lineno(){ return line_; }

	const MethodPtr& fun(){ return fun_; }

	/**
	* \xbind
	* \brief 呼び出し場所のファイル名を返す
	*/
	StringPtr file_name();

	/**
	* \xbind
	* \brief 呼び出し場所の関数名を返す
	*/
	StringPtr fun_name();

	/**
	* \xbind
	* \brief 呼び出し場所の変数フレームオブジェクトを返す
	*/
	const FramePtr& variables_frame(){ return variables_frame_; }

	void set_line(int_t v){ line_ = v; }
	void set_fun(const MethodPtr& v){ fun_ = v; }
	void set_variables_frame(const FramePtr& v){ variables_frame_ = v; }

	void on_visit_members(Visitor& m);

private:
	int_t line_;
	MethodPtr fun_;
	FramePtr variables_frame_;
};

typedef SmartPtr<CallerInfo> CallerInfoPtr;

/**
* \xbind lib::builtin::debug
* \xinherit lib::builtin::Any
* \brief デバッグフック関数で渡されるオブジェクト
*/
class HookInfo : public Base{
public:
	
	/**
	* \xbind
	* \brief フックの種類を返す
	*/
	int_t kind(){ return kind_; } 

	/**
	* \xbind
	* \brief フックされた場所の関数を返す
	*/
	const MethodPtr& fun(){ return fun_; }

	/**
	* \xbind
	* \brief フックされた場所の行数を返す
	*/
	int_t lineno();

	/**
	* \xbind
	* \brief フックされた場所のCodeオブジェクトを返す
	*/
	CodePtr code();

	/**
	* \xbind
	* \brief フックされた場所のファイル名を返す
	*/
	StringPtr file_name();

	/**
	* \xbind
	* \brief フックされた場所の関数名を返す
	*/
	StringPtr fun_name();

	/**
	* \xbind
	* \brief 例外オブジェクトを返す
	*/
	const AnyPtr& exception(){ return exception_; }

	/**
	* \xbind
	* \brief フックされた場所の変数フレームオブジェクトを返す
	*/
	const FramePtr& variables_frame(){ return variables_frame_; }

	/**
	* \brief 呼び出し元の情報を返す
	*/
	CallerInfoPtr caller(uint_t n);

	int_t call_stack_size();

	void set_kind(int_t v){ kind_ = v; }
	void set_fun(const MethodPtr& v, const inst_t* pc){ fun_ = v; pc_ = pc; }
	void set_exception(const AnyPtr& e){ exception_ = e; }
	void set_variables_frame(const FramePtr& v){ variables_frame_ = v; }

	/**
	* \xbind
	* \brief クローンを作る
	* HookInfoオブジェクトは使いまわされるため、情報を保持したい場合、クローンを作る必要がある。
	*/
	SmartPtr<HookInfo> clone();

	void set_vm(const VMachinePtr& v);

	const VMachinePtr& vm();

	void on_visit_members(Visitor& m);

private:

	int_t kind_;
	AnyPtr exception_;
	FramePtr variables_frame_;
	MethodPtr fun_;
	const inst_t* pc_;
	VMachine* vm_;

	friend class VMachine;
};

typedef SmartPtr<HookInfo> HookInfoPtr;

class Debug{};

/**
* \xbind lib::builtin::debug
* \brief デバッグ機能を有効にする
* デバッグ機能はデフォルトでは無効になっている。
*/
void enable();

/**
* \xbind lib::builtin::debug
* \brief デバッグ機能を無効にする
*/
void disable();

/**
* \xbind lib::builtin::debug
* \brief デバッグ機能を強制的に有効にする
* デバッグ機能はデフォルトでは無効になっている。
*/
void enable_force(uint_t count);

/**
* \xbind lib::builtin::debug
* \brief デバッグ機能を強制的に無効にする
*/
uint_t disable_force();

/**
* \xbind lib::builtin::debug
* \brief デバッグ機能が有効かどうか
*/
bool is_enabled();

/**
* \xbind lib::builtin::debug
* \brief デバッグコンパイルが有効かどうか
*/
bool is_debug_compile_enabled();

/**
* \xbind lib::builtin::debug
* \brief デバッグコンパイルを有効にする
*/
void enable_debug_compile();

/**
* \xbind lib::builtin::debug
* \brief デバッグコンパイルを無効にする
*/
void disable_debug_compile();

uint_t hook_setting_bit();

uint_t* hook_setting_bit_ptr();

void set_hook(int_t hooktype, const AnyPtr& hook);

const AnyPtr& hook(int_t hooktype);

/**
* \xbind lib::builtin::debug
* \brief ブレークポイントに達するたびに呼び出されるフック関数を登録する
* \param hook 登録するフック関数
*/
void set_breakpoint_hook(const AnyPtr& hook);

/**
* \xbind lib::builtin::debug
* \brief 関数呼び出しされる度に呼び出されるフック関数を登録する
* \param hook 登録するフック関数
*/
void set_call_hook(const AnyPtr& hook);

/**
* \xbind lib::builtin::debug
* \brief 関数からreturnされる度に呼び出されるフック関数を登録する
* \param hook 登録するフック関数
*/
void set_return_hook(const AnyPtr& hook);

/**
* \xbind lib::builtin::debug
* \brief 例外かthrowされる度に呼び出されるフック関数を登録する
* \param hook 登録するフック関数
*/
void set_throw_hook(const AnyPtr& hook);

/**
* \xbind lib::builtin::debug
* \brief アサートが失敗される度に呼び出されるフック関数を登録する
* \param hook 登録するフック関数
*/
void set_assert_hook(const AnyPtr& hook);

/**
* \xbind lib::builtin::debug
* \brief set_breakpoint_hook関数で登録した関数を取得する
*/
const AnyPtr& breakpoint_hook();

/**
* \xbind lib::builtin::debug
* \brief set_call_hook関数で登録した関数を取得する
*/
const AnyPtr& call_hook();

/**
* \xbind lib::builtin::debug
* \brief set_return_hook関数で登録した関数を取得する
*/
const AnyPtr& return_hook();

/**
* \xbind lib::builtin::debug
* \brief set_throw_hook関数で登録した関数を取得する
*/
const AnyPtr& throw_hook();

/**
* \xbind lib::builtin::debug
* \brief set_assert_hook関数で登録した関数を取得する
*/
const AnyPtr& assert_hook();

/**
* \xbind lib::builtin::debug
* \brief 再定義が有効かどうか
*/
bool is_redefine_enabled();

/**
* \xbind lib::builtin::debug
* \brief 再定義を有効にする
*/
void enable_redefine();

/**
* \xbind lib::builtin::debug
* \brief 再定義を無効にする
*/
void disable_redefine();

//////////////////////////////

enum{
	RUN,
	STEP_OVER,
	STEP_INTO,
	STEP_OUT,
	REDO
};

void call_breakpoint_hook(int_t kind, HookInfoPtr info);

////////////////////////////////////////////////////////////////////////////////////

/**
* \brief デバッガの受信側
*/
class CommandReceiver{
public:

	~CommandReceiver();

	/**
	* \brief デバッガをスタートする
	* \param stream デバッガと通信するためのストリーム
	*/
	bool start(const StreamPtr& stream);

	/**
	* デバッガを更新する
	* 
	*/
	void update();

private:
	ArrayPtr recv_command();

	CodePtr require_source_hook(const StringPtr& name);

	void exec_command(const ArrayPtr& cmd);

	ArrayPtr make_debug_object(const AnyPtr& v, int depth = 3);

	ArrayPtr make_call_stack_info(HookInfoPtr info);

	MapPtr make_eval_expr_info(HookInfoPtr info, int level);

	void send_break(HookInfoPtr info, int level);

	int breakpointhook(HookInfoPtr info);
	int breakpointhook2(HookInfoPtr info);

	void check(HookInfoPtr);

private:
	StreamPtr stream_;
	MapPtr eval_exprs_;
	MapPtr code_map_;
	AnyPtr require_hook_;
	bool pause_;
};

////////////////////////////////////////////////////////////////////////////////////

/**
* \brief デバッガの送信側
*/
class CommandSender{
public:

	CommandSender();

	virtual ~CommandSender(){}

	/**
	* \brief デバッガをスタートする
	* \param stream デバッガと通信するためのストリーム
	*/
	void start(const StreamPtr& stream);

	/**
	* デバッガを更新する
	* 
	*/
	void update();

public:
	// 評価式に関する設定と取得

	void add_eval_expr(const StringPtr& expr);

	void remove_eval_expr(const StringPtr& expr);

	ArrayPtr eval_expr_result(const StringPtr& expr);

public:
	// コールスタックに関する設定と取得

	int call_stack_size();

	StringPtr call_stack_fun_name(int n);
	StringPtr call_stack_fun_name();

	StringPtr call_stack_file_name(int n);
	StringPtr call_stack_file_name();

	int call_stack_lineno(int n);
	int call_stack_lineno();

	void move_call_stack(int n);

public:
	// 要求されたスクリプトファイルに関する設定と取得

	StringPtr required_file();

	void required_source(const CodePtr& code);

	int level(){ return level_;	}

public:
	// ブレークポイントに関する設定と取得

	void add_breakpoint(const StringPtr& path, int n, const StringPtr& cond);
	void remove_breakpoint(const StringPtr& path, int n);

public:
	void pause();

public:
	void update_source(const CodePtr& code){}

public:
	void nostep();
	void start();

	void run();
	void step_over();
	void step_into();
	void step_out();
	void redo();

protected:

	// ブレークした際のシグナル
	virtual void on_breaked(){}

	// ブレークした際のシグナル
	virtual void on_required(){}

	// コンパイルエラーが起こった際のシグナル
	virtual void on_compile_error(){}

	// キャッチされない例外が起こった際のシグナル
	virtual void on_uncatched_exception(){}

protected:
	void send_command(const IDPtr& id);
	void erase_breakpoint(const StringPtr& path, int n);
	void add_breakpoint_inner(const StringPtr& path, int n, const StringPtr& cond);
	void remove_breakpoint_inner(const StringPtr& path, int n);
	void add_eval_expr_inner(const StringPtr& expr);
	void remove_eval_expr_inner(const StringPtr& expr);

private:
	StreamPtr stream_;

	struct CallInfo{
		StringPtr fun_name;
		StringPtr file_name;
		int lineno;
	};

	TArray<CallInfo> call_stack_;

	MapPtr exprs_;
	ArrayPtr breakpoints_;
	StringPtr required_file_;
	int level_;

	IDPtr prev_command_;
};

}

}

#endif // XTAL_DEBUG_H_INCLUDE_GUARD
