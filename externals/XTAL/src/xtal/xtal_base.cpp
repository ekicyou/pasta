#include "xtal.h"
#include "xtal_macro.h"
#include "xtal_stringspace.h"
#include "xtal_objectspace.h"
#include "xtal_details.h"

namespace xtal{

void VirtualMembersN::finalize(RefCountingBase* p){ p->on_finalize(); }
void VirtualMembersN::rawcall(RefCountingBase* p, const VMachinePtr& vm){ p->on_rawcall(vm); }
void VirtualMembersN::def(RefCountingBase* p, const IDPtr& primary_key, const AnyPtr& value, const AnyPtr& secondary_key, int_t accessibility){ p->on_def(primary_key, value, secondary_key, accessibility); }
const AnyPtr& VirtualMembersN::rawmember(RefCountingBase* p, const IDPtr& primary_key, const AnyPtr& secondary_key, bool inherited_too, int_t& accessibility, bool& nocache){ return p->on_rawmember(primary_key, secondary_key, inherited_too, accessibility, nocache); }
const ClassPtr& VirtualMembersN::object_parent(RefCountingBase* p){ return p->on_object_parent(); }
void VirtualMembersN::set_object_parent(RefCountingBase* p, const ClassPtr& parent){ p->on_set_object_parent(parent); }
void VirtualMembersN::visit_members(RefCountingBase* p, Visitor& m){ p->on_visit_members(m); }
void VirtualMembersN::shrink_to_fit(RefCountingBase* p){ p->on_shrink_to_fit(); }

void RefCountingBase::on_rawcall(const VMachinePtr& vm){
	to_smartptr(this)->rawsend(vm, XTAL_DEFINED_ID(op_call));
}

const AnyPtr& RefCountingBase::on_rawmember(const IDPtr& /*primary_key*/, const AnyPtr& /*secondary_key*/, bool /*inherited_too*/, int_t& /*accessibility*/, bool& /*nocache*/){
	return undefined; 
}
	
const ClassPtr& RefCountingBase::on_object_parent(){ 
	return nul<Class>(); 
}

void RefCountingBase::special_initialize(const VirtualMembers* vmembers){
	value_.init_rcbase(vmembers->rcbase_class_type, this);
	vmembers_ = vmembers;
}

void RefCountingBase::object_destroy(){ 
	if(!object_destroyed()){
		if(have_finalizer()){
			finalize();
			
			if(alive_ref_count()){
				return;
			}
		}

		set_object_destroyed_flag();
		virtual_members()->object_destroy(this); 

		if(XTAL_detail_type(*this)==TYPE_BASE){
			XTAL_detail_pvalue(*this)->special_uninitialize();
		}

		if(virtual_members()->is_container){
			environment_->object_space_.inc_objects_destroyed_count();
		}
		else{
			object_free();
		}
	}
}

void Base::special_initialize(){
	value_.init_base(this);
	class_ = nul<Class>().get();
	instance_variables_ = &empty_instance_variables; 
}

void Base::special_initialize(const VirtualMembers* vmembers){
	value_.init_base(this);
	class_ = cpp_class(vmembers->cpp_class_symbol_data).get();
	XTAL_ASSERT(class_!=0);
	instance_variables_ = &empty_instance_variables; 
	vmembers_ = vmembers;

	if(vmembers->finalize!=VirtualMembersN::finalize){
		set_finalizer_flag();
	}
}

void Base::special_uninitialize(){
	if(instance_variables_!=&empty_instance_variables){
		instance_variables_->destroy();
		if(get_class()){
			class_->dec_ref_count();
		}
		instance_variables_ = &empty_instance_variables; 
	}
}

void Base::init_instance_variables(ClassInfo* info){
	if(instance_variables_==&empty_instance_variables){
		if(get_class()){
			class_->inc_ref_count();
		}
	}
	instance_variables_ = instance_variables_->create(info);
}

void Base::set_class(const ClassPtr& c){
	if(instance_variables_==&empty_instance_variables){
		class_ = c.get();
	}
	else{
		if(get_class()){
			class_->dec_ref_count();
		}
		class_ = c.get();
		if(get_class()){
			class_->inc_ref_count();
		}
	}
}

void Base::on_visit_members(Visitor& m){
	if(instance_variables_!=&empty_instance_variables){
		m & class_ & instance_variables_;
	}
}

}

