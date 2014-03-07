#include "xtal.h"
#include "xtal_macro.h"
#include "xtal_stringspace.h"

namespace xtal{

ZipIter::ZipIter(const VMachinePtr& vm){
	next_ = XNew<Array>(vm->ordered_arg_count());
	for(int_t i = 0, len = next_->size(); i<len; ++i){
		next_->set_at(i, vm->arg(i));
	}
}

void ZipIter::common(const VMachinePtr& vm, const IDPtr& id){
	bool all = true;
	ValuesPtr value;
	
	for(int_t i = next_->size()-1; i>=0; --i){
		vm->setup_call(2);
		next_->at(i)->rawsend(vm, id);
		next_->set_at(i, vm->result(0));
		
		if(XTAL_detail_type(value)==TYPE_VALUES){
			value = XNew<Values>(vm->result(1), value);
		}
		else{
			AnyPtr ret = vm->result(1);
			if(XTAL_detail_type(ret)==TYPE_VALUES){
				value = unchecked_ptr_cast<Values>(ret);
			}
			else{
				value = XNew<Values>(ret);
			}
		}
		vm->cleanup_call();
		if(!next_->at(i)){
			all = false;
		}
	}
	
	if(all){
		vm->return_result(to_smartptr(this), value);
	}
	else{
		vm->return_result(null, null);
	}
}

void ZipIter::block_first(const VMachinePtr& vm){
	common(vm, Xid(block_first));
}

void ZipIter::block_next(const VMachinePtr& vm){
	common(vm, Xid(block_next));
}

void ZipIter::block_break(const VMachinePtr& vm){
	IDPtr id = Xid(block_break);
	for(int_t i = 0, len = next_->size(); i<len; ++i){
		vm->setup_call(0);
		next_->at(i)->rawsend(vm, id, undefined, true, true);
		if(!vm->is_executed()){
			vm->return_result();	
		}
		vm->cleanup_call();
	}
	vm->return_result();
}

void ZipIter::on_visit_members(Visitor& m){
	Base::on_visit_members(m);
	m & next_;
}

void DelegateToIterator::on_rawcall(const VMachinePtr& vm){
	vm->arg_this()->send(XTAL_DEFINED_ID(each))->rawsend(vm, member_);
}

void block_break(AnyPtr& target){
	if(target){
		const VMachinePtr& vm = setup_call(0);
		target->rawsend(vm, XTAL_DEFINED_ID(block_break), undefined, true, true);
		if(!vm->is_executed()){
			vm->return_result();
		}
		vm->cleanup_call();
	}
}

bool block_next(BlockValueHolder1& holder, bool first){
	if(holder.it){
		if(!holder.it->block_next_direct(holder.values[0])){
			holder.target = null;
		}
	}
	else{
		const VMachinePtr& vm = setup_call(2);
		holder.target->rawsend(vm, first ? XTAL_DEFINED_ID(block_first) : XTAL_DEFINED_ID(block_next));
		holder.target = vm->result(0);
		holder.values[0] = vm->result(1);
		vm->cleanup_call();
	}
	return holder.target;
}

bool block_next(BlockValueHolder2& holder, bool first){
	if(holder.it){
		if(!holder.it->block_next_direct(holder.values[0], holder.values[1])){
			holder.target = null;
		}
	}
	else{
		const VMachinePtr& vm = setup_call(3);
		holder.target->rawsend(vm, first ? XTAL_DEFINED_ID(block_first) : XTAL_DEFINED_ID(block_next));
		holder.target = vm->result(0);
		holder.values[0] = vm->result(1);
		holder.values[1] = vm->result(2);
		vm->cleanup_call();
	}
	return holder.target;
}

bool block_next(BlockValueHolder3& holder, bool first){
	const VMachinePtr& vm = setup_call(4);
	holder.target->rawsend(vm, first ? XTAL_DEFINED_ID(block_first) : XTAL_DEFINED_ID(block_next));
	holder.target = vm->result(0);
	holder.values[0] = vm->result(1);
	holder.values[1] = vm->result(2);
	holder.values[2] = vm->result(3);
	vm->cleanup_call();
	return holder.target;
}

BlockValueHolder1::BlockValueHolder1(const AnyPtr& tar, bool& not_end)
	:target(tar){
	not_end = tar;
	if(const ArrayPtr& array = ptr_cast<Array>(tar)){ 
		it = unchecked_ptr_cast<ArrayIter>(array->each()); 
	}
	else{ 
		it = ptr_cast<ArrayIter>(tar); 
	}
}

BlockValueHolder1::BlockValueHolder1(const ArrayPtr& tar, bool& not_end)
	:target(tar){
	not_end = tar;
	if(const ArrayPtr& array = tar){ 
		it = unchecked_ptr_cast<ArrayIter>(array->each()); 
	}
}

BlockValueHolder1::~BlockValueHolder1(){ 
	block_break(target); 
}
	
BlockValueHolder2::BlockValueHolder2(const AnyPtr& tar, bool& not_end)
	:target(tar){
	not_end = tar;
	if(const MapPtr& map = ptr_cast<Map>(tar)){ 
		it = unchecked_ptr_cast<MapIter>(map->each()); 
	}
	else{ 
		it = ptr_cast<MapIter>(tar); 
	}
}

BlockValueHolder2::BlockValueHolder2(const MapPtr& tar, bool& not_end)
	:target(tar){
	not_end = tar;
	if(const MapPtr& map = tar){ 
		it = unchecked_ptr_cast<MapIter>(map->each()); 
	}
}

BlockValueHolder2::~BlockValueHolder2(){ 
	block_break(target); 
}

BlockValueHolder3::BlockValueHolder3(const AnyPtr& tar, bool& not_end)
	:target(tar){
	not_end = tar;
}

BlockValueHolder3::~BlockValueHolder3(){ 
	block_break(target); 
}

}
