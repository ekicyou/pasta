/** \file src/xtal/xtal_codebuilder.h
* \brief src/xtal/xtal_codebuilder.h
*/

#ifndef XTAL_CODEBUILDER_H_INCLUDE_GUARD
#define XTAL_CODEBUILDER_H_INCLUDE_GUARD

#pragma once

#include "xtal_parser.h"

#ifndef XTAL_NO_PARSER

namespace xtal{

class CodeBuilder{
public:

	CodeBuilder();

	~CodeBuilder();
	
	CodePtr compile(const StreamPtr& stream, const StringPtr& source_file_name = XTAL_STRING("anonymous"));

	CodePtr compile(const xpeg::ExecutorPtr& scanner, const StringPtr& source_file_name = XTAL_STRING("anonymous"));

	CodePtr eval_compile(const xpeg::ExecutorPtr& scanner);

	AnyPtr errors();

private:

	CodePtr compile_toplevel(const ExprPtr& e, const StringPtr& source_file_name);
	void adjust_values(int_t need_result_count, int_t result_count);
	address16 calc_address(const inst_t* pc, address16 address);
	void opt_jump();
	int_t reserve_label();
	void set_label(int_t lableno);
	void set_jump(int_t offset, int_t labelno);
	void process_labels();
	void put_member_code(const AnyPtr& var, bool q, const ExprPtr& secondary_key);
	void put_define_member_code(const AnyPtr& var, const ExprPtr& secondary_key);
	int_t lookup_instance_variable(const IDPtr& key, bool must = true);
	void put_set_instance_variable_code(int_t value, const IDPtr& var);
	void put_instance_variable_code(int_t result, const IDPtr& var);
	void put_val_code(const AnyPtr& val);
	void put_if_code(int_t target, int_t label_true, int_t label_false);
	void put_if_code(const ExprPtr& cond, int_t label_false, int_t stack_top);
	void put_if_code(const ExprPtr& e, int_t label_true, int_t label_false, int_t stack_top);
	void break_off(int_t to);
	int_t make_result_number(int_t stack_top, int_t result, int_t result_count);
	int_t compile_bin2(const ExprPtr& e, int_t stack_top, int_t result);
	int_t compile_member(const AnyPtr& eterm, const AnyPtr& eprimary, const AnyPtr& esecondary, int_t flags, int_t stack_top, int_t result);
	bool compile_expr(const AnyPtr& p, int_t stack_top, int_t result, int_t result_count, AnyPtr& ret);
	int_t compile_expr(const AnyPtr& p, int_t stack_top, int_t result, int_t result_count = 1);
	int_t compile_expr(const AnyPtr& p, int_t& stack_top);
	void compile_stmt(const AnyPtr& p);	
	void compile_comp_bin(const ExprPtr& e);
	int_t compile_comp_bin2(const ExprPtr& e, int_t stack_top, int_t result);
	int_t compile_send(const AnyPtr& eterm, const AnyPtr& eprimary, const AnyPtr& esecondary, const ExprPtr& args, const ExprPtr& eargs, int_t flags, int_t stack_top, int_t result, int_t result_count);
	int_t compile_property(const AnyPtr& eterm, const AnyPtr& eprimary, const AnyPtr& esecondary, int_t flags, int_t stack_top, int_t result, int_t result_count = 1);
	int_t compile_set_property(const AnyPtr& eterm, const AnyPtr& eprimary, const AnyPtr& esecondary, const AnyPtr& set, int_t flags, int_t stack_top, int_t result, int_t result_count = 1);
	int_t compile_call(int_t target, int_t self, const ExprPtr& args, const ExprPtr& eargs, int_t flags, int_t stack_top, int_t result, int_t result_count);
	void compile_lassign(int_t target, const IDPtr& var);
	void put_if_code(int_t tag, int_t target, int_t lhs, int_t rhs, int_t label_true, int_t label_false, int_t stack_top);
	void put_bin(int_t result, const ExprPtr& e, int_t a, int_t b, int_t stack_top);
	void compile_comp_bin_assert(const AnyPtr& f, const ExprPtr& e, const ExprPtr& str, const ExprPtr& mes, int_t label, int_t stack_top);
	void compile_op_assign(const ExprPtr& e, int_t stack_top);
	void put_incdec(const ExprPtr& e, int_t ret, int_t target, int_t stack_top);
	void compile_incdec(const ExprPtr& e, int_t stack_top);
	void compile_loop_control_statement(const ExprPtr& e);
	void compile_class(const ExprPtr& e, int_t stack_top, int_t result);
	int_t compile_fun(const ExprPtr& e, int_t stack_top, int_t result);
	int_t compile_e(const ExprPtr& e, int_t stack_top, int_t result, int_t result_count);
	bool is_comp_bin(const ExprPtr& e);

