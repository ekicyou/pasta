#include "xtal.h"
#include "xtal_macro.h"

namespace xtal{

Map::Map()
	:default_value_(undefined){}

Map::~Map(){}

void Map::on_visit_members(Visitor& m){
	Base::on_visit_members(m);
	m & table_ & default_value_;
}	

AnyPtr Map::calc_key(const AnyPtr& key){
	int_t t = XTAL_detail_type(key);
	if(t==TYPE_STRING || t==TYPE_LONG_LIVED_STRING){
		if(const StringPtr& str = unchecked_ptr_cast<String>(key)){
			return str->intern();
		}
	}
	return key;
}
	
const AnyPtr& Map::at(const AnyPtr& akey){
	return op_at(akey);
}
	
const AnyPtr& Map::op_at(const AnyPtr& akey){
	const AnyPtr& key = calc_key(akey);
	iterator it = table_.find(key);
	if(it!=end()){
		return it->second;
	}
	else{
		return default_value_;
	}
}

void Map::insert(const AnyPtr& key, const AnyPtr& value){
	const AnyPtr& akey = calc_key(key);
	table_.insert(akey, value);
}

void Map::erase(const AnyPtr& key){
	const AnyPtr& akey = calc_key(key);
	table_.erase(akey);
}

MapPtr Map::op_cat(const MapPtr& a){
	MapPtr ret(clone());
	ret->op_cat_assign(a);
	return ret;
}

const MapPtr& Map::op_cat_assign(const MapPtr& a){
	for(iterator p = a->begin(); p!=a->end(); ++p){
		set_at(p->first, p->second);
	}
	return to_smartptr(this);
}
	
AnyPtr Map::pairs(){
	return xnew<MapIter>(to_smartptr(this), 0);
}

AnyPtr Map::keys(){
	return xnew<MapIter>(to_smartptr(this), 1);
}

AnyPtr Map::values(){
	return xnew<MapIter>(to_smartptr(this), 2);
}

MapPtr Map::clone(){
	MapPtr ret = xnew<Map>();
	for(iterator p = begin(); p!=end(); ++p){
		ret->set_at(p->first, p->second);
	}	
	return ret;
}

void Map::clear(){
	table_.clear();
}	

void Map::assign(const AnyPtr& iterator){
	clear();

	Xfor2(k, v, iterator){
		set_at(k, v);
	}
}
	
void Map::append(const AnyPtr& iterator){
	Xfor2(k, v, iterator){
		set_at(k, v);
	}
}

void Map::push_all(const VMachinePtr& vm){
	for(iterator p = begin(); p!=end(); ++p){
		vm->push_arg(ptr_cast<ID>(p->first), p->second);
	}	
}

StringPtr Map::to_s(){
	if(is_empty()){
		return XTAL_STRING("[:]");
	}

	MemoryStreamPtr ms = xnew<MemoryStream>();
	ms->put_s(XTAL_STRING("["));
	for(iterator p = begin(); p!=end(); ++p){
		if(p!=begin()){
			ms->put_s(XTAL_STRING(", "));
		}
		ms->put_s(p->first);
		ms->put_s(XTAL_STRING(":"));
		ms->put_s(p->second);
	}	
	ms->put_s(XTAL_STRING("]"));
	return ms->to_s();
}

void Map::block_first(const VMachinePtr& vm){
	SmartPtr<MapIter> it = xnew<MapIter>(to_smartptr(this), 0);
	it->block_next(vm);
}

//////////////////////////////////////////////////////////

MapIter::MapIter(const MapPtr& m, int_t type)
	:map_(m), node_(m->begin()), type_(type){
}

void MapIter::block_next(const VMachinePtr& vm){
	if(node_==map_->end()){
		if(type_==0){
			vm->return_result(null, null, null);
		}
		else{
			vm->return_result(null, null);
		}
		return;
	}
	
	switch(type_){
		case 0: vm->return_result(to_smartptr(this), node_->first, node_->second); break;
		case 1: vm->return_result(to_smartptr(this), node_->first); break;
		case 2: vm->return_result(to_smartptr(this), node_->second); break;
		case 3:
			for(;;){
				if(node_->second){
					vm->return_result(to_smartptr(this), node_->first);
					++node_;
					return;
				}

				++node_;

				if(node_==map_->end()){
					vm->return_result(null, null);
					return;
				}
			}
			break;
		default: vm->return_result(null, null); break;
	}

	++node_;
}

bool MapIter::block_next_direct(AnyPtr& rkey, AnyPtr& rval){
	if(node_==map_->end()){
		rkey = null;
		rval = null;
		return false;
	}
	
	switch(type_){
		case 0:
			rkey = node_->first;
			rval = node_->second;
			break;
		case 1: 
			rkey = node_->first;
			rval = null;
			break;

		case 2: 
			rkey = node_->second;
			rval = null;
			break;

		case 3:
			for(;;){
				if(node_->second){
					rkey = node_->first;
					rval = null;
					++node_;
					return true;
				}

				++node_;

				if(node_==map_->end()){
					rkey = null;
					rval = null;
					return false;
				}
			}
			break;
		default: break;
	}

	++node_;
	return true;
}

void MapIter::on_visit_members(Visitor& m){
	Base::on_visit_members(m);
	m & map_;
}

//////////////////////////////////////////////////////////

AnyPtr Set::each(){
	return xnew<MapIter>(to_smartptr(this), 3);
}

StringPtr Set::to_s(){
	MemoryStreamPtr ms = xnew<MemoryStream>();
	ms->put_s(XTAL_STRING("["));
	for(iterator p = begin(); p!=end(); ++p){
		if(p!=begin()){
			ms->put_s(XTAL_STRING(", "));
		}

		if(p->second){
			ms->put_s(p->first);
		}
	}	
	ms->put_s(XTAL_STRING("]"));
	return ms->to_s();
}

XNew<Map>::XNew(){ init(); }

}
