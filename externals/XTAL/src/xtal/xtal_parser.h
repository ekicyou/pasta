/** \file src/xtal/xtal_parser.h
* \brief src/xtal/xtal_parser.h
*/

#ifndef XTAL_PARSER_H_INCLUDE_GUARD
#define XTAL_PARSER_H_INCLUDE_GUARD

#pragma once

#ifndef XTAL_NO_PARSER

namespace xtal{

class Token : public ImmediateValue{
public:

	enum Flag{
		FLAG_LEFT_SPACE = 1<<0,
		FLAG_RIGHT_SPACE = 1<<1
	};
	
	enum Type{	
		TYPE_TOKEN = 0,
		TYPE_INT = 1,
		TYPE_FLOAT = 2,
		TYPE_IDENTIFIER = 3,
		TYPE_KEYWORD = 4
	};

	Token()
		:ImmediateValue(0, (int_t)0){}
	
	Token(int_t type, int_t flags, int_t value)
		:ImmediateValue(type | (flags<<4), value){}

	Token(int_t type, int_t flags, float_t value)
		:ImmediateValue(type | (flags<<4), value){}

	int_t type() const{ return first()&0xf; }

	bool left_space() const{ return (((first()>>4)&0xf) & FLAG_LEFT_SPACE) != 0; }
	
	bool right_space() const{ return (((first()>>4)&0xf) & FLAG_RIGHT_SPACE) != 0; }
	
	int_t ivalue() const{ return second_ivalue(); }
	
	float_t fvalue() const{ return second_fvalue(); }
		
	int_t keyword_number() const{ return second_ivalue(); }

	bool is_end(){ return type()==TYPE_TOKEN && ivalue()==0; }
};

template<> struct CppClassSymbol<Token> : public CppClassSymbol<ImmediateValue>{};


class Tokenizer : public xpeg::Executor{
public:
	Tokenizer(const xpeg::ExecutorPtr& e);

	const AnyPtr& peek();

	const AnyPtr& read();

	StringPtr read_string(int_t open, int_t close);

	int save(){
		return token_pos_;
	}

	void load(int pos){
		token_pos_ = pos;
	}
		
private:
	void tokenize();
	void push_token(int_t v);
	void push_int_token(int_t v);
	void push_float_token(float_t v);
	void push_keyword_token(int_t num);
	void push_identifier_token(const IDPtr& identifier);

	void deplete_space();

	float_t read_finteger();
	int_t read_integer(int_t base);
	bool is_integer_literal();
	void tokenize_number();
	
	int_t test_right_space(int_t ch);

public:
	virtual uint_t read_charactors(AnyPtr* buffer, uint_t max);

	void on_visit_members(Visitor& m){
		xpeg::Executor::on_visit_members(m);
		m & keyword_map_ & ms_;
		for(int_t i=0; i<TOKEN_BUF_SIZE; ++i){
			m & token_buf_[i];
		}
	}

private:
	MapPtr keyword_map_;
	MemoryStreamPtr ms_;

	enum{ TOKEN_BUF_SIZE = 8, TOKEN_BUF_MASK = TOKEN_BUF_SIZE-1 };
	AnyPtr token_buf_[TOKEN_BUF_SIZE];
	int_t token_pos_;
	int_t token_read_;

	uint_t left_space_;

	xpeg::ExecutorPtr executor_;
};

class Parser{
public:
	void parse(const xpeg::ExecutorPtr& scanner);

	void parse_eval(const xpeg::ExecutorPtr& scanner);

private:
	void expect(int_t ch);
	bool check(int_t ch);
	bool eat(int_t ch);
	bool eat_keyword(int_t kw);

	bool cmp_pri(int_t pri, int_t op, int_t r_space, int_t l_space);

	void parse_secondary_key();
	bool parse_term();
	bool parse_expr(int_t pri, int_t space);
	bool parse_expr();
	void expect_parse_expr(int_t pri, int_t space);
	bool parse_post(int_t pri, int_t space);
	bool parse_post2(const Token& ch, int_t pri, int_t space);
	void expect_parse_expr();
	void expect_parse_identifier();
	bool parse_identifier();
	void parse_identifier_or_keyword();
	bool parse_var();
	void parse_if();
	void expect_parse_stmt();
	void parse_else_or_nobreak();
	void parse_try();
	bool parse_stmt();
	void parse_stmts();
	void parse_assert();
	void parse_exprs(bool one = false);
	bool parse_assign_stmt();
	void parse_array();
	void parse_each();
	void parse_while();
	void parse_for();
	bool parse_loop();
	void parse_switch();
	void parse_class(int_t kind);
	void parse_class2(int_t kind);
	void parse_scope();
	void parse_lambda(bool noparam = false);
	void parse_fun(int_t kind);
	void parse_fun2(int_t kind);
	void parse_call();		
	void parse_toplevel();

	bool expr_end();
	bool make_bin_expr(const Token& ch, int_t space, int_t pri, int_t PRI, int_t EXPR);
	void expect_stmt_end();
public:
	SmartPtr<Tokenizer> tokenizer_;
	xpeg::ExecutorPtr executor_;

private:
	AnyPtr last_;

	struct State{
		AnyPtr ch;
		int_t pos;
	};

	State save();

	void load(const State& s);

	const Token& read_token();

	const Token& peek_token();
};

}

#endif


#endif // XTAL_PARSER_H_INCLUDE_GUARD
