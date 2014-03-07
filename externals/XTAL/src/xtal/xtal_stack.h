/** \file src/xtal/xtal_stack.h
* \brief src/xtal/xtal_stack.h
*/

#ifndef XTAL_STACK_H_INCLUDE_GUARD
#define XTAL_STACK_H_INCLUDE_GUARD

#pragma once

namespace xtal{

void* stack_dummy_allocate();

void* stack_allocate(size_t size);

void stack_deallocate(void* p, size_t size);

template<class T>
struct FastStackDefaultValue{
	static T get(){ return T(); }
};

/**
* \internal
* VMachineクラスが使う色々なスタック用に、特定の操作の実行速度を重視して実装したスタックコンテナ。
*
* 速度を稼ぐため、capacity分のオブジェクトは常に生成された状態となっている。
* sizeが減ったり増えたりしたからといって、コンストラクタ、デストラクタを呼んだりはしない。
* そのため、pop関数はポップされたオブジェクトの参照を返しているが、これは安全である。
*/
template<class T>
class FastStack{
	T* begin_; // 確保したメモリの先頭の次を指す。
	T* end_; // 確保したメモリの一番最後の次を指す

public:

	T* current_; // スタックトップの要素を指す

private:

	void upsize_detail(size_t us);

public:

	FastStack();

	FastStack(const FastStack<T>& a);
	
	FastStack<T> &operator =(const FastStack<T>& a);

	~FastStack();

public:

	T* data(){
		return begin_;
	}
	
	T& push_unchecked(const T &val){
		upsize_unchecked(1);
		return top()=val;
	}

	T& push(const T &val){
		upsize(1);
		return top()=val;
	}

	T& push_unchecked(){
		upsize_unchecked(1);
		return top();
	}

	T& push(){
		upsize(1);
		return top();
	}

	T& pop(){
		XTAL_ASSERT(!empty());
		--current_;
		return current_[1];
	}

	T& top(){
		XTAL_ASSERT(!empty());
		return *current_;
	}

	const T& top() const{
		XTAL_ASSERT(!empty());
		return *current_;
	}

	void erase(size_t i);

	void erase(size_t i, size_t n);

	void insert(size_t i, const T& v);
	
	void resize(size_t newsize);

	void downsize(size_t ds){
		XTAL_ASSERT(size()>=ds);
		current_-=ds;
	}

	void downsize_n(size_t newsize){
		XTAL_ASSERT(newsize<=size());
		current_ = begin_-1+newsize;
	}

	void upsize_unchecked(size_t us){
		XTAL_ASSERT(current_+us>=current_);
		current_+=us;
	}

	void upsize(size_t us){		
		XTAL_ASSERT(current_+us>=current_);
		current_+=us;
		if(XTAL_UNLIKELY(current_>=end_)){
			upsize_detail(us);
		}
	}

	size_t size() const{
		return (current_+1)-begin_;
	}

	size_t capacity() const{
		return end_-begin_;
	}

	void reserve(size_t capa);

	T& get(size_t i = 0){
		XTAL_ASSERT(i<size());
		return *(current_-i);
	}

	void set(size_t i, const T& v){
		XTAL_ASSERT(i<size());
		*(current_-i) = v;
	}

	T& operator [](size_t i){
		XTAL_ASSERT(i<size());
		return *(current_-i);
	}

	const T &operator [](size_t i) const{
		XTAL_ASSERT(i<size());
		return *(current_-i);
	}

	T& reverse_at(size_t i){
		XTAL_ASSERT(i<size());
		return *(begin_+i);
	}

	const T &reverse_at(size_t i) const{
		XTAL_ASSERT(i<size());
		return *(begin_+i);
	}

	T& reverse_at_unchecked(size_t i){
		return *(begin_+i);
	}

	const T &reverse_at_unchecked(size_t i) const{
		return *(begin_+i);
	}

	bool empty() const{
		XTAL_ASSERT(begin_<=current_+1);
		return begin_>current_;
	}

	void clear(){
		current_ = begin_-1;
	}
	
	void push(const FastStack<T>& other, size_t sz){
		upsize(sz);
		downsize(sz);
		for(size_t i = sz; i!=0; --i){
			push_unchecked(other[i-1]);
		} 
	}
	
