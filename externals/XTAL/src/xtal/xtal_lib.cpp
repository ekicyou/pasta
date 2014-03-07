#include "xtal.h"
#include "xtal_macro.h"
#include "xtal_codebuilder.h"
#include "xtal_stringspace.h"

namespace xtal{

AutoLoader::AutoLoader()
	:load_path_list_(xnew<Array>()){
}

const AnyPtr& AutoLoader::on_rawmember(const IDPtr& primary_key, const AnyPtr& secondary_key, bool inherited_too, int_t& accessibility, bool& nocache){
	const AnyPtr& ret = Class::on_rawmember(primary_key, secondary_key, inherited_too, accessibility, nocache);
	if(!XTAL_detail_is_undefined(ret)){
		return ret;
	}
	else{

		AnyPtr value;
		Xfor(var, load_path_list_){
			value = undefined;

			CodePtr code;

			if(XTAL_detail_raweq(var, empty_string)){
				code = require_source(primary_key);
			}
			else{
				code = require_source(Xf2("%s/%s", 0, var, 1, primary_key));
			}

			if(code){
				value = code->call();
				
				XTAL_CHECK_EXCEPT(e){
					XTAL_UNUSED_VAR(e);
					return undefined;					
				}
			}
			else{
				XTAL_CATCH_EXCEPT(e){
					if(e->is(cpp_class<CompileError>())){
						XTAL_SET_EXCEPT(e);
						return undefined;
					}
				}
			}

			XTAL_CATCH_EXCEPT(e){ 
				return Class::on_rawmember(primary_key, secondary_key, inherited_too, accessibility, nocache);
			}

			if(!XTAL_detail_is_undefined(value)){
				on_def(primary_key, value, secondary_key, accessibility);
				return Class::on_rawmember(primary_key, secondary_key, inherited_too, accessibility, nocache);
			}
		}

		nocache = true;
		return Class::on_rawmember(primary_key, secondary_key, inherited_too, accessibility, nocache);
	}
}

Lib::Lib(){
	set_object_temporary_name(XTAL_DEFINED_ID(lib));
	set_object_force(1000);
}

void Global::on_def(const IDPtr& primary_key, const AnyPtr& value, const AnyPtr& secondary_key, int_t accessibility){
	if(find_node(primary_key, secondary_key)){
		overwrite_member(primary_key, value, secondary_key, accessibility);
	}
	else{
		Class::on_def(primary_key, value, secondary_key, accessibility);
	}
}

}
