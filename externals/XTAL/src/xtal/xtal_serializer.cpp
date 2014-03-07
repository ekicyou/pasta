#include "xtal.h"
#include "xtal_macro.h"

#include "xtal_serializer.h"

namespace xtal{
	
enum{
	SERIALIZE_VERSION1 = 2,
	SERIALIZE_VERSION2 = 0
};

Serializer::Serializer(const StreamPtr& s)
	:map_(xnew<Map>()), stream_(&*s), pstream_(s){
}

Serializer::~Serializer(){}

void Serializer::serialize(const AnyPtr& v){
	clear();
	inner_serialize(v);
}

AnyPtr Serializer::deserialize(){
	clear();
	AnyPtr ret = inner_deserialize();
	XTAL_CHECK_EXCEPT(e){ 
		XTAL_UNUSED_VAR(e);
		return undefined; 
	}
	return ret;
}

void Serializer::inner_serialize_scope_info(ScopeInfo& info){
	stream_->put_u32be(info.pc);
	stream_->put_u8(info.kind);
	stream_->put_u8(info.flags);
	stream_->put_u16be(info.variable_identifier_offset);
	stream_->put_u16be(info.variable_offset);
	stream_->put_u16be(info.variable_size);
}

void Serializer::inner_serialize(const AnyPtr& v){
	
	// 基本型かチェック
	switch(XTAL_detail_type(v)){
		XTAL_DEFAULT;

		XTAL_CASE(TYPE_NULL){
			stream_->put_u8(TNULL);
			return;
		}
		
		XTAL_CASE(TYPE_UNDEFINED){
			stream_->put_u8(TUNDEFINED);
			return;
		}

		XTAL_CASE(TYPE_BASE){}

		XTAL_CASE(TYPE_INT){
			if(sizeof(int_t)==4){
				stream_->put_u8(TINT32);
				stream_->put_i32be(XTAL_detail_ivalue(v));
			}
			else{
				stream_->put_u8(TINT64);
				stream_->put_i64be(XTAL_detail_ivalue(v));
			}
			return;
		}

		XTAL_CASE(TYPE_FLOAT){
			if(sizeof(float_t)==4){
				stream_->put_u8(TFLOAT32);
				stream_->put_f32be(XTAL_detail_fvalue(v));
			}
			else{
				stream_->put_u8(TFLOAT64);
				stream_->put_f64be(XTAL_detail_fvalue(v));
			}
			return;
		}

		XTAL_CASE(TYPE_FALSE){
			stream_->put_u8(TFALSE);
			return;
		}

		XTAL_CASE(TYPE_TRUE){
			stream_->put_u8(TTRUE);
			return;
		}
	}

	int_t num = register_value(v);
	if(num>=0){
		// 既に保存されているオブジェクトなので参照位置だけ保存する
		stream_->put_u8(REFERENCE);
		stream_->put_u32be(num);
		return;
	}
	
	switch(XTAL_detail_type(v)){
		XTAL_DEFAULT;

		XTAL_CASE4(TYPE_SMALL_STRING, TYPE_LONG_LIVED_STRING, TYPE_INTERNED_STRING, TYPE_STRING){
			StringPtr a = unchecked_ptr_cast<String>(v);
			uint_t sz = a->data_size();
			const char_t* str = a->data();

			int_t type = 0;
			if(a->is_interned()){
				switch(sizeof(char_t)){
					XTAL_DEFAULT{ type = TID8; }
					XTAL_CASE(2){ type = TID16; }
					XTAL_CASE(4){ type = TID32; }
				}
			}
			else{
				switch(sizeof(char_t)){
					XTAL_DEFAULT{ type = TSTRING8; }
					XTAL_CASE(2){ type = TSTRING16; }
					XTAL_CASE(4){ type = TSTRING32; }
				}
			}

			stream_->put_u8((u8)type);
			stream_->put_u32be(sz);
			for(size_t i=0; i<sz; ++i){
				stream_->put_ch_code_be(str[i]);
			}
			return;
		}

		XTAL_CASE(TYPE_ARRAY){
			ArrayPtr a = unchecked_ptr_cast<Array>(v);
			stream_->put_u8(TARRAY);
			stream_->put_u32be(a->size());
			for(uint_t i=0; i<a->size(); ++i){
				inner_serialize(a->at(i));
			}
			return;
		}

		XTAL_CASE(TYPE_VALUES){
			ValuesPtr a = unchecked_ptr_cast<Values>(v);
			stream_->put_u8(TVALUES);
			inner_serialize(a->head());
			inner_serialize(a->tail());
			return;
		}
	}
	
	if(const MapPtr& a = ptr_cast<Map>(v)){
		stream_->put_u8(TMAP);
		stream_->put_u32be(a->size());
		Xfor2(key, value, a){
			inner_serialize(key);
			inner_serialize(value);
		}
		return;
	}

	if(v->is(cpp_class<Code>())){
		CodePtr p = cast<CodePtr>(v);
		stream_->put_u8('x'); stream_->put_u8('t'); stream_->put_u8('a'); stream_->put_u8('l');
		stream_->put_u8(SERIALIZE_VERSION1); stream_->put_u8(SERIALIZE_VERSION2); 
		stream_->put_u8(0); 
		stream_->put_u8(0);
		
		uint_t sz;
		sz = p->code_.size();
		stream_->put_u32be(sz);
		//if(sz!=0){ stream_->write(&p->code_[0], sz); }	
		for(uint_t i=0; i<sz; ++i){
			stream_->put_u16be(p->code_[i]);
		}

		sz = p->scope_info_table_.size();
		stream_->put_u16be((u16)sz);
		for(uint_t i=0; i<sz; ++i){
			ScopeInfo& info = p->scope_info_table_[i];
			inner_serialize_scope_info(info);
		}
		
		sz = p->class_info_table_.size();
		stream_->put_u16be((u16)sz);
		for(uint_t i=0; i<sz; ++i){
			ClassInfo& info = p->class_info_table_[i];
			inner_serialize_scope_info(info);

			stream_->put_u16be(info.instance_variable_size);
			stream_->put_u16be(info.instance_variable_identifier_offset);
			stream_->put_u16be(info.name_number);
			stream_->put_u8(info.mixins);
		}

		sz = p->xfun_info_table_.size();
		stream_->put_u16be((u16)sz);
		for(uint_t i=0; i<sz; ++i){
			FunInfo& info = p->xfun_info_table_[i];
			inner_serialize_scope_info(info);
			
			stream_->put_u16be(info.max_stack);
			stream_->put_u16be(info.max_variable);
			stream_->put_u16be(info.name_number);
			stream_->put_u8(info.min_param_count);
			stream_->put_u8(info.max_param_count);
		}

		sz = p->except_info_table_.size();
		stream_->put_u16be((u16)sz);
		for(uint_t i=0; i<sz; ++i){
			ExceptInfo& info = p->except_info_table_[i];
			stream_->put_u32be(info.catch_pc);
			stream_->put_u32be(info.finally_pc);
			stream_->put_u32be(info.end_pc);
		}
		
		sz = p->lineno_table_.size();
		stream_->put_u16be((u16)sz);
		for(uint_t i=0; i<sz; ++i){
			Code::LineNumberInfo& info = p->lineno_table_[i];
			stream_->put_u32be(info.start_pc);
			stream_->put_u16be(info.lineno);
		}
			
		sz = p->once_table_.size();
		stream_->put_u16be((u16)sz);

		inner_serialize(p->source_file_name_);

		sz = p->identifier_table_.size();
		stream_->put_u16be((u16)sz);
		for(uint_t i=0; i<sz; ++i){
			inner_serialize(p->identifier_table_.at(i));
		}

		sz = p->value_table_.size();
		stream_->put_u16be((u16)sz);
		for(uint_t i=0; i<sz; ++i){
			inner_serialize(p->value_table_.at(i));
		}

		inner_serialize(p->breakpoint_cond_map_);

		return;
	}
	
	// 名前で保存したら読込できそうか
	ArrayPtr name_list = v->object_name_list();
	if(name_list && !name_list->is_empty()){
		stream_->put_u8(NAME);
		inner_serialize(name_list);

		Xfor_cast(ValuesPtr mv, name_list){
			if(first_step){
				if(XTAL_detail_raweq(mv->at(0), Xid(lib))){
					break;
				}
				else{
					break;
				}
			}
		}
	}
	else{
		// serial_newで空オブジェクトを生成するコマンドを埋め込む
		stream_->put_u8(SERIAL_NEW);

		// クラスを埋め込む
		inner_serialize(v->get_class()); 

		// s_saveでserializableなオブジェクトを取り出しserializeする
		inner_serialize(v->s_save());
	}
}

AnyPtr Serializer::inner_deserialize_serial_new(){
	int_t num = append_value(null);

	// serial_newをするクラスを取り出す
	if(ClassPtr c = ptr_cast<Class>(inner_deserialize())){
		// serial_newを呼び出して、保存しておく
		const VMachinePtr& vm = setup_call(1);
		c->s_new(vm);
		AnyPtr ret = vm->result();
		values_.set_at(num, ret);
		vm->cleanup_call();

		ret->s_load(ptr_cast<Map>(inner_deserialize()));

		return ret;
	}

	return undefined;
}

AnyPtr Serializer::inner_deserialize_name(){
	int_t num = append_value(null);
	values_.set_at(num, demangle(inner_deserialize()));
	return values_.at(num);
}

StringPtr Serializer::inner_deserialize_string(int_t charsize, bool bintern){
	StringPtr ret;
	if(uint_t sz = stream_->get_u32be()){
		XMallocGuard guard(sizeof(char_t)*sz);
		char_t* p = (char_t*)guard.get();

		switch(charsize){
			XTAL_DEFAULT{ for(uint_t i = 0; i<sz; ++i){ p[i] = (char_t)stream_->get_u8(); } }
			XTAL_CASE(2){ for(uint_t i = 0; i<sz; ++i){ p[i] = (char_t)stream_->get_u16be(); } }
			XTAL_CASE(4){ for(uint_t i = 0; i<sz; ++i){ p[i] = (char_t)stream_->get_u32be(); } }
		}

		if(bintern){
			ret = intern(p, sz);
		}
		else{
			ret = XNew<String>(p, sz);
		}
	}
	else{
		ret = empty_id;
	}
	append_value(ret);
	return ret;
}

ArrayPtr Serializer::inner_deserialize_array(){
	uint_t sz = stream_->get_u32be();
	ArrayPtr ret = xnew<Array>(sz);
	append_value(ret);
	for(uint_t i = 0; i<sz; ++i){
		ret->set_at(i, inner_deserialize());
	}				
	return ret;
}

ValuesPtr Serializer::inner_deserialize_values(){
	ValuesPtr ret = xnew<Values>(null);
	append_value(ret);
	AnyPtr head = inner_deserialize();
	AnyPtr tail = inner_deserialize();
	ret->set(head, unchecked_ptr_cast<Values>(tail));		
	return ret;
}

MapPtr Serializer::inner_deserialize_map(){
	int_t sz = stream_->get_u32be();
	MapPtr ret = xnew<Map>();
	append_value(ret);
	AnyPtr key;
	for(int_t i = 0; i<sz; ++i){
		key = inner_deserialize();
		ret->set_at(key, inner_deserialize());
	}				
	return ret;
}

void Serializer::inner_deserialize_scope_info(ScopeInfo& info){
	info.pc = stream_->get_u32be();
	info.kind = stream_->get_u8();
	info.flags = stream_->get_u8();
	info.variable_identifier_offset = stream_->get_u16be();
	info.variable_offset = stream_->get_u16be();
	info.variable_size = stream_->get_u16be();		
}

CodePtr Serializer::inner_deserialize_code(){
	CodePtr p = xnew<Code>();
	append_value(p);

	u8 head[7];
	stream_->read(head, 7);

	if(head[0]!='t' || head[1]!='a' || head[2]!='l' ){
		set_runtime_error(Xt("XRE1009"));
		return null;
	}

	if(head[3]!=SERIALIZE_VERSION1 && head[4]!=SERIALIZE_VERSION2){
		set_runtime_error(Xt("XRE1009"));
		return null;
	}

	uint_t sz;
	sz = stream_->get_u32be();
	p->code_.resize(sz);
	//if(sz){ stream_->read(&p->code_[0], sz); }
	for(uint_t i=0; i<sz; ++i){
		p->code_[i] = stream_->get_u16be();
	}	

	sz = stream_->get_u16be();
	p->scope_info_table_.resize(sz);
	for(uint_t i=0; i<sz; ++i){
		ScopeInfo& info = p->scope_info_table_[i];
		inner_deserialize_scope_info(info);
	}
	
	sz = stream_->get_u16be();
	p->class_info_table_.resize(sz);
	for(uint_t i=0; i<sz; ++i){
		ClassInfo& info = p->class_info_table_[i];
		inner_deserialize_scope_info(info);

		info.instance_variable_size = stream_->get_u16be();
		info.instance_variable_identifier_offset = stream_->get_u16be();
		info.name_number = stream_->get_u16be();
		info.mixins = stream_->get_u8();
	}

	sz = stream_->get_u16be();
	p->xfun_info_table_.resize(sz);
	for(uint_t i=0; i<sz; ++i){
		FunInfo& info = p->xfun_info_table_[i];
		inner_deserialize_scope_info(info);

		info.max_stack = stream_->get_u16be();
		info.max_variable = stream_->get_u16be();
		info.name_number = stream_->get_u16be();
		info.min_param_count = stream_->get_u8();
		info.max_param_count = stream_->get_u8();
	}

	sz = stream_->get_u16be();
	p->except_info_table_.resize(sz);
	for(uint_t i=0; i<sz; ++i){
		ExceptInfo& info = p->except_info_table_[i];
		info.catch_pc = stream_->get_u32be();
		info.finally_pc = stream_->get_u32be();
		info.end_pc = stream_->get_u32be();
	}
	
	sz = stream_->get_u16be();
	p->lineno_table_.resize(sz);
	for(uint_t i=0; i<sz; ++i){
		Code::LineNumberInfo& info = p->lineno_table_[i];
		info.start_pc = stream_->get_u32be();
		info.lineno = stream_->get_u16be();
	}

	sz = stream_->get_u16be();
	p->once_table_.resize(sz);
	for(uint_t i=0; i<sz; ++i){
		p->once_table_.set_at(i, undefined);
	}

	p->first_fun_ = xnew<Method>(nul<Frame>(), p, &p->xfun_info_table_[0]);
	
	p->source_file_name_ = ptr_cast<String>(inner_deserialize());

	sz = stream_->get_u16be();
	p->identifier_table_.resize(sz);
	for(uint_t i=0; i<sz; ++i){
		p->identifier_table_.set_at(i, inner_deserialize());
	}

	sz = stream_->get_u16be();
	p->value_table_.resize(sz);
	for(uint_t i=0; i<sz; ++i){
		p->value_table_.set_at(i, inner_deserialize());
	}

	p->breakpoint_cond_map_ = ptr_cast<Map>(inner_deserialize());

	p->generated();

	return p;
}

AnyPtr Serializer::inner_deserialize(){
	if(stream_->eos()){
		return undefined;
	}

	XTAL_CHECK_EXCEPT(e){
		return undefined;
	}

	int_t op = stream_->get_u8();
	switch(op){
		XTAL_DEFAULT;

		XTAL_CASE(SERIAL_NEW){
			return inner_deserialize_serial_new();
		}

		XTAL_CASE(NAME){
			return inner_deserialize_name();
		}	

		XTAL_CASE(REFERENCE){
			return values_.at(stream_->get_u32be());
		}

		XTAL_CASE(TNULL){
			return null;
		}
		
		XTAL_CASE(TUNDEFINED){
			return undefined;
		}

		XTAL_CASE(TINT32){
			return (int_t)stream_->get_i32be();
		}

		XTAL_CASE(TFLOAT32){
			return (float_t)stream_->get_f32be();
		}

		XTAL_CASE(TINT64){
			return (int_t)stream_->get_i64be();
		}

		XTAL_CASE(TFLOAT64){
			return (float_t)stream_->get_f64be();
		}

		XTAL_CASE(TSTRING8){
			return inner_deserialize_string(1, false);
		}

		XTAL_CASE(TID8){
			return inner_deserialize_string(1, true);
		}

		XTAL_CASE(TSTRING16){
			return inner_deserialize_string(2, false);
		}

		XTAL_CASE(TID16){
			return inner_deserialize_string(2, true);
		}

		XTAL_CASE(TSTRING32){
			return inner_deserialize_string(4, false);
		}

		XTAL_CASE(TID32){
			return inner_deserialize_string(4, true);
		}

		XTAL_CASE(TARRAY){
			return inner_deserialize_array();
		}

		XTAL_CASE(TVALUES){
			return inner_deserialize_values();
		}
		
		XTAL_CASE(TMAP){
			return inner_deserialize_map();
		}
		
		XTAL_CASE(TTRUE){
			return true;
		}
		
		XTAL_CASE(TFALSE){
			return false;
		}

		XTAL_CASE('x'){
			return inner_deserialize_code();
		}
	}
	return null;
}

AnyPtr Serializer::demangle(const AnyPtr& n){
	AnyPtr ret;
	Xfor_cast(ValuesPtr mv, n){
		if(first_step){
			if(XTAL_detail_raweq(mv->at(0), Xid(lib))){
				ret = lib();
			}
			else{
				break;
			}
		}
		else{
			if(IDPtr id = ptr_cast<ID>(mv->at(0))){
				ret = ret->member(id, mv->at(1)); 
			}
			else{
				break;	
			}
		}
	}

	if(!ret){
		set_runtime_error(Xt1("XRE1008", object, n));
		return null;
	}

	return ret;
}

int_t Serializer::register_value(const AnyPtr& v){
	Map::iterator it = map_->find_direct(v);
	if(it!=map_->end()){
		return XTAL_detail_ivalue(it->second);
	}
	map_->insert_direct(v, values_.size());
	values_.push_back(v);
	return -1;
}

int_t Serializer::append_value(const AnyPtr& v){
	values_.push_back(v);
	return values_.size()-1;
}

void Serializer::put_tab(int_t tab){
	while(tab--){
		stream_->put_s(XTAL_STRING("\t"));
	}
}

void Serializer::clear(){
	values_.clear();
	map_->clear();
}

}
