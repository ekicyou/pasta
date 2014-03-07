#include "xtal.h"
#include "xtal_macro.h"

#include "xtal_codebuilder.h"
#include "xtal_stringspace.h"

#ifndef XTAL_NO_PARSER

namespace xtal{
	
CodeBuilder::CodeBuilder(){
	implicit_ref_map_ = XNew<Map>();
}

CodeBuilder::~CodeBuilder(){
	while(!fun_frame_stack_.empty()){
		pop_ff();
	}

	while(!class_frame_stack_.empty()){
		pop_cf();
	}
}

void CodeBuilder::error(const AnyPtr& message){
	parser_.executor_->error(message, lineno());
}

CodePtr CodeBuilder::compile(const StreamPtr& stream, const StringPtr& source_file_name){
	xpeg::ExecutorPtr executor = xnew<xpeg::StreamExecutor>(stream, source_file_name);
	eb_ = executor;
	parser_.parse(executor);

	if(!stream->eos()){
		error(Xt("XCE1001"));
	}

	if(executor->has_error()){
		XTAL_SET_EXCEPT(cpp_class<CompileError>()->call(Xt1("XRE1016", name, source_file_name), errors()));
		return nul<Code>();
	}

	prev_inst_op_ = -1;
	eval_ = false;

	return compile_toplevel(ep(executor->tree_back()), source_file_name);
}

CodePtr CodeBuilder::compile(const xpeg::ExecutorPtr& executor, const StringPtr& source_file_name){
	eb_ = executor;
	parser_.parse(executor);

	if(executor->has_error()){
		XTAL_SET_EXCEPT(cpp_class<CompileError>()->call(Xt1("XRE1016", name, source_file_name), errors()));
		return nul<Code>();
	}

	prev_inst_op_ = -1;
	eval_ = false;

	return compile_toplevel(ep(executor->tree_back()), source_file_name);
}

CodePtr CodeBuilder::eval_compile(const xpeg::ExecutorPtr& executor){
	eb_ = executor;

	parser_.parse_eval(executor);
	
	if(executor->has_error()){
		XTAL_SET_EXCEPT(cpp_class<CompileError>()->call(Xt1("XRE1002", name, XTAL_STRING("<eval>")), errors()));
		return nul<Code>();
	}

    ExprPtr e;
    if(executor->tree_back()){
        e = ep(executor->tree_back());
    }

	if(e->itag()==EXPR_DEFINE){
		if(e->bin_lhs()->itag()==EXPR_LVAR){
			eb_->tree_push_back(XTAL_DEFINED_ID(filelocal));
			eb_->tree_splice(EXPR_LVAR, 1);
			eb_->tree_push_back(e->bin_lhs()->lvar_name());
			eb_->tree_push_back(null);
			eb_->tree_splice(EXPR_MEMBER, 3);
			e->set_bin_lhs(ep(eb_->tree_pop_back()));
		}
	}
	else if(e->itag()==EXPR_MDEFINE){
		ExprPtr lhs = e->mdefine_lhs_exprs();
		for(uint_t i=0; i<lhs->size(); ++i){
			ExprPtr e2 = ep(lhs->at(i));
			if(e2->itag()==EXPR_LVAR){
				eb_->tree_push_back(XTAL_DEFINED_ID(filelocal));
				eb_->tree_splice(EXPR_LVAR, 1);
				eb_->tree_push_back(e2->lvar_name());
				eb_->tree_push_back(null);
				eb_->tree_splice(EXPR_MEMBER, 3);
				lhs->set_at(i, ep(eb_->tree_pop_back()));
			}
		}
	}
	else{
		eb_->tree_push_back(e);
		eb_->tree_splice(EXPR_LIST, 1);
		eb_->tree_splice(EXPR_RETURN, 1);
		e = ep(eb_->tree_pop_back());
	}

	eb_->tree_push_back(e);
	eb_->tree_splice(EXPR_LIST, 1);
	eb_->tree_splice(EXPR_TOPLEVEL, 1);
	e = ep(eb_->tree_pop_back());

	prev_inst_op_ = -1;
	eval_ = true;

	return compile_toplevel(e, XTAL_STRING("<eval>"));
}

CodePtr CodeBuilder::compile_toplevel(const ExprPtr& ae, const StringPtr& source_file_name){
	ExprPtr e = ae;
	normalize(e);
	build_scope(e);
	build_scope2(e);

	XTAL_ASSERT(scope_stack_.empty());
	scope_stack_.upsize(1);
	Scope* root = scope_stack_.top();
	scope_stack_.downsize(1);
	optimize_scope(root);
	calc_scope(root, root, 0);

	result_ = XNew<Code>();

	result_->source_file_name_ = source_file_name;
	result_->except_info_table_.push_back(ExceptInfo());

	identifier_map_ = XNew<Map>();
	register_identifier(empty_id);

	value_map_ = XNew<Map>();
	register_value(null);
	
	result_->once_table_.push_back(undefined);

	linenos_.push(0);

	// 関数フレームを作成する
	push_ff();

	// 関数コアを作成
	FunInfo info;
	info.pc = 0;
	info.kind = KIND_FUN;
	info.min_param_count = 0;
	info.max_param_count = 0;
	info.name_number = (u16)register_identifier(XTAL_DEFINED_ID(toplevel));
	info.flags = 0;
	info.variable_offset = 0;
	info.variable_size = 0;
	info.variable_identifier_offset = 0;

	//int esize = scope_stack_.size();
	scope_begin(null);

	info.max_variable = (u16)current_scope().register_max;
	int_t fun_info_table_number = 0;
	result_->xfun_info_table_.push_back(info);

	// 関数本体を処理する
	compile_stmt(e);
	
	break_off(ff().var_frame_count+1);

	eb_->tree_push_back(XTAL_DEFINED_ID(filelocal));
	eb_->tree_splice(EXPR_LVAR, 1);
	eb_->tree_splice(0, 1);
	eb_->tree_splice(EXPR_RETURN, 1);
	compile_stmt(eb_->tree_pop_back());
	
	result_->set_lineno_info(result_->final_lineno()+1);

	put_inst<InstThrow>();

	process_labels();
	
	if(current_scope().scope_chain){
		result_->xfun_info_table_[fun_info_table_number].flags |= FunInfo::FLAG_SCOPE_CHAIN;
	}

    //int eesize = scope_stack_.size();
	scope_end(null);
	delete_scope(root);

	XTAL_ASSERT(scope_stack_.empty());

	// 関数フレームをポップする
	pop_ff();

	if(debug::is_debug_compile_enabled()){
		Xfor2(k, v, implicit_ref_map_){
			if(v){
				Code::ImplcitInfo info = {(u16)k->to_i(), (u16)v->to_i()};
				result_->implicit_table_.push_back(info);
			}
		}
	}

	if(!parser_.executor_->errors()){
		opt_jump();
		result_->generated();
		return result_;
	}
	else{
		XTAL_SET_EXCEPT(cpp_class<CompileError>()->call(Xt1("XRE1016", name, source_file_name), errors()));
		result_ = null;
		return null;
	}
}

CodeBuilder::FunFrame& CodeBuilder::push_ff(){
	fun_frame_stack_.push(new_object_xmalloc<FunFrame>());
	ff().var_frame_count = scope_stack_.size();
	ff().return_value = -1;
	return ff();
}
	
void CodeBuilder::pop_ff(){ 
	delete_object_xfree<FunFrame>(fun_frame_stack_.top());
	fun_frame_stack_.pop(); 
}

CodeBuilder::ClassFrame& CodeBuilder::push_cf(){
	class_frame_stack_.push(new_object_xmalloc<ClassFrame>());
	cf().class_info_num = result_->class_info_table_.size();
	return cf();
}
	
void CodeBuilder::pop_cf(){ 
	delete_object_xfree<ClassFrame>(class_frame_stack_.top());
	class_frame_stack_.pop(); 
}

address16 CodeBuilder::calc_address(const inst_t* pc, address16 address){
	const inst_t* pc2 = pc + address;
	if(XTAL_opc(pc2)==InstGoto::NUMBER){
		return (address16)(pc2+InstGoto::address(pc2) - pc);
	}

	return address;
}

void CodeBuilder::opt_jump(){
	const inst_t* begin = result_->bytecode_data();
	inst_t* pc = (inst_t*)begin;
	const inst_t* end = result_->bytecode_data() + result_->bytecode_size(); 

	while(pc<end){
		switch(XTAL_opc(pc)){
			XTAL_DEFAULT{}

			XTAL_CASE(InstIf::NUMBER){
				InstIf::set_address_true(pc, calc_address(pc, (address16)InstIf::address_true(pc)));
				InstIf::set_address_false(pc, calc_address(pc, (address16)InstIf::address_false(pc)));
			}

			XTAL_CASE(InstGoto::NUMBER){
				InstGoto::set_address(pc, calc_address(pc, (address16)InstGoto::address(pc)));
			}
		}

		int_t size = inst_size(XTAL_opc(pc));
		if(size==0){
			break;
		}

		pc += inst_size(XTAL_opc(pc));
	}
}

AnyPtr CodeBuilder::errors(){
	return parser_.executor_->errors();
}

inst_t* CodeBuilder::code_reserve(size_t size){
	size_t cur = result_->code_.size();
	result_->code_.resize(cur+size);
	return &result_->code_[cur];
}

void CodeBuilder::code_wrote(size_t size){
	inst_t* x = &result_->code_[result_->code_.size()-size];

	if(XTAL_opc(x)==0xff){
		error(Xt("XCE1027"));
	}

	prev_inst_op_ = XTAL_opc(x);
}

int_t CodeBuilder::register_identifier(const IDPtr& v){
	if(!v){ 
		return 0; 
	}

	if(const AnyPtr& pos = identifier_map_->at(v)){ 
		return pos->to_i(); 
	}

	result_->identifier_table_.push_back(v);
	identifier_map_->set_at(v, result_->identifier_table_.size()-1);
	return result_->identifier_table_.size()-1;
}

int_t CodeBuilder::register_value(const AnyPtr& v){
	if(const AnyPtr& pos = value_map_->at(v)){ 
		return pos->to_i(); 
	}

	result_->value_table_.push_back(v);
	value_map_->set_at(v, result_->value_table_.size()-1);
	return result_->value_table_.size()-1;
}

int_t CodeBuilder::append_identifier(const IDPtr& v){
	result_->identifier_table_.push_back(v);

	if(!identifier_map_->at(v)){
		identifier_map_->set_at(v, result_->identifier_table_.size()-1);
	}

	return result_->identifier_table_.size()-1;
}

int_t CodeBuilder::append_value(const AnyPtr& v){
	result_->value_table_.push_back(v);
	return result_->value_table_.size()-1;
}

int_t CodeBuilder::lookup_instance_variable(const IDPtr& key, bool must){
	if(!class_frame_stack_.empty()){
		int ret = 0;
		ClassFrame& cf = this->cf();
		for(size_t i = 0, last = cf.entries.size(); i<last; ++i){
			if(XTAL_detail_raweq(cf.entries[i].name, key)){
				return ret;
			}
			ret++;
		}
	}

	if(must || !eval_){
		error(Xt("XCE1023")->call(Xnamed(name, Xid(_)->cat(key))));
		return 0;
	}

	return -1;
}

void CodeBuilder::put_set_instance_variable_code(int_t value, const IDPtr& var){
	int_t n = lookup_instance_variable(var, false);
	if(n<0){
		put_inst<InstSetInstanceVariableByName>(value, register_identifier(var));
	}
	else{
		put_inst<InstSetInstanceVariable>(value, class_info_num(), lookup_instance_variable(var));
	}
}

void CodeBuilder::put_instance_variable_code(int_t result, const IDPtr& var){
	int_t n = lookup_instance_variable(var, false);
	if(n<0){
		put_inst<InstInstanceVariableByName>(result, register_identifier(var));
	}
	else{
		put_inst<InstInstanceVariable>(result, class_info_num(), lookup_instance_variable(var));
	}
}

int_t CodeBuilder::reserve_label(){
	FunFrame& ff = this->ff();
	ff.labels.resize(ff.labels.size()+1);
	ff.labels[ff.labels.size()-1].pos = -1;
	return ff.labels.size()-1;
}

void CodeBuilder::set_label(int_t labelno){
	prev_inst_op_ = -1;
	ff().labels[labelno].pos = code_size();
}

void CodeBuilder::set_jump(int_t offset, int_t labelno){
	FunFrame::Label::From f;
	f.lineno = linenos_.top();
	f.pos = code_size();
	f.set_pos = f.pos + offset/sizeof(inst_t);
	ff().labels[labelno].froms.push_back(f);
}

void CodeBuilder::process_labels(){
	for(size_t i = 0; i<ff().labels.size(); ++i){
		FunFrame::Label &l = ff().labels[i];
		XTAL_ASSERT(l.pos!=-1);
		for(size_t j = 0; j<l.froms.size(); ++j){
			FunFrame::Label::From &f = l.froms[j];
			address16& buf = *(address16*)&result_->code_[f.set_pos];
			buf = (address16)(l.pos - f.pos); //l.pos - f.set_pos;
		}
	}
	ff().labels.clear();
}

void CodeBuilder::break_off(int_t n){
	for(uint_t scope_count = scope_stack_.size(); scope_count!=(uint_t)n; scope_count--){
		for(uint_t k = 0; k<ff().finallies.size(); ++k){
			if((uint_t)ff().finallies[k].frame_count==scope_count){
				int_t label = reserve_label();
				set_jump(InstPushGoto::OFFSET_address, label);
				put_inst<InstPushGoto>(0);
				put_inst<InstTryEnd>();
				set_label(label);
			}
		}

		Scope& scope = *scope_stack_[scope_stack_.size()-scope_count];
		if(scope_exist(&scope)){
			put_inst<InstScopeEnd>();
		}
	}
}

bool CodeBuilder::is_comp_bin(const ExprPtr& e){
	if(e && EXPR_EQ<=e->itag() && e->itag()<=EXPR_NIS){
		return true;
	}	
	return false;
}

void CodeBuilder::scope_chain(int_t var_frame_size){
	for(int_t i=0; i<var_frame_size; ++i){
		scope_stack_[i]->scope_chain = true;
	}
}

void CodeBuilder::calc_scope(Scope* scope, Scope* fun_scope, int_t base){
	if(scope->kind==Scope::FUN){
		scope->fun_scope = scope;
		scope->register_base = scope->entries.size();
	}
	else if(scope->kind==Scope::TOPLEVEL){
		scope->fun_scope = fun_scope;
		scope->register_base = 0;
	}
	else if(scope->kind==Scope::CLASS){
		scope->fun_scope = fun_scope;
		scope->register_base = 0;
	}
	else{
		scope->fun_scope = fun_scope;
		scope->register_base = base + scope->entries.size();
	}

	scope->register_max = scope->register_base;

	for(uint_t i=0; i<scope->children.size(); ++i){
		calc_scope(scope->children[i], scope->fun_scope, scope->register_base);

		if(scope->children[i]->register_max > scope->register_max){
			scope->register_max = scope->children[i]->register_max;
		}
	}
}

void CodeBuilder::delete_scope(Scope* scope){
	for(uint_t i=0; i<scope->children.size(); ++i){
		delete_scope(scope->children[i]);
	}
	scope->children.clear();

	delete_object_xfree<Scope>(scope);
}

void CodeBuilder::optimize_scope(Scope* scope){
	bool scope_chain_have_possibilities = scope->kind!=Scope::FRAME;

	for(uint_t i=0; i<scope->children.size(); ++i){
		Scope* child = scope->children[i];
		optimize_scope(child);

		if(child->scope_chain_have_possibilities){
			scope_chain_have_possibilities = true;
		}
	}

	scope->scope_chain_have_possibilities = scope_chain_have_possibilities;
}

bool CodeBuilder::scope_exist(Scope* scope){
	return scope->entries.size()!=0 && scope->kind==Scope::FRAME && 
		(scope->scope_chain_have_possibilities || debug::is_debug_compile_enabled());
}

void CodeBuilder::build_scope(const AnyPtr& a){
	ExprPtr e = ep(a);
	if(!e){ return; }

	switch(e->itag()){
		XTAL_DEFAULT{
			XTAL_FOR_EXPR(v, e){
				build_scope(v);
			}	
		}

		XTAL_CASE_N(case EXPR_FOR:){
			var_begin(e, Scope::FRAME);
			var_define(Xid(first_step), true);
			XTAL_FOR_EXPR(v, e){
				build_scope(v);
			}	
			var_end(e);
		}
	
		XTAL_CASE_N(
			case EXPR_SCOPE:
			case EXPR_SWITCH:
		){
			var_begin(e, Scope::FRAME);
			XTAL_FOR_EXPR(v, e){
				build_scope(v);
			}	
			var_end(e);
		}

		XTAL_CASE_N(
			case EXPR_IF:
		){
			var_begin(e, Scope::FRAME);
			
			ExprPtr cond = e->if_cond();
			AnyPtr val = do_expr(cond);
			if(!XTAL_detail_is_undefined(val)){
				if(val){
					build_scope(e->if_body());
				}
				else{
					build_scope(e->if_else());
				}
			}
			else{
				XTAL_FOR_EXPR(v, e){
					build_scope(v);
				}
			}
			var_end(e);
		}

		XTAL_CASE_N(case EXPR_FUN:){
			var_begin(e, Scope::FUN);
			XTAL_FOR_EXPR(v1, e->fun_params()){
				if(const ExprPtr& v = ep(v1)){
					if(const ExprPtr& e2 = ep(v->at(0))){
						if(e2->itag()==EXPR_LVAR){
							var_define(e2->lvar_name(), true);
						}
						else if(e2->itag()==EXPR_IVAR){
							var_define(e2->ivar_name(), true);
						}
					}
					build_scope(v->at(1));
				}
			}

			if(e->fun_extendable_param()){
				var_define(e->fun_extendable_param(), true);
			}

			build_scope(e->fun_body());
			var_end(e);
		}

		XTAL_CASE_N(case EXPR_TOPLEVEL:){
			var_begin(null, Scope::FUN);
			var_begin(e, Scope::TOPLEVEL);
			XTAL_FOR_EXPR(v, e->toplevel_stmts()){
				build_scope(v);
			}
			var_end(e);
			var_end(null);
		}

		XTAL_CASE_N(case EXPR_DEFINE:){
			if(e->bin_lhs()->itag()==EXPR_LVAR){
				var_define(e->bin_lhs()->lvar_name(), false);
			}
			else{
				build_scope(e->bin_lhs());
			}

			build_scope(e->bin_rhs());
		}

		XTAL_CASE_N(case EXPR_MDEFINE:){
			XTAL_FOR_EXPR(v, e->mdefine_lhs_exprs()){
				ExprPtr e2 = ep(v);
				if(e2->itag()==EXPR_LVAR){
					var_define(e2->lvar_name(), false);
				}
				else{
					build_scope(e2);
				}
			}

			build_scope(e->mdefine_rhs_exprs());
		}

		XTAL_CASE_N(case EXPR_CATCH:){
			build_scope(e->catch_body());
			var_begin(e, Scope::FRAME);
			var_define(e->catch_catch_var(), true);
			build_scope(e->catch_catch());
			var_end(e);
		}

		XTAL_CASE_N(case EXPR_TRY:){
			build_scope(e->try_body());

			if(e->try_catch()){
				var_begin(e, Scope::FRAME);
				var_define(e->try_catch_var(), true);
				build_scope(e->try_catch());
				var_end(e);
			}

			build_scope(e->try_finally());
		}

		XTAL_CASE_N(case EXPR_CLASS:){
			build_scope(e->class_mixins());

			var_begin(e, Scope::CLASS);
			
			XTAL_FOR_EXPR(v, e->class_stmts()){
				ExprPtr e2 = ep(v);
				if(e2->itag()==EXPR_CDEFINE_MEMBER){
					if(e2->cdefine_member_ns()){
						var_define_class_member(e2->cdefine_member_name(), e2->cdefine_member_accessibility()->to_i(), true);
					}
					else{
						var_define_class_member(e2->cdefine_member_name(), e2->cdefine_member_accessibility()->to_i(), false);
					}
				}
			}

			build_scope(e->class_stmts());
			var_end(e);
		}
	}
}

void CodeBuilder::build_scope2(const AnyPtr& a){
	ExprPtr e = ep(a);
	if(!e){ return; }

	switch(e->itag()){
		XTAL_DEFAULT{
			XTAL_FOR_EXPR(v, e){
				build_scope2(v);
			}	
		}

		XTAL_CASE_N(case EXPR_LVAR:){
			var_refere(e->lvar_name());
		}

		XTAL_CASE_N(case EXPR_FOR:){
			scope_optimize_begin(e);
			XTAL_FOR_EXPR(v, e){
				build_scope2(v);
			}	
			scope_optimize_end(e);
		}
	
		XTAL_CASE_N(
			case EXPR_SCOPE:
			case EXPR_SWITCH:
		){
			scope_optimize_begin(e);
			XTAL_FOR_EXPR(v, e){
				build_scope2(v);
			}	
			scope_optimize_end(e);
		}

		XTAL_CASE_N(
			case EXPR_IF:
		){
			scope_optimize_begin(e);
			ExprPtr cond = e->if_cond();
			AnyPtr val = do_expr(cond);
			if(!XTAL_detail_is_undefined(val)){
				if(val){
					build_scope2(e->if_body());
				}
				else{
					build_scope2(e->if_else());
				}
			}
			else{
				XTAL_FOR_EXPR(v, e){
					build_scope2(v);
				}	
			}
			scope_optimize_end(e);
		}

		XTAL_CASE_N(case EXPR_FUN:){
			scope_optimize_begin(e);
			XTAL_FOR_EXPR(v1, e->fun_params()){
				if(const ExprPtr& v = ep(v1)){
					build_scope2(v->at(1));
				}
			}

			build_scope2(e->fun_body());
			scope_optimize_end(e);
		}

		XTAL_CASE_N(case EXPR_TOPLEVEL:){
			scope_optimize_begin(null);
			scope_optimize_begin(e);
			XTAL_FOR_EXPR(v, e->toplevel_stmts()){
				build_scope2(v);
			}
			scope_optimize_end(e);
			scope_optimize_end(null);
		}

		XTAL_CASE_N(case EXPR_DEFINE:){
			if(e->bin_lhs()->itag()==EXPR_LVAR){
				
			}
			else{
				build_scope2(e->bin_lhs());
			}

			build_scope2(e->bin_rhs());

			if(e->bin_lhs()->itag()==EXPR_LVAR){
				var_visible(e->bin_lhs()->lvar_name(), true);
			}
		}

		XTAL_CASE_N(case EXPR_MDEFINE:){
			XTAL_FOR_EXPR(v, e->mdefine_lhs_exprs()){
				ExprPtr e2 = ep(v);
				if(e2->itag()==EXPR_LVAR){
					
				}
				else{
					build_scope2(e2);
				}
			}

			build_scope2(e->mdefine_rhs_exprs());

			XTAL_FOR_EXPR(v, e->mdefine_lhs_exprs()){
				ExprPtr e2 = ep(v);
				if(e2->itag()==EXPR_LVAR){
					var_visible(e2->lvar_name(), true);
					var_refere(e2->lvar_name());
				}
			}
		}

		XTAL_CASE_N(
			case EXPR_ASSIGN:
			case EXPR_ADD_ASSIGN:
			case EXPR_SUB_ASSIGN:
			case EXPR_CAT_ASSIGN:
			case EXPR_MUL_ASSIGN:
			case EXPR_DIV_ASSIGN:
			case EXPR_MOD_ASSIGN:
			case EXPR_OR_ASSIGN:
			case EXPR_AND_ASSIGN:
			case EXPR_XOR_ASSIGN:
			case EXPR_SHR_ASSIGN:
			case EXPR_SHL_ASSIGN:
			case EXPR_USHR_ASSIGN:
		){
			if(e->bin_lhs()->itag()==EXPR_LVAR){
				var_assign(e->bin_lhs()->lvar_name());
			}
			else{
				build_scope2(e->bin_lhs());
			}
			build_scope2(e->bin_rhs());
		}

		XTAL_CASE_N(case EXPR_MASSIGN:){
			XTAL_FOR_EXPR(v, e->massign_lhs_exprs()){
				ExprPtr e2 = ep(v);
				if(e2->itag()==EXPR_LVAR){
					var_assign(e2->lvar_name());
				}
				else{
					build_scope2(e2);
				}
			}

			build_scope2(e->massign_rhs_exprs());
		}

		XTAL_CASE_N(
			case EXPR_INC:
			case EXPR_DEC:
		){
			if(e->una_term()->itag()==EXPR_LVAR){
				var_assign(e->una_term()->lvar_name());
			}
			else{
				build_scope2(e->una_term());
			}
		}

		XTAL_CASE_N(case EXPR_CATCH:){
			build_scope2(e->catch_body());
			scope_optimize_begin(e);
			var_assign(e->catch_catch_var());
			build_scope2(e->catch_catch());
			scope_optimize_end(e);
		}

		XTAL_CASE_N(case EXPR_TRY:){
			build_scope2(e->try_body());

			if(e->try_catch()){
				scope_optimize_begin(e);
				var_assign(e->try_catch_var());
				build_scope2(e->try_catch());
				scope_optimize_end(e);
			}

			build_scope2(e->try_finally());
		}

		XTAL_CASE_N(case EXPR_CLASS:){
			build_scope2(e->class_mixins());

			scope_optimize_begin(e);
			build_scope2(e->class_stmts());
			scope_optimize_end(e);
		}
	}
}

void CodeBuilder::normalize(const AnyPtr& a){
	ExprPtr e = ep(a);
	if(!e){
		return;
	}

	switch(e->itag()){

		XTAL_DEFAULT{
			XTAL_FOR_EXPR(v, e){
				normalize(v);
			}	
		}

		XTAL_CASE_N(case EXPR_CDEFINE_MEMBER:){
			if(e->cdefine_member_term()->itag()==EXPR_FUN){
				ExprPtr f = e->cdefine_member_term();
				if(!f->fun_name()){
					f->set_fun_name(e->cdefine_member_name());
				}
			}
			else if(e->cdefine_member_term()->itag()==EXPR_CLASS){
				ExprPtr f = e->cdefine_member_term();
				if(!f->class_name()){
					f->set_class_name(e->cdefine_member_name());
				}
			}

			XTAL_FOR_EXPR(v, e){
				normalize(v);
			}	
		}

		XTAL_CASE_N(case EXPR_DEFINE:){
			if(e->bin_lhs()->itag()==EXPR_LVAR){
				if(e->bin_rhs()->itag()==EXPR_FUN){
					ExprPtr f = e->bin_rhs();
					if(!f->fun_name()){
						f->set_fun_name(e->bin_lhs()->lvar_name());
					}
				}
				else if(e->bin_rhs()->itag()==EXPR_CLASS){
					ExprPtr f = e->bin_rhs();
					if(!f->class_name()){
						f->set_class_name(e->bin_lhs()->lvar_name());
					}
				}
			}

			XTAL_FOR_EXPR(v, e){
				normalize(v);
			}	
		}

		XTAL_CASE_N(case EXPR_ASSIGN:){
			if(e->bin_lhs()->itag()==EXPR_LVAR){
				if(e->bin_rhs()->itag()==EXPR_FUN){
					ExprPtr f = e->bin_rhs();
					if(!f->fun_name()){
						f->set_fun_name(e->bin_lhs()->lvar_name());
					}
				}
				else if(e->bin_rhs()->itag()==EXPR_CLASS){
					ExprPtr f = e->bin_rhs();
					if(!f->class_name()){
						f->set_class_name(e->bin_lhs()->lvar_name());
					}
				}
			}

			XTAL_FOR_EXPR(v, e){
				normalize(v);
			}	
		}

		XTAL_CASE_N(case EXPR_CLASS:){
			int_t method_kind = e->class_kind()==KIND_SINGLETON ? KIND_FUN : KIND_METHOD;
			ExprPtr stmts = xnew<Expr>();
			MapPtr ivar_map = xnew<Map>();
			bool auto_initialize = false;
			Xfor_cast(ExprPtr v, e->class_stmts()->clone()){
				if(v->itag()==EXPR_CDEFINE_IVAR){
					if(v->cdefine_ivar_term()){
						eb_->tree_push_back(v->cdefine_ivar_name());
						eb_->tree_splice(EXPR_IVAR, 1);
						eb_->tree_push_back(v->cdefine_ivar_term());
						eb_->tree_splice(EXPR_ASSIGN, 2);
						stmts->push_back(eb_->tree_pop_back());
						auto_initialize = true;
					}

					// 可触性が付いているので、アクセッサを定義する
					if(v->cdefine_ivar_accessibility()){ 
						IDPtr var = v->cdefine_ivar_name();
						eb_->tree_push_back(v->cdefine_ivar_accessibility());
						eb_->tree_push_back(var);
						eb_->tree_push_back(null);
						
						eb_->tree_push_back(method_kind);
						eb_->tree_push_back(null);
						eb_->tree_push_back(null);
						eb_->tree_push_back(var);
						eb_->tree_splice(EXPR_IVAR, 1);
						eb_->tree_splice(0, 1);
						eb_->tree_splice(EXPR_RETURN, 1);
						eb_->tree_push_back(var);
						eb_->tree_splice(EXPR_FUN, 5);

						eb_->tree_splice(EXPR_CDEFINE_MEMBER, 4);

						e->class_stmts()->push_back(eb_->tree_pop_back());

						IDPtr var2 = Xid(set_)->cat(var);
						eb_->tree_push_back(v->cdefine_ivar_accessibility());
						eb_->tree_push_back(var2);
						eb_->tree_push_back(null);

						eb_->tree_push_back(method_kind);
						eb_->tree_push_back(Xid(value));
						eb_->tree_splice(EXPR_LVAR, 1);
						eb_->tree_push_back(null);
						eb_->tree_splice(0, 2);
						eb_->tree_splice(0, 1);
						eb_->tree_push_back(null);
						eb_->tree_push_back(var);
						eb_->tree_splice(EXPR_IVAR, 1);
						eb_->tree_push_back(Xid(value));
						eb_->tree_splice(EXPR_LVAR, 1);
						eb_->tree_splice(EXPR_ASSIGN, 2);
						eb_->tree_push_back(var2);
						eb_->tree_splice(EXPR_FUN, 5);

						eb_->tree_splice(EXPR_CDEFINE_MEMBER, 4);

						e->class_stmts()->push_back(eb_->tree_pop_back());
					}
				}
			}

			eb_->tree_push_back(KIND_PUBLIC);
			eb_->tree_push_back(Xid(auto_initialize));
			eb_->tree_push_back(null);

			if(auto_initialize){
				eb_->tree_push_back(method_kind);
				eb_->tree_push_back(null);
				eb_->tree_push_back(null);
				eb_->tree_push_back(stmts);
				eb_->tree_splice(EXPR_SCOPE, 1);
				eb_->tree_push_back(Xid(auto_initialize));
				eb_->tree_splice(EXPR_FUN, 5);
			}
			else{
				eb_->tree_splice(EXPR_NULL, 0);
			}

			eb_->tree_splice(EXPR_CDEFINE_MEMBER, 4);

			e->class_stmts()->push_front(eb_->tree_pop_back());

			normalize(e->class_mixins());
			normalize(e->class_stmts());
		}
	}
}


void CodeBuilder::scope_skip(){
	if(scope_stack_.empty()){
		scope_stack_.upsize(1);
	}
	else{
		Scope* p = scope_stack_[0];
		scope_stack_.push(p->children[p->children_pos++]);
	}

	if(current_scope().scope_chain && current_scope().scope_info_num>=0){
		result_->scope_info_table_[current_scope().scope_info_num].flags |= FunInfo::FLAG_SCOPE_CHAIN;
	}

	current_scope().children_pos = 0;
	scope_stack_.pop();
}

int_t CodeBuilder::calc_variable_offset(Scope& scope){
	return ((scope.register_max - scope.register_base) + (int_t)scope.entries.size());
	//return -(scope.register_base - scope.register_max - (int_t)scope.entries.size());
}

void CodeBuilder::scope_begin(const ExprPtr& e){
	if(scope_stack_.empty()){
		scope_stack_.upsize(1);
	}
	else{
		Scope* p = scope_stack_[0];
		scope_stack_.push(p->children[p->children_pos++]);
	}

	XTAL_ASSERT(current_scope().expr==e);
	
	Scope& scope = current_scope();
	if(scope_exist(&scope)){
		int_t scope_info_num = result_->scope_info_table_.size();
		current_scope().scope_info_num = scope_info_num;

		ScopeInfo info;
		info.pc = code_size();
		info.variable_offset = (u16)calc_variable_offset(scope);
		info.variable_size = (u16)current_scope().entries.size();
		info.variable_identifier_offset = (u16)result_->identifier_table_.size();
		for(uint_t i=0; i<scope.entries.size(); ++i){
			Scope::Entry& entry = scope.entries[i];
			result_->identifier_table_.push_back(entry.name);
		}

		put_inst<InstScopeBegin>(scope_info_num);

		result_->scope_info_table_.push_back(info);
	}
	else if(scope.kind==Scope::FUN || scope.kind==Scope::CLASS || scope.kind==Scope::TOPLEVEL){
		int_t scope_info_num = result_->scope_info_table_.size();
		current_scope().scope_info_num = scope_info_num;

		ScopeInfo info;
		info.pc = code_size();
		info.variable_offset = (u16)calc_variable_offset(scope);
		info.variable_size = (u16)current_scope().entries.size();
		info.variable_identifier_offset = (u16)result_->identifier_table_.size();
		for(uint_t i=0; i<scope.entries.size(); ++i){
			Scope::Entry& entry = scope.entries[i];
			result_->identifier_table_.push_back(entry.name);
		}

		result_->scope_info_table_.push_back(info);
	}
}

void CodeBuilder::scope_end(const ExprPtr& e){
	XTAL_ASSERT(current_scope().expr==e);

	if(scope_exist(&current_scope())){
		put_inst<InstScopeEnd>();
	}
		
	if(current_scope().scope_chain && current_scope().scope_info_num>=0){
		result_->scope_info_table_[current_scope().scope_info_num].flags |= FunInfo::FLAG_SCOPE_CHAIN;
	}

	current_scope().children_pos = 0;
	scope_stack_.pop();
}
	
void CodeBuilder::scope_optimize_begin(const ExprPtr& e){
	if(scope_stack_.empty()){
		scope_stack_.upsize(1);
	}
	else{
		Scope* p = scope_stack_[0];
		scope_stack_.push(p->children[p->children_pos++]);
	}
		
	XTAL_ASSERT(current_scope().expr==e);
	Scope* p = scope_stack_[0];

	for(uint_t i=0; i<p->entries.size(); ++i){
		p->entries[i].visible2 = p->entries[i].visible;
	}
}
	
void CodeBuilder::scope_optimize_end(const ExprPtr& e){
	XTAL_ASSERT(current_scope().expr==e);
	Scope* p = scope_stack_[0];

	for(uint_t i=0; i<p->entries.size(); ++i){
		p->entries[i].visible = p->entries[i].visible2;
	}

	if(p->kind==Scope::FRAME){
		for(uint_t i=0; i<p->entries.size();){
			if(!p->entries[i].refered){
				p->entries.erase(i);
			}
			else{
				++i;
			}
		}
	}

	p->children_pos = 0;
	scope_stack_.pop();
}

CodeBuilder::VariableInfo CodeBuilder::var_find(const IDPtr& key, bool force){
	VariableInfo ret;
	ret.depth = 0;
	ret.member_number = 0;
	ret.register_number = 0;
	ret.found = false;
	ret.out_of_fun= false;
	ret.toplevel= false;
	ret.constant= false;
	ret.entry = 0;

	for(size_t i=0, last=scope_stack_.size(); i<last; ++i){
		Scope& scope = *scope_stack_[i];

		for(size_t j = 0, jlast = scope.entries.size(); j<jlast; ++j){
			Scope::Entry& entry = scope.entries[j];

			if(XTAL_detail_raweq(entry.name, key)){
				if(ret.out_of_fun || entry.visible || force){

					ret.member_number = j;

					if(ret.depth==0 && scope.kind==Scope::CLASS){
						ret.out_of_fun = true;
					}

					ret.constant = entry.constant;

					if(scope.kind==Scope::TOPLEVEL){
						ret.toplevel = true;
					}
					else{
						ret.found = true;
					}

					if(ret.found && !ret.out_of_fun){
						ret.register_number = -calc_variable_offset(scope) + (int_t)j;
					}

					ret.entry = &entry;

					if(ret.out_of_fun && !ret.toplevel){
						scope_chain(i);
					}

					return ret;
				}
			}
		}

		if(scope.kind==Scope::FRAME){
			if(scope_exist(&scope)){
				ret.depth++;
			}
		}
		else if(scope.kind==Scope::FUN){
			ret.out_of_fun = true;
			ret.depth++;
		}
		else if(scope.kind==Scope::TOPLEVEL){

		}
		else{
			ret.depth++;
		}
	}

	return ret;
}

void CodeBuilder::var_refere(const IDPtr& key){
	bool out_of_fun = false;

	for(size_t i = 0, last = scope_stack_.size(); i<last; ++i){
		Scope& scope = *scope_stack_[i];

		for(size_t j = 0, jlast = scope.entries.size(); j<jlast; ++j){
			Scope::Entry& entry = scope.entries[scope.entries.size()-1-j];

			if(XTAL_detail_raweq(entry.name, key)){
				if(out_of_fun || entry.visible){
					entry.refered = true;
					return;
				}
			}
		}

		if(scope.kind==Scope::FUN){
			out_of_fun = true;
		}
	}
}

void CodeBuilder::var_visible(const IDPtr& key, bool visible){
	for(size_t i = 0, last = scope_stack_.size(); i<last; ++i){
		Scope& vf = *scope_stack_[i];

		for(size_t j = 0, jlast = vf.entries.size(); j<jlast; ++j){
			Scope::Entry& entry = vf.entries[vf.entries.size()-1-j];

			if(XTAL_detail_raweq(entry.name, key)){
				entry.visible = visible;
				return;
			}
		}
	}
}

void CodeBuilder::var_begin(const ExprPtr& e, int_t kind){
	Scope* p = new_object_xmalloc<Scope>();
	p->expr = e;
	p->kind = kind;
	p->scope_info_num = -1;
	p->scope_chain = false;
	p->scope_chain_have_possibilities = true;
	p->children_pos = 0;
	p->register_base = 0;
	p->register_max = 0;

	if(!scope_stack_.empty()){
		scope_stack_.top()->children.push_back(p);
	}

	scope_stack_.push(p);
}

void CodeBuilder::var_end(const ExprPtr& e){
	XTAL_ASSERT(scope_stack_.top()->expr==e);
	scope_stack_.downsize(1);
}

void CodeBuilder::var_assign(const IDPtr& name){
	var_refere(name);
}

void CodeBuilder::var_define(const IDPtr& name, bool visible){
	for(size_t j = 0, jlast = current_scope().entries.size(); j<jlast; ++j){
		if(XTAL_detail_raweq(current_scope().entries[current_scope().entries.size()-1-j].name, name)){
			error(Xt("XCE1026")->call(Named(Xid(name), name)));
			return;
		}
	}

	Scope::Entry entry;
	entry.name = name;
	entry.refered = false;
	entry.assigned = false;
	entry.constant = false;
	entry.accessibility = 0;
	entry.visible = visible;
	current_scope().entries.push_back(entry);
}

void CodeBuilder::var_define_class_member(const IDPtr& name, int_t accessibility, bool has_secondary){
	if(!has_secondary){
		for(size_t j = 0, jlast = current_scope().entries.size(); j<jlast; ++j){
			if(XTAL_detail_raweq(current_scope().entries[current_scope().entries.size()-1-j].name, name)){
				error(Xt("XCE1026")->call(Named(Xid(name), name)));
				return;
			}
		}
	}

	Scope::Entry entry;
	entry.name = name;
	entry.refered = false;
	entry.assigned = false;
	entry.constant = true;
	entry.accessibility = accessibility;
	entry.visible = has_secondary;
	current_scope().entries.push_back(entry);
}

int_t CodeBuilder::code_size(){
	return result_->code_.size();
}

void CodeBuilder::compile_comp_bin_assert(const AnyPtr& f, const ExprPtr& e, const ExprPtr& str, const ExprPtr& mes, int_t label, int_t stack_top){
	if(is_comp_bin(e->bin_lhs()) || is_comp_bin(e->bin_rhs())){
		error(Xt("XCE1025"));
	}

	int_t target = stack_top++;
	int_t lhs = compile_expr(e->bin_lhs(), stack_top);
	int_t rhs = compile_expr(e->bin_rhs(), stack_top);
	
	{
		int_t label_false = reserve_label();
		put_if_code(e->itag(), target, lhs, rhs, label, label_false, stack_top);

		int_t label_true = reserve_label();
		put_inst<InstLoadConstant>(target, LOAD_TRUE);

		set_jump(InstGoto::OFFSET_address, label_true);
		put_inst<InstGoto>(0);

		set_label(label_false);
		put_inst<InstLoadValue>(target, LOAD_FALSE);
		set_label(label_true);
	}

	int_t label_true = reserve_label();
	set_jump(InstIf::OFFSET_address_false, label_true);
	set_jump(InstIf::OFFSET_address_true, label);
	put_inst<InstIf>(target, 0, 0);
	set_label(label_true);

	int_t vart = stack_top++;
	put_inst<InstLoadConstant>(vart, register_value(f));

	int_t strt = stack_top++;
	if(str){ compile_expr(str, stack_top, strt); }
	else{ put_inst<InstLoadConstant>(strt, register_value(empty_string)); }
	
	int_t lhst = stack_top++;
	put_inst<InstCopy>(lhst, lhs);

	int_t rhst = stack_top++;
	put_inst<InstCopy>(rhst, rhs);

	int_t mest = stack_top++;
	if(mes){ compile_expr(mes, stack_top, mest); }
	else{ put_inst<InstLoadConstant>(mest, register_value(empty_string)); }

	put_inst<InstCallEx>(target, 1, vart, 0, vart+1, 4, 0, 0);
	put_inst<InstAssert>(target);
}	

void CodeBuilder::put_bin(int_t result, const ExprPtr& e, int_t a, int_t b, int_t stack_top){
	int_t tag = e->itag();
	switch(tag){
		XTAL_NODEFAULT;
		XTAL_CASE(EXPR_ADD){ put_inst<InstAdd>(result, a, b, stack_top, 0); }
		XTAL_CASE(EXPR_SUB){ put_inst<InstSub>(result, a, b, stack_top, 0); }
		XTAL_CASE(EXPR_CAT){ put_inst<InstCat>(result, a, b, stack_top, 0); }
		XTAL_CASE(EXPR_MUL){ put_inst<InstMul>(result, a, b, stack_top, 0); }
		XTAL_CASE(EXPR_DIV){ put_inst<InstDiv>(result, a, b, stack_top, 0); }
		XTAL_CASE(EXPR_MOD){ put_inst<InstMod>(result, a, b, stack_top, 0); }

		XTAL_CASE(EXPR_ADD_ASSIGN){ put_inst<InstAdd>(result, a, b, stack_top, 1); }
		XTAL_CASE(EXPR_SUB_ASSIGN){ put_inst<InstSub>(result, a, b, stack_top, 1); }
		XTAL_CASE(EXPR_CAT_ASSIGN){ put_inst<InstCat>(result, a, b, stack_top, 1); }
		XTAL_CASE(EXPR_MUL_ASSIGN){ put_inst<InstMul>(result, a, b, stack_top, 1); }
		XTAL_CASE(EXPR_DIV_ASSIGN){ put_inst<InstDiv>(result, a, b, stack_top, 1); }
		XTAL_CASE(EXPR_MOD_ASSIGN){ put_inst<InstMod>(result, a, b, stack_top, 1); }

		XTAL_CASE(EXPR_AND){ put_inst<InstAnd>(result, a, b, stack_top, 0); }
		XTAL_CASE(EXPR_OR){ put_inst<InstOr>(result, a, b, stack_top, 0); }
		XTAL_CASE(EXPR_XOR){ put_inst<InstXor>(result, a, b, stack_top, 0); }
		XTAL_CASE(EXPR_SHL){ put_inst<InstShl>(result, a, b, stack_top, 0); }
		XTAL_CASE(EXPR_SHR){ put_inst<InstShr>(result, a, b, stack_top, 0); }
		XTAL_CASE(EXPR_USHR){ put_inst<InstUshr>(result, a, b, stack_top, 0); }

		XTAL_CASE(EXPR_AND_ASSIGN){ put_inst<InstAnd>(result, a, b, stack_top, 1); }
		XTAL_CASE(EXPR_OR_ASSIGN){ put_inst<InstOr>(result, a, b, stack_top, 1); }
		XTAL_CASE(EXPR_XOR_ASSIGN){ put_inst<InstXor>(result, a, b, stack_top, 1); }
		XTAL_CASE(EXPR_SHL_ASSIGN){ put_inst<InstShl>(result, a, b, stack_top, 1); }
		XTAL_CASE(EXPR_SHR_ASSIGN){ put_inst<InstShr>(result, a, b, stack_top, 1); }
		XTAL_CASE(EXPR_USHR_ASSIGN){ put_inst<InstUshr>(result, a, b, stack_top, 1); }
	}
}

void CodeBuilder::compile_op_assign(const ExprPtr& e, int_t stack_top){
	ExprPtr lhs = e->bin_lhs();
	ExprPtr rhs = e->bin_rhs();

	if(lhs->itag()==EXPR_LVAR){
		int_t a = compile_expr(lhs, stack_top);
		int_t b = compile_expr(rhs, stack_top);
		if(a<0){
			put_bin(a, e, a, b, stack_top);
		}
		else{
			put_bin(a, e, a, b, stack_top);
			compile_lassign(a, lhs->lvar_name());
		}
	}
	else if(lhs->itag()==EXPR_IVAR){
		int_t a = compile_expr(lhs, stack_top);
		int_t b = compile_expr(rhs, stack_top);
		int_t target = stack_top++;
		put_bin(target, e, a, b, stack_top);
		put_set_instance_variable_code(target, lhs->ivar_name());
	}
	else if(lhs->itag()==EXPR_PROPERTY){
		AnyPtr sec;

		int_t term = compile_expr(lhs->property_term(), stack_top);

		if(lhs->property_ns()){
			sec = compile_expr(lhs->property_ns(), stack_top);
		}
	
		int_t value = compile_expr(rhs, stack_top);
		int_t target = stack_top++;

		compile_property(term, lhs->property_name(), sec, 0, stack_top, target);
		put_bin(target, e, target, value, stack_top);
		compile_set_property(term, lhs->property_name(), sec, target, 0, stack_top, target, 0);
	}
	else if(lhs->itag()==EXPR_PROPERTY_Q){
		AnyPtr sec;

		int_t term = compile_expr(lhs->property_term(), stack_top);

		if(lhs->property_ns()){
			sec = compile_expr(lhs->property_ns(), stack_top);
		}

		int_t value = compile_expr(rhs, stack_top);
		int_t target = stack_top++;

		compile_property(term, lhs->property_name(), sec, MEMBER_FLAG_Q_BIT, stack_top, target);
		put_bin(target, e, target, value, stack_top);
		compile_set_property(term, lhs->property_name(), sec, target, MEMBER_FLAG_Q_BIT, stack_top, target, 0);
	}
	else if(lhs->itag()==EXPR_AT){
		int_t var = compile_expr(lhs->bin_lhs(), stack_top);
		int_t key = compile_expr(lhs->bin_rhs(), stack_top);
		int_t value = compile_expr(rhs, stack_top);
		int_t target = stack_top++;

		put_inst<InstAt>(target, var, key, stack_top);
		put_bin(value, e, target, value, stack_top);
		put_inst<InstSetAt>(var, key, value, stack_top);
	}
}

void CodeBuilder::put_incdec(const ExprPtr& e, int_t ret, int_t target, int_t stack_top){
	if(e->itag()==EXPR_INC)
		put_inst<InstInc>(ret, target, stack_top);
	else
		put_inst<InstDec>(ret, target, stack_top);
}

void CodeBuilder::compile_incdec(const ExprPtr& e, int_t stack_top){
	ExprPtr term = e->una_term();

	if(term->itag()==EXPR_LVAR){
		int_t a = compile_expr(term, stack_top);
		if(a<0){
			put_incdec(e, a, a, stack_top);
		}
		else{
			put_incdec(e, a, a, stack_top);
			compile_lassign(a, term->lvar_name());
		}
	}
	else if(term->itag()==EXPR_IVAR){
		int_t a = compile_expr(term, stack_top);
		put_incdec(e, a, a, stack_top);
		put_set_instance_variable_code(a, term->ivar_name());
	}
	else if(term->itag()==EXPR_PROPERTY){
		AnyPtr sec;

		int_t nterm = compile_expr(term->property_term(), stack_top);

		if(term->property_ns()){
			sec = compile_expr(term->property_ns(), stack_top);
		}

		int_t target = stack_top++;

		compile_property(nterm, term->property_name(), sec, 0, stack_top, target);
		put_incdec(e, target, target, stack_top);
		compile_set_property(nterm, term->property_name(), sec, target, 0, stack_top, target, 0);
	}
	else if(term->itag()==EXPR_PROPERTY_Q){
		AnyPtr sec;

		int_t nterm = compile_expr(term->property_term(), stack_top);

		if(term->property_ns()){
			sec = compile_expr(term->property_ns(), stack_top);
		}

		int_t target = stack_top++;

		compile_property(nterm, term->property_name(), sec, MEMBER_FLAG_Q_BIT, stack_top, target);
		put_incdec(e, target, target, stack_top);
		compile_set_property(nterm, term->property_name(), sec, target, MEMBER_FLAG_Q_BIT, stack_top, target, 0);
	}
	else if(term->itag()==EXPR_AT){
		int_t var = compile_expr(term->bin_lhs(), stack_top);
		int_t key = compile_expr(term->bin_rhs(), stack_top);
		int_t target = stack_top++;

		put_inst<InstAt>(target, var, key, stack_top);
		put_incdec(e, target, target, stack_top);
		put_inst<InstSetAt>(var, key, target, stack_top);
	}
}

void CodeBuilder::compile_loop_control_statement(const ExprPtr& e){
	IDPtr label;
	int_t label_kind = 0;

	if(e->itag()==EXPR_BREAK){
		label = e->break_label();
		label_kind = 0;
	}
	else if(e->itag()==EXPR_CONTINUE){
		label = e->continue_label();
		label_kind = 1;
	}

	if(ff().loops.empty()){
		error(Xt("XCE1006"));
	}
	else{
		if(label){
			bool found = false;
			for(int_t i = 0, last = ff().loops.size(); i<last; ++i){
				if(XTAL_detail_raweq(ff().loops[i].label, label)){
					break_off(ff().loops[i].frame_count);
					set_jump(InstGoto::OFFSET_address, ff().loops[i].control_statement_label[label_kind]);
					put_inst<InstGoto>(0);
					found = true;
					break;
				}
			}

			if(!found){
				error(Xt("XCE1006"));
			}
		}
		else{
			bool found = false;
			for(size_t i = 0, last = ff().loops.size(); i<last; ++i){
				if(!ff().loops[i].have_label){
					break_off(ff().loops[i].frame_count);
					set_jump(InstGoto::OFFSET_address, ff().loops[i].control_statement_label[label_kind]);
					put_inst<InstGoto>(0);		
					found = true;
					break;
				}
			}

			if(!found){
				error(Xt("XCE1006"));
			}
		}
	}
}

void CodeBuilder::compile_class(const ExprPtr& e, int_t stack_top, int_t result){
	push_ff();

	// 継承
	int_t mixins = 0;
	int_t mixin_base = stack_top;
	XTAL_FOR_EXPR(v, e->class_mixins()){
		compile_expr(v, stack_top+mixins+1, stack_top+mixins);
		mixins++;
	}

	scope_begin(e);

	push_cf();

	int_t ivar_num = 0;
	int_t instance_variable_identifier_offset = result_->identifier_table_.size();
	Xfor_cast(ExprPtr v, e->class_stmts()){
		if(v->itag()==EXPR_CDEFINE_IVAR){
			ClassFrame::Entry entry;
			entry.name = v->cdefine_ivar_name();
			cf().entries.push_back(entry);

			result_->identifier_table_.push_back(entry.name);

			ivar_num++;
		}
	}

	int_t class_info_num = result_->class_info_table_.size();

	ClassInfo info;
	info.pc = code_size();
	info.kind = (u8)e->class_kind();
	info.mixins = (u8)mixins;
	info.variable_size = (u16)current_scope().entries.size();
	info.instance_variable_size = (u16)ivar_num;
	info.instance_variable_identifier_offset = (u16)instance_variable_identifier_offset;
	info.name_number = (u16)register_identifier(e->class_name());
	
	info.variable_offset = 0;
	info.variable_identifier_offset = (u16)result_->identifier_table_.size();

	Xfor_cast(ExprPtr v, e->class_stmts()){
		if(v->itag()==EXPR_CDEFINE_MEMBER){
			result_->identifier_table_.push_back(v->cdefine_member_name());
		}
	}

	//for(uint_t i=0; i<current_scope().entries.size(); ++i){
	//	result_->identifier_table_.push_back(intern(Xid(_)->op_cat(current_scope().entries[i].name)));
	//}

	put_inst<InstClassBegin>(mixin_base, class_info_num);
	result_->class_info_table_.push_back(info);

	{
		int_t number = 0;
		XTAL_FOR_EXPR(v, e->class_stmts()){
			ExprPtr v1 = ep(v);
			if(v1->itag()==EXPR_CDEFINE_MEMBER){		
				compile_expr(v1->cdefine_member_term(), stack_top+1, stack_top);

				int_t inumber = 0;
				if(v1->cdefine_member_ns()){
					compile_expr(v1->cdefine_member_ns(), stack_top+2, stack_top+1);
					inumber = number++;
				}
				else{
					put_inst<InstLoadValue>(stack_top+1, LOAD_UNDEFINED);
					inumber = number++;
				}

				put_inst<InstDefineClassMember>(inumber, 
					register_identifier(v1->cdefine_member_name()), stack_top+1,
					v1->cdefine_member_accessibility()->to_i(), stack_top);
			}
		}
	}

	if(scope_stack_.top()->scope_chain){
		result_->class_info_table_[class_info_num].flags |= FunInfo::FLAG_SCOPE_CHAIN;
	}

	put_inst<InstClassEnd>(result);

	pop_cf();
	scope_end(e);
	pop_ff();
}

int_t CodeBuilder::compile_fun(const ExprPtr& e, int_t stack_top, int_t result){
	int_t ordered = 0;
	int_t named = 0;

	Xfor_cast(ExprPtr v, e->fun_params()){
		if(!v->at(0) || (ep(v->at(0))->itag()!=EXPR_LVAR && ep(v->at(0))->itag()!=EXPR_IVAR)){
			error(Xt("XCE1004"));
			return int_t();
		}

		if(v->at(1)){
			named++;
		}
		else{
			if(named!=0){
				error(Xt("XCE1005"));
			}

			ordered++;
		}
	}

	int_t minv = ordered;
	int_t maxv = ordered + named;

	// ゲッター、またはセッター用の最適化を行う
	//if(e->fun_kind()==KIND_METHOD)
	{

		// ゲッタか？
		if(ordered==0 && named==0){
			ExprPtr body = e->fun_body();
			if(body->itag()==EXPR_SCOPE && body->scope_stmts() && body->scope_stmts()->size()==1){
				body = ep(body->scope_stmts()->front());
			}

			if(body->itag()==EXPR_RETURN){
				if(body->return_exprs() && body->return_exprs()->size()==1){
					body = ep(body->return_exprs()->front());
					if(body->itag()==EXPR_IVAR){
						scope_skip();
						put_inst<InstMakeInstanceVariableAccessor>(result, 0, lookup_instance_variable(body->ivar_name()), class_info_num());
						return 1;
					}
				}
			}
		}

		// セッタか？
		if(ordered==1 && named==0){
			ExprPtr body = e->fun_body();
			if(body->itag()==EXPR_SCOPE && body->scope_stmts() && body->scope_stmts()->size()==1){
				body = ep(body->scope_stmts()->front());
			}

			if(body->itag()==EXPR_ASSIGN){
				ExprPtr lhs = body->bin_lhs();
				ExprPtr rhs = body->bin_rhs();

				IDPtr key;
				ExprPtr arg = ep(e->fun_params()->at(0));
				if(arg){
					arg = ep(arg->at(0));
				}

				if(arg->itag()==EXPR_LVAR){
					key = arg->lvar_name();
				}
				else if(arg->itag()==EXPR_IVAR){
					key = arg->ivar_name();
				}

				if(key && lhs->itag()==EXPR_IVAR && rhs->itag()==EXPR_LVAR && XTAL_detail_raweq(rhs->lvar_name(), key)){
					scope_skip();
					put_inst<InstMakeInstanceVariableAccessor>(result, 1, lookup_instance_variable(lhs->ivar_name()), class_info_num());
					return 1;
				}
			}
		}
	}

	// 関数フレームを作成する
	push_ff();

	// 関数コアを作成
	FunInfo fun;
	fun.pc = code_size() + InstMakeFun::ISIZE;
	fun.kind = (u8)e->fun_kind();
	fun.min_param_count = (u8)minv;
	fun.max_param_count = (u8)maxv;
	fun.name_number = (u16)register_identifier(e->fun_name());
	fun.flags = e->fun_extendable_param() ? (u16)FunInfo::FLAG_EXTENDABLE_PARAM : (u16)0;

	scope_begin(e);

	fun.max_variable = (u16)current_scope().register_max;

	// 引数の名前を識別子テーブルに順番に乗せる
	fun.variable_offset = (u16)calc_variable_offset(current_scope());
	fun.variable_size = (u16)current_scope().entries.size();
	fun.variable_identifier_offset = (u16)result_->identifier_table_.size();
	for(uint_t i=0; i<current_scope().entries.size(); ++i){
		result_->identifier_table_.push_back(current_scope().entries[i].name);
	}

	int_t fun_info_table_number = result_->xfun_info_table_.size();
	result_->xfun_info_table_.push_back(fun);

	int_t fun_end_label = reserve_label();
	set_jump(InstMakeFun::OFFSET_address, fun_end_label);
	put_inst<InstMakeFun>(result, fun_info_table_number, 0);

	// デフォルト値を持つ引数を処理する
	{
		int_t i = 0;
		Xfor_cast(ExprPtr v1, e->fun_params()){
			if(const ExprPtr& v = ptr_cast<Expr>(v1->at(1))){
				int_t label_true = reserve_label();
				int_t label_false = reserve_label();

				VariableInfo var = var_find(current_scope().entries[i].name, true);
				
				set_jump(InstIfUndefined::OFFSET_address_true, label_true);
				set_jump(InstIfUndefined::OFFSET_address_false, label_false);
				put_inst<InstIfUndefined>(var.register_number, 0, 0);
				set_label(label_true);

				compile_expr(v, stack_top+1, var.register_number);
								
				set_label(label_false);
			}
			else{
				var_visible(current_scope().entries[i].name, true);
			}	

			++i;
		}
	}

	// 引数にインスタンス変数がある場合に、特別な処理を入れる
	Xfor_cast(ExprPtr v1, e->fun_params()){
		const ExprPtr& v = ep(v1->at(0));
		if(v->itag()==EXPR_IVAR){
			eb_->tree_push_back(v);
			eb_->tree_push_back(v->ivar_name());
			eb_->tree_splice(EXPR_LVAR, 1);
			eb_->tree_splice(EXPR_ASSIGN, 2);
			compile_stmt(eb_->tree_pop_back());
		}
	}

	// 関数本体を処理する
	compile_stmt(e->fun_body());
	
	break_off(ff().var_frame_count+1);

	put_inst<InstReturn>(0, 0);
	set_label(fun_end_label);

	process_labels();

	if(current_scope().scope_chain){
		result_->xfun_info_table_[fun_info_table_number].flags |= FunInfo::FLAG_SCOPE_CHAIN;
	}

	scope_end(e);
	pop_ff();

	return 1;
}

AnyPtr CodeBuilder::do_expr(const AnyPtr& p){
	if(!p){
		return null;
	}

	ExprPtr e = ep(p);

	switch(e->itag()){

		XTAL_DEFAULT;

		XTAL_CASE(EXPR_NULL){ return null; }
		XTAL_CASE(EXPR_TRUE){ return true; }
		XTAL_CASE(EXPR_FALSE){ return false; }
		XTAL_CASE(EXPR_NUMBER){ return e->number_value(); }
		XTAL_CASE(EXPR_STRING){
			if(e->string_kind()==KIND_STRING){
				return e->string_value();
			}
		}

		XTAL_CASE(EXPR_LVAR){
			/*
			VariableInfo var = var_find(e->lvar_name());
			if(var.is_register()){
				
			}

			if(info.pos>=0){
				if((var_frame(info).fun_frames_size==fun_frames_.size() || !entry(info).assigned)){
					entry(info).referenced = true;
					return entry(info).value;
				}
			}
			*/
		}
	}

	return undefined;
}

}

#endif

