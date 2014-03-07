
#include "xtal.h"
#include "xtal_macro.h"


namespace xtal{

const char_t* cast_const_char_t_ptr(const AnyPtr& a){
	return unchecked_ptr_cast<String>(a)->c_str(); 
}

IDPtr cast_IDPtr(const AnyPtr& a){
	if(a){
		const StringPtr& s = unchecked_ptr_cast<String>(a);
		if(!s->is_interned()){
			return s->intern();
		}
	}
	return unchecked_ptr_cast<ID>(a);
}

}