	void push(const FastStack<T>& other, size_t other_offset, size_t sz){
		upsize(sz);
		downsize(sz);
		for(size_t i = sz; i!=0; --i){
			push_unchecked(other[other_offset+i-1]);
		} 
	}
		
	void move(FastStack<T>& other, size_t sz){
		push(other, sz);
		other.downsize(sz);
	}

	void fill_over(const T& val = T()){
		for(size_t i = size(); i<capacity(); ++i){
			reverse_at_unchecked(i)=val;
		}
	}

	friend void swap(FastStack<T>& a, FastStack<T>& b){
		std::swap(a.current_, b.current_);
		std::swap(a.begin_, b.begin_);
		std::swap(a.end_, b.end_);
	}
};

template<class T>
FastStack<T>::FastStack(){
	begin_ = (T*)stack_dummy_allocate()+1;
	current_ = begin_-1;
	end_ = begin_+0;
}

template<class T>
FastStack<T>::~FastStack(){
	for(size_t i = 0, last = capacity(); i<last; ++i){
		begin_[i].~T();
	}
	stack_deallocate(begin_-1, sizeof(T)*(capacity()+1));
}

template<class T>
FastStack<T>::FastStack(const FastStack<T> &a){
	if(a.capacity()==0){
		begin_=(T*)stack_dummy_allocate()+1;
		current_ = begin_-1;
		end_ = begin_+0;
	}
	else{
		begin_=(T*)stack_allocate(sizeof(T)*(a.capacity()+1))+1;
		current_ = begin_+a.size()-1;
		end_ = begin_+a.capacity();

		size_t capa = a.capacity();
		for(size_t i = 0; i<capa; ++i){
			new(&begin_[i]) T(a.begin_[i]);
		}
	}
}

template<class T>
FastStack<T> &FastStack<T>::operator =(const FastStack<T> &a){
	if(this==&a){
		return *this;
	}
	
	if(a.capacity()==0){
		for(size_t i = 0, last = capacity(); i<last; ++i){
			begin_[i].~T();
		}
		stack_deallocate(begin_-1, sizeof(T)*(capacity()+1));

		begin_=(T*)stack_dummy_allocate()+1;
		current_ = begin_-1;
		end_ = begin_+0;
	}
	else{
		T* newp = (T*)stack_allocate(sizeof(T)*(a.capacity()+1))+1;

		for(size_t i = 0, last = capacity(); i<last; ++i){
			begin_[i].~T();
		}
		stack_deallocate(begin_-1, sizeof(T)*(capacity()+1));

		begin_ = newp;
		current_ = begin_+a.size()-1;
		end_ = begin_+a.capacity();

		size_t capa = a.capacity();
		for(size_t i = 0; i<capa; ++i){
			new(&begin_[i]) T(a.begin_[i]);
		}
	}

	return *this;
}

template<class T>
void FastStack<T>::resize(size_t newsize){
	//XTAL_ASSERT(newsize>=0);

	size_t oldsize = size();
	if(newsize>oldsize){
		upsize(newsize-oldsize);
	}
	else{
		downsize_n(newsize);
	}
}

template<class T>
void FastStack<T>::upsize_detail(size_t us){
	//XTAL_ASSERT(us>=0);

	current_ -= us;

	size_t oldsize = size();
	size_t oldcapa = capacity();
	size_t newsize = oldsize+us;
	T* oldp = begin_;
	size_t newcapa = oldcapa==0 ? us : oldsize + us + oldcapa/2 + 2;
	T* newp = (T*)stack_allocate(sizeof(T)*(newcapa+1))+1;

	for(size_t i = 0; i<oldsize; ++i){
		new(&newp[i]) T(oldp[i]);
	}

	for(size_t i = oldsize; i<newcapa; ++i){
		new(&newp[i]) T(FastStackDefaultValue<T>::get());
	}

	for(size_t i = 0; i<oldcapa; ++i){
		oldp[i].~T();
	}

	stack_deallocate(oldp-1, sizeof(T)*(oldcapa+1));

	begin_ = newp;
	current_ = begin_+newsize-1;
	end_ = begin_+newcapa;
}

template<class T>
void FastStack<T>::erase(size_t i){
	for(size_t j = i; j != 0; --j){
		(*this)[j] = (*this)[j-1];
	}
	downsize(1);
}

template<class T>
void FastStack<T>::erase(size_t i, size_t n){
	for(size_t j = i; j != n-1; --j){
		(*this)[j] = (*this)[j-n];
	}
	downsize(n);
}

template<class T>
void FastStack<T>::insert(size_t i, const T& v){
	upsize(1);
	for(size_t j = 0; j != i; ++j){
		(*this)[j] = (*this)[j+1];
	}
	(*this)[i] = v;
}

template<class T>
void FastStack<T>::reserve(size_t capa){
	if(XTAL_LIKELY(capa<=capacity())){
		return;
	}

	size_t diff = capa-size();
	upsize(diff);
	downsize(diff);
}

template<class T>
void visit_members(Visitor& m, const FastStack<T>& value){
	for(int_t i = 0, size = value.capacity(); i<size; ++i){
		m & value.reverse_at_unchecked(i);
	}
}

///////////////////////////////////////////////////////////////////

template<class T>
class PODStack{
	T* begin_; // 確保したメモリの先頭の次を指す。
	T* end_; // 確保したメモリの一番最後の次を指す
	T* current_; // スタックトップの要素を指す

private:

