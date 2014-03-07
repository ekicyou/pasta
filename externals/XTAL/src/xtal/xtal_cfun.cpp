#include "xtal.h"
#include "xtal_macro.h"

namespace xtal{

NativeFunPtr new_native_fun(const param_types_holder_n& pth, const void* val){
	return xnew<NativeMethod>(pth, val);
}

NativeFunPtr new_native_fun(const param_types_holder_n& pth, const void* val, const AnyPtr& this_){
	return xnew<NativeFun>(pth, val, this_);
}

StatelessNativeMethod::StatelessNativeMethod(const param_types_holder_n& pth){
	value_.init_stateless_native_method(&pth);
}

NativeMethod::NativeMethod(const param_types_holder_n& pth, const void* val){
	pth_ = &pth;

	val_size_ = pth.size==1 ? 0 : pth.size;
	min_param_count_ = pth.vm ? 0 : pth.param_n;
	max_param_count_ = (pth.vm || pth.extendable) ? 255 : pth.param_n;

	uint_t data_size = val_size_ + pth.param_n*sizeof(NamedParam);
	data_ = data_size==0 ? 0 : xmalloc(data_size);

	// 関数をコピー
	std::memcpy(data_, val, val_size_);

	if(pth.param_n){
		NamedParam* params = (NamedParam*)((u8*)data_ + val_size_);
		for(int_t i=0; i<pth.param_n; ++i){
			new(&params[i]) NamedParam();
		}
	}
}

NativeMethod::~NativeMethod(){
	const param_types_holder_n& pth = *pth_;
	NamedParam* params = (NamedParam*)((u8*)data_ + val_size_);

	for(int_t i=0; i<pth.param_n; ++i){
		params[i].~NamedParam();
	}

	uint_t data_size = val_size_ + pth.param_n*sizeof(NamedParam);
	xfree(data_, data_size);
}

const NativeFunPtr& NativeMethod::param(int_t i, const IDPtr& key, const AnyPtr& value){
	// iは1始まり
	XTAL_ASSERT(i!=0);

	// 今のところ、可変長引数をとる関数は、paramは設定でない制限あり
	XTAL_ASSERT(pth_->vm==0 && pth_->extendable==0);

	// iが引数の数より大きすぎる
	XTAL_ASSERT(i<=pth_->param_n);

	i--;

	NamedParam* params = (NamedParam*)((u8*)data_ + val_size_);

	// 既に設定済み
	XTAL_ASSERT(XTAL_detail_raweq(params[i].name, null) && XTAL_detail_raweq(params[i].value, undefined));

	params[i].name = key;
	params[i].value = value;

	if(min_param_count_>i){
		min_param_count_ = (u8)i;
	}

	return to_smartptr(this);
}

const NativeFunPtr& NativeMethod::add_param(const IDPtr& key, const AnyPtr& value){
	NamedParam* params = (NamedParam*)((u8*)data_ + val_size_);

	// 今のところ、可変長引数をとる関数は、paramは設定でない制限あり
	XTAL_ASSERT(pth_->vm==0 && pth_->extendable==0);

	// 引数の数よりparamを多く設定している
	XTAL_ASSERT(min_param_count_>0);

	min_param_count_--;
	for(int_t i=min_param_count_; i<max_param_count_-1; ++i){
		params[i] = params[i+1];
	}

	params[max_param_count_-1].name = key;
	params[max_param_count_-1].value = value;
	
	return to_smartptr(this);
}


void NativeMethod::on_visit_members(Visitor& m){
	RefCountingBase::on_visit_members(m);
	NamedParam* params = (NamedParam*)((u8*)data_ + val_size_);

	const param_types_holder_n& pth = *pth_;
	for(uint_t i=0; i<pth.param_n; ++i){
		m & params[i].name & params[i].value;
	}
}

NativeFun::NativeFun(const param_types_holder_n& pth, const void* val, const AnyPtr& this_)
:NativeMethod(pth, val), this_(this_){
}

void NativeFun::on_visit_members(Visitor& m){
	NativeMethod::on_visit_members(m);
	m & this_;
}

void NativeFun::on_rawcall(const VMachinePtr& vm){
	vm->set_arg_this(this_);
	NativeMethod::on_rawcall(vm);
}

DoubleDispatchMethod::DoubleDispatchMethod(const IDPtr& primary_key)
	:primary_key_(primary_key){}

void DoubleDispatchMethod::on_rawcall(const VMachinePtr& vm){
	if(vm->ordered_arg_count()>0){
		vm->arg_this()->rawsend(vm, primary_key_, vm->arg(0)->get_class());
	}
}

DoubleDispatchFun::DoubleDispatchFun(const ClassPtr& klass, const IDPtr& primary_key)
	:klass_(klass), primary_key_(primary_key){}

void DoubleDispatchFun::on_rawcall(const VMachinePtr& vm){
	klass_->member(primary_key_, vm->arg(0)->get_class())->rawcall(vm);
}

void DoubleDispatchFun::on_visit_members(Visitor& m){
	Base::on_visit_members(m);
	m & klass_;
}

DoubleDispatchMethodPtr double_dispatch_method(const IDPtr& primary_key){
	return xnew<DoubleDispatchMethod>(primary_key);
}

DoubleDispatchFunPtr double_dispatch_fun(const ClassPtr& klass, const IDPtr& primary_key){
	return xnew<DoubleDispatchFun>(klass, primary_key);
}

}
