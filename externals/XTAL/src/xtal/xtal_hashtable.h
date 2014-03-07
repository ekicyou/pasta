/** \file src/xtal/xtal_hashtable.h
* \brief src/xtal/xtal_hashtable.h
*/

#ifndef XTAL_HASHTABLE_H_INCLUDE_GUARD
#define XTAL_HASHTABLE_H_INCLUDE_GUARD

#pragma once

namespace xtal{

template<class Key, class Val, class Fun>
class OrderedHashtable{
public:

	typedef std::pair<Key, Val> pair_t;

	struct Node{
		pair_t pair;
		Node* next;
		Node* ordered_next;
		Node* ordered_prev;

		Key& key(){ return pair.first; }
		Val& value(){ return pair.second; }
		
		Node()
			:next(0), ordered_next(0), ordered_prev(0){}

		Node(const Key& key, const Val& value)
			:pair(key, value), next(0), ordered_next(0), ordered_prev(0){}
	};

	typedef Key key_type;
	typedef Val value_type;
	typedef Fun fun_type;

public:

	class iterator{
	public:
		
		iterator(Node* node = 0)
			:node_(node){}
		
		pair_t& operator *() const{
			return node_->pair;
		}

		pair_t* operator ->() const{
			return &node_->pair;
		}

		iterator& operator ++(){
			node_ = node_->ordered_next;
			return *this;
		}

		iterator operator ++(int){
			iterator temp(*this);
			node_ = node_->ordered_next;
			return temp;
		}

		friend bool operator ==(iterator a, iterator b){
			return a.node_ == b.node_;
		}

		friend bool operator !=(iterator a, iterator b){
			return a.node_ != b.node_;
		}

		operator bool() const{
			return node_!=0;
		}

		Node* to_node() const{
			return node_;
		}

	private:
		Node* node_;
	};

	typedef iterator const_iterator;

	iterator begin(){
		return iterator(ordered_head_);
	}

	iterator end(){
		return iterator(0);
	}

	const_iterator begin() const{
		return iterator(ordered_head_);
	}

	const_iterator end() const{
		return iterator(0);
	}

public:

	struct no_use_memory_t{};
	
	OrderedHashtable(no_use_memory_t);

	/**
	* \brief 空のハッシュテーブルを生成する 
	*
	*/
	OrderedHashtable();

	OrderedHashtable(const OrderedHashtable<Key, Val, Fun>& v);

	OrderedHashtable<Key, Val, Fun>& operator=(const OrderedHashtable<Key, Val, Fun>& v);

	~OrderedHashtable();

	void destroy();
		
	/**
	* \brief iに対応する要素を返す
	*
	*/
	iterator find(const Key& key){
		return find(key, Fun::hash(key));
	}

	/**
	* \brief iに対応する要素を返す
	*
	*/
	iterator find(const Key& key, uint_t hash);

	/**
	* \brief iに対応する要素を設定する
	*
	*/	
	std::pair<iterator, bool> insert(const Key& key, const Val& value){
		return insert(key, value, Fun::hash(key));
	}

	/**
	* \brief iに対応する要素を設定する
	*
	*/	
	std::pair<iterator, bool> insert(const Key& key, const Val& value, uint_t hash);

	/**
	* \brief keyに対応する値を削除する
	*
	*/
	void erase(const Key& key);

	iterator erase(iterator it){
		iterator ret = it;
		++ret;
		erase(it->first);
		return ret;
	}

	/**
	* \brief 連想配列の容量を返す
	*
	*/
	uint_t size(){
		return used_size_;
	}

	/**
	* \brief 空か調べる
	*
	*/
	bool empty(){
		return used_size_==0;
	}

	Val& operator [](const Key& key);

	void clear();

	float_t rate(){
		return used_size_/(float_t)size_;
	}
	
	void expand(int_t addsize);

protected:

	uint_t calc_index(uint_t hash){
		return calc_index(hash, size_);
	}

	uint_t calc_index(uint_t hash, uint_t size){
		return hash % size;
	}

