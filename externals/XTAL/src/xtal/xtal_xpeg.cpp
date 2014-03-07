#include "xtal.h"
#include "xtal_bind.h"
#include "xtal_macro.h"
#include "xtal_stringspace.h"

namespace xtal{ namespace xpeg{

enum{ NFA_STATE_START = 0, NFA_STATE_FINAL = 1 };

Executor::Executor(const StringPtr& source_name){
	source_name_ = source_name;

	cap_ = XNew<Map>();
	tree_ = XNew<TreeNode>();
	errors_ = null;
	pos_begin_ = 0;
	match_begin_ = 0;
	match_end_ = 0;

	num_ = 0;
	begin_ = 0;
	max_ = 0;

	mm_ = XNew<MemoryStream>();
	pos_ = 0;
	read_ = 0;
	base_ = 0;
	record_pos_ = -1;

	n_ch_ = XTAL_STRING("\n");
	r_ch_ = XTAL_STRING("\r");

	lineno_ = 1;

	expand();
}

Executor::~Executor(){
	for(uint_t i=0; i<num_; ++i){
		for(int j=0, sz=ONE_BLOCK_SIZE; j<sz; ++j){
			begin_[i][j] = null;
		}

		xfree(begin_[i], sizeof(AnyPtr)*ONE_BLOCK_SIZE);
	}
	xfree(begin_, sizeof(AnyPtr*)*max_);
}

bool Executor::match(const AnyPtr& pattern){
	ElementPtr nfa = elem(pattern);

	pos_begin_ = pos();
	for(;;){
		match_begin_ = pos();
		if(match_inner(nfa)){
			match_end_ = pos();
			return !errors_;
		}

		if(eos()){
			return false;
		}

		read();
	}
}

bool Executor::parse(const AnyPtr& pattern){
	return match_inner(elem(pattern)) && (!errors_ || errors_->empty());
}

AnyPtr Executor::captures(){
	MapPtr ret = xnew<Map>();

	ret->set_at(empty_id, at(empty_string));
	Xfor2(k, v, cap_){
		XTAL_UNUSED_VAR(v);
		const StringPtr& k2 = unchecked_ptr_cast<String>(k);
		ret->set_at(k2, at(k2));
	}

	return ret->each();
}

AnyPtr Executor::captures_values(){
	MapPtr ret = xnew<Map>();

	ret->set_at(empty_id, at(empty_string));
	Xfor2(k, v, cap_){
		XTAL_UNUSED_VAR(v);
		const StringPtr& k2 = unchecked_ptr_cast<String>(k);
		ret->set_at(k2, call(k2));
	}

	return ret->each();
}

StringPtr Executor::at(const StringPtr& key){
	if(XTAL_detail_raweq(key, empty_id)){
		return capture(match_begin_, match_end_);
	}
	else{
		if(const SmartPtr<Cap>& temp = unchecked_ptr_cast<Cap>(cap_->at(key))){
			if(temp->end>=0 && temp->end-temp->begin>0){
				return capture(temp->begin, temp->end);
			}
			else{
				if(temp->end==temp->begin){
					return empty_string;
				}
			}
		}
	}
	return nul<String>();
}

AnyPtr Executor::call(const StringPtr& key){
	if(XTAL_detail_raweq(key, empty_id)){
		return capture_values(match_begin_, match_end_);
	}
	else{		
		const SmartPtr<Cap>& temp = unchecked_ptr_cast<Cap>(cap_->at(key));

		if(temp->end>=0 && temp->end-temp->begin>0){
			return capture_values(temp->begin, temp->end);
		}
		else{
			if(temp->end==temp->begin){
				return xnew<Array>();
			}
			else{
				return null;
			}
		}
	}
}

StringPtr Executor::prefix(){
	return capture(pos_begin_, match_begin_);
}

StringPtr Executor::suffix(){
	return capture(match_end_);
}

AnyPtr Executor::prefix_values(){
	return capture_values(pos_begin_, match_begin_)->each();
}

AnyPtr Executor::suffix_values(){
	return capture_values(match_end_)->each();
}

int_t Executor::peek_ascii(int_t n){
	return XTAL_detail_chvalue(peek(n));
}

int_t Executor::read_ascii(){
	return XTAL_detail_chvalue(read());
}

const StringPtr& Executor::peek_s(int_t n){
	const AnyPtr& ret = peek(n);
	if(XTAL_detail_is_undefined(ret)){
		return empty_string;
	}
	return unchecked_ptr_cast<String>(ret);
}

const StringPtr& Executor::read_s(){
	const AnyPtr& ret = read();
	if(XTAL_detail_is_undefined(ret)){
		return empty_string;
	}
	return unchecked_ptr_cast<String>(ret);
}

bool Executor::eat(const AnyPtr& v){
	if(XTAL_detail_raweq(peek(), v)){
		skip();
		return true;
	}
	return false;
}

AnyPtr Executor::errors(){
	if(!errors_ || errors_->empty()) return null;
	return errors_->each();
}

void Executor::clear_errors(){
	if(errors_) return errors_->clear();
}

bool Executor::has_error(){
	return errors_ && !errors_->empty();
}

void Executor::error(const AnyPtr& message, int_t line){
	if(!errors_){
		errors_ = XNew<Array>();
	}

	if(errors_->size()<10){
		errors_->push_back(Xf3("%s:%d:%s", 0, source_name_, 1, line==0 ? lineno() : line, 2, message));
	}
}

bool Executor::match_inner(const ElementPtr& e){
	if(!e->nfa){
		e->nfa = XNew<NFA>(e);
	}

	const NFAPtr& nfa = e->nfa;

	// メモ化したい
	// (スキャナーの位置、NFAのポインタ値) がキー
	Key key;
	key.pos = pos();
	key.ptr = nfa.get();

	// すでにメモ化してないかチェック
	if(memotable_t::Node* it=memotable_.find(key)){
		load(it->value().state);
		tree_->op_cat_assign(it->value().tree);
		return true;
	}

	int_t nodenum = tree_->size();
	uint_t mins = stack_.size();

	for(uint_t i=0, sz=nfa->cap_list_->size(); i<sz; ++i){
		cap_->set_at(nfa->cap_list_->at(i), xnew<Cap>());
	}

	bool match = false;
	State match_pos = {0, 0};
	State first_pos = save();

	push(mins, NFA_STATE_START, nodenum, first_pos);

	while(stack_.size()>mins){
		StackInfo se = stack_.pop();
		int cur_state = se.state;
		
		State& pos = se.pos;
		NFA::State& state = nfa->states_[cur_state];

		if(cur_state==NFA_STATE_FINAL){
			if(match_pos.pos <= pos.pos){
				match_pos = pos;
			}
			match = true;
		}

		bool fail = true;
		for(const TransPtr* tr=&state.trans; *tr; tr=&(*tr)->next){
			load(pos);

			if(se.nodes<tree_->size()){
				tree_->resize(se.nodes);
			}

			if(test((*tr)->ch)){
				push(mins, (*tr)->to, tree_->size(), save());
				fail = false;
			}
		}

		if(fail){
			if(match){
				break;
			}
		}
		else{
			switch(state.capture_kind){
				XTAL_NODEFAULT;
				XTAL_CASE(NFA::CAPTURE_NONE){}

				XTAL_CASE(NFA::CAPTURE_BEGIN){
					const SmartPtr<Cap>& temp = unchecked_ptr_cast<Cap>(cap_->at(state.capture_name));
					temp->begin = pos.pos;
					temp->end = -1;
				}

				XTAL_CASE(NFA::CAPTURE_END){
					const SmartPtr<Cap>& temp = unchecked_ptr_cast<Cap>(cap_->at(state.capture_name));
					temp->end = pos.pos;
				}
			}
		}
	}

	stack_.downsize_n(mins);

	if(match){
		Value value;
		value.state = match_pos;
		value.tree = tree_->slice(nodenum, tree_->size()-nodenum);
		memotable_.insert(key, value);
		load(match_pos);
		return true;
	}

	tree_->resize(nodenum);

	load(first_pos);
	return false;
}

bool Executor::test(const ElementPtr& e){
	switch(e->type){
		XTAL_NODEFAULT;

		case Element::TYPE_CONCAT:
		case Element::TYPE_OR:
		case Element::TYPE_MORE0:
		case Element::TYPE_MORE1:
		case Element::TYPE_01:
		case Element::TYPE_EMPTY:
		case Element::TYPE_CAP:
			XTAL_ASSERT(false);
			break;

		XTAL_CASE(Element::TYPE_INVALID){
			return true;
		}

		XTAL_CASE(Element::TYPE_ANY){
			if(eos()){ return false; }
			read();
			return !e->inv;
		}

		XTAL_CASE(Element::TYPE_BOS){
			return bos()!=e->inv;
		}

		XTAL_CASE(Element::TYPE_EOS){
			return eos()!=e->inv;
		}

		XTAL_CASE(Element::TYPE_BOL){
			return bol()!=e->inv;
		}

		XTAL_CASE(Element::TYPE_EOL){
			if(!eol()){ return e->inv; }
			skip_eol();
			return !e->inv;
		}

		XTAL_CASE(Element::TYPE_EQL){
			if(eos()){ return false; }
			if(XTAL_detail_raweq(read(), e->param1)){ return !e->inv; }
			return e->inv;
		}

		XTAL_CASE(Element::TYPE_INT_RANGE){
			if(eos()){ return false; }
			const AnyPtr& a = peek();
			if(XTAL_detail_is_ivalue(a)){
				return unchecked_ptr_cast<Int>(a)->op_in_IntRange(unchecked_ptr_cast<IntRange>(e->param1))!=e->inv;
			}
			return e->inv;
		}

		XTAL_CASE(Element::TYPE_FLOAT_RANGE){
			if(eos()){ return false; }
			const AnyPtr& a = peek();
			if(XTAL_detail_is_fvalue(a)){
				return unchecked_ptr_cast<Float>(a)->op_in_FloatRange(unchecked_ptr_cast<FloatRange>(e->param1))!=e->inv;
			}
			return e->inv;
		}

		XTAL_CASE(Element::TYPE_CH_RANGE){
			if(eos()){ return false; }
			const AnyPtr& a = read();
			if(XTAL_detail_type(a)==TYPE_SMALL_STRING){
				return unchecked_ptr_cast<String>(a)->op_in(unchecked_ptr_cast<ChRange>(e->param1))!=e->inv;
			}
			return e->inv;
		}

		XTAL_CASE(Element::TYPE_PRED){
			if(eos()){ return false; }
			if(e->param1->call(read())){ return !e->inv; }
			return e->inv;
		}

		XTAL_CASE(Element::TYPE_CH_SET){
			if(eos()){ return false; }
			const MapPtr& data = unchecked_ptr_cast<Map>(e->param1);
			if(data->at(read())){ return !e->inv; }
			return e->inv;
		}

		XTAL_CASE(Element::TYPE_CALL){
			if(eos()){ return false; }
			AnyPtr ret = e->param1->call(to_smartptr(this));
			return (ret || XTAL_detail_is_undefined(ret))!=e->inv;
		}

		XTAL_CASE(Element::TYPE_GREED){
			return match_inner(unchecked_ptr_cast<Element>(e->param1))!=e->inv;
		}

		XTAL_CASE(Element::TYPE_LOOKAHEAD){
			State state = save();
			bool ret = match_inner(unchecked_ptr_cast<Element>(e->param1));
			load(state);
			return ret!=e->inv;
		}

		XTAL_CASE(Element::TYPE_LOOKBEHIND){
			State state = save();
			State fict_state = state;
			fict_state.pos = fict_state.pos > (uint_t)XTAL_detail_ivalue(e->param2) ? fict_state.pos-e->param3 : 0;
			load(fict_state);
			bool ret = match_inner(unchecked_ptr_cast<Element>(e->param1));
			load(state);
			return ret!=e->inv;
		}

		XTAL_CASE(Element::TYPE_LEAF){
			int_t ipos = pos();
			if(match_inner(unchecked_ptr_cast<Element>(e->param1))){
				if(tree_){
					if(e->param3!=0){
						tree_->push_back(capture_values(ipos, pos()));
					}
					else{
						tree_->push_back(capture(ipos, pos()));
					}
				}
				return !e->inv;
			}
			return e->inv;
		}

		XTAL_CASE(Element::TYPE_NODE){
			if(tree_){
				int_t nodenum = tree_->size() - e->param3;
				if(nodenum<0){ nodenum = 0; }

				if(match_inner(unchecked_ptr_cast<Element>(e->param1))){
					TreeNodePtr node = xnew<TreeNode>();
					node->set_tag(e->param2->to_s()->intern());
					node->set_lineno(lineno());
					node->assign(tree_->splice(nodenum, tree_->size()-nodenum));
					tree_->push_back(node);
					return !e->inv;
				}
			}
			else{
				if(match_inner(unchecked_ptr_cast<Element>(e->param1))){
					return !e->inv;
				}
			}
			return e->inv;
		}

		XTAL_CASE(Element::TYPE_BACKREF){
			const SmartPtr<Cap>& temp = unchecked_ptr_cast<Cap>(cap_->at(e->param1));
			if(temp && temp->end>=0 && temp->end-temp->begin>0){
				return (eat_capture(temp->begin, temp->end))!=e->inv;
			}
			return e->inv;
		}

		XTAL_CASE(Element::TYPE_ERROR){
			if(errors_){
				errors_->push_back(e->param1->call(Named(Xid(line), lineno())));
			}
			return !e->inv;
		}

		XTAL_CASE(Element::TYPE_DECL){
			return match_inner(unchecked_ptr_cast<Element>(e->param1))!=e->inv;
		}
	}

	return false;
}


/////

const AnyPtr& Executor::peek(int_t n){
	while(pos_+n >= read_){
		int_t now_read = 0;

		if((read_>>ONE_BLOCK_SHIFT)-base_==num_){
			expand();
		}

		now_read = on_read(&access(read_), ONE_BLOCK_SIZE-(read_&ONE_BLOCK_MASK));

		if(now_read<=0){
			return undefined;
		}

		read_ += now_read;
	}
	
	return access(pos_+n);
}

Executor::State Executor::save(){
	State state;
	state.lineno = lineno_;
	state.pos = pos_;
	state.ch = last_;
	return state;
}

void Executor::load(const State& state){
	pos_ = state.pos;
	lineno_ = state.lineno;
	last_ = state.ch;
}

const AnyPtr& Executor::read(){
	const AnyPtr& ret = peek();
	last_ = ret;

	if(XTAL_detail_raweq(ret, n_ch_)){
		lineno_++;
	}

	pos_ += 1;
	return  ret;
}

bool Executor::bol(){
	if(pos_==0){
		return true;
	}

	const AnyPtr& ch = access(pos_-1);
	return XTAL_detail_raweq(ch, n_ch_) || XTAL_detail_raweq(ch, r_ch_);
}

bool Executor::eol(){
	const AnyPtr& ch = peek();
	return XTAL_detail_raweq(ch, r_ch_) || XTAL_detail_raweq(ch, n_ch_);
}

void Executor::skip(uint_t n){
	for(uint_t i=0; i<n; ++i){
		read();
	}
}

void Executor::skip(){
	read();
}

void Executor::skip_eol(){
	const AnyPtr& ch = peek();
	if(XTAL_detail_raweq(ch, r_ch_)){
		if(XTAL_detail_raweq(peek(1), n_ch_)){
			skip(2);
		}
		else{
			skip(1);
		}
	}
	else if(XTAL_detail_raweq(ch, n_ch_)){
		skip(1);
	}
}

ArrayPtr Executor::capture_values(int_t begin, int_t end){
	ArrayPtr ret = xnew<Array>();
	for(int_t i=begin; i<end; ++i){
		ret->push_back(access(i));
	}
	return ret;
}

ArrayPtr Executor::capture_values(int_t begin){
	ArrayPtr ret = xnew<Array>();
	int_t saved = pos_;
	pos_ = begin;
	while(!eos()){
		ret->push_back(read());
	}
	pos_ = saved;
	return ret;
}
	
StringPtr Executor::capture(int_t begin, int_t end){
	mm_->clear();
	for(int_t i=begin; i<end; ++i){
		mm_->put_s(access(i));
	}
	return mm_->to_s();
}

StringPtr Executor::capture(int_t begin){
	mm_->clear();
	int_t saved = pos_;
	pos_ = begin;
	while(!eos()){
		mm_->put_s(read());
	}
	pos_ = saved;
	return mm_->to_s();
}

bool Executor::eat_capture(int_t begin, int_t end){
	for(int_t i=begin; i<end; ++i){
		if(!XTAL_detail_raweq(peek(i-begin), access(i))){
			return false;
		}
	}
	skip(end-begin);
	return true;
}

void Executor::begin_record(){
	record_pos_ = pos_;
}

StringPtr Executor::end_record(){
	if(record_pos_<0){
		return empty_string;
	}

	int_t begin = record_pos_;
	record_pos_ = -1;
	return capture(begin, pos_);
}

bool Executor::eat_ascii(int_t ch){
	if(peek_ascii()==ch){
		read_ascii();
		return true;
	}
	return false;
}

void Executor::expand(){
	if(max_==num_){
		uint_t newmax = max_ + max_/2 + 4;
		AnyPtr** newp = (AnyPtr**)xmalloc(sizeof(AnyPtr*)*newmax);

		if(begin_){
			std::memcpy(newp, begin_, sizeof(AnyPtr*)*num_);
		}

		for(uint_t i=num_; i<newmax; ++i){
			newp[i] = 0;
		}

		xfree(begin_, sizeof(AnyPtr*)*max_);
		begin_ = newp;
		max_ = newmax;
	}

	//XTAL_ASSERT(begin_[num_-base_]==0);

	begin_[num_] = (AnyPtr*)xmalloc(sizeof(AnyPtr)*ONE_BLOCK_SIZE);
	std::memset(begin_[num_], 0, sizeof(AnyPtr)*ONE_BLOCK_SIZE);
	num_++;
}
	
void Executor::bin(){
	if(record_pos_>=0){
		return;
	}

	uint_t n = (pos_>>ONE_BLOCK_SHIFT)-base_;

	if(n>0){
		for(uint_t i=0; i<num_; ++i){
			if(n+i>=num_){
				break;
			}

			std::swap(begin_[i], begin_[n+i]);
		}

		base_ += n;
	}
}

void Executor::tree_splice(int_t itag, int_t num){
	tree_splice(Int(itag), num, lineno_);	
}

void Executor::tree_splice(const AnyPtr& tag, int_t num){
	tree_splice(tag, num, lineno_);
}

void Executor::tree_splice(const AnyPtr& tag, int_t num, int_t lineno){
	TreeNodePtr ret = xnew<TreeNode>(tag, lineno);
	
	if((int_t)tree_->size()<num){
		tree_->resize(num);
	}

	for(uint_t i=tree_->size()-num; i<tree_->size(); ++i){
		ret->push_back(tree_->at(i));
	}

	tree_->resize(tree_->size()-num);
	tree_->push_back(ret);
}


/////////////////////////////////////////////////

NFA::NFA(const ElementPtr& node){
	e_ = XNew<Element>(Element::TYPE_INVALID);

	cap_max_ = 0;
	cap_list_ = XNew<Array>();

	gen_state(); // NFA_STATE_START
	gen_state(); // NFA_STATE_FINAL

	gen_nfa(NFA_STATE_START, node, NFA_STATE_FINAL, 0);
}

int NFA::gen_state(){
	State state;
	state.capture_kind = 0;
	states_.push_back(state);
	return states_.size() - 1;
}

void NFA::add_transition(int from, const AnyPtr& ch, int to){
	TransPtr x = xnew<Trans>();
	x->to = to;
	x->ch = ptr_cast<Element>(ch);

	x->next = states_[from].trans;
	states_[from].trans = x;
}

void NFA::gen_nfa(int entry, const AnyPtr& a, int exit, int depth){
	const ElementPtr& t = unchecked_ptr_cast<Element>(a);

	switch(t->type){
		XTAL_DEFAULT{
			//         ch
			//  entry -----> exit
			add_transition(entry, t, exit);
		}

		XTAL_CASE(Element::TYPE_CONCAT){
			//         left         right
			//  entry ------> step -------> exit
			int step = gen_state();
			gen_nfa(entry, t->param1, step, depth+1);
			gen_nfa(step, t->param2, exit, depth+1);
		}

		XTAL_CASE(Element::TYPE_OR){
			//               left
			//        ----------------->
			//  entry -----> step -----> exit
			//          e         right
			int step = gen_state();
			gen_nfa(entry, t->param1, exit, depth+1);
			add_transition(entry, e_, step);
			gen_nfa(step, t->param2, exit, depth+1);
		}

		XTAL_CASE2(Element::TYPE_MORE0, Element::TYPE_MORE1){
			//                       e
			//         e          <------        e
			//  entry ---> before ------> after ---> exit
			//    |                left                ^
			//    >------->------------------->------>-|
			//                      e
			//----------------------------------------------
			//                       e
			//         e          <------        e
			//  entry ---> before ------> after ---> exit
			//                     left
			int before = gen_state();
			int after = gen_state();
			
			if(t->param3==1){
				// exitに向かう方が優先
				if(t->type == Element::TYPE_MORE0){ add_transition(entry, e_, exit); }
				add_transition(entry, e_, before);

				// exitに向かう方が優先
				add_transition(after, e_, exit);
				add_transition(after, e_, before);
			}
			else{
				// beforeに向かう方が優先
				add_transition(entry, e_, before);
				if(t->type == Element::TYPE_MORE0){ add_transition(entry, e_, exit); }

				// beforeに向かう方が優先
				add_transition(after, e_, before);
				add_transition(after, e_, exit);
			}

			// beforeからafterへの遷移 
			gen_nfa(before, t->param1, after, depth+1);
		}

		XTAL_CASE(Element::TYPE_01){
			//           e
			//        ------>
			//  entry ------> exit
			//         left
			if(t->param3==1){
				// eを経由する方が優先
				add_transition(entry, e_, exit);
				gen_nfa(entry, t->param1, exit, depth+1);
			}
			else{
				// leftを経由する方が優先
				gen_nfa(entry, t->param1, exit, depth+1);
				add_transition(entry, e_, exit);
			}
		}

		XTAL_CASE(Element::TYPE_EMPTY){
			//         e
			//  entry ---> exit
			add_transition(entry, e_, exit);
		}

		XTAL_CASE(Element::TYPE_CAP){
			int before = gen_state();
			int after = gen_state();

			states_[before].capture_kind = CAPTURE_BEGIN;
			states_[after].capture_kind = CAPTURE_END;
			states_[before].capture_name = unchecked_ptr_cast<ID>(t->param2);
			states_[after].capture_name = unchecked_ptr_cast<ID>(t->param2);
			cap_list_->push_back(t->param2);
			cap_max_++;

			add_transition(entry, e_, before);
			gen_nfa(before, t->param1, after, depth+1);
			add_transition(after, e_, exit);
		}
	}
}

////////////////////////////////////////////////////////////////////////////////

int_t IteratorExecutor::on_read(AnyPtr* buf, int_t size){
	if(iterator_){
		return 0;
	}

	for(int_t i=0; i<size; ++i){
		const VMachinePtr& vm = setup_call();
		iterator_->rawsend(vm, first_ ? XTAL_DEFINED_ID(block_first) : XTAL_DEFINED_ID(block_next));
		first_ = false;
		iterator_ = vm->result(0);
		if(!iterator_){
			vm->cleanup_call();
			return i;
		}
		buf[i] = vm->result(1);
		vm->cleanup_call();
	}

	return size;
}

////////////////////////////////////////////////////////////////////////////////

Element::Element(int_t type, const AnyPtr& param1, const AnyPtr& param2, int_t param3)
	:type((u8)type), inv(false), param1(param1), param2(param2), param3(param3){

}
	
Element::~Element(){}

ElementPtr Element::op_com() const{
	ElementPtr ret = xnew<Element>(type, param1, param2, param3);
	ret->inv = !inv;
	return ret;
}

////////////////////////////////////////////////////////////////////////////////

ElementPtr elem(const AnyPtr& a){
	if(const ElementPtr& p = ptr_cast<Element>(a)){
		return p;
	}

	int_t type = -1;
	if(can_cast<IntRange>(a)){
		type = Element::TYPE_INT_RANGE;
	}
	else if(can_cast<FloatRange>(a)){
		type = Element::TYPE_FLOAT_RANGE;
	}
	else if(can_cast<ChRange>(a)){
		type = Element::TYPE_CH_RANGE;
	}
	else if(can_cast<Fun>(a)){
		type = Element::TYPE_CALL;
	}
	
	if(type>0){
		return xnew<Element>(type, a);
	}
	else{
		if(const StringPtr& p = ptr_cast<String>(a)){
			if(p->is_empty()){
				return xnew<Element>(Element::TYPE_EMPTY);
			}
			
			if(p->is_ch()){
				return xnew<Element>(Element::TYPE_EQL, p);
			}

			AnyPtr str = null;
			Xfor(v, p->each()){
				if(str){
					str = XNew<Element>(Element::TYPE_CONCAT, str, xnew<Element>(Element::TYPE_EQL, v));
				}
				else{
					str = elem(v);
				}
			}
			
			return elem(str);
		}
	}
	
	set_runtime_error(Xt("XRE1026"));
	return nul<Element>();
}

}

}
