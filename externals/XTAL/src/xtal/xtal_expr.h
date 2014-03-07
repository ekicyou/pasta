/** \file src/xtal/xtal_expr.h
* \brief src/xtal/xtal_expr.h
*/

#ifndef XTAL_EXPR_H_INCLUDE_GUARD
#define XTAL_EXPR_H_INCLUDE_GUARD

#pragma once

namespace xtal{

enum ExprType{

	EXPR_LIST,
	
	EXPR_NULL,
	EXPR_UNDEFINED,
	EXPR_TRUE,
	EXPR_FALSE,
	EXPR_CALLEE,
	EXPR_THIS,
	EXPR_NUMBER,
	EXPR_STRING,
	EXPR_ARRAY,
	EXPR_MAP,
	EXPR_VALUES,

	EXPR_ADD,
	EXPR_SUB,
	EXPR_CAT,
	EXPR_MUL,
	EXPR_DIV,
	EXPR_MOD,
	EXPR_AND,
	EXPR_OR,
	EXPR_XOR,
	EXPR_SHL,
	EXPR_SHR,
	EXPR_USHR,

	EXPR_EQ,
	EXPR_NE,
	EXPR_LT,
	EXPR_LE,
	EXPR_GT,
	EXPR_GE,
	EXPR_RAWEQ,
	EXPR_RAWNE,
	EXPR_IN,
	EXPR_NIN,
	EXPR_IS,
	EXPR_NIS,

	EXPR_ANDAND,
	EXPR_OROR,

	EXPR_CATCH,

	EXPR_POS,
	EXPR_NEG,
	EXPR_COM,
	EXPR_NOT,

	EXPR_RANGE,

	EXPR_FUN,
	EXPR_CLASS,

	EXPR_ONCE,

	EXPR_IVAR,
	EXPR_LVAR,
	EXPR_AT,
	EXPR_Q,
	EXPR_MEMBER,
	EXPR_MEMBER_Q,
	EXPR_PROPERTY,
	EXPR_PROPERTY_Q,
	EXPR_CALL,
	EXPR_YIELD,

	EXPR_INC,
	EXPR_DEC,

	EXPR_ADD_ASSIGN,
	EXPR_SUB_ASSIGN,
	EXPR_CAT_ASSIGN,
	EXPR_MUL_ASSIGN,
	EXPR_DIV_ASSIGN,
	EXPR_MOD_ASSIGN,
	EXPR_AND_ASSIGN,
	EXPR_OR_ASSIGN,
	EXPR_XOR_ASSIGN,
	EXPR_SHL_ASSIGN,
	EXPR_SHR_ASSIGN,
	EXPR_USHR_ASSIGN,

	EXPR_RETURN,
	EXPR_ASSERT,
	EXPR_THROW,
	EXPR_TRY,
	EXPR_IF,
	EXPR_FOR,
	EXPR_MASSIGN,
	EXPR_MDEFINE,
	EXPR_ASSIGN,
	EXPR_DEFINE,
	EXPR_CDEFINE_MEMBER,
	EXPR_CDEFINE_IVAR,
	EXPR_BREAK,
	EXPR_CONTINUE,
	EXPR_BRACKET,
	EXPR_SCOPE,
	EXPR_SWITCH,
	EXPR_TOPLEVEL,

	EXPR_MAX
};


#define XTAL_DEF_MEMBER(N, Type, Name) \
	Caster<Type>::type Name(){ return unchecked_cast<Type>(at(N)); }\
	const ExprPtr& set_##Name(Type v){ set_at(N, v); return to_smartptr(this); }


typedef TreeNode Expr;
typedef TreeNodePtr ExprPtr;
const ExprPtr& ep(const AnyPtr& a);

/*
* \brief 構文木のノード
*/
class TreeNode : public Array{
public:
	enum{ TYPE = TYPE_TREE_NODE };

	TreeNode();
	
	TreeNode(int_t tag, int_t lineno = 0);
	
	TreeNode(const AnyPtr& tag, int_t lineno = 0);

	const AnyPtr& tag(){
		return tag_;
	}

	int_t itag(){
		return XTAL_detail_ivalue(tag_);
	}

	void set_tag(const AnyPtr& tag){
		tag_ = tag;
	}
	
	int_t lineno(){
		return lineno_;
	}

	void set_lineno(int_t lineno){
		lineno_ = lineno;
	}

	const AnyPtr& at(int_t i);

	void set_at(int_t i, const AnyPtr& v);

	template<class T>
	typename Caster<const T&>::type at_cast(int_t i){
		return cast<const T&>(at(i));
	}

	void on_visit_members(Visitor& m){
		Array::on_visit_members(m);
		m & tag_;
	}

private:
	AnyPtr tag_;
	int_t lineno_;

public:

	XTAL_DEF_MEMBER(0, const ExprPtr&, una_term);

	XTAL_DEF_MEMBER(0, const ExprPtr&, bin_lhs);
	XTAL_DEF_MEMBER(1, const ExprPtr&, bin_rhs);

	XTAL_DEF_MEMBER(0, const ExprPtr&, q_cond);
	XTAL_DEF_MEMBER(1, const ExprPtr&, q_true);
	XTAL_DEF_MEMBER(2, const ExprPtr&, q_false);

	XTAL_DEF_MEMBER(0, const AnyPtr&, number_value);

	XTAL_DEF_MEMBER(0, int_t, string_kind);
	XTAL_DEF_MEMBER(1, const StringPtr&, string_value);

	XTAL_DEF_MEMBER(0, int_t, range_kind);
	XTAL_DEF_MEMBER(1, const ExprPtr&, range_lhs);
	XTAL_DEF_MEMBER(2, const ExprPtr&, range_rhs);

