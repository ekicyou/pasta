/** \file src/xtal/xtal_except.h
* \brief src/xtal/xtal_except.h
*/

#ifndef XTAL_EXCEPT_H_INCLUDE_GUARD
#define XTAL_EXCEPT_H_INCLUDE_GUARD

#pragma once

namespace xtal{

/**
* \xbind lib::builtin
* \brief 例外
*/
class Exception : public Base{
public:

	Exception(const AnyPtr& message = empty_string);

	/**
	* \xbind
	* \brief 初期化する
	*/
	void initialize(const AnyPtr& message = empty_string);

	/**
	* \xbind
	* \brief バックトレースを追加する
	*/
	void append_backtrace(const AnyPtr& file, int_t line, const AnyPtr& function_name = empty_string);

	/**
	* \xbind
	* \brief 文字列化する
	*/
	StringPtr to_s();

	/**
	* \xbind
	* \brief メッセージを返す
	*/
	const StringPtr& message(){
		return message_;
	}

	/**
	* \xbind
	* \brief バックトレースの情報を要素とするIteratorを返す
	*/
	AnyPtr backtrace(){
		return backtrace_->each();
	}

	void on_visit_members(Visitor& m){
		Base::on_visit_members(m);
		m & message_ & backtrace_;
	}

private:
	StringPtr message_;
	ArrayPtr backtrace_;
};

class StandardError{};
class RuntimeError{};
class IOError{};
class EOSError{};
class LogicError{};
class CastError{};
class ArgumentError{};
class YieldError{};
class InstanceVariableError{};
class UnsupportedError{};
class RedefinedError{};
class AccessibilityError{};
class AssertionFailed{};
class CompileError{};

AnyPtr unsupported_error(const AnyPtr& target, const IDPtr& primary_key, const AnyPtr& secondary_key);
AnyPtr filelocal_unsupported_error(const CodePtr& code, const IDPtr& primary_key);

void set_unsupported_error(const AnyPtr& target, const IDPtr& primary_key, const AnyPtr& secondary_key, const VMachinePtr& vm = vmachine());
void set_runtime_error(const AnyPtr& arg, const VMachinePtr& vm = vmachine());
void set_argument_type_error(const AnyPtr& object, int_t no, const ClassPtr& required, const ClassPtr& type, const VMachinePtr& vm = vmachine());
void set_argument_num_error(const AnyPtr& funtion_name, int_t n, int_t min_count, int_t max_count, const VMachinePtr& vm = vmachine());

/**
* \brief 例外を設定する
* \hideinitializer
*/
#define XTAL_SET_EXCEPT(e) ::xtal::vmachine()->set_except(e)

/**
* \brief 例外が送出されているなら取り出す
* \hideinitializer
*/
#define XTAL_CATCH_EXCEPT(e) if(const ::xtal::ExceptionPtr& e = (::xtal::unchecked_ptr_cast< ::xtal::Exception>(::xtal::vmachine()->catch_except())))

/**
* \brief 例外が送出されているか調べる
* XTAL_CATCH_EXCEPTと違い、例外は設定されたまま。
* \hideinitializer
*/
#define XTAL_CHECK_EXCEPT(e) if(const ::xtal::ExceptionPtr& e = (::xtal::unchecked_ptr_cast< ::xtal::Exception>(::xtal::vmachine()->except())))

}

#endif // XTAL_EXCEPT_H_INCLUDE_GUARD