	template<class T> void put_inst(){ 
		T::set(code_reserve(T::ISIZE));
		code_wrote(T::ISIZE);
	}

	template<class T> void put_inst(int m1){ 
		T::set(code_reserve(T::ISIZE), m1);
		code_wrote(T::ISIZE);
	}

	template<class T> void put_inst(int m1, int m2){ 
		T::set(code_reserve(T::ISIZE), m1, m2);
		code_wrote(T::ISIZE);
	}

	template<class T> void put_inst(int m1, int m2, int m3){ 
		T::set(code_reserve(T::ISIZE), m1, m2, m3);
		code_wrote(T::ISIZE);
	}

	template<class T> void put_inst(int m1, int m2, int m3, int m4){ 
		T::set(code_reserve(T::ISIZE), m1, m2, m3, m4);
		code_wrote(T::ISIZE);
	}

	template<class T> void put_inst(int m1, int m2, int m3, int m4, int m5){ 
		T::set(code_reserve(T::ISIZE), m1, m2, m3, m4, m5);
		code_wrote(T::ISIZE);
	}

	template<class T> void put_inst(int m1, int m2, int m3, int m4, int m5, int m6){ 
		T::set(code_reserve(T::ISIZE), m1, m2, m3, m4, m5, m6);
		code_wrote(T::ISIZE);
	}

	template<class T> void put_inst(int m1, int m2, int m3, int m4, int m5, int m6, int m7){ 
		T::set(code_reserve(T::ISIZE), m1, m2, m3, m4, m5, m6, m7);
		code_wrote(T::ISIZE);
	}

	template<class T> void put_inst(int m1, int m2, int m3, int m4, int m5, int m6, int m7, int m8){ 
		T::set(code_reserve(T::ISIZE), m1, m2, m3, m4, m5, m6, m7, m8);
		code_wrote(T::ISIZE);
	}

	template<class T> void put_inst(int m1, int m2, int m3, int m4, int m5, int m6, int m7, int m8, int m9){ 
		T::set(code_reserve(T::ISIZE), m1, m2, m3, m4, m5, m6, m7, m8, m9);
		code_wrote(T::ISIZE);
	}

	inst_t* code_reserve(size_t size);

	void code_wrote(size_t size);

	int_t code_size();
	
private:

	struct FunFrame{
		int_t var_frame_count;
		int_t fun_info_num;
		int_t return_value;

		struct Label{
			int_t pos;
			struct From{
				int_t lineno;
				int_t pos;
				int_t set_pos;
			};
			PODArray<From> froms;
		};
		
		TArray<Label> labels;
		
		struct Loop{
			// ラベル名
			IDPtr label;

			// フレームの数
			int_t frame_count;

			// breakとcontinueのラベル番号
			int_t control_statement_label[2];

			// 対応するラベルを持っているか
			bool have_label;
		};
		
		FastStack<Loop> loops;

		struct Finally{
			int_t frame_count;
			int_t finally_label;
		};

		PODStack<Finally> finallies;
	};

private:

	struct ClassFrame{
		struct Entry{
			IDPtr name;
		};

		TArray<Entry> entries;
		int_t class_info_num;
	};

private:

	AnyPtr do_expr(const AnyPtr& p);

	int_t lineno(){ return linenos_.empty() ? 0 : linenos_.top(); }

	int_t register_identifier_or_compile_expr(const AnyPtr& var);
	int_t register_identifier(const IDPtr& v);
	int_t register_value(const AnyPtr& v);
	int_t append_identifier(const IDPtr& identifier);
	int_t append_value(const AnyPtr& v);

	void error(const AnyPtr& message);

private:

	struct Scope{
		enum{
			CLASS,
			FRAME,
			FUN,
			TOPLEVEL
		};