	XTAL_DEF_MEMBER(0, const ExprPtr&, array_values);
	
	XTAL_DEF_MEMBER(0, const ExprPtr&, map_values);

	XTAL_DEF_MEMBER(0, const ExprPtr&, multi_value_exprs);

	XTAL_DEF_MEMBER(0, const ExprPtr&, return_exprs);
	
	XTAL_DEF_MEMBER(0, const ExprPtr&, yield_exprs);

	XTAL_DEF_MEMBER(0, const ExprPtr&, assert_cond);
	XTAL_DEF_MEMBER(1, const ExprPtr&, assert_string);
	XTAL_DEF_MEMBER(2, const ExprPtr&, assert_message);

	XTAL_DEF_MEMBER(0, const ExprPtr&, throw_expr);

	XTAL_DEF_MEMBER(0, const ExprPtr&, try_body);
	XTAL_DEF_MEMBER(1, IDPtr, try_catch_var);
	XTAL_DEF_MEMBER(2, const ExprPtr&, try_catch);
	XTAL_DEF_MEMBER(3, const ExprPtr&, try_finally);

	XTAL_DEF_MEMBER(0, const ExprPtr&, catch_body);
	XTAL_DEF_MEMBER(1, IDPtr, catch_catch_var);
	XTAL_DEF_MEMBER(2, const ExprPtr&, catch_catch);

	XTAL_DEF_MEMBER(0, const ExprPtr&, if_cond);
	XTAL_DEF_MEMBER(1, const ExprPtr&, if_body);
	XTAL_DEF_MEMBER(2, const ExprPtr&, if_else);

	XTAL_DEF_MEMBER(0, IDPtr, for_label);
	XTAL_DEF_MEMBER(1, const ExprPtr&, for_init);
	XTAL_DEF_MEMBER(2, const ExprPtr&, for_cond);
	XTAL_DEF_MEMBER(3, const ExprPtr&, for_next);
	XTAL_DEF_MEMBER(4, const ExprPtr&, for_body);
	XTAL_DEF_MEMBER(5, const ExprPtr&, for_else);
	XTAL_DEF_MEMBER(6, const ExprPtr&, for_nobreak);

	XTAL_DEF_MEMBER(0, int_t, fun_kind);
	XTAL_DEF_MEMBER(1, const ExprPtr&, fun_params);
	XTAL_DEF_MEMBER(2, IDPtr, fun_extendable_param);
	XTAL_DEF_MEMBER(3, const ExprPtr&, fun_body);
	XTAL_DEF_MEMBER(4, IDPtr, fun_name);

	XTAL_DEF_MEMBER(0, const ExprPtr&, massign_lhs_exprs);
	XTAL_DEF_MEMBER(1, const ExprPtr&, massign_rhs_exprs);

	XTAL_DEF_MEMBER(0, const ExprPtr&, mdefine_lhs_exprs);
	XTAL_DEF_MEMBER(1, const ExprPtr&, mdefine_rhs_exprs);

	XTAL_DEF_MEMBER(0, IDPtr, ivar_name);

	XTAL_DEF_MEMBER(0, IDPtr, lvar_name);

	XTAL_DEF_MEMBER(0, const ExprPtr&, member_term);
	XTAL_DEF_MEMBER(1, const AnyPtr&, member_name);
	XTAL_DEF_MEMBER(2, const ExprPtr&, member_ns);

	XTAL_DEF_MEMBER(0, const ExprPtr&, property_term);
	XTAL_DEF_MEMBER(1, const AnyPtr&, property_name);
	XTAL_DEF_MEMBER(2, const ExprPtr&, property_ns);

	XTAL_DEF_MEMBER(0, const ExprPtr&, call_term);
	XTAL_DEF_MEMBER(1, const ExprPtr&, call_args);
	XTAL_DEF_MEMBER(2, const ExprPtr&, call_extendable_arg);

	XTAL_DEF_MEMBER(0, const AnyPtr&, cdefine_member_accessibility);
	XTAL_DEF_MEMBER(1, IDPtr, cdefine_member_name);
	XTAL_DEF_MEMBER(2, const ExprPtr&, cdefine_member_ns);
	XTAL_DEF_MEMBER(3, const ExprPtr&, cdefine_member_term);

	XTAL_DEF_MEMBER(0, const AnyPtr&, cdefine_ivar_accessibility);
	XTAL_DEF_MEMBER(1, IDPtr, cdefine_ivar_name);
	XTAL_DEF_MEMBER(2, const ExprPtr&, cdefine_ivar_term);

	XTAL_DEF_MEMBER(0, IDPtr, break_label);

	XTAL_DEF_MEMBER(0, IDPtr, continue_label);

	XTAL_DEF_MEMBER(0, int_t, class_kind);
	XTAL_DEF_MEMBER(1, const ExprPtr&, class_mixins);
	XTAL_DEF_MEMBER(2, const ExprPtr&, class_stmts);
	XTAL_DEF_MEMBER(3, IDPtr, class_name);

	XTAL_DEF_MEMBER(0, const ExprPtr&, switch_cond);
	XTAL_DEF_MEMBER(1, const ExprPtr&, switch_cases);
	XTAL_DEF_MEMBER(2, const ExprPtr&, switch_default);

	XTAL_DEF_MEMBER(0, const ExprPtr&, scope_stmts);

	XTAL_DEF_MEMBER(0, const ExprPtr&, toplevel_stmts);
};

}


#endif // XTAL_EXPR_H_INCLUDE_GUARD
