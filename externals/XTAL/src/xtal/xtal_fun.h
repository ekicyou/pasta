/** \file src/xtal/xtal_fun.h
* \brief src/xtal/xtal_fun.h
*/

#ifndef XTAL_FUN_H_INCLUDE_GUARD
#define XTAL_FUN_H_INCLUDE_GUARD

#pragma once

namespace xtal{

/**
* \xbind lib::builtin
* \xinherit lib::builtin::Any
* \brief 引数オブジェクト
*/
class Arguments : public Base{
public:

	/**
	* \xbind
	* \brief 順番指定引数、名前付き引数を渡して引数オブジェクトを構築する
	*/
	Arguments(const AnyPtr& ordered = undefined, const AnyPtr& named = undefined);

	const AnyPtr& op_at_int(uint_t index);

	const AnyPtr& op_at_string(const StringPtr& key);

	/**
	* \xbind
	* \brief keyに対応する引数を返す
	* keyが整数であれば、順番指定引数、文字列であれば名前付き引数を返す
	*/
	const AnyPtr& op_at(const AnyPtr& key){
		if(XTAL_detail_is_ivalue(key)){
			return op_at_int(XTAL_detail_ivalue(key));
		}
		return op_at_string(key->to_s());
	}

	/**
	* \xbind
	* \brief 順番指定引数の数を返す
	*/
	uint_t length();
	
	/**
	* \xbind
	* \brief 順番指定引数を列挙するためのイテレータを返す
	*/
	AnyPtr ordered_arguments();
	
	/**
	* \xbind
	* \brief 名前付き引数を列挙するためのイテレータを返す
	*/
	AnyPtr named_arguments();

	AnyPtr each();

	StringPtr to_s();

public:

	void add_ordered(const AnyPtr& v);

	void add_named(const AnyPtr& k, const AnyPtr& v);

	void add_named(const VMachinePtr& vm);

	uint_t ordered_size();

	uint_t named_size();

	void on_visit_members(Visitor& m);

private:

	ArrayPtr ordered_;
	MapPtr named_;
};

class ArgumentsIter : public Base{
public:

	ArgumentsIter(const ArgumentsPtr& a);
			
	void block_next(const VMachinePtr& vm);

	void on_visit_members(Visitor& m);

private:
	SmartPtr<ArrayIter> ait_;
	SmartPtr<MapIter> mit_;
	int_t index_;
};

class InstanceVariableGetter : public Any{
public:

	InstanceVariableGetter(int_t number, ClassInfo* info){
		value_.init_instance_variable_getter(number, info);
	}

public:

	int_t number() const{
		return value_.immediate_first_value();
	}

	ClassInfo* class_info() const{
		return (ClassInfo*)value_.immediate_second_vpvalue();
	}

	void on_rawcall(const VMachinePtr& vm) const;

public:

	operator const AnyPtr&() const{
		return ap(*this);
	}
};

class InstanceVariableSetter : public Any{
public:

	InstanceVariableSetter(int_t number, ClassInfo* info){
		value_.init_instance_variable_setter(number, info);
	}

public:

	int_t number() const{
		return value_.immediate_first_value();
	}

	ClassInfo* class_info() const{
		return (ClassInfo*)value_.immediate_second_vpvalue();
	}

	void on_rawcall(const VMachinePtr& vm) const;

public:

	operator const AnyPtr&() const{
		return ap(*this);
	}
};
/**
* \xbind lib::builtin
* \xinherit lib::builtin::Any
* \brief メソッドオブジェクト
*/
class Method : public HaveParentRefCountingBase{
public:
	enum{ TYPE = TYPE_METHOD };

	Method(const FramePtr& outer, const CodePtr& code, FunInfo* info);

	const FramePtr& outer(){ return outer_; }

	const CodePtr& code(){ return code_; }

	int_t pc(){ return info_->pc; }

	const inst_t* source(){ return source_; }

	const IDPtr& param_name_at(size_t i);

	int_t param_size();

	bool extendable_param();

	FunInfo* info(){ return info_; }

	//void set_info(FunInfo* fc){ info_ = fc; }

	bool check_arg(const VMachinePtr& vm);

	const IDPtr& object_temporary_name();

public:
		
	void on_rawcall(const VMachinePtr& vm);
	
	void on_visit_members(Visitor& m){
		HaveParentRefCountingBase::on_visit_members(m);
		m & outer_ & code_;
	}

private:
	BasePtr<Frame> outer_;
	BasePtr<Code> code_;
	FunInfo* info_;
	const inst_t* source_;

	friend class VMachine;
};

/**
* \xbind lib::builtin
* \xinherit lib::builtin::Method
* \brief 関数オブジェクト
*/
class Fun : public Method{
public:
	enum{ TYPE = TYPE_FUN };

	Fun(const FramePtr& outer, const AnyPtr& athis, const CodePtr& code, FunInfo* info);

public:
	
	void on_rawcall(const VMachinePtr& vm);

	void on_visit_members(Visitor& m){
		Method::on_visit_members(m);
		m & this_;
	}

	const AnyPtr& self(){
		return this_;
	}

protected:

	AnyPtr this_;
};

/**
* \xbind lib::builtin
* \xinherit lib::builtin::Fun
* \brief ラムダオブジェクト
*/
class Lambda : public Fun{
public:
	enum{ TYPE = TYPE_LAMBDA };

	Lambda(const FramePtr& outer, const AnyPtr& th, const CodePtr& code, FunInfo* info);

public:
	
	void on_rawcall(const VMachinePtr& vm);
};

/**
* \xbind lib::builtin
* \xinherit lib::builtin::Fun
* \brief ファイバーオブジェクト
*/
class Fiber : public Fun{
public:
	enum{ TYPE = TYPE_FIBER };

	Fiber(const FramePtr& outer, const AnyPtr& th, const CodePtr& code, FunInfo* info);

			
public:
	void block_next(const VMachinePtr& vm){
		call_helper(vm, true);
	}

	/**
	* \brief 実行を強制停止する
	*/
	void halt();

	/**
	* \brief ファイバーオブジェクトが実行中かどうか
	*/
	bool is_alive(){
		return alive_;
	}

	/**
	* \brief ファイバーを強制停止し、初期状態に戻す
	*/
	const FiberPtr& reset();

public:
	void on_rawcall(const VMachinePtr& vm){
		call_helper(vm, false);
	}

	void on_finalize();

	void on_visit_members(Visitor& m){
		Fun::on_visit_members(m);
		m & vm_;
	}

private:
	void call_helper(const VMachinePtr& vm, bool add_succ_or_fail_result);

private:

	VMachinePtr vm_;
	const inst_t* resume_pc_;
	bool alive_;
	bool calling_;
};

class BindedThis : public Base{
public:

	BindedThis(const AnyPtr& fun, const AnyPtr& athis)
		:fun_(fun), this_(athis){
	}

public:
	
	void on_rawcall(const VMachinePtr& vm);

	void on_visit_members(Visitor& m){
		Base::on_visit_members(m);
		m & fun_ & this_;
	}

protected:
	AnyPtr fun_;
	AnyPtr this_;
};

inline AnyPtr bind_this(const AnyPtr& method, const AnyPtr& athis){
	return xnew<BindedThis>(method, athis);
}

}


#endif // XTAL_FUN_H_INCLUDE_GUARD
