#include "xtal.h"
#include "xtal_bind.h"
#include "xtal_macro.h"
#include "xtal_stringspace.h"

namespace xtal{

namespace{

void filelocal_check_implicit_lookup(const AnyPtr& a){
	if(const CodePtr& code = ptr_cast<Code>(a)){	// C³ by ¯‚­‚¸”Þ•û‚É
		code->check_implicit_lookup();
	}
}

void Class_inherit(const ClassPtr& cls, const ClassPtr cls2){
	cls->inherit(cls2);
}

}

Code::Code(){
	enable_redefine_ = false;

	set_singleton();
	set_object_temporary_name(XTAL_DEFINED_ID(filelocal));
	set_object_force(500);
}

Code::~Code(){

}

void Code::reload(const CodePtr& new_code){
	if(!new_code){
		return;
	}

	new_code->enable_redefine();
	new_code->call();

	overwrite(new_code);

	code_ = new_code->code_;

	identifier_table_ = new_code->identifier_table_;
	value_table_ = new_code->value_table_;
	once_table_ = new_code->once_table_;
	
	source_file_name_ = new_code->source_file_name_;
	first_fun_ = new_code->first_fun_;

	breakpoint_cond_map_ = new_code->breakpoint_cond_map_;

	xfun_info_table_ = new_code->xfun_info_table_;
	scope_info_table_ = new_code->scope_info_table_;
	class_info_table_ = new_code->class_info_table_;
	except_info_table_ = new_code->except_info_table_;

	lineno_table_ = new_code->lineno_table_;

	implicit_table_ = new_code->implicit_table_;
}

void Code::generated(){
	set_code(to_smartptr(this));
	if(scope_info_table_.size()>1){
		ClassInfo cinfo;
		(ScopeInfo&)cinfo = scope_info_table_[1];
		class_info_table_.push_back(cinfo);

		set_info((ClassInfo*)&class_info_table_.back());
		resize_member_direct(class_info_table_.back().variable_size);

		make_members_force(Frame::FLAG_NOCACHE);
	}

	inherit(builtin());
	
	XTAL_BIND_DIRECT(this){
		Xdef_method_alias(inherit, &Class_inherit);
		Xdef_method_alias(check_implicit_lookup, &filelocal_check_implicit_lookup);
		def(XTAL_DEFINED_ID(filelocal), to_smartptr(this));
	}

	first_fun_ = XNew<Method>(nul<Frame>(), to_smartptr(this), &xfun_info_table_[0]);
}

void Code::add_breakpoint(int_t lineno, const AnyPtr& cond){
	for(uint_t i=0, sz=lineno_table_.size(); i<sz; ++i){
		LineNumberInfo& info = lineno_table_[i];
		if(info.lineno==lineno && (code_[info.start_pc] == InstLine::NUMBER || code_[info.start_pc] == InstBreakPoint::NUMBER)){
			code_[info.start_pc] = InstBreakPoint::NUMBER;

			if(cond){
				if(!breakpoint_cond_map_){
					breakpoint_cond_map_ = xnew<Map>();
				}
				breakpoint_cond_map_->set_at(lineno, cond);
			}
		}
	}
}

void Code::remove_breakpoint(int_t lineno){
	for(uint_t i=0, sz=lineno_table_.size(); i<sz; ++i){
		LineNumberInfo& info = lineno_table_[i];
		if(info.lineno==lineno && code_[info.start_pc] == InstBreakPoint::NUMBER){
			code_[info.start_pc] = InstLine::NUMBER;

			if(breakpoint_cond_map_){
				breakpoint_cond_map_->erase(lineno);
			}
		}
	}
}
		
CodePtr Code::breakpoint_cond(int_t lineno){
	if(breakpoint_cond_map_){
		return ptr_cast<Code>(breakpoint_cond_map_->at(lineno));
	}
	return nul<Code>();
}

bool Code::set_lineno_info(uint_t line){
	if(lineno_table_.empty()){

	}
	else{
		if(lineno_table_.back().lineno==line || lineno_table_.back().start_pc==(u32)code_.size()){
			return false;
		}
	}

	LineNumberInfo lnt = {(u32)code_.size(), (u16)line};
	lineno_table_.push_back(lnt);
	return true;
}
	
int_t Code::final_lineno(){
	if(lineno_table_.empty()){
		return 0;
	}
	return lineno_table_.back().lineno;
}

int_t Code::compliant_lineno(const inst_t* p){
	if(LineNumberInfo* lni = compliant_lineno_info(p)){
		return lni->lineno;
	}
	return 0;
}

const inst_t* Code::compliant_pc(int_t lineno){
	int_t nearv = 0xffffff;
	int_t neari = -1;
	for(uint_t i=0; i<lineno_table_.size(); ++i){
		int_t diff = lineno_table_[i].lineno - lineno;
		if(diff>=0 && diff<nearv){
			nearv = diff;
			neari = i;
		}
	}

	return neari<0 ? 0 : bytecode_data() + lineno_table_[neari].start_pc;
}

Code::LineNumberInfo* Code::compliant_lineno_info(const inst_t* p){
	if(!lineno_table_.empty()){
		LineNumberInfo* begin = &lineno_table_[0];
		LineNumberInfo* end = begin+lineno_table_.size();
		LineNumberInfo* it=
			std::upper_bound(
				begin,
				end,
				static_cast<int_t>(p-bytecode_data()),
				LineNumberCmp()
			);

		if(it!=end){
			if(it==begin){
				return &*it;
			}
			--it;
			return &*it;
		}
	}

	return 0;
}

void Code::on_rawcall(const VMachinePtr& vm){
	if(enable_redefine_){
		debug::enable_redefine();
	}

	if(vm->has_arguments()){
		overwrite_member(XTAL_DEFINED_ID(arg), vm->make_arguments());
		vm->setup_call();
		vm->set_arg_this(to_smartptr(this));
		first_fun_->rawcall(vm);
		vm->return_result(vm->result_and_cleanup_call());
	}
	else{
		if(enable_redefine_){
			vm->setup_call();
			vm->set_arg_this(to_smartptr(this));
			first_fun_->rawcall(vm);
			vm->return_result(vm->result_and_cleanup_call());
		}
		else{
			vm->set_arg_this(to_smartptr(this));
			first_fun_->rawcall(vm);
		}
	}

	if(enable_redefine_){
		debug::disable_redefine();
	}
}

void Code::find_near_variable_inner(const IDPtr& primary_key, const ScopeInfo& info, IDPtr& pick, int_t& minv){
	for(uint_t j=0; j<info.variable_size; ++j){
		IDPtr id = identifier(info.variable_identifier_offset+j);
		int_t dist = edit_distance(primary_key, id);
		if(dist!=0 && dist<minv){
			minv = dist;
			pick = id;
		}
	}
}
	
IDPtr Code::find_near_variable(const IDPtr& primary_key){
	int_t minv = 0xffffff;
	IDPtr pick;

	if(const ClassPtr& klass = filelocal()){
		pick = klass->find_near_member2(primary_key, undefined, minv);
	}

	for(uint_t i=0; i<xfun_info_table_.size(); ++i){
		find_near_variable_inner(primary_key, xfun_info_table_[i], pick, minv);
	}

	for(uint_t i=0; i<scope_info_table_.size(); ++i){
		find_near_variable_inner(primary_key, scope_info_table_[i], pick, minv);
	}

	for(uint_t i=0; i<class_info_table_.size(); ++i){
		find_near_variable_inner(primary_key, class_info_table_[i], pick, minv);
	}

	return pick;
}

void Code::check_implicit_lookup(){
	ArrayPtr ary;
	for(uint_t i=0; i<implicit_table_.size(); ++i){
		const IDPtr& id = unchecked_ptr_cast<ID>(identifier_table_.at(implicit_table_[i].id));
		const AnyPtr& ret = member(id);
		if(XTAL_detail_raweq(undefined, ret)){
			if(!ary){
				ary = XNew<Array>();
			}

			ary->push_back(Xf2("%s(%d)", 0, filelocal_unsupported_error(to_smartptr(this), id), 1, implicit_table_[i].lineno));
		}
	}

	if(ary){
		set_runtime_error(Xt1("XRE1030", name, ary));
	}
}

StringPtr Code::inspect(){

#ifdef XTAL_DEBUG

	MemoryStreamPtr ms = xnew<MemoryStream>();

	ms->put_s(XTAL_STRING("identifier_table\n"));
	for(uint_t i=0; i<identifier_table_.size(); ++i){
		Xf("\t%04d:%s\n")->call(i, identifier_table_.at(i));
		ms->put_s(Xf("\t%04d:%s\n")->call(i, identifier_table_.at(i))->to_s());
	}

	ms->put_s(XTAL_STRING("value_table\n"));
	for(uint_t i=0; i<value_table_.size(); ++i){
		ms->put_s(Xf("\t%04d:%s\n")->call(i, value_table_.at(i))->to_s());
	}
	
	ms->put_s(XTAL_STRING("\n"));
	ms->put_s(inspect_range(0, bytecode_size()));

	ms->seek(0);
	return ms->get_s(ms->size());
	
#else

	return empty_string;

#endif
}

StringPtr Code::inspect_range(int_t start, int_t end){
	return xtal::inspect_range(to_smartptr(this), bytecode_data()+start, bytecode_data()+end);
}

}
