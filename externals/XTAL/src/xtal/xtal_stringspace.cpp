#include "xtal.h"
#include "xtal_macro.h"

#define XATL_DEFINE_IDs
#include "xtal_stringspace.h"
#undef XATL_DEFINE_IDs

namespace xtal{

void StringSpace::initialize(){
	nodes_ = 0;
	nodes_size_ = 0;
	nodes_capa_ = 0;
	expand_nodes(127);
	nodes_[nodes_size_++] = 0;

	blocks_ = 0;
	blocks_size_ = 0;
	blocks_capa_ = 0;
	add_block();
	alloc(16);

	buckets_ = 0;
	buckets_size_ = 0;
	buckets_capa_ = 0;
	expand_buckets(127);

	for(uint_t i=0; i<sizeof(id_list)/sizeof(*id_list); ++i){
		const LongLivedString& str = XTAL_LONG_LIVED_STRING(id_list[i]);
		uint_t hashcode, size;
		string_data_size_and_hashcode(str.str(), size, hashcode);
		register_string(str.str(), size, hashcode, true);
	}
}

void StringSpace::uninitialize(){
	for(uint_t i=0; i<nodes_size_; ++i){
		Node* node = nodes_[i];
		if(node && node->flags==2){
			xfree(node->pointer(), (node->size+1)*sizeof(char_t));
		}
	}

	for(uint_t i=0; i<blocks_size_; ++i){
		xfree(blocks_[i], sizeof(Block)+LIMIT);
	}

	xfree(blocks_, sizeof(Block*)*blocks_capa_);
	xfree(buckets_, sizeof(node_t)*buckets_capa_);
	xfree(nodes_, sizeof(Node*)*nodes_capa_);
}

const char_t* StringSpace::register_string(const char_t* str, uint_t size, uint_t hashcode, bool long_lived){
	uint_t hash = (hashcode&0xffff) ^ ((hashcode>>16)&0xffff);

	uint_t hn = hash % buckets_capa_;
	Node* node = nodes_[buckets_[hn]];
	while(node!=0){
		char_t* data = node->data();
		if(node->hash==hash && string_compare(data, node->size, str, size)==0){
			return data;
		}
		node = nodes_[node->next];
	}

	if(size >= (1<<16) || buckets_size_ >= (1<<16)-1){
		return XTAL_L("<error>");		
	}

	buckets_size_++;

	Node* p;
	char_t* newstr;
	if(!long_lived || size<8){
		if(size<48){
			p = (Node*)alloc(sizeof(Node) + (size+1)*sizeof(char_t));
			p->flags = 0;
			newstr = p->buf();
		}
		else{
			p = (Node*)alloc(sizeof(Node) + sizeof(char_t*));
			p->flags = 2;
			newstr = (char_t*)xmalloc((size+1)*sizeof(char_t));
			p->set_pointer(newstr);
		}

		string_copy(newstr, str, size);
		newstr[size] = 0;
	}
	else{
		p = (Node*)alloc(sizeof(Node) + sizeof(char_t*));
		p->flags = 1;
		newstr = (char_t*)str;
		p->set_pointer(newstr);
	}

	p->size = (strsize_t)size;
	p->hash = (u16)hash;

	p->next = buckets_[hn];
	buckets_[hn] = (node_t)nodes_size_;

	if(nodes_capa_==nodes_size_){
		expand_nodes(1);
	}
	nodes_[nodes_size_++] = p;

	if(buckets_size_ > buckets_capa_){
		expand_buckets(1);
	}

	return newstr;
}

void* StringSpace::Block::alloc(int size){
	size = align(size, sizeof(int_t));
	if(pos+size>=LIMIT){ return 0; }
	void* ret = ((char*)XTAL_STRUCT_TAIL(this) + pos);
	pos += size;
	return ret;
}

void StringSpace::add_block(){
	if(blocks_size_==blocks_capa_){
		uint_t newcapa = blocks_capa_ + 16;
		Block** newblocks = (Block**)xmalloc(sizeof(Block*)*newcapa);
		std::memcpy(newblocks, blocks_, sizeof(Block*)*blocks_capa_);
		xfree(blocks_, sizeof(Block*)*blocks_capa_);
		blocks_ = newblocks;
		blocks_capa_ = newcapa;
	}

	Block* p = (Block*)xmalloc(sizeof(Block)+LIMIT);
	p->pos = 0;
	blocks_[blocks_size_++] = p;
	current_block_ = p;
}

void StringSpace::expand_buckets(uint_t n){
	uint_t newcapa = buckets_capa_*2 + n;
	node_t* newbuckets = (node_t*)xmalloc(sizeof(node_t)*newcapa);
	for(uint_t i=0; i<newcapa; ++i){
		newbuckets[i] = 0;
	}

	for(uint_t i=0; i<nodes_size_; ++i){
		Node* node = nodes_[i];
		if(node){
			uint_t hn2 = node->hash % newcapa;
			node->next = newbuckets[hn2]; 
			newbuckets[hn2] = (node_t)i;
		}	
	}

	xfree(buckets_, sizeof(node_t)*buckets_capa_);
	buckets_ = newbuckets;
	buckets_capa_ = newcapa;
}

void StringSpace::expand_nodes(uint_t n){
	uint_t newcapa = nodes_capa_*2 + n;
	Node** newnodes = (Node**)xmalloc(sizeof(Node*)*newcapa);
	std::memcpy(newnodes, nodes_, sizeof(Node*)*nodes_capa_);
	xfree(nodes_, sizeof(Node*)*nodes_capa_);
	nodes_ = newnodes;
	nodes_capa_ = newcapa;
}

void* StringSpace::alloc(int size){
	void* p = current_block_->alloc(size);
	if(!p){
		add_block();
		p = current_block_->alloc(size);
	}
	return p;
}


}
