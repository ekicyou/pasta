#include "xtal.h"
#include "xtal_macro.h"

namespace xtal{

MembersIter::MembersIter(const FramePtr& a)
	:frame_(a), node_(0), pos_(0){
	move_next();
}

void MembersIter::on_visit_members(Visitor& m){
	Base::on_visit_members(m);
	m & frame_;
}

void MembersIter::move_next(){
	while(true){
	if(node_){
			if(node_->accessibility()==KIND_PUBLIC){
				return;
			}
			node_ = node_->next;
		}
		else{
			if(pos_==frame_->buckets_capa_){
				return;
			}
			node_ = frame_->buckets_[pos_++];
		}
	}
}

void MembersIter::block_next(const VMachinePtr& vm){
	if(node_){
		if(node_->flags&Frame::FLAG_NODE3){
			vm->return_result(to_smartptr(this), ((Frame::Node3*)node_)->primary_key, ((Frame::Node3*)node_)->secondary_key, frame_->members_.at(node_->num));
		}
		else{
			vm->return_result(to_smartptr(this), ((Frame::Node2*)node_)->primary_key, undefined, frame_->members_.at(node_->num));
		}
		node_ = node_->next;
		move_next();
		return;
	}

	vm->return_result(null, null, null, null);
}


MembersIter2::MembersIter2(const FramePtr& a)
	:frame_(a), it_(0){
}

void MembersIter2::on_visit_members(Visitor& m){
	Base::on_visit_members(m);
	m & frame_;
}

void MembersIter2::block_next(const VMachinePtr& vm){
	if(it_ < frame_->scope_info_->variable_size){
		vm->return_result(to_smartptr(this), frame_->code_->identifier(frame_->scope_info_->variable_identifier_offset+it_), undefined, frame_->members_.at(it_));
		++it_;
		return;
	}
	else{
		vm->return_result(null, null, null, null);
		return;
	}
}

Frame::Frame(const FramePtr& outer, const CodePtr& code, ScopeInfo* info)
	:outer_(outer), code_(code), scope_info_(info ? info : &empty_class_info), 
	members_(scope_info_->variable_size), buckets_(0), buckets_capa_(0), flags_(FLAG_ORPHAN){
}

Frame::Frame()
	:scope_info_(&empty_class_info), 
	members_(0),  buckets_(0), buckets_capa_(0), flags_(FLAG_ORPHAN){}

Frame::~Frame(){
	if(!orphan()){
		members_.detach();
	}

	for(uint_t i=0; i<buckets_capa_; ++i){
		Node* node = buckets_[i];
		while(node!=0){
			Node* next = node->next;
			if(node->flags&FLAG_NODE3){
				delete_object_xfree<Node3>((Node3*)node);
			}
			else{
				delete_object_xfree<Node2>((Node2*)node);
			}
			node = next;
		}	
	}
	xfree(buckets_, sizeof(Node*)*buckets_capa_);
}

void Frame::expand_buckets(){
	uint_t newcapa = buckets_capa_==0 ? 5 : buckets_capa_*2 + 1;

	while(members_.size() >= newcapa){
		newcapa =  newcapa*2 + 1;
	}

	Node** newbuckets = (Node**)xmalloc(sizeof(Node*)*newcapa);
	for(uint_t i=0; i<newcapa; ++i){
		newbuckets[i] = 0;
	}

	for(uint_t i=0; i<buckets_capa_; ++i){
		Node* node = buckets_[i];
		while(node!=0){
			Node* next = node->next;

			uint_t hn;
			if(node->flags&FLAG_NODE3){
				hn = hashcode(((Node3*)node)->primary_key,((Node3*)node)->secondary_key) % newcapa;
			}
			else{
				hn = hashcode(((Node2*)node)->primary_key, undefined) % newcapa;
			}

			node->next = newbuckets[hn]; 
			newbuckets[hn] = node;
			node = next;
		}	
	}

	xfree(buckets_, sizeof(Node*)*buckets_capa_);
	buckets_ = newbuckets;
	buckets_capa_ = newcapa;
}

Frame::Node* Frame::find_node(const IDPtr& primary_key, const AnyPtr& secondary_key){
	if(buckets_capa_==0){
		return 0;
	}

	uint_t hn = hashcode(primary_key, secondary_key) % buckets_capa_;
	Node* node = buckets_[hn];
	while(node!=0){
		if(node->flags&FLAG_NODE3){
			if(XTAL_detail_raweq(primary_key, ((Node3*)node)->primary_key) && XTAL_detail_raweq(secondary_key, ((Node3*)node)->secondary_key)){
				return node;
			}
		}
		else{
			if(XTAL_detail_raweq(primary_key, ((Node2*)node)->primary_key) && XTAL_detail_is_undefined(secondary_key)){
				return node;
			}
		}
		node = node->next;
	}

	return 0;
}

Frame::Node* Frame::insert_node(const IDPtr& primary_key, const AnyPtr& secondary_key, uint_t n){		
	if(Node* node = find_node(primary_key, secondary_key)){
		return node;
	}

	if(members_.size() >= buckets_capa_*0.7f){
		expand_buckets();
	}

	Node* node;
	if(XTAL_detail_is_undefined(secondary_key)){
		Node2* n2 = new_object_xmalloc<Node2>();
		n2->primary_key = primary_key;
		node = n2;
		node->flags = 0;
	}
	else{
		Node3* n3 = new_object_xmalloc<Node3>();
		n3->primary_key = primary_key;
		n3->secondary_key = secondary_key;
		node = n3;
		node->flags = FLAG_NODE3;
	}

	node->num = (u16)n;

	uint_t hn = hashcode(primary_key, secondary_key) % buckets_capa_;
	node->next = buckets_[hn];
	buckets_[hn] = node;

	return node;
}

void Frame::make_members(){
	if(!initialized_members()){
		make_members_force(0);
	}
}

void Frame::make_members_force(int_t flags){
	if(code_ && scope_info_){
		for(int_t i=0; i<scope_info_->variable_size; ++i){
			Node* node = insert_node(code_->identifier(scope_info_->variable_identifier_offset+i), undefined, i);
			node->flags |= flags;
		}
		set_initialized_members();
	}
}

const CodePtr& Frame::code(){ 
	return code_; 
}

void Frame::set_code(const CodePtr& code){
	code_ = code;
}

void Frame::on_visit_members(Visitor& m){
	HaveParentBase::on_visit_members(m);

	if(flags_ & FLAG_ORPHAN){
		m & members_;
	}

	m & outer_ & code_;

	for(uint_t i=0; i<buckets_capa_; ++i){
		Node* node = buckets_[i];
		while(node!=0){
			if(node->flags&FLAG_NODE3){
				m & ((Node3*)node)->secondary_key;
			}
			node = node->next;
		}	
	}
}

AnyPtr Frame::members(){
	if(!orphan()){
		return XNew<MembersIter2>(to_smartptr(this));
	}

	make_members();
	return XNew<MembersIter>(to_smartptr(this));
}

const AnyPtr& Frame::on_rawmember(const IDPtr& primary_key, const AnyPtr& secondary_key, bool inherited_too, int_t& accessibility, bool& nocache){
	nocache = true;

	make_members_force(0);

	if(Node* node = find_node(primary_key, secondary_key)){
		return member_direct(node->num);
	}

	return undefined;
}

bool Frame::replace_member(const IDPtr& primary_key, const AnyPtr& value){
	make_members();

	if(Node* node = find_node(primary_key, undefined)){
		if(!node->nocache()){
			invalidate_cache_member();
		}
		set_member_direct(node->num, value);
		return true;
	}

	return false;
}

}