	void upsize_detail(size_t us);

public:

	PODStack();

	PODStack(const PODStack<T>& a);
	
	PODStack<T> &operator =(const PODStack<T>& a);

	~PODStack();

public:

	T* data(){
		return begin_;
	}
	
	T& push_unchecked(const T &val){
		upsize_unchecked(1);
		return top()=val;
	}

	T& push(const T &val){
		upsize(1);
		return top()=val;
	}

	T& push_unchecked(){
		upsize_unchecked(1);
		return top();
	}

	T& push(){
		upsize(1);
		return top();
	}

	T& pop(){
		XTAL_ASSERT(!empty());
		--current_;
		return current_[1];
	}

	T& top(){
		XTAL_ASSERT(!empty());
		return *current_;
	}

	const T& top() const{
		XTAL_ASSERT(!empty());
		return *current_;
	}

	void erase(size_t i);

	void erase(size_t i, size_t n);

	void insert(size_t i, const T& v);
	
	void resize(size_t newsize);

	void downsize(size_t ds){
		XTAL_ASSERT(size()>=ds);
		current_-=ds;
	}

	void downsize_n(size_t newsize){
		XTAL_ASSERT(newsize<=size());
		current_ = begin_-1+newsize;
	}

	void upsize_unchecked(size_t us){
		XTAL_ASSERT(current_+us>=current_);
		current_+=us;
	}

	void upsize(size_t us){		
		XTAL_ASSERT(current_+us>=current_);
		current_+=us;
		if(current_>=end_){
			upsize_detail(us);
		}
	}

	size_t size() const{
		return (current_+1)-begin_;
	}

	size_t capacity() const{
		return end_-begin_;
	}

	void reserve(size_t capa);

	T& get(size_t i = 0){
		XTAL_ASSERT(i<size());
		return *(current_-i);
	}

	void set(size_t i, const T& v){
		XTAL_ASSERT(i<size());
		*(current_-i) = v;
	}

	T& operator [](size_t i){
		XTAL_ASSERT(i<size());
		return *(current_-i);
	}

	const T &operator [](size_t i) const{
		XTAL_ASSERT(i<size());
		return *(current_-i);
	}

	T& reverse_at(size_t i){
		XTAL_ASSERT(i<size());
		return *(begin_+i);
	}

	const T &reverse_at(size_t i) const{
		XTAL_ASSERT(i<size());
		return *(begin_+i);
	}

	T& reverse_at_unchecked(size_t i){
		return *(begin_+i);
	}

	const T &reverse_at_unchecked(size_t i) const{
		return *(begin_+i);
	}

	bool empty() const{
		XTAL_ASSERT(begin_<=current_+1);
		return begin_>current_;
	}

	void clear(){
		current_ = begin_-1;
	}
	
