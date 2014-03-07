#include "xtal.h"
#include "xtal_macro.h"

namespace xtal{

VMachine::VMachine(){
	exit_fiber_ = false;
	end_code_ = InstExit::NUMBER;
	throw_code_ = InstThrow::NUMBER;
	resume_pc_ = 0;

	variables_top_ = 0;
	XTAL_VM_set_variables_top(0);
//	stack_.reserve(4);

	ready();

	parent_vm_ = 0;

	thread_yield_count_ = 1000;

	static uint_t dummy = 0;
	hook_setting_bit_ = &dummy;
}

VMachine::~VMachine(){
	variables_.clear();
	for(int_t i=0, size=fun_frame_stack_.capacity(); i<size; ++i){
		if(FunFrame* p = fun_frame_stack_.reverse_at_unchecked(i)){
			delete_object_xfree<FunFrame>(p);
		}
	}

	while(scopes_.size()){
		pop_scope();
	}
}

void VMachine::reset(){
	exit_fiber_ = false;
	stack_.resize(0);
	except_frames_.resize(0);
	fun_frame_stack_.resize(0);

	ready();
	
	while(scopes_.size()){
		pop_scope();
	}

	XTAL_VM_set_variables_top(0);

	except_[0] = null;
	except_[1] = null;
	except_[2] = null;
	debug_info_ = null;

	parent_vm_ = 0;

	for(int_t i=0, size=(int)variables_.size(); i<size; ++i){
		variables_.set_at(i, null);
	}
}

void VMachine::push_arg(const AnyPtr& value){
	XTAL_ASSERT(XTAL_VM_ff().named_arg_count==0); //順番指定引数は 名前指定引数の前だけ

	set_local_variable(XTAL_VM_ff().ordered_arg_count, value);
	XTAL_VM_ff().ordered_arg_count++;
}
	
void VMachine::push_arg(const IDPtr& name, const AnyPtr& value){
	XTAL_ASSERT(XTAL_VM_ff().ordered_arg_count==0); // 名前指定引数は順番指定引数の後だけ

	int_t offset = XTAL_VM_ff().ordered_arg_count + XTAL_VM_ff().named_arg_count*2;
	set_local_variable(offset+0, name);
	set_local_variable(offset+1, value);
	XTAL_VM_ff().named_arg_count++;
}

void VMachine::push_ordered_args(const ArrayPtr& p){ 
	Xfor(v, p){
		push_arg(v);
	}
}

void VMachine::push_named_args(const MapPtr& p){ 
	Xfor2(k, v, p){
		push_arg(ptr_cast<ID>(k), v);
	}
}
	
void VMachine::insert_arg(int_t index, const AnyPtr& value){
	for(int i=XTAL_VM_ff().ordered_arg_count+XTAL_VM_ff().named_arg_count*2 - 1; i>index; --i){
		set_local_variable(i, local_variable(i-1));
	}

	set_local_variable(index, value);
	XTAL_VM_ff().ordered_arg_count++;
}

void VMachine::push_arg(const char_t* s){ push_arg(AnyPtr(s)); }
void VMachine::push_arg(const LongLivedString& s){ push_arg(AnyPtr(s)); }
void VMachine::push_arg(char v){ push_arg(Int(v)); }
void VMachine::push_arg(signed char v){ push_arg(Int(v)); }
void VMachine::push_arg(unsigned char v){ push_arg(Int(v)); }
void VMachine::push_arg(short v){ push_arg(Int(v)); }
void VMachine::push_arg(unsigned short v){ push_arg(Int(v)); }
void VMachine::push_arg(int v){ push_arg(Int(v)); }
void VMachine::push_arg(unsigned int v){ push_arg(Int(v)); }
void VMachine::push_arg(long v){ push_arg(Int((int_t)v)); }
void VMachine::push_arg(unsigned long v){ push_arg(Int((int_t)v)); }
void VMachine::push_arg(long long v){ push_arg(Int((int_t)v)); }
void VMachine::push_arg(unsigned long long v){ push_arg(Int((int_t)v)); }
void VMachine::push_arg(float v){ push_arg(Float(v)); }
void VMachine::push_arg(double v){ push_arg(Float((float_t)v)); }
void VMachine::push_arg(long double v){ push_arg(Float((float_t)v)); }
void VMachine::push_arg(bool v){ push_arg(Bool(v)); }
	
void VMachine::push_arg(const ArgumentsPtr& args){
	push_args(args, 0, XTAL_VM_ff().ordered_arg_count, XTAL_VM_ff().named_arg_count);
	XTAL_VM_ff().ordered_arg_count += args->ordered_size();
	XTAL_VM_ff().named_arg_count += args->named_size();
}

void VMachine::ready(){
	FunFrame& f = *push_ff_simple();
	f.is_executed = 0;
	f.need_result_count = 0;
	f.ordered_arg_count = 0;
	f.named_arg_count = 0;
	f.next_pc = 0;
	f.poped_pc = &end_code_;
	f.result = 0;
	f.prev_stack_base = 0;
	f.scope_lower = scopes_.size();
	f.self = undefined;
	f.fun = null;
	f.outer = null;

	result_base_ = f.result;
}

void VMachine::setup_call(int_t need_result_count){
	int_t base;
	FunFrame& f = XTAL_VM_ff();
	if(f.fun){
		base = 128;
	}
	else{
		base = f.ordered_arg_count+f.named_arg_count*2+1;
	}


	CallState call_state;
	call_state.aself = undefined;
	call_state.poped_pc = &end_code_;
	call_state.result = base;
	call_state.need_result_count = need_result_count;
	call_state.stack_base = base;
	call_state.ordered_arg_count = 0;
	call_state.named_arg_count = 0;
	push_ff(call_state);	
}

void VMachine::replace_result(const AnyPtr& result){
	FunFrame& f = XTAL_VM_ff();

	if(f.is_executed==2){
		set_local_variable(result_base_, result);
		return;
	}

	const inst_t* temp = f.poped_pc;
	f.poped_pc = &end_code_;

	execute();

	if(!except()){
		XTAL_VM_ff().next_pc = temp;
		set_local_variable(result_base_, result);
	}
}
	
void VMachine::execute(Method* fun, const inst_t* start_pc){
	setup_call(0);
	carry_over(fun);
	FunFrame& f = XTAL_VM_ff();
	f.poped_pc = &end_code_;
	execute_inner(start_pc ? start_pc : f.next_pc);
}

void VMachine::execute(){
	FunFrame& f = XTAL_VM_ff();

	// 既に実行されている
	if(f.is_executed==2){
		return;
	}

	// 関数も戻り値も設定されていない
	if(f.is_executed==0){
		return;
	}

	// 例外がクリアされていないなら実行できない
	if(except_[0]){
		return;
	}

	execute_inner(f.next_pc);
}
	
const AnyPtr& VMachine::result(int_t pos){
	execute();
	return XTAL_VM_local_variable(result_base_+pos);
}
	
const AnyPtr& VMachine::result_and_cleanup_call(int_t pos){
	execute();
	XTAL_VM_ff().is_executed = 0;
	return XTAL_VM_local_variable(result_base_+pos);
}
	
void VMachine::cleanup_call(){
	execute();
	XTAL_VM_ff().is_executed = 0;
}

const AnyPtr& VMachine::arg(int_t pos){
	FunFrame& f = XTAL_VM_ff();
	if(pos<f.ordered_arg_count)
		return XTAL_VM_local_variable(pos);
	return undefined;
}

const AnyPtr& VMachine::arg(int_t pos, const IDPtr& name){
	FunFrame& f = XTAL_VM_ff();
	if(pos<f.ordered_arg_count){
		return XTAL_VM_local_variable(pos);
	}
	return arg(name);
}

const AnyPtr& VMachine::arg_default(int_t pos, const AnyPtr& def){
	FunFrame& f = XTAL_VM_ff();
	if(pos<f.ordered_arg_count){
		return XTAL_VM_local_variable(pos);
	}
	return def;
}
	
const AnyPtr& VMachine::arg_default(const IDPtr& name, const AnyPtr& def){
	FunFrame& f = XTAL_VM_ff();
	for(int_t i = 0, sz = f.named_arg_count; i<sz; ++i){
		if(XTAL_detail_raweq(XTAL_VM_local_variable(f.ordered_arg_count + i*2 + 0), name)){
			return XTAL_VM_local_variable(f.ordered_arg_count + i*2 + 1);
		}
	}
	return def;
}
	
const AnyPtr& VMachine::arg_default(int_t pos, const IDPtr& name, const AnyPtr& def){
	FunFrame& f = XTAL_VM_ff();
	if(pos<f.ordered_arg_count){
		return XTAL_VM_local_variable(pos);
	}
	return arg_default(name, def);
}

bool VMachine::adjust_args(const NamedParam* params, int_t num){
	FunFrame& f = XTAL_VM_ff();
	int_t base = f.ordered_arg_count + f.named_arg_count*2;
	int_t offset = base;
	
	char argcheck[138]; // バイトコードの仕様からいって128を超えることはありえない
	for(int_t i=0, sz=f.named_arg_count; i<sz; ++i){
		argcheck[i] = 0;
	}

	for(int_t j=f.ordered_arg_count; j<num; ++j){
		bool hit = false;
		for(int_t i = 0, sz = f.named_arg_count; i<sz; ++i){
			// 名前付き引数に同じ名前のものがあった
			if(XTAL_detail_raweq(XTAL_VM_local_variable(f.ordered_arg_count + i*2 + 0), params[j].name)){
				argcheck[i] = 1;
				set_local_variable(offset++, XTAL_VM_local_variable(f.ordered_arg_count + i*2 + 1));
				hit = true;
				break;
			}
		}

		if(!hit){
			set_local_variable(offset++, params[j].value);
		}
	}

	for(int_t i=0, sz=f.named_arg_count; i<sz; ++i){
		if(argcheck[i]==0){
			set_except(cpp_class<ArgumentError>()->call(
				Xt2("XRE1036",
					object, f.fun->object_name(),
					name, XTAL_VM_local_variable(f.ordered_arg_count + i*2 + 0)
				)
			));
			return false;
		}
	}

	int_t top = XTAL_VM_variables_top();
	variables_.move(top + f.ordered_arg_count, base + top, offset-base);
	return true;
}

bool VMachine::adjust_args(Method* names){
	FunFrame& f = XTAL_VM_ff();
	int_t num = names->param_size();
	int_t base = f.ordered_arg_count + f.named_arg_count*2;
	int_t offset = base;

	char argcheck[138]; // バイトコードの仕様からいって128を超えることはありえない
	for(int_t i=0, sz=f.named_arg_count; i<sz; ++i){
		argcheck[i] = 0;
	}

	for(int_t j=f.ordered_arg_count; j<num; ++j){
		bool hit = false;
		for(int_t i = 0, sz = f.named_arg_count; i<sz; ++i){
			if(XTAL_detail_raweq(XTAL_VM_local_variable(f.ordered_arg_count + i*2 + 0), names->param_name_at(j))){
				argcheck[i] = 1;
				set_local_variable(offset++, XTAL_VM_local_variable(f.ordered_arg_count + i*2 + 1));
				hit = true;
				break;
			}
		}

		if(!hit){
			set_local_variable(offset++, undefined);
		}
	}

	for(int_t i=0, sz=f.named_arg_count; i<sz; ++i){
		if(argcheck[i]==0){
			set_except(cpp_class<ArgumentError>()->call(
				Xt2("XRE1036",
					object, f.fun->object_name(),
					name, XTAL_VM_local_variable(f.ordered_arg_count + i*2 + 0)
				)
			));
			return false;
		}
	}

	int_t top = XTAL_VM_variables_top();
	variables_.move(top + f.ordered_arg_count, base + top, offset-base);
	return true;
}

void VMachine::return_result(){
	XTAL_ASSERT(!is_executed());
	pop_ff(0, 0);
}

void VMachine::return_result(const AnyPtr& value1){
	XTAL_ASSERT(!is_executed());
	if(!XTAL_detail_is_undefined(value1)){
		set_local_variable(0, value1);
		pop_ff(0, 1);
	}
	else{
		pop_ff(0, 0);
	}
}
	
void VMachine::return_result(const AnyPtr& value1, const AnyPtr& value2){
	XTAL_ASSERT(!is_executed());
	set_local_variable(0, value1);
	set_local_variable(1, value2);
	pop_ff(0, 2);
}

void VMachine::return_result(const AnyPtr& value1, const AnyPtr& value2, const AnyPtr& value3){
	XTAL_ASSERT(!is_executed());
	set_local_variable(0, value1);
	set_local_variable(1, value2);
	set_local_variable(2, value3);
	pop_ff(0, 3);
}
	
void VMachine::return_result(const AnyPtr& value1, const AnyPtr& value2, const AnyPtr& value3, const AnyPtr& value4){
	XTAL_ASSERT(!is_executed());
	set_local_variable(0, value1);
	set_local_variable(1, value2);
	set_local_variable(2, value3);
	set_local_variable(3, value4);
	pop_ff(0, 4);
}

void VMachine::return_result_mv(const ValuesPtr& values){
	XTAL_ASSERT(!is_executed());
	int_t size = values->size();
	for(int_t i=0; i<size; ++i){
		set_local_variable(i, values->at(i));
	}
	pop_ff(0, size);
}

void VMachine::return_result(const char_t* s){ return_result(xnew<String>(s)); }
void VMachine::return_result(const LongLivedString& s){ return_result(xnew<String>(s)); }
void VMachine::return_result(const IDPtr& s){ return_result(unchecked_ptr_cast<Any>(s)); }

void VMachine::return_result(char v){ return_result(Int(v)); }
void VMachine::return_result(signed char v){ return_result(Int(v)); }
void VMachine::return_result(unsigned char v){ return_result(Int(v)); }
void VMachine::return_result(short v){ return_result(Int(v)); }
void VMachine::return_result(unsigned short v){ return_result(Int(v)); }
void VMachine::return_result(int v){ return_result(Int(v)); }
void VMachine::return_result(unsigned int v){ return_result(Int(v)); }
void VMachine::return_result(long v){ return_result(Int(v)); }
void VMachine::return_result(unsigned long v){ return_result(Int(v)); }
void VMachine::return_result(long long v){ return_result(Int((int_t)v)); }
void VMachine::return_result(unsigned long long v){ return_result(Int((int_t)v)); }

void VMachine::return_result(float v){ return_result(Float(v)); }
void VMachine::return_result(double v){ return_result(Float((float_t)v)); }
void VMachine::return_result(long double v){ return_result(Float((float_t)v)); }
void VMachine::return_result(bool v){ return_result(Bool(v)); }

void VMachine::present_for_vm(Fiber* fun, VMachine* vm, bool add_succ_or_fail_result){
	if(const AnyPtr& e = catch_except()){
		vm->set_except_x(e);
		vm->stack_.push(e);
		vm->fun_frame_stack_.top()->next_pc = &throw_code_;
		resume_pc_ = 0;
		return;
	}

	// 結果をvmに渡す
	int_t base;
	if(add_succ_or_fail_result){
		if(resume_pc_!=0){
			vm->set_local_variable(0, to_smartptr(fun));
		}
		else{
			vm->set_local_variable(0, null);
		}
		base = 1;
	}
	else{
		base = 0;
	}

	for(int_t i=0; i<yield_result_count_; ++i){
		vm->set_local_variable(base+i, XTAL_VM_local_variable(yield_base_+i));
	}	

	vm->pop_ff(0, base + yield_result_count_);
}

const inst_t* VMachine::start_fiber(Fiber* fun, VMachine* vm, bool add_succ_or_fail_result){
	yield_result_count_ = 0;

	CallState call_state;
	call_state.aself = vm->arg_this();
	call_state.poped_pc = &end_code_;
	call_state.result = XTAL_VM_variables_top();
	call_state.need_result_count = vm->need_result_count();
	call_state.stack_base = 0;
	call_state.ordered_arg_count = vm->ordered_arg_count();
	call_state.named_arg_count = vm->named_arg_count();
	push_ff(call_state);
	
	move_variables(vm, vm->ordered_arg_count()+vm->named_arg_count()*2);

	resume_pc_ = 0;

	carry_over(fun);
	XTAL_VM_ff().yieldable = true;

	parent_vm_ = vm;
	execute_inner(XTAL_VM_ff().next_pc);
	parent_vm_ = 0;
	//XTAL_ASSERT(resume_pc_);
	present_for_vm(fun, vm, add_succ_or_fail_result);
	return resume_pc_;
}

const inst_t* VMachine::resume_fiber(Fiber* fun, const inst_t* pc, VMachine* vm, bool add_succ_or_fail_result){
	yield_result_count_ = 0;
	XTAL_VM_ff().next_pc = pc;
	resume_pc_ = 0;

	if(yield_need_result_count_!=0){
		vm->adjust_values2(0, vm->ordered_arg_count()+vm->named_arg_count()*2, yield_need_result_count_);

		for(int_t i=0; i<yield_need_result_count_; ++i){
			set_local_variable(yield_result_+i, vm->local_variable(i));
		}
	}

	parent_vm_ = vm;
	execute_inner(XTAL_VM_ff().next_pc);
	parent_vm_ = 0;
	present_for_vm(fun, vm, add_succ_or_fail_result);
	return resume_pc_;
}

void VMachine::exit_fiber(){
	exit_fiber_ = true;
	yield_result_count_ = 0;
	XTAL_VM_ff().next_pc = &throw_code_;
	resume_pc_ = 0;

	stack_.push(undefined);
	execute_inner(&throw_code_);

	reset();
}

const inst_t* VMachine::check_accessibility(CallState& call_state, int_t accessibility){
	if(accessibility & KIND_PRIVATE){
		if(!XTAL_detail_raweq(ap(call_state.aself)->get_class(), ap(call_state.acls))){
			return push_except(call_state.pc, cpp_class<AccessibilityError>()->call(Xt4("XRE1017",
				object, ap(call_state.acls)->object_name(), 
				name, ap(call_state.aprimary), 
				secondary_key, ap(call_state.asecondary), 
				accessibility, Xid(private)))
			);
		}
	}
	else if(accessibility & KIND_PROTECTED){
		if(!ap(call_state.aself)->is(ap(call_state.acls))){
			return push_except(call_state.pc, cpp_class<AccessibilityError>()->call(Xt4("XRE1017",
				object, ap(call_state.acls)->object_name(), 
				primary_key, ap(call_state.aprimary), 
				secondary_key, ap(call_state.asecondary), 
				accessibility, Xid(protected)))
			);
		}
	}
	return 0;
}

void VMachine::push_args(const ArgumentsPtr& args, int_t stack_base, int_t ordered_arg_count, int_t named_arg_count){
	uint_t baseplus = args->ordered_size();
	if(!named_arg_count){
		for(uint_t i = 0; i<baseplus; ++i){
			set_local_variable(stack_base+ordered_arg_count+i, args->op_at_int(i));
		}
	}
	else{
		int_t offset = named_arg_count*2;
		for(int_t i = 0; i<offset; ++i){
			set_local_variable(stack_base+ordered_arg_count+i+baseplus, XTAL_VM_local_variable(ordered_arg_count+i));
		}

		for(uint_t i = 0; i<baseplus; ++i){
			set_local_variable(stack_base+ordered_arg_count+i, args->op_at_int(i));
		}
	}

	int_t i=0;
	Xfor2(key, value, args->named_arguments()){
		set_local_variable(stack_base+baseplus+ordered_arg_count+named_arg_count*2+i*2+0, key);
		set_local_variable(stack_base+baseplus+ordered_arg_count+named_arg_count*2+i*2+1, value);
		++i;
	}
}

ArgumentsPtr VMachine::make_arguments(int_t lower){
	ArgumentsPtr p = xnew<Arguments>();
	int_t osize = ordered_arg_count();
	for(int_t i = lower; i<osize; ++i){
		p->add_ordered(arg(i));
	}

	for(int_t i = 0, size = named_arg_count(); i<size; ++i){
		p->add_named(XTAL_VM_local_variable(osize+i*2+0), XTAL_VM_local_variable(osize+i*2+1));
	}
	return p;
}

bool VMachine::has_arguments(){
	return ordered_arg_count()!=0 || named_arg_count()!=0;
}

ArgumentsPtr VMachine::inner_make_arguments(const NamedParam* params, int_t num){
	ArgumentsPtr p = xnew<Arguments>();

	int_t param_size =num;

	for(int_t i = param_size, size = XTAL_VM_ff().ordered_arg_count; i<size; ++i){
		p->add_ordered(XTAL_VM_local_variable(i));
	}

	for(int_t i = 0, size = XTAL_VM_ff().named_arg_count; i<size; ++i){
		IDPtr name = unchecked_ptr_cast<ID>(XTAL_VM_local_variable(XTAL_VM_ff().ordered_arg_count+i*2+0));
		bool found = false;
		for(int_t j = 0; j<param_size; ++j){
			if(XTAL_detail_raweq(params[j].name, name)){
				found = true;
				break;
			}
		}

		if(!found){
			p->add_named(name, XTAL_VM_local_variable(XTAL_VM_ff().ordered_arg_count+i*2+1));
		}
	}

	return p;
}

ArgumentsPtr VMachine::inner_make_arguments(Method* fun){
	ArgumentsPtr p = xnew<Arguments>();

	int_t param_size = fun->param_size();

	for(int_t i = param_size, size = XTAL_VM_ff().ordered_arg_count; i<size; ++i){
		p->add_ordered(XTAL_VM_local_variable(i));
	}

	for(int_t i = 0, size = XTAL_VM_ff().named_arg_count; i<size; ++i){
		IDPtr name = unchecked_ptr_cast<ID>(XTAL_VM_local_variable(XTAL_VM_ff().ordered_arg_count+i*2+0));
		bool found = false;
		for(int_t j = 0; j<param_size; ++j){
			if(XTAL_detail_raweq(fun->param_name_at(j), name)){
				found = true;
				break;
			}
		}

		if(!found){
			p->add_named(name, XTAL_VM_local_variable(XTAL_VM_ff().ordered_arg_count+i*2+1));
		}
	}

	return p;
}

AnyPtr VMachine::append_backtrace(const inst_t* pc, const AnyPtr& e){
	if(e){
		AnyPtr ep = e;
		if(!ep->is(cpp_class<Exception>())){
			ep = cpp_class<RuntimeError>()->call(ep);
		}

		if(MethodPtr fun = XTAL_VM_ff().fun){
			if(CodePtr code = fun->code()){
				//if((pc !=  code->bytecode_data() + code->bytecode_size()-1)){
				if((code->bytecode_data() <= pc && pc < code->bytecode_data()+code->bytecode_size()-1)){
					unchecked_ptr_cast<Exception>(ep)->append_backtrace(
						code->source_file_name(),
						code->compliant_lineno(pc),
						fun->object_name());
				}
			}
		}
		return ep;
	}
	return e;
}

void VMachine::set_except_0(const AnyPtr& e){
	except_[0] = e;
}
	
debug::CallerInfoPtr VMachine::caller(uint_t n){
	//if(n==0){
	//	return nul<debug::CallerInfo>();
	//}

	if((int_t)n>call_stack_size()){
		return nul<debug::CallerInfo>();
	}

	if(n>=fun_frame_stack_.size()-1){
		if(parent_vm_){
			return parent_vm_->caller(n-(fun_frame_stack_.size()-1));
		}
		return nul<debug::CallerInfo>();
	}

	FunFrame& f = *fun_frame_stack_[n];
	FunFrame& pf = n==0 ? f : *fun_frame_stack_[n-1];
	int_t scope_lower = n==0 ? 0 : fun_frame_stack_[n-1]->scope_lower;

	debug::CallerInfoPtr ret = xnew<debug::CallerInfo>();

	if(!f.fun){
		ret->set_line(0);
		ret->set_fun(nul<Fun>());
		return ret;
	}

	int_t line = f.fun->code()->compliant_lineno(pf.poped_pc-1);
	ret->set_line(line);
	ret->set_fun(f.fun);

	make_outer_outer(0, 0, true);
	ret->set_variables_frame(scopes_.reverse_at(scope_lower==0 ? 0 : scope_lower-1).frame);
	return ret;
}

int_t VMachine::call_stack_size(){
	int_t n = parent_vm_ ? parent_vm_->call_stack_size()-1 : -1; 
	return n + fun_frame_stack_.size()-1;
}

AnyPtr VMachine::eval_local_variable(const IDPtr& var, uint_t call_n){
	if(call_n<fun_frame_stack_.size()-1){
		int_t scope_upper = fun_frame_stack_[call_n-1]->scope_lower;
		int_t scope_lower = fun_frame_stack_[call_n]->scope_lower;

		FramePtr frame;
		for(int_t i=0; i<scope_upper - scope_lower; ++i){
			frame = scopes_.reverse_at(scope_upper-i-1).frame;
			const AnyPtr& ret = frame->member(var);
			if(!XTAL_detail_is_undefined(ret)){
				return ret;
			}
		}

		frame = fun_frame_stack_[call_n]->outer;
		while(frame){
			const AnyPtr& ret = frame->member(var);
			if(!XTAL_detail_is_undefined(ret)){
				return ret;
			}

			frame = frame->outer();
		}

		if(fun_frame_stack_[call_n]->fun){
			const AnyPtr& ret = fun_frame_stack_[call_n]->code->member(var);
			if(!XTAL_detail_is_undefined(ret)){
				return ret;
			}
		}
	}
	else{
		if(parent_vm_){
			return parent_vm_->eval_local_variable(var, call_n-(fun_frame_stack_.size()-1));
		}
	}
		
	return undefined;
}

bool VMachine::eval_set_local_variable(const IDPtr& var, const AnyPtr& value, uint_t call_n){
	if(call_n<fun_frame_stack_.size()-1){

		int_t scope_upper = fun_frame_stack_[call_n-1]->scope_lower;
		int_t scope_lower = fun_frame_stack_[call_n]->scope_lower;

		FramePtr frame;
		for(int_t i=0; i<scope_upper - scope_lower; ++i){
			frame = scopes_.reverse_at(scope_upper-i-1).frame;

			if(frame->replace_member(var, value)){
				return true;
			}
		}

		frame = fun_frame_stack_[call_n]->outer;
		while(frame){
			if(frame->replace_member(var, value)){
				return true;
			}

			frame = frame->outer();
		}

		if(fun_frame_stack_[call_n]->fun){
			if(fun_frame_stack_[call_n]->code->replace_member(var, value)){
				return true;
			}
		}
	}
	else{
		if(parent_vm_){
			return parent_vm_->eval_set_local_variable(var, value, call_n-(fun_frame_stack_.size()-1));
		}
	}
		
	return false;
}

AnyPtr VMachine::eval_instance_variable(const AnyPtr& self, const IDPtr& key){
	if(XTAL_detail_type(self)!=TYPE_BASE){
		return undefined;
	}

	ClassPtr klass = self->get_class();
	ArrayPtr ary = klass->ancestors()->to_a();
	ary->push_back(klass);

	if(InstanceVariables* iv = XTAL_detail_pvalue(self)->instance_variables()){
		Xfor_cast(ClassPtr p, ary){
			if(!p->is(cpp_class<Code>())){
				if(const CodePtr& code = p->code()){
					ClassInfo* info = p->info();
					for(uint_t i=0; i<info->instance_variable_size; ++i){
						if(XTAL_detail_raweq(key, code->identifier(info->instance_variable_identifier_offset+i))){
							return iv->variable(i, info);
						}
					}
				}
			}
		}
	}
	return undefined;
}

bool VMachine::eval_set_instance_variable(const AnyPtr& self, const IDPtr& key, const AnyPtr& value){
	if(XTAL_detail_type(self)!=TYPE_BASE){
		return false;
	}

	ClassPtr klass = self->get_class();
	ArrayPtr ary = klass->ancestors()->to_a();
	ary->push_back(klass);

	Xfor_cast(ClassPtr p, ary){
		if(InstanceVariables* iv = XTAL_detail_pvalue(self)->instance_variables()){
			if(CodePtr code = p->code()){
				ClassInfo* info = p->info();
				for(uint_t i=0; i<info->instance_variable_size; ++i){
					if(XTAL_detail_raweq(key, code->identifier(info->instance_variable_identifier_offset+i))){
						iv->set_variable(i, info, value);
						return true;
					}
				}
			}
		}
	}
	return false;
}

void VMachine::breakpoint_hook(const inst_t* pc, const MethodPtr& fun, int_t kind){
	if(!debug_info_){ 
		debug_info_ = XNew<debug::HookInfo>(); 
	}

	debug_info_->set_kind(kind);
	debug_info_->set_fun(fun, pc);
	debug_info_->set_exception(except_[0]);
	debug_info_->set_vm(to_smartptr(this));
	debug_info_->set_variables_frame(null);

	if(kind>=BREAKPOINT_LINE_PROFILE && kind<=BREAKPOINT_CALL_PROFILE){

	}
	else if(kind>=BREAKPOINT_LINE_LIGHT_WEIGHT && kind<=BREAKPOINT_CALL_LIGHT_WEIGHT){

	}
	else{
		if(!scopes_.empty()){
			make_outer_outer(0, 0, true);
			debug_info_->set_variables_frame(scopes_.top().frame);
		}
	}

	VMachinePtr oldvm = set_vmachine(vmachine_take_over());
	debug::call_breakpoint_hook(kind, debug_info_);
	vmachine_take_back(set_vmachine(oldvm));
}

void VMachine::pop_ff_non(){
	FunFrame& f = *fun_frame_stack_.top();

	f.fun = null;
	f.outer = null;
	f.self = null;

	pop_ff_simple();
	current_fun_frame_->is_executed = 2;
	current_fun_frame_->next_pc = f.poped_pc; 
}

const inst_t* VMachine::catch_body(const inst_t* pc, const ExceptFrame& nef){
	AnyPtr e = catch_except();

	ExceptFrame ef;
	if(except_frames_.empty() || except_frames_.top().fun_frame_size<nef.fun_frame_size){
		ef = nef;
	}
	else{
		ef = except_frames_.top();
	}

	// Xtalの関数を脱出していく
	while((size_t)ef.fun_frame_size<fun_frame_stack_.size()){
		while(scopes_.size()>XTAL_VM_ff().scope_lower){
			pop_scope();
		}

		check_breakpoint_hook(pc, BREAKPOINT_RETURN);
		pop_ff_non();
		pc = XTAL_VM_ff().next_pc;
		e = append_backtrace(pc, e);

		// Cの関数にぶつかった
		if(pc==&end_code_){
			ef.info = 0;
			break;
		}
	}

	//XTAL_ASSERT(scopes_.size()>=ef.scope_size);

	while(scopes_.size()<ef.scope_size){
		push_scope();
	}

	while(scopes_.size()>ef.scope_size){
		pop_scope();
	}

	stack_.resize(ef.stack_size);
	XTAL_VM_set_variables_top(ef.variables_top);

	if(ef.info){
		CodePtr code = XTAL_VM_ff().fun->code();
		if(ef.info->catch_pc && e){
			pc = ef.info->catch_pc +  code->bytecode_data();
			stack_.push(AnyPtr(ef.info->end_pc));
			stack_.push(e);
		}
		else{
			pc = ef.info->finally_pc +  code->bytecode_data();
			stack_.push(e);
			stack_.push(AnyPtr(code->bytecode_size()-1));
			const inst_t* pcode = code->bytecode_data() + (code->bytecode_size()-1);
			pcode = pcode;
		}

		except_frames_.downsize(1);
		return pc;
	}
	else{
		set_except_0(e);
	}

	while(scopes_.size()>XTAL_VM_ff().scope_lower){
		pop_scope();
	}

	pop_ff_non();

	return 0;
}

const AnyPtr& VMachine::catch_except(){
	except_[2] = except();
	except_[0] = null;
	return except_[2];
}

void VMachine::set_except(const AnyPtr& e){
	if(!except_[0]){
		except_[0] = e;
	}
	else{
		//XTAL_ASSERT(false); // 例外をハンドルせずに次の例外を設定した
	}
}

void VMachine::set_except_x(const AnyPtr& e){
	if(!except_[0]){
		except_[0] = e;
	}
}

const inst_t* VMachine::push_except(const inst_t* pc){
	stack_.push(except_[0]);
	except_[0] = null;
	throw_pc_ = pc+1;
	return &throw_code_;
}
	
const inst_t* VMachine::push_except(const inst_t* pc, const AnyPtr& e){
	stack_.push(e);
	except_[0] = null;
	throw_pc_ = pc+1;
	return &throw_code_;
}

void VMachine::on_visit_members(Visitor& m){
	Base::on_visit_members(m);
	m & debug_info_ & except_[0] & except_[1] & except_[2];

	for(int_t i=0, size=stack_.size(); i<size; ++i){
		m & stack_[i];
	}

	for(int_t i=0, size=fun_frame_stack_.size(); i<size; ++i){
		if(FunFrame* f = fun_frame_stack_[i]){
			m & f->fun;
			m & f->self;
			m & f->outer;
		}
	}

	for(int_t i=0, size=scopes_.size(); i<size; ++i){
		m & scopes_[i].frame;
	}

	for(int_t i=0, size=variables_.size(); i<size; ++i){
		m & variables_.at(i);
	}
}

void VMachine::print_info(){
#ifdef XTAL_DEBUG_PRINT
	std::printf("stack size %d\n", stack_.size());
	for(uint_t i=0; i<stack_.size(); ++i){
//		std::printf("\tstack value %d = %s\n", i, ap(stack_[i])->to_s()->c_str());
	}

	std::printf("fun_frames size %d\n", fun_frames_.size());
	std::printf("except_frames size %d\n", except_frames_.size());
	std::printf("scopes size %d\n", scopes_.size());
#endif
}

AnyPtr VMachine::eval(const CodePtr& code, uint_t n){
	if(!code){
		return undefined;
	}

	debug::CallerInfoPtr cp = caller(n);
	if(!cp || !cp->fun()){
		return undefined;
	}

	AnyPtr self = fun_frame_stack_[n]->self;

	// eval中の例外の発生による影響範囲抑制のため、関数呼び出しを一段はさむ
	setup_call(1); 
	n += 1;

	setup_call(1);
	set_arg_this(self);
	code->first_fun()->rawcall(to_smartptr(this)); //evalコンパイルされたCodeを実行

	execute_inner(XTAL_VM_ff().next_pc, n+1);

	XTAL_VM_ff().is_executed = 0;

	return_result(XTAL_VM_local_variable(result_base_+0));

	return result_and_cleanup_call();
}

}//namespace

