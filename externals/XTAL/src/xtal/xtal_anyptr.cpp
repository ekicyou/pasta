#include "xtal.h"
#include "xtal_macro.h"

namespace xtal{

SmartPtr<Any>::SmartPtr(const char_t* str){
	value_.init_primitive(TYPE_NULL);
	*this = XNew<String>(str);
}

SmartPtr<Any>::SmartPtr(const LongLivedString& str){
	value_.init_primitive(TYPE_NULL);
	*this = XNew<String>(str);
}

void SmartPtr<Any>::set_unknown_pointer(const Base* p){
	XTAL_detail_copy(*this, *p);
	XTAL_detail_inc_ref_count(*this);
}

void SmartPtr<Any>::set_unknown_pointer(const RefCountingBase* p){
	XTAL_detail_copy(*this, *p);
	XTAL_detail_inc_ref_count(*this);
}

void SmartPtr<Any>::set_unknown_pointer(const Any* p){
	XTAL_detail_copy(*this, *p);
	XTAL_detail_inc_ref_count(*this);
}

void SmartPtr<Any>::set_unknown_pointer(const AnyPtr* p){
	XTAL_detail_copy(*this, *p);
	XTAL_detail_inc_ref_count(*this);
}

void SmartPtr<Any>::init(const Any& a){
	XTAL_detail_copy(*this, a);
	XTAL_detail_inc_ref_count(*this);
}

void SmartPtr<Any>::unref_init(const Any& a){
	XTAL_detail_inc_ref_count(a);
	XTAL_detail_dec_ref_count(*this);
	XTAL_detail_copy(*this, a);
}

void SmartPtr<Any>::ref(){
	XTAL_detail_inc_ref_count(*this);
}

void SmartPtr<Any>::unref(){
	XTAL_detail_dec_ref_count(*this);
}

SmartPtr<Any>::SmartPtr(const SmartPtr<Any>& p)
	:Any(p){
	XTAL_detail_inc_ref_count(*this);
}

SmartPtr<Any>::~SmartPtr(){
#ifdef XTAL_CHECK_REF_COUNT
	if(!environment()){
		XTAL_ASSERT(XTAL_detail_type(*this)<TYPE_BASE);
	}
#endif

	XTAL_detail_dec_ref_count(*this);
}	

SmartPtr<Any>::SmartPtr(const IDPtr& p){
	Any::operator=(p);
	XTAL_ASSERT(XTAL_detail_type(*this)<TYPE_BASE);
}

SmartPtr<Any>& SmartPtr<Any>::operator =(const IDPtr& p){
	XTAL_detail_dec_ref_count(*this);
	Any::operator=(p);
	XTAL_ASSERT(XTAL_detail_type(*this)<TYPE_BASE);
	return *this;
}

void visit_members(Visitor& m, RefCountingBase* p){
	XTAL_ASSERT((int)p->ref_count() >= -m.value());
	p->add_ref_count(m.value());
}

void visit_members(Visitor& m, const Any& p){
	if(XTAL_detail_is_rcpvalue(p)){
		XTAL_ASSERT((int)XTAL_detail_rcpvalue(p)->ref_count() >= -m.value());
		XTAL_detail_rcpvalue(p)->add_ref_count(m.value());
	}
}

//////////////////////////////

CppClassSymbolData* CppClassSymbolData::head = 0;

void CppClassSymbolData::init_bind(int n, const char_t* xtname, XTAL_bind_t b){
	if(n==0){
		flags |= FLAG_NAME | FLAG_BIND0;
		prebind = b;
		name = xtname;
		next = head;
		head = this;
	}
	else if(n==1){
		flags |= FLAG_NAME | FLAG_BIND1;
		bind[0] = b;
		name = xtname;
	}
	else{
		flags |= FLAG_BIND2;
		bind[1] = b;
	}
}

}