		struct Entry{
			// 変数の名前
			IDPtr name;

			// 静的に値が決まるなら非undefinedな値が入っている
			AnyPtr value;

			// 代入があるかどうか
			bool assigned;

			// 定数属性があるかどうか
			bool constant;

			// 可視かどうか
			bool visible;
			bool visible2;

			// 参照されているかどうか
			bool refered;

			// public, prentriesotected, private
			int_t accessibility;
		};

		// このスコープの種類
		int_t kind;

		// 変数のエントリ
		TArray<Entry> entries;

		// 対応するScopeInfoのインデックス
		int_t scope_info_num;

		// 変数のレジスタ番号算出のベースとなる番号
		int_t register_base;
		int_t register_max;

		// 直近の関数スコープへのポインタ
		Scope* fun_scope;

		// 親のスコープとチェインするかどうか
		bool scope_chain;

		bool scope_chain_have_possibilities;

		// 子供のスコープ
		PODArray<Scope*> children;

		// 現在指している子供の位置
		int_t children_pos;

		// 対応する式
		ExprPtr expr;
	};

	struct VariableInfo{
		// スコープの深さ
		int_t depth;

		// そのスコープのメンバー番号
		int_t member_number;

		// レジスタ番号
		int_t register_number;

		// 見つかったかどうか
		bool found;

		// 関数の外側まで探したかどうか
		bool out_of_fun;

		// トップレベルで見つかったかどうか
		bool toplevel;

		// 定数かどうか
		bool constant;

		Scope::Entry* entry;

		bool is_register(){
			return found && !out_of_fun;
		}
	};

	VariableInfo var_find(const IDPtr& key, bool force = false);
	void var_visible(const IDPtr& key, bool visible);
	void var_refere(const IDPtr& key);
	void var_begin(const ExprPtr& e, int_t kind);
	//void var_define_stmts(const ExprPtr& stmts, bool visible);
	//void var_define_stmt(const AnyPtr& stmt, bool visible);
	void var_define(const IDPtr& name, bool visible);
	void var_assign(const IDPtr& name);
	void var_define_class_member(const IDPtr& name, int_t accessibility, bool visible);
	void var_end(const ExprPtr& e);

	int_t calc_variable_offset(Scope& socpe);
	void scope_begin(const ExprPtr& e);
	void scope_end(const ExprPtr& e);
	void scope_skip();
	void scope_chain(int_t var_frame_size);

	void scope_optimize_begin(const ExprPtr& e);
	void scope_optimize_end(const ExprPtr& e);

	PODStack<Scope*> scope_stack_;
	Scope& current_scope(){ return *scope_stack_.top(); }

	void build_scope(const AnyPtr& a);
	void build_scope2(const AnyPtr& a);
	void calc_scope(Scope* scope, Scope* fun_scope, int_t base);
	void optimize_scope(Scope* scope);
	void delete_scope(Scope* scope);
	void normalize(const AnyPtr& a);

	bool scope_exist(Scope* scope);
public:

	FunFrame& ff(){ return *fun_frame_stack_.top(); }
	FunFrame& push_ff();
	void pop_ff();

	ClassFrame& cf(){ return *class_frame_stack_.top(); }
	ClassFrame& push_cf();
	void pop_cf();

	int_t class_info_num(){
		return class_frame_stack_.empty() ? 0 : cf().class_info_num;
	}

private:

	MapPtr value_map_;
	MapPtr identifier_map_;
	MapPtr implicit_ref_map_;
	
	Parser parser_;
	CodePtr result_;
	
	PODStack<FunFrame*> fun_frame_stack_;
	PODStack<ClassFrame*> class_frame_stack_;

	PODStack<int_t> linenos_;

	xpeg::ExecutorPtr eb_;

	int_t prev_inst_op_;

	bool eval_;

private:
	XTAL_DISALLOW_COPY_AND_ASSIGN(CodeBuilder);
};

#define XTAL_FOR_EXPR(var, target) \
	if(const ExprPtr& expr = target)\
	for(uint_t i=0; i<expr->size(); ++i)\
	if(bool not_end = true)\
	for(const ::xtal::AnyPtr& var = expr->at(i); not_end; not_end=false)


}

#endif


#endif // XTAL_CODEBUILDER_H_INCLUDE_GUARD
