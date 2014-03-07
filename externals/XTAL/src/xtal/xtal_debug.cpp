#include "xtal.h"
#include "xtal_macro.h"
#include "xtal_stringspace.h"

namespace xtal{ namespace debug{

StringPtr CallerInfo::file_name(){ 
	return fun_ ? fun_->code()->source_file_name() : StringPtr(XTAL_STRING("C++ Function")); 
}

StringPtr CallerInfo::fun_name(){ 
	return fun_ ? fun_->object_name() : empty_string; 
}

void CallerInfo::on_visit_members(Visitor& m){
	Base::on_visit_members(m);
	m & fun_ & variables_frame_;
}

void HookInfo::on_visit_members(Visitor& m){
	Base::on_visit_members(m);
	m & /*vm_ & */ exception_ & variables_frame_ & fun_;
}

int_t HookInfo::lineno(){
	if(fun_){ return fun_->code()->compliant_lineno(pc_); }
	return 0;
}

CodePtr HookInfo::code(){
	if(fun_){ return fun_->code(); }
	return nul<Code>();
}

StringPtr HookInfo::file_name(){
	if(fun_){ return fun_->code()->source_file_name(); }
	return XTAL_DEFINED_ID(UNKNOWN);
}

StringPtr HookInfo::fun_name(){
	if(fun_){ return fun_->code()->object_name(); }
	return XTAL_DEFINED_ID(UNKNOWN);
}

SmartPtr<HookInfo> HookInfo::clone(){
	SmartPtr<HookInfo> ret = xnew<HookInfo>();
	ret->kind_ = kind_;
	ret->fun_ = fun_;
	ret->pc_ = pc_;
	ret->exception_ = exception_;
	ret->variables_frame_ = variables_frame_;
	ret->vm_ = vm_;
	return ret;
}
		
void HookInfo::set_vm(const VMachinePtr& v){ 
	vm_ = &*v; 
}
	
const VMachinePtr& HookInfo::vm(){ 
	return to_smartptr(vm_); 
}

CallerInfoPtr HookInfo::caller(uint_t n){
	return vm_->caller(n);
}

int_t HookInfo::call_stack_size(){
	return vm_->call_stack_size();
}

enum{
	BSTATE_NONE,
	BSTATE_GO,
	BSTATE_STEP_OVER,
	BSTATE_STEP_INTO,
	BSTATE_STEP_OUT,
	BSTATE_STEP_OUT2
};

class DebugData : public Base{
public:
	DebugData(){
		redefine_count_ = 0;
		enable_count_ = 0;
		hook_setting_bit_ = 0;
		saved_hook_setting_bit_ = 0;
		breakpoint_state_ = BSTATE_GO;
		debug_compile_count_ = 0;
	}

	uint_t redefine_count_;
	uint_t enable_count_;
	uint_t hook_setting_bit_;
	uint_t saved_hook_setting_bit_;

	AnyPtr hooks_[BREAKPOINT_MAX];

	int_t breakpoint_state_;
	int_t breakpoint_call_stack_size_;