	void set_zero();

protected:

	Node** begin_;
	Node* ordered_head_;
	Node* ordered_tail_;
	uint_t size_;
	uint_t used_size_;
};

template<class Key, class Val, class Fun>
void OrderedHashtable<Key, Val, Fun>::set_zero(){
	size_ = 0;
	begin_ = 0;
	used_size_ = 0;
	ordered_head_ = 0;
	ordered_tail_ = 0;
}

template<class Key, class Val, class Fun>
OrderedHashtable<Key, Val, Fun>::OrderedHashtable(no_use_memory_t){
	set_zero();
}

template<class Key, class Val, class Fun>
OrderedHashtable<Key, Val, Fun>::OrderedHashtable(){
	set_zero();
	expand(5);
}

template<class Key, class Val, class Fun>
OrderedHashtable<Key, Val, Fun>::OrderedHashtable(const OrderedHashtable<Key, Val, Fun>& v){
	set_zero();
	expand(5);

	for(const_iterator it=v.begin(); it!=v.end(); ++it){
		insert(it->first, it->second);
	}
}

template<class Key, class Val, class Fun>
OrderedHashtable<Key, Val, Fun>& OrderedHashtable<Key, Val, Fun>::operator=(const OrderedHashtable<Key, Val, Fun>& v){
	if(this==&v){ return *this; }
	clear();
	for(const_iterator it=v.begin(); it!=v.end(); ++it){
		insert(it->first, it->second);
	}
	return *this;
}

template<class Key, class Val, class Fun>
OrderedHashtable<Key, Val, Fun>::~OrderedHashtable(){
	clear();
	xfree(begin_, sizeof(Node*)*size_);
}

template<class Key, class Val, class Fun>
void OrderedHashtable<Key, Val, Fun>::destroy(){
	clear();
	xfree(begin_, sizeof(Node*)*size_);
	set_zero();
}

template<class Key, class Val, class Fun>
typename OrderedHashtable<Key, Val, Fun>::iterator OrderedHashtable<Key, Val, Fun>::find(const Key& key, uint_t hash){
	Node* p = begin_[calc_index(hash)];
	while(p){
		if(Fun::eq(p->pair.first, key)){
			return iterator(p);
		}
		p = p->next;
	}
	return end();
}

template<class Key, class Val, class Fun>
Val& OrderedHashtable<Key, Val, Fun>::operator [](const Key& key){
	uint_t hash = Fun::hash(key);
	Node** p = &begin_[calc_index(hash)];
	while(*p){
		if(Fun::eq((*p)->pair.first, key)){
			return (*p)->pair.second;
		}
		p = &(*p)->next;
	}

	*p = (Node*)xmalloc(sizeof(Node));
	Node* ret = *p;
	new(ret) Node(key, Val());

	if(ordered_tail_){
		ordered_tail_->ordered_next = ret;
		(ret)->ordered_prev = ordered_tail_;
		ordered_tail_ = ret;
	}
	else{
		ordered_head_ = ret;
		ordered_tail_ = ret;
	}

	used_size_++;
	if(rate()>0.8f){
		expand(1);
	}

	return ret->pair.second;
}

template<class Key, class Val, class Fun>
std::pair<typename OrderedHashtable<Key, Val, Fun>::iterator, bool> OrderedHashtable<Key, Val, Fun>::insert(const Key& key, const Val& value, uint_t hash){
	Node* p = begin_[calc_index(hash)];
	while(p){
		if(Fun::eq(p->pair.first, key)){
			p->pair.second = value;
			return std::pair<iterator, bool>(p, false);
		}
		p = p->next;
	}

	used_size_++;
	if(rate()>0.8f){
		expand(1);
	}

	Node* ret = (Node*)xmalloc(sizeof(Node));
	new(ret) Node(key, value);

	Node** pp = &begin_[calc_index(hash)];
	ret->next = *pp;
	*pp = ret;

	if(ordered_tail_){
		ordered_tail_->ordered_next = ret;
		(ret)->ordered_prev = ordered_tail_;
		ordered_tail_ = ret;
	}
	else{
		ordered_head_ = ret;
		ordered_tail_ = ret;
	}

	return std::pair<iterator, bool>(ret, true);
}

template<class Key, class Val, class Fun>
void OrderedHashtable<Key, Val, Fun>::erase(const Key& key){
	uint_t hash = Fun::hash(key);
	uint_t pos = calc_index(hash);
	Node* p = begin_[pos];
	Node* prev = 0;
	while(p){
		if(Fun::eq(p->pair.first, key)){
			if(prev){
				prev->next = p->next;
			}
			else{
				begin_[pos] = p->next;
			}

			if(p==ordered_head_){
				ordered_head_ = p->ordered_next;
			}

			if(p==ordered_tail_){
				ordered_tail_ = p->ordered_prev;
			}

			if(p->ordered_next) p->ordered_next->ordered_prev = p->ordered_prev;
			if(p->ordered_prev) p->ordered_prev->ordered_next = p->ordered_next;

			p->~Node();
			xfree(p, sizeof(Node));

			used_size_--;
			break;
		}
		prev = p;
		p = p->next;
	}
}

template<class Key, class Val, class Fun>
void OrderedHashtable<Key, Val, Fun>::clear(){
	for(Node* p = ordered_head_; p;){
		Node* next = p->ordered_next;
		p->~Node();
		xfree(p, sizeof(Node));
		p = next;
	}

	for(uint_t i = 0; i<size_; ++i){
		begin_[i] = 0;
	}

	used_size_ = 0;
	ordered_head_ = 0;
	ordered_tail_ = 0;
}

template<class Key, class Val, class Fun>
void OrderedHashtable<Key, Val, Fun>::expand(int_t addsize){
	Node** oldbegin = begin_;
	uint_t oldsize = size_;

	size_ = size_*2 + addsize;
	begin_ = (Node**)xmalloc(sizeof(Node*)*size_);
	
	for(uint_t i = 0; i<size_; ++i){
		begin_[i] = 0;
	}

	for(Node* p = ordered_head_; p; p=p->ordered_next){
		p->next = 0;
	}

	for(Node* p = ordered_head_; p; p=p->ordered_next){
		Node** temp = &begin_[calc_index(Fun::hash(p->pair.first))];
		while(*temp){
			temp = &(*temp)->next;
		}
		*temp = p;
	}

	xfree(oldbegin, sizeof(Node*)*oldsize);
}

template<class Key, class Val, class Fun>
void visit_members(Visitor& m, const OrderedHashtable<Key, Val, Fun>& values){
	for(typename OrderedHashtable<Key, Val, Fun>::const_iterator it = values.begin(); it!=values.end(); ++it){
		m & it->first & it->second;
	}
}

template<class Key, class Val, class Fun>
class Hashtable{
public:

