#include "xtal.h"

namespace xtal{

namespace{
	int stack_dummy_buf[16] = {0};
}

void* stack_dummy_allocate(){
	return stack_dummy_buf;
}

void* stack_allocate(size_t size){
	return xmalloc(size);
}

void stack_deallocate(void* p, size_t size){
	if(stack_dummy_buf==p){
		return;
	}
	XTAL_ASSERT(!((char*)stack_dummy_buf-64 <= (char*)p && (char*)p <= (char*)stack_dummy_buf+64));
	xfree(p, size);
}


PODArrayBase::PODArrayBase(uint_t one_size, uint_t size){
	one_size_ = one_size;
	capa_ = size;
	size_ = size;

	if(capa_!=0){
		values_ = (AnyPtr*)xmalloc(one_size_*capa_);
	}
	else{
		values_ = 0;
	}
}

void PODArrayBase::init(const void* values, uint_t size){
	capa_ = size;
	size_ = size;
	if(capa_!=0){
		values_ = (AnyPtr*)xmalloc(one_size_*capa_);
		std::memcpy(values_, values, one_size_*size);
	}
	else{
		values_ = 0;
	}
}


PODArrayBase::PODArrayBase(const PODArrayBase& v){
	one_size_ = v.one_size_;
	init(((PODArrayBase&)v).data(), ((PODArrayBase&)v).size());
}

PODArrayBase& PODArrayBase::operator =(const PODArrayBase& v){
	PODArrayBase temp(v);
	std::swap(one_size_, temp.one_size_);
	std::swap(values_, temp.values_);
	std::swap(size_, temp.size_);
	std::swap(capa_, temp.capa_);
	return *this;
}

PODArrayBase::~PODArrayBase(){
	destroy();
}

void PODArrayBase::destroy(){
	xfree(values_, one_size_*capa_);
	size_ = 0;
	capa_ = 0;
	values_ = 0;

}

void PODArrayBase::clear(){
	size_ = 0;
}

void PODArrayBase::shrink_to_fit(){
	if(size_!=capa_){
		PODArrayBase temp(*this);
		std::swap(values_, temp.values_);
		std::swap(one_size_, temp.one_size_);
		std::swap(size_, temp.size_);
		std::swap(capa_, temp.capa_);
	}
}

void PODArrayBase::resize(uint_t sz){
	if(sz<size_){
		downsize(size_-sz);
	}
	else if(sz>size_){
		upsize(sz-size_);
	}
}

void PODArrayBase::upsize(uint_t sz){
	if(size_+sz>capa_){ // todo overflow check
		if(capa_!=0){
			uint_t newcapa = size_ + sz + capa_/2 + 2;
			void* newp = xmalloc(one_size_*newcapa);
			std::memcpy(newp, values_, one_size_*size_);
			xfree(values_, one_size_*capa_);
			values_ = newp;
			size_ += sz;
			capa_ = newcapa;
		}
		else{
			// 一番最初のリサイズは、きっかりに取る
			uint_t newcapa = sz;
			values_ = xmalloc(one_size_*newcapa);
			size_ = sz;
			capa_ = newcapa;
		}
	}
	else{
		size_ += sz;
	}
}

void PODArrayBase::downsize(uint_t sz){
	if(sz>size_){
		sz = size_;
	}
	size_ -= sz;
}

void PODArrayBase::erase(int_t start, int_t n){
	if(n==0){
		return;
	}

	XTAL_ASSERT(0<=start && (uint_t)start<size_);

	if(size_-(start+n)!=0){
		std::memmove(plusp(values_, start), plusp(values_, start+n), (size_-(start+n))*one_size_);
	}
	size_ -= n;
}

void PODArrayBase::insert(int_t i, const void* v){
	if(capa_==size_){
		upsize(1);
	}
	else{
		size_++;
	}
	std::memmove(plusp(values_, i+1), plusp(values_, i), (size_-(i+1))*one_size_);
	std::memcpy(plusp(values_, i), v, one_size_);
}

void PODArrayBase::push_back(const void* v){
	if(capa_==size_){
		upsize(1);
	}
	else{
		size_++;
	}
	std::memcpy(plusp(values_, size_-1), v, one_size_);
}

void PODArrayBase::pop_back(){
	XTAL_ASSERT(!empty());
	size_--;
}

void PODArrayBase::set_at(int_t i, const void* v){
	XTAL_ASSERT(0<=i && (uint_t)i<size_);
	std::memcpy(plusp(values_, i), v, one_size_);
}

}
