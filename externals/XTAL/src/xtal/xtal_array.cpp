#include "xtal.h"
#include "xtal_macro.h"

namespace xtal{

namespace{

void fill_undefined(Any* begin, uint_t size){
	Any v = undefined;
	for(uint_t i=0; i<size; ++i){
		begin[i] = v;
	}
}

}

void xxmemmove(int_t* ss1, const int_t* ss2, size_t n){
	if(n != 0){
		if(ss1 < ss2){
			const int_t *t = ss2 + n;
			do{
				*ss1++ = *ss2++;
			}while(ss2 != t);
		}
		else if(ss1 > ss2){
			const int_t *t = ss2;
			ss1 += n;
			ss2 += n;
			do{
				*--ss1 = *--ss2;
			}while(ss2 != t);
		}
	}
}

void xxmemcpy(int_t* ss1, const int_t* ss2, size_t n){
	if(n != 0){
		const int_t* t = ss2 + n;
		do{
			*ss1++ = *ss2++;
		}while(ss2 != t);
	}
}

void xxmemset(int_t* s, int_t c, size_t n){
	if(n != 0){
		int_t* addr = s;
		int_t* end = addr + n;
		for(; addr < end; ++addr){
			*addr = c;
		}
	}
}

//////////////////////////////////////////////////////////////

xarray::xarray(uint_t size){
	capa_ = size;
	size_ = size;

	if(capa_!=0){
		values_ = (AnyPtr*)xmalloc(sizeof(AnyPtr)*capa_);
		fill_undefined(values_, size_);
	}
	else{
		values_ = 0;
	}
}

void xarray::init(const AnyPtr* values, uint_t size){
	capa_ = size;
	size_ = size;
	if(capa_!=0){
		values_ = (AnyPtr*)xmalloc(sizeof(AnyPtr)*capa_);

		for(uint_t i=0; i<size; ++i){
			XTAL_detail_copy(values_[i], values[i]);
			XTAL_detail_inc_ref_count(values_[i]);
		}
	}
	else{
		values_ = 0;
	}
}

void xarray::reflesh(){
	if(size_){
		AnyPtr* newp = (AnyPtr*)xmalloc(sizeof(AnyPtr)*size_);
		xmemcpy(&newp[0], &values_[0], size_);
		for(uint_t i=0; i<size_; ++i){
			XTAL_detail_inc_ref_count(values_[i]);
		}
		capa_ = size_;
		values_ = newp;
	}
	else{
		capa_ = 0;
		values_ = 0;
	}
}

xarray::xarray(const AnyPtr* first, const AnyPtr* end){
	init(first, end-first);
}

xarray::xarray(const xarray& v){
	init(((xarray&)v).data(), ((xarray&)v).size());
}

xarray& xarray::operator =(const xarray& v){
	xarray temp(v);
	std::swap(values_, temp.values_);
	std::swap(size_, temp.size_);
	std::swap(capa_, temp.capa_);
	return *this;
}

xarray::~xarray(){
	destroy();
}

void xarray::destroy(){
	clear();
	xfree(values_, sizeof(AnyPtr)*capa_);
	capa_ = 0;
	values_ = 0;

}

void xarray::clear(){
	for(uint_t i=0; i<size_; ++i){
		XTAL_detail_dec_ref_count(values_[i]);
	}
	size_ = 0;
}

void xarray::shrink_to_fit(){
	if(size_!=capa_){
		xarray temp(*this);
		std::swap(values_, temp.values_);
		std::swap(size_, temp.size_);
		std::swap(capa_, temp.capa_);
	}
}

void xarray::resize(uint_t sz){
	if(sz<size_){
		downsize(size_-sz);
	}
	else if(sz>size_){
		upsize(sz-size_);
	}
}

void xarray::upsize(uint_t sz){
	if(size_+sz>capa_){ // todo overflow check
		if(capa_!=0){
			uint_t newcapa = size_ + sz + capa_/2 + 2;
			AnyPtr* newp = (AnyPtr*)xmalloc(sizeof(AnyPtr)*newcapa);
			xmemcpy(newp, values_, size_);
			fill_undefined(&newp[size_], sz);
			xfree(values_, sizeof(AnyPtr)*capa_);
			values_ = newp;
			size_ += sz;
			capa_ = newcapa;
		}
		else{
			// 一番最初のリサイズは、きっかりに取る
			uint_t newcapa = sz;
			values_ = (AnyPtr*)xmalloc(sizeof(AnyPtr)*newcapa);
			fill_undefined(&values_[0], sz);
			size_ = sz;
			capa_ = newcapa;
		}
	}
	else{
		fill_undefined(values_ + size_, sz);
		size_ += sz;
	}
}

void xarray::downsize(uint_t sz){
	XTAL_ASSERT(sz<=size_);
	for(uint_t i=size_-sz; i<size_; ++i){
		XTAL_detail_dec_ref_count(values_[i]);
	}
	size_ -= sz;
}

void xarray::erase(int_t start, int_t n){
	if(n==0){
		return;
	}

	int_t pos = start;
	XTAL_ASSERT(0<=pos && (uint_t)pos<size_);

	for(int_t i=0; i<n; ++i){
		XTAL_detail_dec_ref_count(values_[pos+i]);
	}

	if(size_-(pos+n)!=0){
		xmemmove(&values_[pos], &values_[pos+n], size_-(pos+n));
	}
	size_ -= n;
}

void xarray::move(int_t dest, int_t src, int_t n){
	if(n==0 || dest==src){
		return;
	}

	for(int_t i=0; i<n; ++i){
		XTAL_detail_inc_ref_count(values_[src+i]);
		XTAL_detail_dec_ref_count(values_[dest+i]);
	}

	xmemmove(&values_[dest], &values_[src], n);
}


void xarray::insert(int_t i, const AnyPtr& v){
	if(capa_==size_){
		upsize(1);
	}
	else{
		size_++;
	}

	int_t pos = i;

	xmemmove(&values_[pos+1], &values_[pos], size_-(pos+1));
	XTAL_detail_copy(values_[pos], v);
	XTAL_detail_inc_ref_count(values_[pos]);
}

void xarray::push_back(const AnyPtr& v){
	if(capa_==size_){
		upsize(1);
	}
	else{
		size_++;
	}

	XTAL_detail_copy(values_[size_-1], v);
	XTAL_detail_inc_ref_count(values_[size_-1]);
}

void xarray::pop_back(){
	XTAL_ASSERT(!empty());
	XTAL_detail_dec_ref_count(values_[size_-1]);
	size_--;
}

void visit_members(Visitor& m, const xarray& values){
	for(uint_t i=0; i<values.size(); ++i){
		m & values.at(i);
	}
}

//////////////////////////////////////////////////////////////

#define XTAL_ARRAY_CALC_OFFSET(i, size, ret) \
	do{\
		if(i<0){\
			i = size + i;\
			if(i>=0){\
				break;\
			}\
		}\
		else{\
			if((uint_t)i < size){\
				break;\
			}\
		}\
		throw_index_error();\
		ret;\
	}while(0)


void Array::on_visit_members(Visitor& m){
	m & values_;
}

Array::Array(uint_t size)
:values_(size){}

Array::Array(const AnyPtr* first, const AnyPtr* end)
:values_(first, end){}

const AnyPtr& Array::op_at(int_t i){
	XTAL_ARRAY_CALC_OFFSET(i, size(), return undefined);
	return values_.at(i);
}

void Array::op_set_at(int_t i, const AnyPtr& v){
	XTAL_ARRAY_CALC_OFFSET(i, size(), return);
	values_.set_at(i, v);
}

void Array::erase(int_t start, int_t n){
	int_t pos = start;
	XTAL_ARRAY_CALC_OFFSET(pos, size(), return);

	if(n<0 || (uint_t)(n + pos)>size()){
		throw_index_error();
		return;
	}

	values_.erase(pos, n);
}

void Array::insert(int_t i, const AnyPtr& v){
	int_t pos = i;
	XTAL_ARRAY_CALC_OFFSET(pos, size()+1, return);
	values_.insert(pos, v);
}

void Array::push_back(const AnyPtr& v){
	insert(size(), v);
}

void Array::pop_back(){
	erase(size()-1, 1);
}

ArrayPtr Array::slice(int_t start, int_t n){
	if(n==0){
		return XNew<Array>(0);
	}

	int_t pos = start;
	XTAL_ARRAY_CALC_OFFSET(pos, size(), return nul<Array>());

	if(n<0 || (uint_t)(n + pos)>size()){
		throw_index_error();
		return nul<Array>();
	}

	return XNew<Array>(values_.data()+pos, values_.data()+pos+n);
}

ArrayPtr Array::splice(int_t start, int_t n){
	ArrayPtr ret = slice(start, n);
	erase(start, n);
	return ret;
}

ArrayPtr Array::clone(){
	return XNew<Array>(data(), data()+size());
}

ArrayPtr Array::op_cat(const ArrayPtr& a){
	ArrayPtr ret(clone());
	ret->op_cat_assign(a);
	return ret;
}

const ArrayPtr& Array::op_cat_assign(const ArrayPtr& a){
	for(uint_t i = 0, size = a->size(); i<size; ++i){
		push_back(a->at(i));
	}
	return to_smartptr(this);
}

StringPtr Array::join(const StringPtr& sep){
	MemoryStreamPtr ms = xnew<MemoryStream>();
	if(XTAL_detail_raweq(sep, empty_string)){
		for(uint_t i=0, sz=size(); i<sz; ++i){
			ms->put_s(at(i));
		}
	}
	else{
		for(uint_t i=0, sz=size(); i<sz; ++i){
			if(i!=0){
				ms->put_s(sep);
			}
			ms->put_s(at(i));
		}
	}
	return ms->to_s();
}

StringPtr Array::to_s(){
	MemoryStreamPtr ms = xnew<MemoryStream>();
	ms->put_s(XTAL_STRING("["));
	ms->put_s(join(XTAL_STRING(", ")));
	ms->put_s(XTAL_STRING("]"));
	return ms->to_s();
}

bool Array::op_eq(const ArrayPtr& other){
	if(size()!=other->size())
		return false;
	
	const VMachinePtr& vm = vmachine();
	for(uint_t i=0, size=other->size(); i<size; ++i){
		if(!XTAL_detail_raweq(at(i), other->at(i))){
			vm->setup_call(1);
			vm->push_arg(other->at(i));
			at(i)->rawsend(vm, Xid(op_eq));
			if(!vm->is_executed() || !vm->result()){
				vm->cleanup_call();
				return false;
			}
			vm->cleanup_call();
		}
	}
	return true;
}

AnyPtr Array::each(){
	return xnew<ArrayIter>(to_smartptr(this), &values_, false);
}

AnyPtr Array::reverse(){
	return xnew<ArrayIter>(to_smartptr(this), &values_, true);
}
	
void Array::block_first(const VMachinePtr& vm){
	SmartPtr<ArrayIter> it = xnew<ArrayIter>(to_smartptr(this), &values_);
	it->block_next(vm);
}

void Array::assign(const AnyPtr& iterator){
	clear();

	uint_t i = 0;
	Xfor(v, iterator){
		if(i>=size()){
			upsize(1);
		}

		set_at(i, v);
		++i;
	}
}

void Array::append(const AnyPtr& iterator){
	Xfor(v, iterator){
		push_back(v);
	}
}

void Array::throw_index_error(){
	set_runtime_error(Xt("XRE1020"));
}

//////////////////////////////////////////////////

ArrayIter::ArrayIter(const AnyPtr& a, xarray* v, bool reverse)
	:ref_(a), values_(v), index_(0), reverse_(reverse){
}

void ArrayIter::block_next(const VMachinePtr& vm){
	++index_;
	if(index_<=values_->size()){
		vm->return_result(to_smartptr(this), values_->at(reverse_ ? values_->size()-index_ : index_-1));
	}
	else{
		vm->return_result(null, null);
	}
}

bool ArrayIter::block_next_direct(AnyPtr& ret){
	++index_;
	if(index_<=values_->size()){
		ret = values_->at(reverse_ ? values_->size()-index_ : index_-1);
		return true;
	}
	else{
		ret = null;
		return false;
	}
}

void ArrayIter::on_visit_members(Visitor& m){
	Base::on_visit_members(m);
	m & ref_;
}

XNew<Array>::XNew(uint_t size){ init(size);	}
XNew<Array>::XNew(const AnyPtr* first, const AnyPtr* end){ init(first, end); }

}
