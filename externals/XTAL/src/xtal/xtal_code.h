/** \file src/xtal/xtal_code.h
* \brief src/xtal/xtal_code.h
*/

#ifndef XTAL_CODE_H_INCLUDE_GUARD
#define XTAL_CODE_H_INCLUDE_GUARD

#pragma once

namespace xtal{

/**
* \brief コンパイルされたバイトコード
*/
class Code : public Class{
public:
	
	Code();

	~Code();

public:

	/**
	* \brief コードオブジェクトをリロードする
	*/
	void reload(const CodePtr& new_code);

	/**
	* \brief コードに対応したソース行数を返す。
	*/
	int_t compliant_lineno(const inst_t* p);

	/**
	* \brief ソース行数に対応したコード位置を返す。
	*/
	const inst_t* compliant_pc(int_t p);
		
	bool set_lineno_info(uint_t line);

	int_t final_lineno();

	/**
	* \brief バイトコードのデータを返す
	*/
	const inst_t* bytecode_data(){
		return code_.data();
	}

	/**
	* \brief バイトコードのサイズを返す
	*/
	int_t bytecode_size(){
		return (int_t)code_.size();
	}

	/**
	* \brief このコードをcallしたときだけは再定義してもエラーにならないようにする
	*/
	void enable_redefine(){
		enable_redefine_ = true;
	}

	/**
	* \brief シンボルテーブルからi番目のシンボルを取り出す。
	*/
	const IDPtr& identifier(int_t i){
		return unchecked_ptr_cast<ID>(identifier_table_.at(i));
	}

	const IDPtr* identifier_data(){
		return &identifier(0);
	}

	uint_t identifier_size(){
		return identifier_table_.size();
	}

	/**
	* \brief 値テーブルからi番目の値を取り出す。
	*/
	const AnyPtr& value(int_t i){
		return value_table_.at(i);
	}

	const AnyPtr* value_data(){
		return (AnyPtr*)value_table_.data();
	}

	/**
	* \brief onceテーブルからi番目の値を取り出す。
	*/
	const AnyPtr& once_value(int_t i){
		return once_table_.at(i);
	}

	/**
	* \brief onceテーブルのi番目に値を設定する。
	*/
	void set_once_value(int_t i, const AnyPtr& v){
		once_table_.set_at(i, v);
	}

	const StringPtr& source_file_name(){ 
		return source_file_name_; 
	}

	void set_source_file_name(const StringPtr& file_name){
		source_file_name_ = file_name;
	}

	const ClassPtr& filelocal(){ 
		return to_smartptr(static_cast<Class*>(this)); 
	}

	ScopeInfo* scope_info(uint_t i){
		XTAL_ASSERT(i<scope_info_table_.size());
		return &scope_info_table_[i];
	}

	ClassInfo* class_info(uint_t i){
		XTAL_ASSERT(i<class_info_table_.size());
		return &class_info_table_[i];
	}

	FunInfo* fun_info(uint_t i){
		XTAL_ASSERT(i<xfun_info_table_.size());
		return &xfun_info_table_[i];
	}	

	ExceptInfo* except_info(uint_t i){
		XTAL_ASSERT(i<except_info_table_.size());
		return &except_info_table_[i];
	}

	const MethodPtr& first_fun(){
		return first_fun_;
	}

	StringPtr inspect();

	StringPtr inspect_range(int_t start, int_t end);

	void on_rawcall(const VMachinePtr& vm);

	void check_implicit_lookup();

	IDPtr find_near_variable(const IDPtr& primary_key);

	void add_breakpoint(int_t lineno, const AnyPtr& cond = undefined);

	void remove_breakpoint(int_t lineno);

	CodePtr breakpoint_cond(int_t lineno);

	void on_visit_members(Visitor& m){
		Class::on_visit_members(m);
		m & identifier_table_ & value_table_ & source_file_name_ & first_fun_ & once_table_;
	}

	void generated();

protected:

	void find_near_variable_inner(const IDPtr& primary_key, const ScopeInfo& info, IDPtr& pick, int_t& minv);

private:

	friend class CodeBuilder;
	friend class VMachine;
	friend class Serializer;

	typedef PODArray<inst_t> code_t;
	code_t code_;

	xarray identifier_table_;
	xarray value_table_;
	xarray once_table_;
	
	StringPtr source_file_name_;
	MethodPtr first_fun_;

	bool enable_redefine_;

	MapPtr breakpoint_cond_map_;

private:

	PODArray<FunInfo> xfun_info_table_;
	PODArray<ScopeInfo> scope_info_table_;
	PODArray<ClassInfo> class_info_table_;
	PODArray<ExceptInfo> except_info_table_;

	struct LineNumberInfo{
		u32 start_pc;
		u16 lineno;
	};

	struct LineNumberCmp{
		bool operator ()(const LineNumberInfo& lnt, uint_t pc){
			return lnt.start_pc<pc;
		}
		bool operator ()(uint_t pc, const LineNumberInfo& lnt){
			return pc<lnt.start_pc;
		}
		bool operator ()(const LineNumberInfo& l, const LineNumberInfo& r){
			return l.start_pc<r.start_pc;
		}
	};

	PODArray<LineNumberInfo> lineno_table_;

	LineNumberInfo* compliant_lineno_info(const inst_t* p);

	struct ImplcitInfo{
		u16 id;
		u16 lineno;
	};

	PODArray<ImplcitInfo> implicit_table_;
};

}

#endif // XTAL_CODE_H_INCLUDE_GUARD
