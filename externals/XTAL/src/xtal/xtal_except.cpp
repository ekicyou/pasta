#include "xtal.h"
#include "xtal_macro.h"

namespace xtal{

Exception::Exception(const AnyPtr& message){
	initialize(message);
}

void Exception::initialize(const AnyPtr& message){
	message_ = message->to_s();
	backtrace_ = XNew<Array>();
}

void Exception::append_backtrace(const AnyPtr& file, int_t line, const AnyPtr& function_name){
	backtrace_->push_back(Xf3("\t%s:%d: in %s", 0, file, 1, line, 2, function_name));
}

StringPtr Exception::to_s(){
	MemoryStreamPtr mm = xnew<MemoryStream>();
	mm->put_s(get_class()->object_name());
	mm->put_s(Xs(": "));
	mm->put_s(message_->to_s());
	mm->put_s(Xs("\n"));
	mm->put_s(backtrace_->join(Xs("\n")));
	return mm->to_s();
}

AnyPtr unsupported_error(const AnyPtr& target, const IDPtr& primary_key, const AnyPtr& secondary_key){

#ifdef XTAL_DEBUG
	IDPtr pick;
	if(const ClassPtr& klass = ptr_cast<Class>(target)){
		pick = klass->find_near_member(primary_key, secondary_key);
		if(XTAL_detail_raweq(pick, primary_key)){
			pick = null;
		}
	}

	if(pick){
		if(!XTAL_detail_raweq(secondary_key, undefined)){
			return (cpp_class<UnsupportedError>()->call(Xt("XRE1021")->call(
				Named(Xid(object), Xf3("%s::%s#%s", 0, target->object_name(), 1, primary_key, 2, secondary_key)),
				Named(Xid(pick), pick)
			)));
		}
		else{
			return (cpp_class<UnsupportedError>()->call(Xt("XRE1021")->call(
				Named(Xid(object), Xf2("%s::%s", 0, target->object_name(), 1, primary_key)),
				Named(Xid(pick), pick)
			)));
		}
	}
#endif

	if(!XTAL_detail_is_undefined(secondary_key)){
		return (cpp_class<UnsupportedError>()->call(Xt("XRE1015")->call(
			Named(Xid(object), Xf3("%s::%s#%s", 0, target->object_name(), 1, primary_key, 2, secondary_key))
		)));
	}
	else{
		return (cpp_class<UnsupportedError>()->call(Xt("XRE1015")->call(
			Named(Xid(object), Xf2("%s::%s", 0, target->object_name(), 1, primary_key))
		)));
	}
}

AnyPtr filelocal_unsupported_error(const CodePtr& code, const IDPtr& primary_key){
	IDPtr pick = code->find_near_variable(primary_key);

	if(pick){
		return (cpp_class<UnsupportedError>()->call(Xt2("XRE1021", object, primary_key, pick, pick)));	
	}
	else{
		return (cpp_class<UnsupportedError>()->call(Xt1("XRE1015", object, primary_key)));	
	}	
}

void set_unsupported_error(const AnyPtr& target, const IDPtr& primary_key, const AnyPtr& secondary_key, const VMachinePtr& vm){
	vm->set_except(unsupported_error(target, primary_key, secondary_key));
}

void set_runtime_error(const AnyPtr& arg, const VMachinePtr& vm){
	vm->set_except(cpp_class<RuntimeError>()->call(arg));
}

void set_argument_type_error(const AnyPtr& object, int_t no, const ClassPtr& required, const ClassPtr& type, const VMachinePtr& vm){
	vm->set_except(cpp_class<ArgumentError>()->call(
		Xt4("XRE1001", 
			object, object, 
			no, no, required, 
			required->object_name(), 
			type, type->object_name())));
}

void set_argument_num_error(const AnyPtr& funtion_name, int_t n, int_t min_count, int_t max_count, const VMachinePtr& vm){
	if(min_count==0 && max_count==0){
		vm->set_except(cpp_class<ArgumentError>()->call(Xt2("XRE1007", object, funtion_name, value, n)));
	}
	else if(max_count<0){
		vm->set_except(cpp_class<ArgumentError>()->call(
			Xt3("XRE1005",
				object, funtion_name,
				min, min_count,
				value, n
			)
		));
	}
	else{
		vm->set_except(cpp_class<ArgumentError>()->call(
			Xt4("XRE1006", 
				object, funtion_name,
				min, min_count,
				max, max_count,
				value, n
			)
		));
	}
}

}