	void push(const PODStack<T>& other, size_t sz){
		upsize(sz);
		downsize(sz);
		for(size_t i = sz; i!=0; --i){
			push_unchecked(other[i-1]);
		} 
	}
	
	void push(const PODStack<T>& other, size_t other_offset, size_t sz){
		upsize(sz);
		downsize(sz);
		for(size_t i = sz; i!=0; --i){
			push_unchecked(other[other_offset+i-1]);
		} 
	}
		
	void move(PODStack<T>& other, size_t sz){
		push(other, sz);
		other.downsize(sz);
	}

	void fill_over(const T& val = T()){
		for(size_t i = size(); i<capacity(); ++i){
			reverse_at_unchecked(i)=val;
		}
	}

	friend void swap(PODStack<T>& a, PODStack<T>& b){
		std::swap(a.current_, b.current_);
		std::swap(a.begin_, b.begin_);
		std::swap(a.end_, b.end_);
	}
};

template<class T>
PODStack<T>::PODStack(){
	begin_=(T*)stack_dummy_allocate()+1;
	current_ = begin_-1;
	end_ = begin_+0;
}

template<class T>
PODStack<T>::~PODStack(){
	stack_deallocate(begin_-1, sizeof(T)*(capacity()+1));
}

template<class T>
PODStack<T>::PODStack(const PODStack<T> &a){
	if(a.capacity()==0){
		begin_=(T*)stack_dummy_allocate()+1;
		current_ = begin_-1;
		end_ = begin_+0;
	}
	else{
		begin_=(T*)stack_allocate(sizeof(T)*(a.capacity()+1))+1;
		current_ = begin_+a.size()-1;
		end_ = begin_+a.capacity();
		size_t capa = a.capacity();
		std::memcpy(begin_, a.begin_, sizeof(T)*capa);
	}
}

template<class T>
PODStack<T> &PODStack<T>::operator =(const PODStack<T> &a){
	if(this==&a){
		return *this;
	}
	
	if(a.capacity()==0){
		stack_deallocate(begin_-1, sizeof(T)*(capacity()+1));
		begin_=(T*)stack_dummy_allocate()+1;
		current_ = begin_-1;
		end_ = begin_+0;
	}
	else{
		T* newp = (T*)stack_allocate(sizeof(T)*(a.capacity()+1))+1;
		stack_deallocate(begin_-1, sizeof(T)*(capacity()+1));

		begin_ = newp;
		current_ = begin_+a.size()-1;
		end_ = begin_+a.capacity();
		size_t capa = a.capacity();
		std::memcpy(begin_, a.begin_, sizeof(T)*capa);
	}

	return *this;
}

template<class T>
void PODStack<T>::resize(size_t newsize){
	XTAL_ASSERT(newsize>=0);

	size_t oldsize = size();
	if(newsize>oldsize){
		upsize(newsize-oldsize);
	}
	else{
		downsize_n(newsize);
	}
}

template<class T>
void PODStack<T>::upsize_detail(size_t us){
	//XTAL_ASSERT(us>=0);

	current_-=us;

	size_t oldsize = size();
	size_t oldcapa = capacity();
	size_t newsize = oldsize+us;
	T* oldp = begin_;
	size_t newcapa = oldcapa==0 ? us : 4 + us + oldcapa + oldcapa/2;
	T* newp = (T*)stack_allocate(sizeof(T)*(newcapa+1))+1;
	std::memcpy(newp, oldp, sizeof(T)*oldsize);
	stack_deallocate(oldp-1, sizeof(T)*(oldcapa+1));
	begin_ = newp;
	current_ = begin_+newsize-1;
	end_ = begin_+newcapa;
}

template<class T>
void PODStack<T>::erase(size_t i){
	for(size_t j = i; j != 0; --j){
		(*this)[j] = (*this)[j-1];
	}
	downsize(1);
}

template<class T>
void PODStack<T>::erase(size_t i, size_t n){
	for(size_t j = i; j != n-1; --j){
		(*this)[j] = (*this)[j-n];
	}
	downsize(n);
}

template<class T>
void PODStack<T>::insert(size_t i, const T& v){
	upsize(1);
	for(size_t j = 0; j != i; ++j){
		(*this)[j] = (*this)[j+1];
	}
	(*this)[i] = v;
}

template<class T>
void PODStack<T>::reserve(size_t capa){
	if(capa<=capacity()){
		return;
	}

	size_t diff = capa-size();
	upsize(diff);
	downsize(diff);
}

///////////////////////////////////////////////////////////////////

class PODArrayBase{
public:
	PODArrayBase(uint_t one_size, uint_t size = 0);
	PODArrayBase(const PODArrayBase& v);
	PODArrayBase& operator =(const PODArrayBase& v);
	~PODArrayBase();

