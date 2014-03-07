#include "xtal.h"
#include "xtal_macro.h"

#include "xtal_codebuilder.h"

#ifndef XTAL_NO_PARSER	

namespace xtal{

void CodeBuilder::compile_lassign(int_t target, const IDPtr& var){
	VariableInfo varinfo = var_find(var);
	if(varinfo.found){
		if(varinfo.constant){
			error(Xt("XCE1019")->call(Named(Xid(name), var)));
		}

		if(varinfo.is_register()){
			put_inst<InstCopy>(varinfo.register_number, target);
		}
		else{
			put_inst<InstSetLocalVariable>(target, varinfo.member_number, varinfo.depth);
		}
	}
	else{
		if(varinfo.toplevel){
			put_inst<InstSetFilelocalVariable>(target, varinfo.member_number);
		}
		else{
			if(eval_){
				put_inst<InstSetFilelocalVariableByName>(target, register_identifier(var));
			}
			else{
				error(Xt("XCE1009")->call(Named(Xid(name), var)));
			}
		}
	}	
}

int_t CodeBuilder::compile_bin2(const ExprPtr& e, int_t stack_top, int_t result){
	int_t lhs = compile_expr(e->bin_lhs(), stack_top);
	int_t rhs = compile_expr(e->bin_rhs(), stack_top);
	put_bin(result, e, lhs, rhs, stack_top);
	return 1;
}

int_t CodeBuilder::compile_comp_bin2(const ExprPtr& e, int_t stack_top, int_t result){
	if(is_comp_bin(e->bin_lhs()) || is_comp_bin(e->bin_rhs())){
		error(Xt("XCE1025"));
	}

	int_t label_false = reserve_label();
	put_if_code(e, label_false, stack_top);

	int_t label_true = reserve_label();
	put_inst<InstLoadValue>(result, LOAD_TRUE);

	set_jump(InstGoto::OFFSET_address, label_true);
	put_inst<InstGoto>(0);

	set_label(label_false);

	put_inst<InstLoadValue>(result, LOAD_FALSE);

	set_label(label_true);

	return 1;
}

void CodeBuilder::put_if_code(int_t target, int_t label_true, int_t label_false){
	set_jump(InstIf::OFFSET_address_true, label_true);
	set_jump(InstIf::OFFSET_address_false, label_false);
	put_inst<InstIf>(target, 0, 0);
}

void CodeBuilder::put_if_code(int_t tag, int_t target, int_t lhs, int_t rhs, int_t label_true, int_t label_false, int_t stack_top){

	switch(tag){
		XTAL_NODEFAULT;

		XTAL_CASE(EXPR_EQ){
			put_inst<InstIfEq>(lhs, rhs, stack_top);
			put_if_code(target, label_true, label_false);
		}

		XTAL_CASE(EXPR_NE){
			put_inst<InstIfEq>(lhs, rhs, stack_top);
			put_if_code(target, label_false, label_true);
		}

		XTAL_CASE(EXPR_LT){
			put_inst<InstIfLt>(lhs, rhs, stack_top);
			put_if_code(target, label_true, label_false);
		}

		XTAL_CASE(EXPR_GT){
			put_inst<InstIfLt>(rhs, lhs, stack_top);
			put_if_code(target, label_true, label_false);
		}

		XTAL_CASE(EXPR_GE){
			put_inst<InstIfLt>(lhs, rhs, stack_top);
			put_if_code(target, label_false, label_true);
		}

		XTAL_CASE(EXPR_LE){
			put_inst<InstIfLt>(rhs, lhs, stack_top);
			put_if_code(target, label_false, label_true);
		}

		XTAL_CASE(EXPR_RAWEQ){
			put_inst<InstIfRawEq>(lhs, rhs, stack_top);
			put_if_code(target, label_true, label_false);
		}

		XTAL_CASE(EXPR_RAWNE){
			put_inst<InstIfRawEq>(lhs, rhs, stack_top);
			put_if_code(target, label_false, label_true);
		}

		XTAL_CASE(EXPR_IN){
			put_inst<InstIfIn>(lhs, rhs, stack_top);
			put_if_code(target, label_true, label_false);
		}

		XTAL_CASE(EXPR_NIN){
			put_inst<InstIfIn>(lhs, rhs, stack_top);
			put_if_code(target, label_false, label_true);
		}

		XTAL_CASE(EXPR_IS){
			put_inst<InstIfIs>(lhs, rhs, stack_top);
			put_if_code(target, label_true, label_false);
		}

		XTAL_CASE(EXPR_NIS){
			put_inst<InstIfIs>(lhs, rhs, stack_top);
			put_if_code(target, label_false, label_true);
		}
	}
}

void CodeBuilder::put_if_code(const ExprPtr& e, int_t label_false, int_t stack_top){
	int_t label_true = reserve_label();
	put_if_code(e, label_true, label_false, stack_top);
	set_label(label_true);
}

void CodeBuilder::put_if_code(const ExprPtr& e, int_t label_true, int_t label_false, int_t stack_top){
	if(e->itag()==EXPR_AND){
		error(Xt("XCE1029"));
	}

	if(is_comp_bin(e)){
		int_t target = stack_top++;
		int_t lhs = compile_expr(e->bin_lhs(), stack_top);
		int_t rhs = compile_expr(e->bin_rhs(), stack_top);
		put_if_code(e->itag(), target, lhs, rhs, label_true, label_false, stack_top);
	}
	else{
		if(e->itag()==EXPR_NOT){
			int_t lhs = compile_expr(e->una_term(), stack_top);
			put_if_code(lhs, label_false, label_true);
		}
		else{
			int_t lhs = compile_expr(e, stack_top);
			put_if_code(lhs, label_true, label_false);
		}
	}
}
	

int_t CodeBuilder::compile_member(const AnyPtr& eterm, const AnyPtr& eprimary, const AnyPtr& esecondary, int_t flags, int_t stack_top, int_t result){
	int_t target = compile_expr(eterm, stack_top);
	int_t primary = 0;
	int_t secondary = 0;

	if(IDPtr id = ptr_cast<ID>(eprimary)){
		primary = register_identifier(id);
	}
	else{
		primary = stack_top++; compile_expr(eprimary, stack_top, primary);
		compile_property(primary, Xid(to_s), null, 0, stack_top, primary, 1);
		compile_property(primary, Xid(intern), null, 0, stack_top, primary, 1);
		flags |= MEMBER_FLAG_P_BIT;
	}

	if(esecondary){
		secondary = compile_expr(esecondary, stack_top);
		flags |= MEMBER_FLAG_S_BIT;
	}

	if(flags==0){
		put_inst<InstMember>(result, target, primary);
	}
	else{
		put_inst<InstMemberEx>(result, target, primary, secondary, flags);
	}

	return 1;
}

int_t CodeBuilder::compile_property(const AnyPtr& eterm, const AnyPtr& eprimary, const AnyPtr& esecondary, int_t flags, int_t stack_top, int_t result, int_t result_count){
	return compile_send(eterm, eprimary, esecondary, nul<Expr>(), nul<Expr>(), flags, stack_top, result, result_count);
}	

int_t CodeBuilder::compile_set_property(const AnyPtr& eterm, const AnyPtr& eprimary, const AnyPtr& esecondary, const AnyPtr& set, int_t flags, int_t stack_top, int_t result, int_t result_count){
	eb_->tree_push_back(null);
	eb_->tree_push_back(set);
	eb_->tree_splice(EXPR_LIST, 2);
	eb_->tree_splice(EXPR_LIST, 1);
	ExprPtr args = ep(eb_->tree_pop_back());

	if(ptr_cast<Expr>(eprimary)){ 
		eb_->tree_push_back(KIND_STRING);
		eb_->tree_push_back(Xid(set_));
		eb_->tree_splice(EXPR_STRING, 2);
		eb_->tree_push_back(ep(eprimary));
		eb_->tree_splice(EXPR_CAT, 2);
		return compile_send(eterm, eb_->tree_pop_back(), esecondary, args, nul<Expr>(), flags, stack_top, result, result_count);
	}
	else{
		return compile_send(eterm, Xid(set_)->cat(ptr_cast<ID>(eprimary))->intern(), esecondary, args, nul<Expr>(), flags, stack_top, result, result_count);
	}
}

int_t CodeBuilder::compile_send(const AnyPtr& eterm, const AnyPtr& eprimary, const AnyPtr& esecondary, const ExprPtr& args, const ExprPtr& eargs, int_t flags, int_t stack_top, int_t result, int_t result_count){
	int_t target = compile_expr(eterm, stack_top);
	int_t primary = 0;
	int_t secondary = 0;

	if(IDPtr id = ptr_cast<ID>(eprimary)){
		primary = register_identifier(id);
	}
	else{
		primary = stack_top++; compile_expr(eprimary, stack_top, primary);
		compile_property(primary, Xid(to_s), null, 0, stack_top, primary, 1);
		compile_property(primary, Xid(intern), null, 0, stack_top, primary, 1);
		flags |= MEMBER_FLAG_P_BIT;
	}

	if(esecondary){
		secondary = compile_expr(esecondary, stack_top);
		flags |= MEMBER_FLAG_S_BIT;
	}

	int_t stack_base = stack_top;
	int_t ordered = 0;
	int_t named = 0;

	if(args){
		if(args->length()==1){
			ExprPtr a1 = ep(args->at(0));
			if(!a1->at(0) && XTAL_detail_is_ivalue(a1->at(1)) && XTAL_detail_ivalue(a1->at(1))>=0){
				stack_base = XTAL_detail_ivalue(a1->at(1));
				ordered = 1;
			}
		}
		
		if(ordered==0){
			Xfor_cast(ExprPtr v, args){
				if(v->at(0)){
					named++;
				}
				else{
					if(named!=0){
						error(Xt("XCE1005"));
					}

					ordered++;

					compile_expr(v->at(1), stack_top+1, stack_top);
					stack_top++;
				}
			}
			
			Xfor_cast(ExprPtr v, args){
				if(v->at(0)){
					const ExprPtr& k = ptr_cast<Expr>(v->at(0));

					put_inst<InstLoadConstant>(stack_top++, register_value(k->lvar_name()));
					compile_expr(v->at(1), stack_top+1, stack_top);
					stack_top++;
				}
			}
		}
	}

	if(eargs){
		compile_expr(eargs, stack_top+1, stack_top);
		flags |= CALL_FLAG_ARGS_BIT;
	}

	if(flags==0 && named==0){
		if(result_count==1 && ordered==0){
			put_inst<InstProperty>(result, primary, target, stack_base);
			return 1;
		}

		if(result_count==0 && ordered==1){
			put_inst<InstSetProperty>(primary, target, stack_base);
			return 0;
		}
	}

	if(named==0 && flags==0){
		put_inst<InstSend>(result, result_count, target, primary, secondary, stack_base, ordered);
	}
	else{
		put_inst<InstSendEx>(result, result_count, target, primary, secondary, stack_base, ordered, named, flags);
	}
	return result_count;
}

int_t CodeBuilder::compile_call(int_t target, int_t self, const ExprPtr& args, const ExprPtr& eargs, int_t flags, int_t stack_top, int_t result, int_t result_count){
	int_t stack_base = stack_top;
	int_t ordered = 0;
	int_t named = 0;

	if(args){
		Xfor_cast(ExprPtr v, args){
			if(v->at(0)){
				named++;
			}
			else{
				if(named!=0){
					error(Xt("XCE1005"));
				}

				ordered++;

				compile_expr(ep(v->at(1)), stack_top+1, stack_top);
				stack_top++;
			}
		}
		
		Xfor_cast(ExprPtr v, args){
			if(v->at(0)){
				const ExprPtr& k = ptr_cast<Expr>(v->at(0));

				put_inst<InstLoadConstant>(stack_top++, register_value(k->lvar_name()));
				compile_expr(ep(v->at(1)), stack_top+1, stack_top);
				stack_top++;
			}
		}
	}

	if(eargs){
		compile_expr(eargs, stack_top+1, stack_top);
		flags |= CALL_FLAG_ARGS_BIT;
	}
		
	if(named==0 && flags==0){
		put_inst<InstCall>(result, result_count, target, stack_base, ordered);
	}
	else{
		put_inst<InstCallEx>(result, result_count, target, self, stack_base, ordered, named, flags);
	}

	return result_count;
}

int_t CodeBuilder::compile_expr(const AnyPtr& p, int_t& stack_top){
	if(XTAL_detail_is_ivalue(p)){
		return XTAL_detail_ivalue(p);
	}

	if(ep(p)->itag()==EXPR_LVAR){
		VariableInfo var = var_find(ep(p)->lvar_name());
		if(var.is_register()){
			return var.register_number;
		}
	}
	
	compile_expr(p, stack_top+1, stack_top);

	return stack_top++;
}

int_t CodeBuilder::compile_expr(const AnyPtr& p, int_t stack_top, int_t result, int_t result_count){
	if(XTAL_detail_is_ivalue(p)){
		put_inst<InstCopy>(result, XTAL_detail_ivalue(p));
		return 1;
	}

	ExprPtr e = ep(p);

	int_t temp = stack_top;
	int_t ret = compile_e(e, temp, result, result_count);

	if(result_count!=0 && ret!=result_count){
		put_inst<InstAdjustValues>(result, ret, result_count);
	}

	return ret;
}
	
void CodeBuilder::compile_stmt(const AnyPtr& p){
	if(!p){
		return;
	}

	ExprPtr e = ep(p);

	linenos_.push(e->lineno());

	if(debug::is_debug_compile_enabled() && e->lineno()!=0){
		if(result_->set_lineno_info(e->lineno())){
			put_inst<InstLine>();
		}
	}
	else{
		if(e->lineno()!=0){
			result_->set_lineno_info(e->lineno());
		}
	}

	compile_e(e, 0, 0, 0);

	//if(debug::is_debug_compile_enabled() && e->lineno()!=0){
	if(e->lineno()!=0){
		result_->set_lineno_info(e->lineno());
	}

	linenos_.pop();
}

int_t CodeBuilder::compile_e(const ExprPtr& e, int_t stack_top, int_t result, int_t result_count){
	if(!e){
		error(Xt("XCE1001"));
		return 0;
	}

	switch(e->itag()){

		XTAL_DEFAULT{}

		XTAL_CASE_N(case EXPR_LIST:){
			//compile_expr(e, 0);
			return int_t(0);
		}

		XTAL_CASE_N(case EXPR_NULL:){
			put_inst<InstLoadValue>(result, LOAD_NULL);
			return 1;
		}

		XTAL_CASE_N(case EXPR_UNDEFINED:){
			put_inst<InstLoadValue>(result, LOAD_UNDEFINED);
			return 1;
		}

		XTAL_CASE_N(case EXPR_TRUE:){
			put_inst<InstLoadValue>(result, LOAD_TRUE);
			return 1;
		}

		XTAL_CASE_N(case EXPR_FALSE:){
			put_inst<InstLoadValue>(result, LOAD_FALSE);
			return 1;
		}

		XTAL_CASE_N(case EXPR_CALLEE:){
			put_inst<InstLoadCallee>(result);
			return 1;
		}

		XTAL_CASE_N(case EXPR_THIS:){
			put_inst<InstLoadThis>(result);
			return 1;
		}

		XTAL_CASE_N(case EXPR_NUMBER:){
			AnyPtr nvalue = e->number_value();
			if(XTAL_detail_is_ivalue(nvalue)){
				int_t value = XTAL_detail_ivalue(nvalue);
				if(value==(i8)value){ 
					put_inst<InstLoadInt1Byte>(result, value);
				}
				else{ 
					put_inst<InstLoadConstant>(result, register_value(value));
				}
			}
			else{
				float_t value = XTAL_detail_fvalue(nvalue);
				if(value==(i8)value){ 
					put_inst<InstLoadFloat1Byte>(result, (i8)value);
				}
				else{ 
					put_inst<InstLoadConstant>(result, register_value(value));
				}
			}

			return 1;
		}

		XTAL_CASE_N(case EXPR_STRING:){
			if(e->string_kind()==KIND_TEXT){
				put_inst<InstLoadConstant>(result, register_value(text(e->string_value()->to_s())));
			}
			else if(e->string_kind()==KIND_FORMAT){
				put_inst<InstLoadConstant>(result, register_value(format(e->string_value()->to_s())));
			}
			else{
				put_inst<InstLoadConstant>(result, register_value(e->string_value()));
			}

			return 1;
		}

		XTAL_CASE_N(case EXPR_ARRAY:){
			if(result_count!=0){
				put_inst<InstMakeArray>(result);
				XTAL_FOR_EXPR(v, e->array_values()){
					compile_expr(v, stack_top+1, stack_top);
					put_inst<InstArrayAppend>(result, stack_top);
				}
				return 1;
			}
			else{
				XTAL_FOR_EXPR(v, e->array_values()){
					compile_expr(v, stack_top+1, stack_top);
				}
				return 0;
			}
		}

		XTAL_CASE_N(case EXPR_MAP:){
			if(result_count!=0){
				put_inst<InstMakeMap>(result);
				Xfor_cast(ArrayPtr v, e->map_values()){
					compile_expr(v->at(0), stack_top+1, stack_top);
					compile_expr(v->at(1), stack_top+2, stack_top+1);
					put_inst<InstMapInsert>(result, stack_top, stack_top+1);
				}
				return 1;
			}
			else{
				Xfor_cast(ArrayPtr v, e->map_values()){
					compile_expr(v->at(0), stack_top+1, stack_top);
					compile_expr(v->at(1), stack_top+2, stack_top+1);
				}
				return 0;
			}
		}

		XTAL_CASE_N(case EXPR_VALUES:){
			if(e->multi_value_exprs()){
				int_t stack_base = stack_top;

				XTAL_FOR_EXPR(v, e->multi_value_exprs()){
					compile_expr(v, stack_top);	
				}

				if(e->multi_value_exprs()->size()!=(uint_t)result_count){
					put_inst<InstAdjustValues>(stack_base, e->multi_value_exprs()->size(), 1);
				}

				put_inst<InstCopy>(result, stack_base);
				return 1;
			}
			put_inst<InstLoadValue>(result, LOAD_UNDEFINED);
			return 1;
		}

		XTAL_CASE_N(
			case EXPR_ADD: case EXPR_SUB: case EXPR_CAT:
			case EXPR_MUL: case EXPR_DIV: case EXPR_MOD:
			case EXPR_AND: case EXPR_OR: case EXPR_XOR:
			case EXPR_SHL: case EXPR_SHR: case EXPR_USHR:
		){
			return compile_bin2(e, stack_top, result);
		}

		XTAL_CASE_N(
			case EXPR_EQ: case EXPR_NE:
			case EXPR_LT: case EXPR_LE:
			case EXPR_GT: case EXPR_GE:
			case EXPR_RAWEQ: case EXPR_RAWNE:
			case EXPR_IN: case EXPR_NIN:
			case EXPR_IS: case EXPR_NIS:
		){
			return compile_comp_bin2(e, stack_top, result);
		}

		XTAL_CASE_N(case EXPR_ANDAND:){
			int_t label_true = reserve_label();
			int_t label_false = reserve_label();

			if(e->bin_lhs()->itag()==EXPR_AND){
				error(Xt("XCE1029"));
			}

			compile_expr(e->bin_lhs(), stack_top, result);
			
			set_jump(InstIf::OFFSET_address_true, label_true);
			set_jump(InstIf::OFFSET_address_false, label_false);
			put_inst<InstIf>(result, 0, 0);
			set_label(label_true);
			
			compile_expr(e->bin_rhs(), stack_top, result);
			
			set_label(label_false);
			return 1;
		}

		XTAL_CASE_N(case EXPR_OROR:){
			int_t label_true = reserve_label();
			int_t label_false = reserve_label();

			if(e->bin_lhs()->itag()==EXPR_AND){
				error(Xt("XCE1029"));
			}

			compile_expr(e->bin_lhs(), stack_top, result);
			
			set_jump(InstIf::OFFSET_address_true, label_true);
			set_jump(InstIf::OFFSET_address_false, label_false);
			put_inst<InstIf>(result, 0, 0);
			set_label(label_false);
			
			compile_expr(e->bin_rhs(), stack_top, result);
				
			set_label(label_true);
			return 1;
		}

		XTAL_CASE_N(case EXPR_CATCH:){
			int_t end_label = reserve_label();

			int_t n = result_->except_info_table_.size();
			result_->except_info_table_.push_back(ExceptInfo());
			put_inst<InstTryBegin>(n);

			compile_expr(e->catch_body(), stack_top, result);

			put_inst<InstTryEnd>();

			// catch節のコードを埋め込む
			{
				result_->except_info_table_[n].catch_pc = code_size();

				// 例外を受け取るために変数スコープを構築
				scope_begin(e);

				put_inst<InstPop>(stack_top);
				compile_lassign(stack_top, e->catch_catch_var());

				put_inst<InstPop>(stack_top);
				compile_expr(e->catch_catch(), stack_top+1, stack_top);
				put_inst<InstPush>(stack_top);
				
				scope_end(e);

				put_inst<InstPop>(result);
			}
				
			set_label(end_label);
			result_->except_info_table_[n].finally_pc = code_size();
			result_->except_info_table_[n].end_pc = code_size();
			return 1;
		}

		XTAL_CASE_N(case EXPR_POS:){
			compile_expr(e->una_term(), stack_top, result);
			put_inst<InstPos>(result, result, stack_top);
			return 1;
		}

		XTAL_CASE_N(case EXPR_NEG:){
			compile_expr(e->una_term(), stack_top, result);
			put_inst<InstNeg>(result, result, stack_top);
			return 1;
		}

		XTAL_CASE_N(case EXPR_COM:){
			compile_expr(e->una_term(), stack_top, result);
			put_inst<InstCom>(result, result, stack_top);
			return 1;
		}

		XTAL_CASE_N(case EXPR_NOT:){
			compile_expr(e->una_term(), stack_top, result);
			put_inst<InstNot>(result, result);
			return 1;
		}

		XTAL_CASE_N(case EXPR_RANGE:){
			int_t lhs = stack_top++; compile_expr(e->range_lhs(), stack_top, lhs);
			int_t rhs = stack_top++; compile_expr(e->range_rhs(), stack_top, rhs);
			put_inst<InstRange>(result, (int_t)e->range_kind(), lhs, rhs, stack_top);
			return 1;
		}

		XTAL_CASE_N(case EXPR_FUN:){
			return compile_fun(e, stack_top, result);
		}

		XTAL_CASE_N(case EXPR_CLASS:){
			compile_class(e, stack_top, result);
			return 1;
		}

		XTAL_CASE_N(case EXPR_ONCE:){
			int_t label_end = reserve_label();
			
			set_jump(InstOnce::OFFSET_address, label_end);
			int_t num = result_->once_table_.size();
			result_->once_table_.push_back(undefined);
			put_inst<InstOnce>(result, 0, num);
						
			compile_expr(e->una_term(), stack_top, result);
			put_inst<InstSetOnce>(result, num);
			
			set_label(label_end);

			return 1;
		}

		XTAL_CASE_N(case EXPR_IVAR:){
			put_instance_variable_code(result, e->ivar_name());
			return 1;
		}

		XTAL_CASE_N(case EXPR_LVAR:){
			VariableInfo var = var_find(e->lvar_name());
			if(var.found){
				if(var.is_register()){
					put_inst<InstCopy>(result, var.register_number);
				}
				else{
					put_inst<InstLocalVariable>(result, var.member_number, var.depth);
				}
			}
			else{
				if(var.toplevel){
					put_inst<InstFilelocalVariable>(result, var.member_number);
				}
				else{
					int_t id = register_identifier(e->lvar_name());
					put_inst<InstFilelocalVariableByName>(result, id);
					implicit_ref_map_->set_at(id, lineno());
				}
			}

			return 1;
		}

		XTAL_CASE_N(case EXPR_AT:){
			int_t lhs = compile_expr(e->bin_lhs(), stack_top);
			int_t rhs = compile_expr(e->bin_rhs(), stack_top);
			put_inst<InstAt>(result, lhs, rhs, stack_top);

			return 1;
		}

		XTAL_CASE_N(case EXPR_Q:){
			int_t label_true = reserve_label();
			int_t label_false = reserve_label();
			int_t label_end = reserve_label();

			compile_expr(e->q_cond(), stack_top, result);

			set_jump(InstIf::OFFSET_address_true, label_true);
			set_jump(InstIf::OFFSET_address_false, label_false);
			put_inst<InstIf>(result, 0, 0);

			set_label(label_true);	

			compile_expr(e->q_true(), stack_top, result);

			set_jump(InstGoto::OFFSET_address, label_end);
			put_inst<InstGoto>(0);

			set_label(label_false);
			
			compile_expr(e->q_false(), stack_top, result);
			
			set_label(label_end);
			return 1;
		}

		XTAL_CASE_N(case EXPR_MEMBER:){
			return compile_member(e->member_term(), e->member_name(), e->member_ns(), 0, stack_top, result);
		}

		XTAL_CASE_N(case EXPR_MEMBER_Q:){
			return compile_member(e->member_term(), e->member_name(), e->member_ns(), MEMBER_FLAG_Q_BIT, stack_top, result);
		}

		XTAL_CASE_N(case EXPR_PROPERTY_Q:){
			return compile_property(e->property_term(), e->property_name(), e->property_ns(), MEMBER_FLAG_Q_BIT, stack_top, result, result_count);
		}

		XTAL_CASE_N(case EXPR_PROPERTY:){
			return compile_property(e->property_term(), e->property_name(), e->property_ns(), 0, stack_top, result, result_count);
		}

		XTAL_CASE_N(case EXPR_CALL:){
			ExprPtr cterm = e->call_term();

			if(cterm->itag()==EXPR_PROPERTY){
				return compile_send(cterm->property_term(), cterm->property_name(), cterm->property_ns(), e->call_args(), e->call_extendable_arg(), 0, stack_top, result, result_count);
			}
			else if(cterm->itag()==EXPR_PROPERTY_Q){
				return compile_send(cterm->property_term(), cterm->property_name(), cterm->property_ns(), e->call_args(), e->call_extendable_arg(), MEMBER_FLAG_Q_BIT, stack_top, result, result_count);
			}
			else{
				int_t target = compile_expr(e->call_term(), stack_top);
				return compile_call(target, 0, e->call_args(), e->call_extendable_arg(), 0, stack_top, result, result_count);
			}
		}

		XTAL_CASE_N(case EXPR_YIELD:){
			int_t count = 0;
			int_t base = stack_top;
			XTAL_FOR_EXPR(v, e->yield_exprs()){
				compile_expr(v, stack_top+1, stack_top);
				count++;
				stack_top++;
			}
				
			put_inst<InstYield>(result, result_count, base, count);
			return result_count;
		}

		XTAL_CASE_N(case EXPR_RETURN:){
			int_t count = 0;
			int_t base = stack_top;
			XTAL_FOR_EXPR(v, e->return_exprs()){
				compile_expr(v, stack_top+1, stack_top);
				count++;
				stack_top++;
			}

			break_off(ff().var_frame_count+1);

			if(ff().return_value<0){
				ff().return_value = count;
			}

			put_inst<InstReturn>(base, count);
			if(count>=256){
				error(Xt("XCE1022"));
			}	
			
			return 0;
		}

		XTAL_CASE_N(case EXPR_INC:){
			compile_incdec(e, stack_top);
			return 0;
		}

		XTAL_CASE_N(case EXPR_DEC:){
			compile_incdec(e, stack_top);
			return 0;
		}

		XTAL_CASE_N(case EXPR_ADD_ASSIGN:){
			compile_op_assign(e, stack_top);
			return 0;
		}

		XTAL_CASE_N(case EXPR_SUB_ASSIGN:){
			compile_op_assign(e, stack_top);
			return 0;
		}

		XTAL_CASE_N(case EXPR_CAT_ASSIGN:){
			compile_op_assign(e, stack_top);
			return 0;
		}

		XTAL_CASE_N(case EXPR_MUL_ASSIGN:){
			compile_op_assign(e, stack_top);
			return 0;
		}

		XTAL_CASE_N(case EXPR_DIV_ASSIGN:){
			compile_op_assign(e, stack_top);
			return 0;
		}

		XTAL_CASE_N(case EXPR_MOD_ASSIGN:){
			compile_op_assign(e, stack_top);
			return 0;
		}

		XTAL_CASE_N(case EXPR_AND_ASSIGN:){
			compile_op_assign(e, stack_top);
			return 0;
		}

		XTAL_CASE_N(case EXPR_OR_ASSIGN:){
			compile_op_assign(e, stack_top);
			return 0;
		}

		XTAL_CASE_N(case EXPR_XOR_ASSIGN:){
			compile_op_assign(e, stack_top);
			return 0;
		}

		XTAL_CASE_N(case EXPR_SHL_ASSIGN:){
			compile_op_assign(e, stack_top);
			return 0;
		}

		XTAL_CASE_N(case EXPR_SHR_ASSIGN:){
			compile_op_assign(e, stack_top);
			return 0;
		}

		XTAL_CASE_N(case EXPR_USHR_ASSIGN:){
			compile_op_assign(e, stack_top);
			return 0;
		}

		XTAL_CASE_N(case EXPR_ASSERT:){
			int_t label_end = reserve_label();

			set_jump(InstIfDebug::OFFSET_address, label_end);
			put_inst<InstIfDebug>(0);

			if(ExprPtr e2 = e->assert_cond()){
				switch(e2->itag()){
				XTAL_DEFAULT{
					int_t target = compile_expr(e2, stack_top);

					int_t label_true = reserve_label();
					set_jump(InstIf::OFFSET_address_false, label_true);
					set_jump(InstIf::OFFSET_address_true, label_end);
					put_inst<InstIf>(target, 0, 0);

					set_label(label_true);

					int_t vart = stack_top++;
					put_inst<InstLoadConstant>(vart, register_value(Xs("%s : %s")));

					int_t strt = stack_top++;
					if(e->assert_string()){ compile_expr(e->assert_string(), stack_top, strt); }
					else{ put_inst<InstLoadConstant>(strt, register_value(empty_string)); }
					
					int_t mest = stack_top++;
					if(e->assert_message()){ compile_expr(e->assert_message(), stack_top, mest); }
					else{ put_inst<InstLoadConstant>(mest, register_value(empty_string)); }

					put_inst<InstCallEx>(target, 1, vart, 0, vart+1, 2, 0, 0);
					put_inst<InstAssert>(target);
				}

				XTAL_CASE(EXPR_EQ){ compile_comp_bin_assert(Xs("%s : ![%s == %s] : %s"), e2, e->assert_string(), e->assert_message(), label_end, stack_top); }
				XTAL_CASE(EXPR_NE){ compile_comp_bin_assert(Xs("%s : ![%s !=  %s] : %s"), e2, e->assert_string(), e->assert_message(), label_end, stack_top); }
				XTAL_CASE(EXPR_LT){ compile_comp_bin_assert(Xs("%s : ![%s <  %s] : %s"), e2, e->assert_string(), e->assert_message(), label_end, stack_top); }
				XTAL_CASE(EXPR_GT){ compile_comp_bin_assert(Xs("%s : ![%s >  %s] : %s"), e2, e->assert_string(), e->assert_message(), label_end, stack_top); }
				XTAL_CASE(EXPR_LE){ compile_comp_bin_assert(Xs("%s : ![%s <=  %s] : %s"), e2, e->assert_string(), e->assert_message(), label_end, stack_top); }
				XTAL_CASE(EXPR_GE){ compile_comp_bin_assert(Xs("%s : ![%s >=  %s] : %s"), e2, e->assert_string(), e->assert_message(), label_end, stack_top); }
				XTAL_CASE(EXPR_RAWEQ){ compile_comp_bin_assert(Xs("%s : ![%s ===  %s] : %s"), e2, e->assert_string(), e->assert_message(), label_end, stack_top); }
				XTAL_CASE(EXPR_RAWNE){ compile_comp_bin_assert(Xs("%s : ![%s !==  %s] : %s"), e2, e->assert_string(), e->assert_message(), label_end, stack_top); }
				XTAL_CASE(EXPR_IN){ compile_comp_bin_assert(Xs("%s : ![%s in  %s] : %s"), e2, e->assert_string(), e->assert_message(), label_end, stack_top); }
				XTAL_CASE(EXPR_NIN){ compile_comp_bin_assert(Xs("%s : ![%s !in  %s] : %s"), e2, e->assert_string(), e->assert_message(), label_end, stack_top); }
				XTAL_CASE(EXPR_IS){ compile_comp_bin_assert(Xs("%s : ![%s is  %s] : %s"), e2, e->assert_string(), e->assert_message(), label_end, stack_top); }
				XTAL_CASE(EXPR_NIS){ compile_comp_bin_assert(Xs("%s : ![%s !is  %s] : %s"), e2, e->assert_string(), e->assert_message(), label_end, stack_top); }
				}
			}
			else{		
				int_t target = stack_top++;

				int_t vart = stack_top++;
				put_inst<InstLoadConstant>(vart, register_value(Xs("%s : %s")));

				int_t strt = stack_top++;
				if(e->assert_string()){ compile_expr(e->assert_string(), stack_top, strt); }
				else{ put_inst<InstLoadConstant>(strt, register_value(empty_string)); }
				
				int_t mest = stack_top++;
				if(e->assert_message()){ compile_expr(e->assert_message(), stack_top, mest); }
				else{ put_inst<InstLoadConstant>(mest, register_value(empty_string)); }

				put_inst<InstCallEx>(target, 1, vart, 0, vart+1, 2, 0, 0);
				put_inst<InstAssert>(target);
			}

			set_label(label_end);
			
			return 0;
		}

		XTAL_CASE_N(case EXPR_THROW:){
			compile_expr(e->una_term(), stack_top, result);
			put_inst<InstPush>(result);
			put_inst<InstThrow>();
			return 0;
		}

		XTAL_CASE_N(case EXPR_TRY:){
			int_t finally_label = reserve_label();
			int_t end_label = reserve_label();

			int_t n = result_->except_info_table_.size();
			result_->except_info_table_.push_back(ExceptInfo());
			put_inst<InstTryBegin>(n);

			CodeBuilder::FunFrame::Finally exc;
			exc.frame_count = scope_stack_.size();
			exc.finally_label = finally_label;
			ff().finallies.push(exc);

			compile_stmt(e->try_body());
			
			set_jump(InstPushGoto::OFFSET_address, end_label);
			put_inst<InstPushGoto>(0);
			put_inst<InstTryEnd>();

			// catch節のコードを埋め込む
			if(e->try_catch()){
				result_->except_info_table_[n].catch_pc = code_size();
				
				// catch節の中での例外に備え、例外フレームを構築。

				int_t n2 = result_->except_info_table_.size();
				result_->except_info_table_.push_back(ExceptInfo());
				put_inst<InstTryBegin>(n2);

				CodeBuilder::FunFrame::Finally exc;
				exc.frame_count = scope_stack_.size();
				exc.finally_label = finally_label;
				ff().finallies.push(exc);

				// 例外を受け取るために変数スコープを構築
				scope_begin(e);
				scope_chain(1);

				put_inst<InstPop>(stack_top);
				compile_lassign(stack_top, e->try_catch_var());
				compile_stmt(e->try_catch());
				
				scope_end(e);

				put_inst<InstTryEnd>();
				ff().finallies.pop();

				result_->except_info_table_[n2].finally_pc = code_size();
				result_->except_info_table_[n2].end_pc = code_size();
			}
			
			set_label(finally_label);

			result_->except_info_table_[n].finally_pc = code_size();

			// finally節のコードを埋め込む
			compile_stmt(e->try_finally());
			
			ff().finallies.pop();

			put_inst<InstPopGoto>();

			set_label(end_label);
			result_->except_info_table_[n].end_pc = code_size();
			return 0;
		}

		XTAL_CASE_N(case EXPR_IF:){		
			scope_begin(e);

			ExprPtr cond = e->if_cond();
			if(cond->itag()==EXPR_DEFINE && cond->bin_lhs()->itag()==EXPR_LVAR){
				VariableInfo var = var_find(cond->bin_lhs()->lvar_name(), true);
				if(var.found){
					if(var.is_register()){
						compile_expr(cond->bin_rhs(), stack_top, var.register_number);
						var_visible(cond->bin_lhs()->lvar_name(), true);
					}
					else{
						XTAL_ASSERT(false);
					}

					// 変数参照を条件式とする
					cond = cond->bin_lhs();
				}
				else{
					cond = cond->bin_rhs();
				}
			}

			AnyPtr val = do_expr(cond);
			if(!XTAL_detail_is_undefined(val)){
				if(val){
					compile_stmt(e->if_body());
				}
				else{
					compile_stmt(e->if_else());
				}
			}
			else{
				int_t label_false = reserve_label();
				int_t label_end = reserve_label();

				put_if_code(cond, label_false, stack_top);

				compile_stmt(e->if_body());
				
				if(e->if_else()){
					set_jump(InstGoto::OFFSET_address, label_end);
					put_inst<InstGoto>(0);
				}
				
				set_label(label_false);
				compile_stmt(e->if_else());
			
				set_label(label_end);
			}

			scope_end(e);
			
			return 0;
		}

		XTAL_CASE_N(case EXPR_FOR:){
			scope_begin(e);
			
			VariableInfo first_step_info = var_find(Xid(first_step));

			int_t label_cond = reserve_label();
			int_t label_false_q = reserve_label();
			int_t label_false = reserve_label();
			int_t label_break = reserve_label();
			int_t label_continue = reserve_label();
			int_t label_body = reserve_label();

			FunFrame::Loop loop;
			loop.control_statement_label[0] = label_break;
			loop.control_statement_label[1] = label_continue;
			loop.label = e->for_label();
			loop.frame_count = scope_stack_.size();
			loop.have_label = false;
			ff().loops.push(loop);

			compile_stmt(e->for_init());

			// 条件式をコンパイル
			if(e->for_cond()){
				if(first_step_info.found){
					put_inst<InstLoadValue>(first_step_info.register_number, LOAD_TRUE);
				}
				put_if_code(e->for_cond(), label_false, stack_top);
			}

			set_label(label_body);

			// ループ本体をコンパイル
			compile_stmt(e->for_body());

			set_label(label_continue);

			// next部をコンパイル
			if(e->for_next()){
				compile_stmt(e->for_next());
			}

			set_label(label_cond);

			// 条件式をコンパイル 2回目
			if(e->for_cond()){
				if(first_step_info.found){
					put_inst<InstLoadValue>(first_step_info.register_number, LOAD_FALSE);
				}
				put_if_code(e->for_cond(), label_false_q, stack_top);
			}

			/*if(referenced_first_step){
				// ループ本体をコンパイル 2回目
				compile_stmt(e->for_body());

				// label_continue部分にジャンプ
				set_jump(InstGoto::OFFSET_address, label_continue);
				put_inst<InstGoto>(0);
			}
			else*/{
				// label_body部分にジャンプ
				set_jump(InstGoto::OFFSET_address, label_body);
				put_inst<InstGoto>(0);
			}
			
			ff().loops.pop();
			
			set_label(label_false);
			if(e->for_else()){
				compile_stmt(e->for_else());
			}

			set_label(label_false_q);
			if(e->for_nobreak()){
				compile_stmt(e->for_nobreak());
			}

			set_label(label_break);

			scope_end(e);
			return 0;
		}

		XTAL_CASE_N(case EXPR_MASSIGN:){
			ExprPtr lhs = e->massign_lhs_exprs();
			ExprPtr rhs = e->massign_rhs_exprs();

			int_t lhs_stack_base = stack_top;

			// 左辺をすべて評価する
			for(uint_t i=0; i<lhs->size(); ++i){
				ExprPtr term = ep(lhs->at(i));

				if(term->itag()==EXPR_LVAR){

				}
				else if(term->itag()==EXPR_IVAR){

				}
				else if(term->itag()==EXPR_PROPERTY){
					int_t nterm = stack_top++; compile_expr(term->property_term(), stack_top, nterm);

					if(term->property_ns()){
						int_t sec = stack_top++; compile_expr(term->property_ns(), stack_top, sec);
					}
					else{
						int_t sec = stack_top++; put_inst<InstLoadValue>(sec, LOAD_UNDEFINED);
					}
				}
				else if(term->itag()==EXPR_PROPERTY_Q){
					int_t nterm = stack_top++; compile_expr(term->property_term(), stack_top, nterm);

					if(term->property_ns()){
						int_t sec = stack_top++; compile_expr(term->property_ns(), stack_top, sec);
					}
					else{
						int_t sec = stack_top++; put_inst<InstLoadValue>(sec, LOAD_UNDEFINED);
					}
				}
				else if(term->itag()==EXPR_AT){
					int_t var = stack_top++; compile_expr(term->bin_lhs(), stack_top, var); 
					int_t key = stack_top++; compile_expr(term->bin_rhs(), stack_top, key); 
				}
			}

			// スタックトップを保存
			int_t rhs_stack_base = stack_top;

			// 右辺をすべて評価する
			for(uint_t i=0; i<lhs->size(); ++i){
				// 右辺最後の要素
				if(i==rhs->size()-1){
					int_t rrc = lhs->size() - i;
					compile_expr(rhs->at(i), stack_top+1, stack_top, rrc);
					stack_top += rrc;
					break;
				}
				// 左辺最後の要素
				else if(i==lhs->size()-1){
					int_t stack_base2 = stack_top;
					for(; i<rhs->size(); ++i){
						compile_expr(rhs->at(i), stack_top+1, stack_top);
						stack_top++;
					}

					if(rhs->size()!=lhs->size()){
						put_inst<InstAdjustValues>(stack_base2, rhs->size()-(lhs->size()-1), 1);
					}
					break;
				}
				else{
					compile_expr(rhs->at(i), stack_top+1, stack_top);
					stack_top++;
				}
			}


			// 左辺に代入する
			for(uint_t i=0; i<lhs->size(); ++i){
				ExprPtr term = ep(lhs->at(i));

				if(term->itag()==EXPR_LVAR){
					int_t target = rhs_stack_base++;
					compile_lassign(target, term->ivar_name());
				}
				else if(term->itag()==EXPR_IVAR){
					int_t target = rhs_stack_base++;
					put_set_instance_variable_code(target, term->ivar_name());
				}
				else if(term->itag()==EXPR_PROPERTY){
					int_t nterm = lhs_stack_base++;
					int_t sec = lhs_stack_base++;
					int_t target = rhs_stack_base++;
					compile_set_property(nterm, term->property_name(), sec, target, 0, stack_top, target);
				}
				else if(term->itag()==EXPR_PROPERTY_Q){
					int_t nterm = lhs_stack_base++;
					int_t sec = lhs_stack_base++;
					int_t target = rhs_stack_base++;
					compile_set_property(nterm, term->property_name(), sec, target, MEMBER_FLAG_Q_BIT, stack_top, target);
				}
				else if(term->itag()==EXPR_AT){
					int_t var = lhs_stack_base++;
					int_t key = lhs_stack_base++;
					int_t target = rhs_stack_base++;
					put_inst<InstSetAt>(var, key, target, stack_top);
				}
				else{
					error(Xt("XCE1012"));
				}
			}

			return 0;
		}

		XTAL_CASE_N(case EXPR_MDEFINE:){
			ExprPtr lhs = e->massign_lhs_exprs();
			ExprPtr rhs = e->massign_rhs_exprs();

			int_t lhs_stack_base = stack_top;

			// 左辺をすべて評価する
			for(uint_t i=0; i<lhs->size(); ++i){
				ExprPtr term = ep(lhs->at(i));

				if(term->itag()==EXPR_LVAR){

				}
				else if(term->itag()==EXPR_MEMBER){
					int_t nterm = stack_top++; compile_expr(term->member_term(), stack_top, nterm);

					if(can_cast<ID>(term->member_name())){
					
					}
					else{
						int_t primary = stack_top++; compile_expr(term->member_name(), stack_top, primary);
						compile_property(primary, Xid(to_s), null, 0, stack_top, primary, 1);
						compile_property(primary, Xid(intern), null, 0, stack_top, primary, 1);
					}

					if(term->member_ns()){
						int_t sec = stack_top++; compile_expr(term->member_ns(), stack_top, sec);
					}
					else{
						int_t sec = stack_top++; put_inst<InstLoadValue>(sec, LOAD_UNDEFINED);
					}
				}
			}

			// スタックトップを保存
			int_t rhs_stack_base = stack_top;

			// 右辺をすべて評価する
			for(uint_t i=0; i<lhs->size(); ++i){
				// 右辺最後の要素
				if(i==rhs->size()-1){
					int_t rrc = lhs->size() - i;
					compile_expr(rhs->at(i), stack_top+1, stack_top, rrc);
					stack_top += rrc;
					break;
				}
				// 左辺最後の要素
				else if(i==lhs->size()-1){
					int_t stack_base2 = stack_top;
					for(; i<rhs->size(); ++i){
						compile_expr(rhs->at(i), stack_top+1, stack_top);
						stack_top++;
					}

					if(rhs->size()!=lhs->size()){
						put_inst<InstAdjustValues>(stack_base2, rhs->size()-(lhs->size()-1), 1);
					}
					break;
				}
				else{
					compile_expr(rhs->at(i), stack_top+1, stack_top);
					stack_top++;
				}
			}

			// 左辺に代入する
			for(uint_t i=0; i<lhs->size(); ++i){
				ExprPtr term = ep(lhs->at(i));

				if(term->itag()==EXPR_LVAR){
					int_t target = rhs_stack_base++;
					VariableInfo var = var_find(term->lvar_name(), true);
					var_visible(term->lvar_name(), true);
					if(var.is_register()){
						put_inst<InstCopy>(var.register_number, target);
					}
					else{
						if(var.toplevel){
							put_inst<InstSetFilelocalVariable>(target, var.member_number);
						}
						else{
							XTAL_ASSERT(false);
						}
					}
				}
				else if(term->itag()==EXPR_MEMBER){
					int_t flags = 0;
					int_t nterm = lhs_stack_base++;

					int_t primary;
					if(IDPtr id = ptr_cast<ID>(term->member_name())){
						primary = register_identifier(id);
					}
					else{
						primary = lhs_stack_base++;
						flags |= MEMBER_FLAG_P_BIT;		
					}

					int_t sec = lhs_stack_base++;
					if(term->member_ns()){
						flags |= MEMBER_FLAG_S_BIT;		
					}
					
					int_t target = rhs_stack_base++;
					put_inst<InstDefineMember>(nterm, primary, sec, flags, target);
				}
				else{
					error(Xt("XCE1012"));
				}
			}

			return 0;
		}

		XTAL_CASE_N(case EXPR_ASSIGN:){
			if(e->bin_lhs()->itag()==EXPR_LVAR){
				VariableInfo var = var_find(e->bin_lhs()->lvar_name());
				if(var.is_register()){
					compile_expr(e->bin_rhs(), stack_top, var.register_number);
				}
				else{
					int_t target = compile_expr(e->bin_rhs(), stack_top);
					compile_lassign(target, e->bin_lhs()->lvar_name());
				}
			}
			else if(e->bin_lhs()->itag()==EXPR_IVAR){
				int_t target = compile_expr(e->bin_rhs(), stack_top);
				put_set_instance_variable_code(target, e->bin_lhs()->ivar_name());
			}
			else if(e->bin_lhs()->itag()==EXPR_PROPERTY){
				compile_set_property(e->bin_lhs()->property_term(), e->bin_lhs()->property_name(), e->bin_lhs()->property_ns(),
					e->bin_rhs(), 0, stack_top, result, result_count);
			}
			else if(e->bin_lhs()->itag()==EXPR_PROPERTY_Q){
				compile_set_property(e->bin_lhs()->property_term(), e->bin_lhs()->property_name(), e->bin_lhs()->property_ns(),
					e->bin_rhs(), MEMBER_FLAG_Q_BIT, stack_top, result, result_count);
			}
			else if(e->bin_lhs()->itag()==EXPR_AT){
				int_t target = compile_expr(e->bin_lhs()->bin_lhs(), stack_top);
				int_t key = compile_expr(e->bin_lhs()->bin_rhs(), stack_top);
				int_t value = compile_expr(e->bin_rhs(), stack_top);
				put_inst<InstSetAt>(target, key, value, stack_top);
			}
			else{
				error(Xt("XCE1012"));
			}

			return 0;
		}

		XTAL_CASE_N(case EXPR_DEFINE:){
			if(e->bin_lhs()->itag()==EXPR_LVAR){
				VariableInfo var = var_find(e->bin_lhs()->lvar_name(), true);

				if(e->bin_rhs()->itag()==EXPR_CLASS){
					e->bin_rhs()->set_class_name(e->bin_lhs()->lvar_name());
				}
				else if(e->bin_rhs()->itag()==EXPR_FUN){
					e->bin_rhs()->set_fun_name(e->bin_lhs()->lvar_name());
				}

				if(var.is_register()){
					compile_expr(e->bin_rhs(), stack_top, var.register_number);
					var_visible(e->bin_lhs()->lvar_name(), true);
				}
				else{
					if(var.toplevel){
						compile_expr(e->bin_rhs(), stack_top+1, stack_top);
						put_inst<InstSetFilelocalVariable>(stack_top, var.member_number);
						var_visible(e->bin_lhs()->lvar_name(), true);
					}
					else{
						compile_expr(e->bin_rhs(), stack_top+1, stack_top);
						//XTAL_ASSERT(false);
					}
				}
			}
			else if(e->bin_lhs()->itag()==EXPR_MEMBER){
				int_t lhs = compile_expr(e->bin_lhs()->member_term(), stack_top);
				int_t rhs = compile_expr(e->bin_rhs(), stack_top);

				int_t flags = 0;
				int_t primary = 0;
				int_t secondary = 0;

				if(IDPtr id = ptr_cast<ID>(e->bin_lhs()->member_name())){
					primary = register_identifier(id);
				}
				else{
					primary = stack_top++; compile_expr(e->bin_lhs()->member_name(), stack_top, primary);
					compile_property(primary, Xid(to_s), null, 0, stack_top, primary, 1);
					compile_property(primary, Xid(intern), null, 0, stack_top, primary, 1);
					flags |= MEMBER_FLAG_P_BIT;		
				}

				if(e->bin_lhs()->member_ns()){
					secondary = compile_expr(e->bin_lhs()->member_ns(), stack_top);
					flags |= MEMBER_FLAG_S_BIT;		
				}
				
				put_inst<InstDefineMember>(lhs, primary, secondary, flags, rhs);
			}
			else{
				error(Xt("XCE1030"));
			}

			return 0;
		}

		XTAL_CASE_N(case EXPR_CDEFINE_MEMBER:){

			return 0;
		}

		XTAL_CASE_N(case EXPR_CDEFINE_IVAR:){
			return 0;
		}

		XTAL_CASE_N(case EXPR_BREAK:){
			compile_loop_control_statement(e);
			return 0;
		}

		XTAL_CASE_N(case EXPR_CONTINUE:){
			compile_loop_control_statement(e);
			return 0;
		}

		XTAL_CASE_N(case EXPR_BRACKET:){
			return compile_expr(e->una_term(), stack_top, result, result_count);
		}

		XTAL_CASE_N(case EXPR_SCOPE:){
			scope_begin(e);{
				XTAL_FOR_EXPR(v, e->scope_stmts()){
					compile_stmt(v);
				}
			}scope_end(e);
			return 0;
		}

		XTAL_CASE_N(case EXPR_SWITCH:){
			ExprPtr cond = e->switch_cond();
				
			scope_begin(e);

			if(cond->itag()==EXPR_DEFINE && cond->bin_lhs()->itag()==EXPR_LVAR){
				VariableInfo var = var_find(cond->bin_lhs()->lvar_name(), true);
				if(var.found){
					if(var.is_register()){
						compile_expr(cond->bin_rhs(), stack_top, var.register_number);
						var_visible(cond->bin_lhs()->lvar_name(), true);
					}
					else{
						XTAL_ASSERT(false);
					}

					// 変数参照を条件式とする
					cond = cond->bin_lhs();
				}
				else{
					cond = cond->bin_rhs();
				}
			}

			int_t label_jump = reserve_label();
			int_t label_end = reserve_label();
			int_t label_default = reserve_label();

			int_t base = stack_top++;

			set_jump(InstOnce::OFFSET_address, label_jump);
			int_t num = result_->once_table_.size();
			result_->once_table_.push_back(undefined);
			put_inst<InstOnce>(base, 0, num);

			MapPtr case_map = xnew<Map>();
			ArrayPtr case_array = xnew<Array>();
			ArrayPtr jump_array = xnew<Array>();
			ExprPtr default_case = e->switch_default();
			Xfor_cast(ExprPtr v, e->switch_cases()){
				// vはcase ひとつが入っている
				// v->at(0) には条件のリスト
				// v->at(1) には条件がマッチした場合の実行文が入っている

				case_array->push_back(v->at(1));
				jump_array->push_back(reserve_label());
				Xfor_cast(ExprPtr k, v->at(0)){
					case_map->set_at(k, case_array->size()-1);
				}
			}

			put_inst<InstMakeMap>(result);
			Xfor2(k, v, case_map){
				compile_expr(k, stack_top+1, stack_top);
				set_jump(InstPushGoto::OFFSET_address, jump_array->at(v->to_i())->to_i());
				put_inst<InstPushGoto>(0);
				put_inst<InstPop>(stack_top+1);
				put_inst<InstMapInsert>(base, stack_top, stack_top+1);
			}

			set_jump(InstPushGoto::OFFSET_address, label_default);
			put_inst<InstPushGoto>(0);
			put_inst<InstPop>(stack_top+1);
			put_inst<InstMapSetDefault>(base, stack_top+1);
						
			put_inst<InstSetOnce>(base, num);
			
			set_label(label_jump);

			compile_expr(cond, stack_top+1, stack_top);
			put_inst<InstAt>(base, base, stack_top, stack_top+1);
			put_inst<InstPush>(base);
			put_inst<InstPopGoto>();

			Xfor2(k, v, case_map){
				int_t index = v->to_i();
				AnyPtr snt = case_array->at(index);

				if(ep(snt)){
					set_label(jump_array->at(index)->to_i());
					compile_stmt(snt);
					case_array->set_at(index, label_end);
					set_jump(InstGoto::OFFSET_address, label_end);
					put_inst<InstGoto>(0);
				}
				else{
					set_jump(InstGoto::OFFSET_address, case_array->at(index)->to_i());
					put_inst<InstGoto>(0);
				}
			}

			set_label(label_default);

			compile_stmt(default_case);
			set_jump(InstGoto::OFFSET_address, label_end);
			put_inst<InstGoto>(0);

			set_label(label_end);	

			scope_end(e);
			return 0;
		}

		XTAL_CASE_N(case EXPR_TOPLEVEL:){
			scope_begin(e);{
				XTAL_FOR_EXPR(v, e->toplevel_stmts()){
					compile_stmt(v);
				}
			}scope_end(e);
			return 0;
		}
	}

	return 0;
}

}

#endif