	typedef std::pair<Key, Val> pair_t;

	struct Node{
		pair_t pair;
		Node* next;

		Key& key(){ return pair.first; }
		Val& value(){ return pair.second; }
		
		Node()
			:next(0){}

		Node(const Key& key, const Val& value)
			:pair(key, value), next(0){}
	};

	typedef Key key_type;
	typedef Val value_type;
	typedef Fun fun_type;

public:

	class iterator{
	public:
		iterator(Node** last)
			:first_(last), last_(last), node_(0){
		}

		iterator(Node** first, Node** last)
			:first_(first), last_(last), node_(0){
			++(*this);
		}

		iterator(Node** first, Node** last, Node* node)
			:first_(first), last_(last), node_(node){}

		pair_t& operator *() const{
			return node_->pair;
		}

		pair_t* operator ->() const{
			return &node_->pair;
		}

		iterator& operator ++();

		iterator operator ++(int);

		friend bool operator ==(iterator a, iterator b){
			return a.node_ == b.node_;
		}

		friend bool operator !=(iterator a, iterator b){
			return a.node_ != b.node_;
		}

		Node* to_node() const{
			return node_;
		}

	private:
		Node** first_;
		Node** last_;
		Node* node_;
	};

	typedef iterator const_iterator;

	iterator begin(){
		return iterator(begin_, begin_+size_);
	}