	uint_t length() const{ return size_; }
	uint_t size() const{ return size_; }
	uint_t capacity() const{ return capa_; }
	void resize(uint_t sz);
	void upsize(uint_t sz);
	void downsize(uint_t sz);
	const void* at(int_t i) const{ return plusp(values_, i); }
	void set_at(int_t i, const void* v);
	void push_front(const void* v){ insert(0, v); }
	void pop_front(){ erase(0); }
	void push_back(const void* v);
	void pop_back();
	const void* front() const{ return at(0); }
	const void* back() const{ return at(size()-1); }
	void erase(int_t i, int_t n = 1);
	void insert(int_t i, const void* v);
	bool empty() const{ return size_ == 0; }
	bool is_empty() const{ return size_ == 0; }
	void clear();
	void shrink_to_fit();
	void* operator [](size_t i){ return plusp(values_, i); }
	const void* operator [](size_t i) const{ return plusp(values_, i); }
	void* data(){ return values_; }
	void init(const void* values, uint_t size);
	void destroy();

private:
	void* plusp(const void* p, size_t v) const{ return (u8*)p + v*one_size_; }
	void* minusp(const void* p, size_t v) const{ return (u8*)p - v*one_size_; }
	void* addp(void*& p, size_t v){ return p = plusp(p, v); }
	void* subp(void*& p, size_t v){ return p = minusp(p, v); }
	void* incp(void*& p){ return addp(p, 1); }
	void* decp(void*& p){ return subp(p, 1); }

protected:
	void* values_;
	uint_t size_;
	uint_t capa_;
	uint_t one_size_;
};

/**
* \brief 配列
*/
template<class T>
class PODArray{
public:
	PODArray(uint_t size = 0):impl_(sizeof(T), size){}
	uint_t length() const{ return impl_.length(); }
	uint_t size() const{ return impl_.size(); }
	uint_t capacity() const{ return impl_.capacity(); }
	void resize(uint_t sz){ return impl_.resize(sz); }
	void upsize(uint_t sz){ return impl_.upsize(sz); }
	void downsize(uint_t sz){ return impl_.downsize(sz); }
	const T& at(int_t i) const{ return *(T*)impl_.at(i); }
	void set_at(int_t i, const T& v){ return impl_.set_at(i, v); }
	void push_front(const T& v){ return impl_.push_front(&v); }
	void pop_front(){ return impl_.pop_front(); }
	void push_back(const T& v){ return impl_.push_back(&v); }
	void pop_back(){ return impl_.pop_back(); }
	const T& front() const{ return *(T*)impl_.front(); }
	const T& back() const{ return *(T*)impl_.back(); }
	void erase(int_t i, int_t n = 1){ return impl_.erase(i, n); }
	void insert(int_t i, const T& v){ return impl_.insert(i, &v); }
	bool empty() const{ return impl_.empty(); }
	bool is_empty() const{ return impl_.is_empty(); }
	void clear(){ return impl_.clear(); }
	void shrink_to_fit(){ return impl_.shrink_to_fit(); }
	T* data(){ return (T*)impl_.data(); }
	void destroy(){ return impl_.destroy(); }
	T& operator [](size_t i){ return *(T*)impl_[i]; }
	const T& operator [](size_t i) const{ return *(T*)impl_[i]; }
protected:
	PODArrayBase impl_;
};

/**
* \brief 配列
*/
template<class T>
class TArray{
public:
	TArray(){ capa_ = 0; size_ = 0; values_ = 0; }
	TArray(const T* first, const T* end){ init(first, end-first); }
	TArray(const TArray& v){ init(v.values_, v.size()); }
	TArray& operator =(const TArray<T>& v);
	~TArray(){ destroy(); }

