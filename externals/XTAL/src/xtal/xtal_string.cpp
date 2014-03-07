#include "xtal.h"
#include "xtal_macro.h"

#include "xtal_stringspace.h"
#include "xtal_details.h"

namespace xtal{

uint_t string_hashcode(const char_t* data, uint_t len){
	uint_t h = hash_seed ^ len;

	while(len >= 4){
		uint_t k = data[0];
		k |= data[1] << 8;
		k |= data[2] << 16;
		k |= data[3] << 24;

		k *= hash_m; 
		k ^= k >> hash_r; 
		k *= hash_m;

		h *= hash_m;
		h ^= k;

		data += 4;
		len -= 4;
	}
	
	switch(len){
	case 3: h ^= data[2] << 16;
	case 2: h ^= data[1] << 8;
	case 1: h ^= data[0];
	        h *= hash_m;
	};

	h ^= h >> 13;
	h *= hash_m;
	h ^= h >> 15;

	return h;
}

void string_data_size_and_hashcode(const char_t* str, uint_t& size, uint_t& hash){
	size = string_data_size(str);
	hash = string_hashcode(str, size);
}

uint_t string_length(const char_t* str){
	ChMaker chm;
	uint_t length = 0;
	uint_t i=0;
	while(str[i]){
		chm.clear();
		while(!chm.is_completed()){
			if(str[i]){ 
				chm.add(str[i++]); 
			} 
			else{ 
				break; 
			}
		}
		length += 1;
	}
	return length;
}

uint_t string_data_size(const char_t* str){
	uint_t ret = 0;
	while(*str++){
		ret++;
	}
	return ret;
}

int_t string_compare(const char_t* a, uint_t asize, const char_t* b, uint_t bsize){
	if(a==b){ return 0; }
	if(asize<bsize){ return -1; }
	if(bsize<asize){ return 1; }

	for(uint_t i=0; i<asize; ++i){
		int_t diff = (int_t)((uchar_t*)a)[i] - (int_t)((uchar_t*)b)[i];
		if(diff!=0){
			return diff;
		}
	}

	return 0;
}

void string_copy(char_t* a, const char_t* b, uint_t size){
	while(size!=0){
		*a++ = *b++;
		size--;
	}
}

bool string_is_ch(const char_t* str, uint_t size){
	ChMaker chm;
	for(uint_t i=0; i<size; ++i){
		chm.add(str[i]);
		if(chm.is_completed()){
			return i+1==size;
		}
	}
	return false;
}

void ChRangeIter::block_next(const VMachinePtr& vm){
	if(ch_cmp(it_->data(), it_->data_size(), end_->data(), end_->data_size())>0){
		vm->return_result(null, null);
		return;
	}

	StringPtr temp = it_;
	char_t buf[8];
	int_t sz = ch_inc(it_->data(), it_->data_size(), buf, 8);
	it_ = XNew<String>(buf, sz);
	vm->return_result(to_smartptr(this), temp);
}

AnyPtr ChRange::each(){
	return XNew<ChRangeIter>(to_smartptr(this));
}

////////////////////////////////////////////////////////////////

int_t edit_distance(const StringPtr& str1, const StringPtr& str2){
	return edit_distance(str1->data(), str1->data_size()*sizeof(char_t), str2->data(), str2->data_size()*sizeof(char_t));
}

////////////////////////////////////////////////////////////////

StringData* String::new_string_data(uint_t size){
	StringData* p = new(make_object<StringData>()) StringData(size);
	value_.init_rcbase(TYPE_STRING, p);
	//register_gc(p);
	return p;
}

void String::init_small_string(const char_t* str, uint_t size){
	value_.init_small_string(str, size);
}

void String::init_string(const char_t* str, uint_t size){
	if(size<SMALL_STRING_MAX){
		init_small_string(str, size);
	}
	else{
		if(string_is_ch(str, size)){
			*this = ID(str, size);
		}
		else{
			StringData* sd = new_string_data(size);
			string_copy(sd->buf(), str, size);
		}
	}
}
	
void String::init_long_lived_string(const char_t* str, uint_t size){
	if(size<SMALL_STRING_MAX){
		init_small_string(str, size);
	}
	else{
		if(string_is_ch(str, size)){
			*this = ID(str, size);
		}
		else{
			value_.init_long_lived_string(str, size);
		}
	}
}

String::String(){
	value_.init_small_string(0);
}

String::String(const char_t* str){
	init_string(str, string_data_size(str));
}

String::String(const char_t* str, uint_t size){
	init_string(str, size);
}

String::String(const char_t* str1, uint_t size1, const char_t* str2, uint_t size2){
	if(size1==0){
		init_string(str2, size2);
		return;
	}
	else if(size2==0){
		init_string(str1, size1);
		return;
	}

	uint_t sz = size1 + size2;
	if(sz<SMALL_STRING_MAX){
		value_.init_small_string(sz);
		string_copy(XTAL_detail_svalue(*this), str1, size1);
		string_copy(XTAL_detail_svalue(*this)+size1, str2, size2);
	}
	else{
		StringData* sd = new_string_data(sz);
		string_copy(sd->buf(), str1, size1);
		string_copy(sd->buf()+size1, str2, size2);
	}
}

String::String(const String& s)
	:Any(s){
}
	
uint_t String::length() const{
	return string_length(c_str());
}

const char_t* String::c_str() const{
	return data();
}

const char_t* String::data() const{
	switch(XTAL_detail_type(*this)){
		XTAL_NODEFAULT;
		XTAL_CASE(TYPE_SMALL_STRING){ return XTAL_detail_svalue(*this); }
		XTAL_CASE(TYPE_LONG_LIVED_STRING){ return XTAL_detail_spvalue(*this); }
		XTAL_CASE(TYPE_INTERNED_STRING){ return XTAL_detail_spvalue(*this); }
		XTAL_CASE(TYPE_STRING){ return ((StringData*)XTAL_detail_rcpvalue(*this))->buf(); }
	}
	return XTAL_L("");
}

uint_t String::data_size() const{
	switch(XTAL_detail_type(*this)){
		XTAL_NODEFAULT;

		XTAL_CASE3(TYPE_SMALL_STRING, TYPE_INTERNED_STRING, TYPE_LONG_LIVED_STRING){ 
			return XTAL_detail_ssize(*this); 
		}

		XTAL_CASE(TYPE_STRING){ return ((StringData*)XTAL_detail_rcpvalue(*this))->data_size(); }
	}
	return 0;
}

const StringPtr& String::clone() const{
	return to_smartptr(this);
}

IDPtr String::intern() const{
	switch(XTAL_detail_type(*this)){
		XTAL_NODEFAULT;
		XTAL_CASE2(TYPE_SMALL_STRING, TYPE_INTERNED_STRING){ return unchecked_ptr_cast<ID>(ap(*this)); }
		XTAL_CASE(TYPE_LONG_LIVED_STRING){ return xtal::intern(XTAL_detail_spvalue(*this), XTAL_detail_ssize(*this)); }
		XTAL_CASE(TYPE_STRING){
			StringData* p = ((StringData*)XTAL_detail_rcpvalue(*this));
			return xtal::intern(p->buf(), p->data_size());
		}
	}
	return empty_id;
}

bool String::is_interned() const{
	switch(XTAL_detail_type(*this)){
		XTAL_NODEFAULT;
		XTAL_CASE2(TYPE_SMALL_STRING, TYPE_INTERNED_STRING){ return true; }
		XTAL_CASE2(TYPE_LONG_LIVED_STRING, TYPE_STRING){ return false; }
	}
	return false;
}

const StringPtr& String::to_s() const{
	return to_smartptr(this);
}

int_t String::to_i() const{
	const char_t* str = data();
	int_t ret = 0;
	int_t start = 0, sign = 1;
	
	if(str[0]=='-'){
		sign = -1;
		start = 1;
	}
	else if(str[0]=='+'){
		start = 1;
	}

	for(uint_t i=start, sz=data_size(); i<sz; ++i){
		ret *= 10;
		ret += str[i] - '0';
	}
	return ret*sign; 
}

float_t String::to_f() const{
	const char_t* str = data();
	float_t ret = 0;
	float_t scale = 10;
	int_t start = 0, sign = 1;
	
	if(str[0]=='-'){
		sign = -1;
		start = 1;
	}
	else if(str[0]=='+'){
		start = 1;
	}

	for(uint_t i=start, sz=data_size(); i<sz; ++i){
		if(str[i]=='.'){
			scale = 0.1f;
			continue;
		}

		ret *= scale;
		ret += str[i] - '0';
	} 
	return ret*sign; 
}

AnyPtr String::each() const{
	return send(XTAL_DEFINED_ID(each));
}

bool String::is_ch() const{
	return string_is_ch(data(), data_size());
}

bool String::is_empty() const{
	return XTAL_detail_raweq(*this, empty_string)!=0;
}

int_t String::ascii() const{
	if(data_size()==0){
		return -1;
	}

	int_t ch = (uchar_t)c_str()[0];
	return ch<128 ? ch : 255;
}
	
bool String::op_in(const ChRangePtr& range) const{
	const char_t* str = c_str();
	return ch_cmp(str, data_size(), range->left()->c_str(), range->left()->data_size())>=0 &&
		ch_cmp(str, data_size(), range->right()->c_str(), range->right()->data_size())<=0;
}

ChRangePtr String::op_range(const StringPtr& right, int_t kind) const{
	if(kind!=RANGE_CLOSED){
		set_runtime_error(Xt("XRE1025"));
		return xnew<ChRange>(empty_string, empty_string);
	}

	if(length()==1 && right->length()==1){
		return xnew<ChRange>(to_smartptr(this), right);
	}
	else{
		set_runtime_error(Xt("XRE1023"));
		return xnew<ChRange>(empty_string, empty_string);
	}
}

StringPtr String::op_cat(const StringPtr& v) const{
	return XNew<String>(data(), data_size(), v->data(), v->data_size());
}

bool String::op_eq(const StringPtr& v) const{ 
	return string_compare(data(), data_size(), v->data(), v->data_size())==0; 
}

bool String::op_lt(const StringPtr& v) const{
	return string_compare(data(), data_size(), v->data(), v->data_size()) < 0;
}

StringPtr String::cat(const StringPtr& v) const{
	return op_cat(v);
}

AnyPtr String::scan(const AnyPtr& pattern) const{
	return send(Xid(scan), pattern);
}

AnyPtr String::split(const AnyPtr& pattern) const{
	return send(Xid(split), pattern);
}
	
bool String::match(const AnyPtr& pattern) const{
	return send(Xid(match), pattern);
}

StringPtr String::gsub(const AnyPtr& pattern, const AnyPtr& fn) const{
	return ptr_cast<String>(send(Xid(gsub), pattern, fn));
}

StringPtr String::sub(const AnyPtr& pattern, const AnyPtr& fn) const{
	return ptr_cast<String>(send(Xid(sub), pattern, fn));
}

String::iterator String::begin() const{
	return iterator(data());
}

String::iterator String::end() const{
	return iterator(data() + data_size());
}

////////////////////////////////////////////////////////////////

StringData::StringData(uint_t size){
	value_.init_rcbase(TYPE_STRING, this);
	data_size_ = size;
	buf_ = (char_t*)xmalloc(sizeof(char_t)*(size+1));
	buf()[size] = 0;
}

StringData::~StringData(){
	xfree(buf_, sizeof(char_t)*(data_size()+1));
}

////////////////////////////////////////////////////////////////

SmartPtr<ID>::SmartPtr(const char_t* v)
	:Any(ID(v)){}

SmartPtr<ID>::SmartPtr(const StringPtr& v)
	:Any(ID(v)){}

SmartPtr<ID>::SmartPtr(const LongLivedString& v)
	:Any(ID(v)){}

SmartPtr<ID>::SmartPtr(const ID& v)
	:Any(v){}

}
