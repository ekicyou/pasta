#include "xtal.h"
#include "xtal_macro.h"

namespace xtal{

NullPtr null(unchecked_ptr_cast<Null>(ap(Null())));
UndefinedPtr undefined(unchecked_ptr_cast<Undefined>(ap(Undefined())));

IDPtr empty_id(unchecked_ptr_cast<ID>(ap(String())));
StringPtr empty_string(empty_id);

FunInfo empty_xfun_info;
ScopeInfo empty_scope_info;
ClassInfo empty_class_info;

ExceptInfo empty_except_info;
InstanceVariables empty_instance_variables = {&empty_class_info};

undeleter_t undeleter;
deleter_t deleter;

bool Int::op_in_IntRange(const IntRangePtr& range){
	return range->begin() <= XTAL_detail_ivalue(*this) && XTAL_detail_ivalue(*this) < range->end();
}

bool Int::op_in_FloatRange(const FloatRangePtr& range){
	return (range->is_left_closed() ? (range->left() <= XTAL_detail_ivalue(*this)) : (range->left() < XTAL_detail_ivalue(*this))) && (range->is_right_closed() ? (XTAL_detail_ivalue(*this) <= range->right()) : (XTAL_detail_ivalue(*this) < range->right()));
}

IntRangePtr Int::op_range_Int(int_t right, int_t kind){
	return xnew<IntRange>(XTAL_detail_ivalue(*this), right, kind);	
}

FloatRangePtr Int::op_range_Float(float_t right, int_t kind){
	return xnew<FloatRange>(XTAL_detail_fvalue(*this), right, kind);	
}

FloatRangePtr Float::op_range_Int(int_t right, int_t kind){
	return xnew<FloatRange>(XTAL_detail_fvalue(*this), (float_t)right, kind);	
}

FloatRangePtr Float::op_range_Float(float_t right, int_t kind){
	return xnew<FloatRange>(XTAL_detail_fvalue(*this), right, kind);	
}

bool Float::op_in_IntRange(const IntRangePtr& range){
	return range->begin() <= XTAL_detail_fvalue(*this) && XTAL_detail_fvalue(*this) < range->end();
}

bool Float::op_in_FloatRange(const FloatRangePtr& range){
	return (range->is_left_closed() ? (range->left() <= XTAL_detail_fvalue(*this)) : (range->left() < XTAL_detail_fvalue(*this))) && (range->is_right_closed() ? (XTAL_detail_fvalue(*this) <= range->right()) : (XTAL_detail_fvalue(*this) < range->right()));
}


AnyPtr IntRange::each(){
	return xnew<IntRangeIter>(to_smartptr(this));
}

void IntRangeIter::block_next(const VMachinePtr& vm){
	if(it_<end_){
		vm->return_result(to_smartptr(this), it_);
		++it_;
	}
	else{
		vm->return_result(null, null);
	}
}

Values::Values(const AnyPtr& head)
	:head_(head), tail_(unchecked_ptr_cast<Values>(undefined)){
}

Values::Values(const AnyPtr& head, const ValuesPtr& tail)
	:head_(head), tail_(tail){
}

Values::~Values(){
}
	
void Values::block_next(const VMachinePtr& vm){
	if(!XTAL_detail_is_undefined(tail_)){
		vm->return_result(tail_, head_);
	}
	else{
		vm->return_result(0, head_);
	}
}

int_t Values::size(){
	const ValuesPtr* cur = &to_smartptr(this);
	int_t size = 1;
	while(true){
		if(XTAL_detail_is_undefined((*cur)->tail_)){
			return size;
		}
		cur = &(*cur)->tail_;
		++size;
	}
}

const AnyPtr& Values::at(int_t i){
	const ValuesPtr* cur = &to_smartptr(this);
	const AnyPtr* ret = &head_;
	for(int_t n=0; n<i; ++n){
		if(XTAL_detail_is_undefined((*cur)->tail_)){
			ret = &(AnyPtr&)undefined;
			break;
		}
		cur = &(*cur)->tail_;
		ret = &(*cur)->head_;
	}
	return *ret;
}

bool Values::op_eq(const ValuesPtr& other){
	const VMachinePtr& vm = vmachine();

	const ValuesPtr* cur1 = &to_smartptr(this);
	const ValuesPtr* cur2 = &other;

	while(true){
		if(!XTAL_detail_raweq((*cur1)->head_, (*cur2)->head_)){
			vm->setup_call(1);
			vm->push_arg((*cur2)->head_);
			(*cur1)->head_->rawsend(vm, Xid(op_eq), undefined, true, true);

			if(!vm->is_executed() || !vm->result()){
				vm->cleanup_call();
				return false;
			}
			vm->cleanup_call();
		}

		if(XTAL_detail_is_undefined((*cur1)->tail_)){
			if(XTAL_detail_is_undefined((*cur2)->tail_)){
				return true;
			}
			else{
				return false;
			}
		}

		if(!(*cur2)->tail_){
			return false;
		}

		cur1 = &(*cur1)->tail_;
		cur2 = &(*cur2)->tail_;
	}

	return true;
}

StringPtr Values::to_s(){
	MemoryStreamPtr ms = xnew<MemoryStream>();
	ms->put_s(XTAL_STRING("("));

	const ValuesPtr* cur = &to_smartptr(this);
	for(int_t n=0; ; ++n){
		if(n!=0){
			ms->put_s(XTAL_STRING(", "));
		}

		ms->put_s((*cur)->head_);

		if(XTAL_detail_is_undefined((*cur)->tail_)){
			break;
		}
		cur = &(*cur)->tail_;
	}

	ms->put_s(XTAL_STRING(")"));
	return ms->to_s();

}

void Values::set(const AnyPtr& head, const ValuesPtr& tail){
	head_ = head;
	tail_ = tail;
}

ValuesPtr mv(const AnyPtr& v1, const AnyPtr& v2){
	return xnew<Values>(v1, xnew<Values>(v2));
}

///////////////////////////////////

const ClassPtr& HaveParent::object_parent(){
	return parent_;
}

void HaveParent::set_object_parent(const ClassPtr& parent){
	if(!parent_ || parent_->object_force()<parent->object_force()){
		parent_ = parent;
	}
}

}