	void destroy();
	void clear();
	uint_t size() const{ return size_; }
	void resize(uint_t sz);
	void upsize(uint_t sz);
	void downsize(uint_t sz);
	void push_back(const T& v);
	void push(const T& v){ push_back(v); }
	void pop_back();
	void pop(){ pop_back(); }
	void erase(int_t i, int_t n = 1);

	T& operator [](size_t i){ return values_[i]; }
	const T& operator [](size_t i) const{ return values_[i]; }

private:
	void init(const T* values, uint_t size);

private:
	T* values_;
	uint_t size_;
	uint_t capa_;

};

template<class T>
TArray<T>& TArray<T>::operator =(const TArray<T>& v){
	TArray<T> temp(v);
	std::swap(values_, temp.values_);
	std::swap(size_, temp.size_);
	std::swap(capa_, temp.capa_);
	return *this;
}

template<class T>
void TArray<T>::destroy(){
	clear();
	xfree(values_, sizeof(T)*capa_);
	capa_ = 0;
	values_ = 0;
}

template<class T>
void TArray<T>::clear(){
	for(uint_t i=0; i<size_; ++i){
		values_[i].~T();
	}
	size_ = 0;
}

template<class T>
void TArray<T>::resize(uint_t sz){
	if(sz<size_){
		downsize(size_-sz);
	}
	else if(sz>size_){
		upsize(sz-size_);
	}
}

template<class T>
void TArray<T>::upsize(uint_t sz){
	if(size_+sz>capa_){ // todo overflow check
		if(capa_!=0){
			uint_t newcapa = size_+sz+capa_+1;
			T* newp = (T*)xmalloc(sizeof(T)*newcapa);
			
			for(uint_t i=0; i<size_; ++i){
				new(newp+i) T(*(values_+i));
			}

			for(uint_t i=size_; i<size_+sz; ++i){
				new(newp+i) T();
			}

			for(uint_t i=0; i<size_; ++i){
				(values_+i)->~T();
			}

			xfree(values_, sizeof(T)*capa_);
			values_ = newp;
			size_ += sz;
			capa_ = newcapa;
		}
		else{
			// 一番最初のリサイズは、きっかりに取る
			uint_t newcapa = sz;
			values_ = (T*)xmalloc(sizeof(T)*newcapa);
			for(uint_t i=0; i<sz; ++i){
				new(values_+i) T();
			}
			size_ = sz;
			capa_ = newcapa;
		}
	}
	else{
		for(uint_t i=size_; i<size_+sz; ++i){
			new(values_+i) T();
		}
		size_ += sz;
	}
}

template<class T>
void TArray<T>::downsize(uint_t sz){
	for(uint_t i=size_-sz; i<size_; ++i){
		values_[i].~T();
	}
	size_ -= sz;
}

template<class T>
void TArray<T>::push_back(const T& v){
	if(capa_==size_){
		upsize(1);
		(*this)[size_-1] = v;
	}
	else{
		size_++;
		new(values_+size_-1) T(v);
	}
}

template<class T>
void TArray<T>::pop_back(){
	XTAL_ASSERT(size_ != 0);
	size_--;
	values_[size_].~T();
}

template<class T>
void TArray<T>::init(const T* values, uint_t size){
	capa_ = size;
	size_ = size;
	if(capa_!=0){
		values_ = (T*)xmalloc(sizeof(T)*capa_);
		for(uint_t i=0; i<size; ++i){
			new(values_+i) T(*(values+i));
		}
	}
	else{
		values_ = 0;
	}
}

template<class T>
void TArray<T>::erase(int_t start, int_t n){
	if(n==0){
		return;
	}

	XTAL_ASSERT(0<=start && (uint_t)start<size_);

	if(size_-(start+n)!=0){
		for(uint_t i=0; i<(size_-(start+n)); ++i){
			new(values_+start+i) T(*(values_+start+n+i));
		}
	}
	size_ -= n;
}

}


#endif // XTAL_STACK_H_INCLUDE_GUARD