	iterator end(){
		return iterator(begin_+size_);
	}

	const_iterator begin() const{
		return iterator(begin_, begin_+size_);
	}

	const_iterator end() const{
		return iterator(begin_+size_);
	}

public:

	struct no_use_memory_t{};
	
	Hashtable(no_use_memory_t);

	/**
	* \brief 空のハッシュテーブルを生成する 
	*
	*/
	Hashtable();

	Hashtable(const Hashtable<Key, Val, Fun>& v);

	Hashtable<Key, Val, Fun>& operator=(const Hashtable<Key, Val, Fun>& v);

	~Hashtable();

	void destroy();
		
	/**
	* \brief iに対応する要素を返す
	*
	*/
	Node* find(const Key& key){
		return find(key, Fun::hash(key));
	}

	/**
	* \brief iに対応する要素を返す
	*
	*/
	Node* find(const Key& key, uint_t hash);

	template<class NKey>
	iterator find(const NKey& key, uint_t hash){
		Node** pp = &begin_[calc_index(hash)];
		Node* p = *pp;
		while(p){
			if(Fun::eq(p->pair.first, key)){
				return iterator(pp, begin_+size_, p);
			}
			p = p->next;
		}
		return end();
	}

	template<class NKey>
	Node* find(const NKey& key){
		return find(key, Fun::hash(key));
	}

	/**
	* \brief iに対応する要素を設定する
	*
	*/	
	Node* insert(const Key& key, const Val& value){
		return insert(key, value, Fun::hash(key));
	}

	/**
	* \brief iに対応する要素を設定する
	*
	*/	
	Node* insert(const Key& key, const Val& value, uint_t hash);

	/**
	* \brief keyに対応する値を削除する
	*
	*/
	void erase(const Key& key);

	/**
	* \brief 連想配列の容量を返す
	*
	*/
	uint_t size(){
		return used_size_;
	}

	/**
	* \brief 空か調べる
	*
	*/
	bool empty(){
		return used_size_==0;
	}

	void clear();

	float_t rate(){
		return used_size_/(float_t)size_;
	}
	
	void expand(int_t addsize);

protected:

	uint_t calc_index(uint_t hash){
		return calc_index(hash, size_);
	}

	uint_t calc_index(uint_t hash, uint_t size){
		return hash % size;
	}