	uint_t debug_compile_count_;
};

namespace{

void bitchange(const SmartPtr<DebugData>& d, bool b, int_t type){
	if(b){
		d->hook_setting_bit_ |= 1<<type;
	}
	else{
		d->hook_setting_bit_ &= ~(1<<type);
	}
}

}

void enable(){
	const SmartPtr<DebugData>& d = cpp_value<DebugData>();
	if(d->enable_count_==0){
		d->saved_hook_setting_bit_ = d->hook_setting_bit_;
	}
	d->enable_count_++;
}

void disable(){
	const SmartPtr<DebugData>& d = cpp_value<DebugData>();
	d->enable_count_--;

	if(d->enable_count_<=0){
		d->saved_hook_setting_bit_ = d->hook_setting_bit_;
		d->hook_setting_bit_ = 0;
	}
}

bool is_enabled(){
	const SmartPtr<DebugData>& d = cpp_value<DebugData>();
	return d->enable_count_>0;
}

void enable_force(uint_t count){
	const SmartPtr<DebugData>& d = cpp_value<DebugData>();
	if(d->enable_count_==0){
		d->hook_setting_bit_ = d->saved_hook_setting_bit_;
	}
	d->enable_count_ = count;
}

uint_t disable_force(){
	const SmartPtr<DebugData>& d = cpp_value<DebugData>();
	int_t temp = d->enable_count_;
	d->enable_count_ = 0;

	if(d->enable_count_<=0){
		d->saved_hook_setting_bit_ = d->hook_setting_bit_;
		d->hook_setting_bit_ = 0;
	}

	return temp;
}

bool is_debug_compile_enabled(){
	const SmartPtr<DebugData>& d = cpp_value<DebugData>();
	return d->debug_compile_count_>0;
}

void enable_debug_compile(){
	const SmartPtr<DebugData>& d = cpp_value<DebugData>();
	d->debug_compile_count_++;
}

void disable_debug_compile(){
	const SmartPtr<DebugData>& d = cpp_value<DebugData>();
	d->debug_compile_count_--;
}

bool is_redefine_enabled(){
	const SmartPtr<DebugData>& d = cpp_value<DebugData>();
	return d->redefine_count_>0;
}

void enable_redefine(){
	const SmartPtr<DebugData>& d = cpp_value<DebugData>();
	d->redefine_count_++;
}

void disable_redefine(){
	const SmartPtr<DebugData>& d = cpp_value<DebugData>();
	d->redefine_count_--;
}


uint_t hook_setting_bit(){
	const SmartPtr<DebugData>& d = cpp_value<DebugData>();
	return d->hook_setting_bit_;
}

uint_t* hook_setting_bit_ptr(){
	const SmartPtr<DebugData>& d = cpp_value<DebugData>();
	return &d->hook_setting_bit_;
}

void set_hook(int_t hooktype, const AnyPtr& hook){
	const SmartPtr<DebugData>& d = cpp_value<DebugData>();
	d->hooks_[hooktype] = hook;
	bitchange(d, hook, hooktype);
}

const AnyPtr& hook(int_t hooktype){
	const SmartPtr<DebugData>& d = cpp_value<DebugData>();
	return d->hooks_[hooktype];
}

void set_breakpoint_hook(const AnyPtr& hook){
	set_hook(BREAKPOINT, hook);
}

void set_call_hook(const AnyPtr& hook){
	set_hook(BREAKPOINT_CALL, hook);
}

void set_return_hook(const AnyPtr& hook){
	set_hook(BREAKPOINT_RETURN, hook);
}

void set_throw_hook(const AnyPtr& hook){
	set_hook(BREAKPOINT_THROW, hook);
}

void set_assert_hook(const AnyPtr& hook){
	set_hook(BREAKPOINT_ASSERT, hook);
}

const AnyPtr& breakpoint_hook(){
	return hook(BREAKPOINT);
}

const AnyPtr& call_hook(){
	return hook(BREAKPOINT_CALL);
}

const AnyPtr& return_hook(){
	return hook(BREAKPOINT_RETURN);
}

const AnyPtr& throw_hook(){
	return hook(BREAKPOINT_THROW);
}

const AnyPtr& assert_hook(){
	return hook(BREAKPOINT_ASSERT);
}

void call_breakpoint_hook(int_t kind, HookInfoPtr info){
	const SmartPtr<DebugData>& d = cpp_value<DebugData>();

	if(kind>=BREAKPOINT_LINE_LIGHT_WEIGHT && kind<=BREAKPOINT_CALL_LIGHT_WEIGHT){
		d->hooks_[kind]->call(info);
		return;
	}

	if(kind>=BREAKPOINT_LINE_PROFILE && kind<=BREAKPOINT_CALL_PROFILE){
		d->hooks_[kind]->call(info);
		return;
	}

	AnyPtr hook = (kind>=BREAKPOINT && kind<=BREAKPOINT_INNER_CALL) ? d->hooks_[BREAKPOINT] : d->hooks_[kind];

	while(true){
		switch(d->breakpoint_state_){
			XTAL_NODEFAULT;

			XTAL_CASE(BSTATE_NONE){
				int_t count = disable_force();
				AnyPtr hookret = AnyPtr(0);

				if(hook){
					hookret = hook->call(info);
				}

				int_t ret = XTAL_detail_is_ivalue(hookret) ? hookret->to_i() : -1;
				enable_force(count);

				switch(ret){
					XTAL_DEFAULT{

					}

					XTAL_CASE(RUN){
						d->breakpoint_state_ = BSTATE_NONE;
						bitchange(d, false, BREAKPOINT_INNER_LINE);
						bitchange(d, false, BREAKPOINT_INNER_RETURN);
						bitchange(d, false, BREAKPOINT_INNER_CALL);
					}

					XTAL_CASE(STEP_OVER){
						d->breakpoint_state_ = BSTATE_STEP_OVER;
						bitchange(d, true, BREAKPOINT_INNER_LINE);
						bitchange(d, true, BREAKPOINT_INNER_RETURN);
						bitchange(d, true, BREAKPOINT_INNER_CALL);
					}

					XTAL_CASE(STEP_INTO){
						d->breakpoint_state_ = BSTATE_STEP_INTO;
						bitchange(d, true, BREAKPOINT_INNER_LINE);
						bitchange(d, true, BREAKPOINT_INNER_RETURN);
						bitchange(d, false, BREAKPOINT_INNER_CALL);
					}

					XTAL_CASE(STEP_OUT){
						d->breakpoint_state_ = BSTATE_STEP_OUT;
						bitchange(d, true, BREAKPOINT_INNER_LINE);
						bitchange(d, true, BREAKPOINT_INNER_RETURN);
						bitchange(d, false, BREAKPOINT_INNER_CALL);
					}

					XTAL_CASE(REDO){
						return;
					}
				}

				d->breakpoint_call_stack_size_ = info->call_stack_size();
			}

			XTAL_CASE(BSTATE_GO){
				d->breakpoint_state_ = BSTATE_NONE;
				continue;
			}

			XTAL_CASE(BSTATE_STEP_OVER){
				if(kind==BREAKPOINT_INNER_LINE || kind==BREAKPOINT_INNER_RETURN || kind==BREAKPOINT){
					d->breakpoint_state_ = BSTATE_NONE;
					continue;
				}

				if(kind==BREAKPOINT_INNER_CALL){
					d->breakpoint_state_ = BSTATE_STEP_OUT2;
					bitchange(d, true, BREAKPOINT_INNER_LINE);
					bitchange(d, true, BREAKPOINT_INNER_RETURN);
					bitchange(d, false, BREAKPOINT_INNER_CALL);
					break;
				}
			}

			XTAL_CASE(BSTATE_STEP_INTO){
				d->breakpoint_state_ = BSTATE_NONE;
				continue;
			}

			XTAL_CASE(BSTATE_STEP_OUT){
				if(kind==BREAKPOINT || info->call_stack_size() < d->breakpoint_call_stack_size_){
					d->breakpoint_state_ = BSTATE_NONE;
					continue;
				}
			}

			XTAL_CASE(BSTATE_STEP_OUT2){
				if(kind==BREAKPOINT_INNER_RETURN && info->call_stack_size() == d->breakpoint_call_stack_size_){
					break;
				}

				if(kind==BREAKPOINT || info->call_stack_size() <= d->breakpoint_call_stack_size_){
					d->breakpoint_state_ = BSTATE_NONE;
					continue;
				}
			}
		}
		break;
	}
}


CommandReceiver::~CommandReceiver(){
	set_breakpoint_hook(null);
	remove_require_source_hook(require_hook_);
	set_hook(BREAKPOINT_LINE_LIGHT_WEIGHT, null);
	set_hook(BREAKPOINT_LINE, null);
	require_hook_ = null;
}

bool CommandReceiver::start(const StreamPtr& stream){
	stream_ = stream;

	set_hook(BREAKPOINT_LINE_LIGHT_WEIGHT, bind_this(method(&CommandReceiver::check), undeleter(this)));

	set_breakpoint_hook(bind_this(method(&CommandReceiver::breakpointhook), undeleter(this)));

	require_hook_ = bind_this(method(&CommandReceiver::require_source_hook), undeleter(this));
	append_require_source_hook(require_hook_);

	eval_exprs_ = xnew<Map>();
	code_map_ = xnew<Map>();
	pause_ = false;

	while(ArrayPtr cmd = recv_command()){
		AnyPtr type = cmd->at(0);
		if(XTAL_detail_raweq(type, Xid(start))){
			break;
		}
		exec_command(cmd);
	}

	return true;
}
	
void CommandReceiver::check(HookInfoPtr){
	update();
	if(pause_){
		set_hook(BREAKPOINT_LINE, bind_this(method(&CommandReceiver::breakpointhook), undeleter(this)));
	}
}

void CommandReceiver::update(){
	// 次のコマンドが到着していたらコマンドをデシリアライズして実行する
	while(stream_->available()){
		exec_command(ptr_cast<Array>(stream_->deserialize()));
	}
}

ArrayPtr CommandReceiver::recv_command(){
	return ptr_cast<Array>(stream_->deserialize());
}

CodePtr CommandReceiver::require_source_hook(const StringPtr& name){
	ArrayPtr a = xnew<Array>();
	a->push_back(Xid(require));
	a->push_back(name);
	stream_->serialize(a);

	CodePtr ret;
	while(ArrayPtr cmd = recv_command()){
		AnyPtr type = cmd->at(0);
		if(XTAL_detail_raweq(type, Xid(required_source))){
			ret = ptr_cast<Code>(cmd->at(1));
			if(ret){
				code_map_->set_at(ret->source_file_name(), ret);
			}
			else{
				break;
			}

			continue;
		}

		if(XTAL_detail_raweq(type, Xid(start))){
			break;
		}

		exec_command(cmd);
	}

	return ret;
}

void CommandReceiver::exec_command(const ArrayPtr& cmd){
	if(!cmd){
		return;
	}

	AnyPtr type = cmd->at(0);

	if(XTAL_detail_raweq(type, Xid(add_breakpoint))){ // ブレークポイントの追加
		if(CodePtr code = ptr_cast<Code>(code_map_->at(cmd->at(1)))){
			code->add_breakpoint(cmd->at(2)->to_i(), cmd->at(3));
		}

		return;
	}

	if(XTAL_detail_raweq(type, Xid(remove_breakpoint))){ // ブレークポイントの削除
		if(CodePtr code = ptr_cast<Code>(code_map_->at(cmd->at(1)))){
			code->remove_breakpoint(cmd->at(2)->to_i());
		}

		return;
	}

	if(XTAL_detail_raweq(type, Xid(add_eval_expr))){ // 評価式の追加
		eval_exprs_->set_at(cmd->at(1), cmd->at(2));
		return;
	}

	if(XTAL_detail_raweq(type, Xid(remove_eval_expr))){ // 評価式の削除
		eval_exprs_->erase(cmd->at(1));
		return;
	}

	if(XTAL_detail_raweq(type, Xid(pause))){ // 一時停止
		pause_ = true;
		return;
	}
}

ArrayPtr CommandReceiver::make_debug_object(const AnyPtr& v, int depth){
	ArrayPtr ret = xnew<Array>(3);
	ret->set_at(0, v->get_class()->to_s());
	ret->set_at(1, v->to_s());

	// 基本型かチェック
	switch(XTAL_detail_type(v)){
		case TYPE_NULL:
		case TYPE_UNDEFINED:
		case TYPE_INT:
		case TYPE_FLOAT:
		case TYPE_FALSE:
		case TYPE_TRUE:
		case TYPE_SMALL_STRING:
		case TYPE_LONG_LIVED_STRING:
		case TYPE_INTERNED_STRING:
		case TYPE_STRING:
			return ret;
	}

	if(depth<=0){
		ret->set_at(2, "...");
		return ret;
	}

	switch(XTAL_detail_type(v)){
		XTAL_DEFAULT{}

		XTAL_CASE(TYPE_ARRAY){
			ArrayPtr children = xnew<Array>();
			Xfor(it, v){
				children->push_back(make_debug_object(it, depth-1));
			}
			ret->set_at(2, children);
			return ret;
		}

		XTAL_CASE(TYPE_VALUES){
			ArrayPtr children = xnew<Array>();
			Xfor(it, v){
				children->push_back(make_debug_object(it, depth-1));
			}
			ret->set_at(2, children);
			return ret;
		}
	}

	if(const MapPtr& a = ptr_cast<Map>(v)){
		MapPtr children = xnew<Map>();
        Xfor2(key, val, a){
			children->set_at(key->to_s(), make_debug_object(val, depth-1));
		}
		ret->set_at(2, children);
		return ret;
	}

	if(const ClassPtr& a = ptr_cast<Class>(v)){
		MapPtr children = xnew<Map>();
		Xfor3(key, skey, val, a->members()){
			children->set_at(key->to_s(), make_debug_object(val, depth-1));
		}
		ret->set_at(2, children);
		return ret;
	}

	AnyPtr data = v->s_save();
	if(const MapPtr& a = ptr_cast<Map>(data)){
		MapPtr children = xnew<Map>();
		Xfor2(key, val, a){
			Xfor2(key2, val2, val){
				children->set_at(key2->to_s(), make_debug_object(val2, depth-1));
			}
		}
		ret->set_at(2, children);
		return ret;
	}

	return ret;
}

ArrayPtr CommandReceiver::make_call_stack_info(HookInfoPtr info){
	ArrayPtr ret = xnew<Array>();

	{
		ArrayPtr record = xnew<Array>(3);
		record->set_at(0, info->fun_name());
		record->set_at(1, info->file_name());
		record->set_at(2, info->lineno());
		ret->push_back(record);
	}

	for(int i=2; i<info->call_stack_size(); ++i){
		ArrayPtr record = xnew<Array>(3);
		if(CallerInfoPtr caller = info->caller(i)){
			record->set_at(0, caller->fun_name());
			record->set_at(1, caller->file_name());
			record->set_at(2, caller->lineno());
		}
		ret->push_back(record);
	}

	return ret;
}

MapPtr CommandReceiver::make_eval_expr_info(HookInfoPtr info, int level){
	MapPtr ret = xnew<Map>();
	Xfor2(key, value, eval_exprs_){
		if(CodePtr code = ptr_cast<Code>(value)){
			AnyPtr val = info->vm()->eval(code, level);
			
			if(AnyPtr e = info->vm()->catch_except()){
				ret->set_at(key, null);
			}
			else{
				ret->set_at(key, make_debug_object(val));
			}
		}
		else{
			ret->set_at(key, null);
		}
	}

	return ret;
}

void CommandReceiver::send_break(HookInfoPtr info, int level){
	ArrayPtr data = xnew<Array>();
	data->push_back(Xid(break));
	data->push_back(make_eval_expr_info(info, level));
	data->push_back(make_call_stack_info(info));
	data->push_back(level);
	stream_->serialize(data);
}

int CommandReceiver::breakpointhook(HookInfoPtr info){
	int ret = breakpointhook2(info);

	if(pause_){
		set_hook(BREAKPOINT_LINE, null);
		pause_ = false;
	}

	return ret;
}

int CommandReceiver::breakpointhook2(HookInfoPtr info){
	if(!pause_){
		if(info->kind()==BREAKPOINT){
			if(CodePtr code = ptr_cast<Code>(code_map_->at(info->file_name()))){
				if(code->breakpoint_cond(info->lineno())){
					AnyPtr val = info->vm()->eval(code->breakpoint_cond(info->lineno()), 0);
					if(!info->vm()->catch_except() && !val){
						return REDO;
					}
				}
			}
		}
	}

	int level = 0;
	send_break(info, level);
	while(true){
		if(ArrayPtr cmd = ptr_cast<Array>(stream_->deserialize())){
			AnyPtr type = cmd->at(0);

			if(XTAL_detail_raweq(type, Xid(move_callstack))){
				level = cmd->at(1)->to_i();
				send_break(info, level);
				continue;
			}

			if(XTAL_detail_raweq(type, Xid(nostep))){
				send_break(info, level);
				continue;
			}

			if(XTAL_detail_raweq(type, Xid(run))){
				return RUN;
			}

			if(XTAL_detail_raweq(type, Xid(step_into))){
				return STEP_INTO;
			}

			if(XTAL_detail_raweq(type, Xid(step_over))){
				return STEP_OVER;
			}

			if(XTAL_detail_raweq(type, Xid(step_out))){
				return STEP_OUT;
			}

			exec_command(cmd);
		}
		else{
			return RUN;
		}
	}
}


////////////////////////////////////////////////////////

CommandSender::CommandSender(){
	level_ = 0;
	prev_command_ = Xid(run);
	exprs_ = xnew<Map>();
	breakpoints_ = xnew<Array>();
}

void CommandSender::add_eval_expr(const StringPtr& expr){
    MapPtr a = xnew<Map>();
	exprs_->set_at(expr, null);

	add_eval_expr_inner(expr);
}

void CommandSender::add_eval_expr_inner(const StringPtr& expr){
    if(!stream_ || stream_->eos()){
		return;
	}

	if(CodePtr eval_code = eval_compile(expr)){
	    ArrayPtr a = xnew<Array>();
		a->push_back(Xid(add_eval_expr));
	    a->push_back(expr);
		a->push_back(eval_code);
	    stream_->serialize(a);
	}
	else{
		XTAL_CATCH_EXCEPT(e){
		
		}
	}
}

void CommandSender::remove_eval_expr(const StringPtr& expr){
	exprs_->erase(expr);

	remove_eval_expr_inner(expr);
}

void CommandSender::remove_eval_expr_inner(const StringPtr& expr){
    if(!stream_ || stream_->eos()){
		return;
	}

    ArrayPtr a = xnew<Array>();
    a->push_back(Xid(remove_eval_expr));
    a->push_back(expr->intern());
    stream_->serialize(a);
}

ArrayPtr CommandSender::eval_expr_result(const StringPtr& expr){
	return ptr_cast<Array>(exprs_->at(expr));
}

int CommandSender::call_stack_size(){
	return call_stack_.size();
}

StringPtr CommandSender::call_stack_fun_name(int n){
	return call_stack_[n].fun_name;
}

StringPtr CommandSender::call_stack_file_name(int n){
	return call_stack_[n].file_name;
}

int CommandSender::call_stack_lineno(int n){
	return call_stack_[n].lineno;
}

StringPtr CommandSender::call_stack_fun_name(){
	return call_stack_[level_].fun_name;
}

StringPtr CommandSender::call_stack_file_name(){
	return call_stack_[level_].file_name;
}

int CommandSender::call_stack_lineno(){
	return call_stack_[level_].lineno;
}

StringPtr CommandSender::required_file(){
	return required_file_;
}

void CommandSender::run(){
	send_command(Xid(run));
}

void CommandSender::step_over(){
	send_command(Xid(step_over));
}

void CommandSender::step_into(){
	send_command(Xid(step_into));
}

void CommandSender::step_out(){
	send_command(Xid(step_out));
}

void CommandSender::redo(){
	send_command(prev_command_);
}
	
void CommandSender::erase_breakpoint(const StringPtr& path, int n){
	for(uint_t i=0; i<breakpoints_->size(); ++i){
		ArrayPtr d = ptr_cast<Array>(breakpoints_->at(i));
		if(!XTAL_detail_raweq(d->at(0), path) || !XTAL_detail_raweq(d->at(1), AnyPtr(n))){
			breakpoints_->erase(i);
			break;
		}		
	}
}

void CommandSender::add_breakpoint(const StringPtr& path, int n, const StringPtr& cond){
	erase_breakpoint(path, n);

	ArrayPtr v = xnew<Array>();
	v->push_back(path);
	v->push_back(n);
	v->push_back(cond);
	breakpoints_->push_back(v);

	add_breakpoint_inner(path, n, cond);
}

void CommandSender::add_breakpoint_inner(const StringPtr& path, int n, const StringPtr& cond){
    if(!stream_ || stream_->eos()){
		return;
	}

	ArrayPtr a = xnew<Array>();
	a->push_back(Xid(add_breakpoint));
	a->push_back(path);
	a->push_back(n);
	if(!cond || cond->empty()){
		a->push_back(null);
	}
	else{
		if(CodePtr eval_code = eval_compile(cond)){
			a->push_back(eval_code);
		}
		else{
			a->push_back(null);
			XTAL_CATCH_EXCEPT(e){
				XTAL_UNUSED_VAR(e);
			}
		}
	}

	stream_->serialize(a);
}

void CommandSender::remove_breakpoint(const StringPtr& path, int n){
	erase_breakpoint(path, n);

	remove_breakpoint_inner(path, n);
}

void CommandSender::remove_breakpoint_inner(const StringPtr& path, int n){
    if(!stream_ || stream_->eos()){
		return;
	}

	ArrayPtr a = xnew<Array>();
	a->push_back(Xid(remove_breakpoint));
	a->push_back(path);
	a->push_back(n);
	stream_->serialize(a);
}

void CommandSender::move_call_stack(int n){
    if(!stream_ || stream_->eos()){
		return;
	}

	ArrayPtr a = xnew<Array>();
	a->push_back(Xid(move_callstack));
	a->push_back(n);
	stream_->serialize(a);
}

void CommandSender::nostep(){
    if(!stream_ || stream_->eos()){
		return;
	}

	ArrayPtr a = xnew<Array>();
	a->push_back(Xid(nostep));
	stream_->serialize(a);
}

void CommandSender::start(){
    if(!stream_ || stream_->eos()){
		return;
	}

	ArrayPtr a = xnew<Array>();
	a->push_back(Xid(start));
	stream_->serialize(a);
}

void CommandSender::required_source(const CodePtr& code){
    if(!stream_ || stream_->eos()){
		return;
	}

	Xfor_cast(ArrayPtr d, breakpoints_){
		if(code->source_file_name()->op_eq(ptr_cast<String>(d->at(0)))){
			code->add_breakpoint(d->at(1)->to_i(), d->at(2));
		}
	}

	ArrayPtr a = xnew<Array>();
	a->push_back(Xid(required_source));
	a->push_back(code);
	stream_->serialize(a);
}
	
void CommandSender::pause(){
    if(!stream_ || stream_->eos()){
		return;
	}

	ArrayPtr a = xnew<Array>();
	a->push_back(Xid(pause));
	stream_->serialize(a);
}

void CommandSender::send_command(const IDPtr& id){
    if(!stream_ || stream_->eos()){
		return;
	}

	prev_command_ = id;
	ArrayPtr a = xnew<Array>();
	a->push_back(id);
	stream_->serialize(a);
}

void CommandSender::update(){
    if(!stream_ || stream_->eos()){
		return;
	}

	while(stream_->available()!=0){
		ArrayPtr command = ptr_cast<Array>(stream_->deserialize());
		AnyPtr type = command->at(0);

		if(raweq(type, Xid(break))){
			MapPtr exprs = ptr_cast<Map>(command->at(1));
			ArrayPtr callStack = ptr_cast<Array>(command->at(2));

			Xfor2(key, value, exprs){
				exprs_->set_at(key, value);
			}

			call_stack_.resize(callStack->size());
			for(uint_t i=0; i<callStack->size(); ++i){
				ArrayPtr record = ptr_cast<Array>(callStack->at(i));
				call_stack_[i].fun_name = record->at(0)->to_s();
				call_stack_[i].file_name = record->at(1)->to_s();
				call_stack_[i].lineno = record->at(2)->to_i();
			}

			level_ = command->at(3)->to_i();

			on_breaked();
			continue;
		}

		if(raweq(type, Xid(require))){
			required_file_ = command->at(1)->to_s();
			on_required();
			continue;
		}
	}
}

void CommandSender::start(const StreamPtr& stream){
	stream_ = stream;

	// 式をすべて転送する
	Xfor2(k, v, exprs_){
		add_eval_expr_inner(ptr_cast<String>(k));
	}

	send_command(Xid(start));

	update();
}

}

}