	void set_zero();

protected:
	Node** begin_;
	uint_t size_;
	uint_t used_size_;
};

template<class Key, class Val, class Fun>
void Hashtable<Key, Val, Fun>::set_zero(){
	size_ = 0;
	begin_ = 0;
	used_size_ = 0;
}

template<class Key, class Val, class Fun>
typename Hashtable<Key, Val, Fun>::iterator& Hashtable<Key, Val, Fun>::iterator::operator ++(){
	if(node_){
		if(node_->next){
			node_ = node_->next;
			return *this;
		}

		node_ = 0;
		++first_;
	}

	for(;;){
		if(first_==last_){
			node_ = 0;
			return *this;
		}

		if(*first_){
			node_ = *first_;
			return *this;
		}

		++first_;
	}
}

template<class Key, class Val, class Fun>
typename Hashtable<Key, Val, Fun>::iterator Hashtable<Key, Val, Fun>::iterator::operator ++(int){
	iterator temp(*this);
	++(*this);
	return temp;
}

template<class Key, class Val, class Fun>
Hashtable<Key, Val, Fun>::Hashtable(no_use_memory_t){
	set_zero();
}

template<class Key, class Val, class Fun>
Hashtable<Key, Val, Fun>::Hashtable(){
	set_zero();
	expand(5);
}

template<class Key, class Val, class Fun>
Hashtable<Key, Val, Fun>::Hashtable(const Hashtable<Key, Val, Fun>& v){
	set_zero();
	expand(5);

	for(const_iterator it=v.begin(); it!=v.end(); ++it){
		insert(it->first, it->second);
	}
}

template<class Key, class Val, class Fun>
Hashtable<Key, Val, Fun>& Hashtable<Key, Val, Fun>::operator=(const Hashtable<Key, Val, Fun>& v){
	if(this==&v){ return *this; }
	clear();
	for(const_iterator it=v.begin(); it!=v.end(); ++it){
		insert(it->first, it->second);
	}
	return *this;
}

template<class Key, class Val, class Fun>
Hashtable<Key, Val, Fun>::~Hashtable(){
	clear();
	xfree(begin_, sizeof(Node*)*size_);
}

template<class Key, class Val, class Fun>
void Hashtable<Key, Val, Fun>::destroy(){
	clear();
	xfree(begin_, sizeof(Node*)*size_);
	set_zero();
}

template<class Key, class Val, class Fun>
typename Hashtable<Key, Val, Fun>::Node* Hashtable<Key, Val, Fun>::find(const Key& key, uint_t hash){
	Node* p = begin_[calc_index(hash)];
	while(p){
		if(Fun::eq(p->pair.first, key)){
			return p;
		}
		p = p->next;
	}
	return 0;
}

template<class Key, class Val, class Fun>
typename Hashtable<Key, Val, Fun>::Node* Hashtable<Key, Val, Fun>::insert(const Key& key, const Val& value, uint_t hash){
	Node* p = begin_[calc_index(hash)];
	while(p){
		if(Fun::eq(p->pair.first, key)){
			p->pair.second = value;
			return p;
		}
		p = p->next;
	}

	used_size_++;
	if(rate()>0.8f){
		expand(1);
	}

	Node* ret = (Node*)xmalloc(sizeof(Node));
	new(ret) Node(key, value);

	Node** pp = &begin_[calc_index(hash)];
	ret->next = *pp;
	*pp = ret;

	return ret;
}

template<class Key, class Val, class Fun>
void Hashtable<Key, Val, Fun>::erase(const Key& key){
	uint_t hash = Fun::hash(key);
	uint_t pos = calc_index(hash);
	Node* p = begin_[pos];
	Node* prev = 0;
	while(p){
		if(Fun::eq(p->pair.first, key)){
			if(prev){
				prev->next = p->next;
			}
			else{
				begin_[pos] = p->next;
			}

			p->~Node();
			xfree(p, sizeof(Node));

			used_size_--;
			break;
		}
		prev = p;
		p = p->next;
	}
}

template<class Key, class Val, class Fun>
void Hashtable<Key, Val, Fun>::clear(){
	for(iterator it=begin(), last=end(); it!=last; ){
		Node* node = it.to_node();
		++it;
		node->~Node();
		xfree(node, sizeof(Node));
	}

	for(uint_t i = 0; i<size_; ++i){
		begin_[i] = 0;
	}

	used_size_ = 0;
}

template<class Key, class Val, class Fun>
void Hashtable<Key, Val, Fun>::expand(int_t addsize){
	uint_t size2 = size_*2 + addsize;
	Node** begin2 = (Node**)xmalloc(sizeof(Node*)*size2);
	
	for(uint_t i=0; i<size2; ++i){
		begin2[i] = 0;
	}

	for(iterator it=begin(), last=end(); it!=last; ){
		Node** p = &begin2[calc_index(Fun::hash(it->first), size2)];
		Node* node = it.to_node();
		++it;
		node->next = *p;
		*p = node;
	}
	xfree(begin_, sizeof(Node*)*size_);
	begin_ = begin2;
	size_ = size2;
}

template<class Key, class Val, class Fun>
void visit_members(Visitor& m, const Hashtable<Key, Val, Fun>& values){
	for(typename Hashtable<Key, Val, Fun>::const_iterator it = values.begin(); it!=values.end(); ++it){
		m & it->first & it->second;
	}
}

}

#endif // XTAL_HASHTABLE_H_INCLUDE_GUARD
