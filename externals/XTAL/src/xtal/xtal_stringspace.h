/** \file src/xtal/xtal_stringspace.h
* \brief src/xtal/xtal_stringspace.h
*/

#if !defined(XTAL_STRINGSPACE_H_INCLUDE_GUARD) || defined(XATL_DEFINE_IDs)

namespace xtal{

#ifndef XATL_DEFINE_IDs

#define XTAL_DEFINE_ID(x) id_##x,
#define XTAL_DEFINE_ID2(x, y) id_##x = id_##y,

struct DefinedID{ enum type{
	id_,

#else

#ifdef XTAL_DEFINE_ID
#undef XTAL_DEFINE_ID
#endif

#ifdef XTAL_DEFINE_ID2
#undef XTAL_DEFINE_ID2
#endif

#define XTAL_DEFINE_ID(x) XTAL_L(#x),
#define XTAL_DEFINE_ID2(x, y) 

static const char_t* id_list[] = {

#endif
		
	XTAL_DEFINE_ID(if)
	XTAL_DEFINE_ID2(keyword_begin, if)
	XTAL_DEFINE_ID(for)
	XTAL_DEFINE_ID(else)
	XTAL_DEFINE_ID(fun)
	XTAL_DEFINE_ID(method)
	XTAL_DEFINE_ID(do)
	XTAL_DEFINE_ID(while)
	XTAL_DEFINE_ID(continue)
	XTAL_DEFINE_ID(break)
	XTAL_DEFINE_ID(fiber)
	XTAL_DEFINE_ID(yield)
	XTAL_DEFINE_ID(return)
	XTAL_DEFINE_ID(once)
	XTAL_DEFINE_ID(null)
	XTAL_DEFINE_ID(undefined)
	XTAL_DEFINE_ID(false)
	XTAL_DEFINE_ID(true)
	XTAL_DEFINE_ID(xtal)
	XTAL_DEFINE_ID(try)
	XTAL_DEFINE_ID(catch)
	XTAL_DEFINE_ID(finally)
	XTAL_DEFINE_ID(throw)
	XTAL_DEFINE_ID(class)
	XTAL_DEFINE_ID(callee)
	XTAL_DEFINE_ID(this)
	XTAL_DEFINE_ID(dofun)
	XTAL_DEFINE_ID(is)
	XTAL_DEFINE_ID(in)
	XTAL_DEFINE_ID(assert)
	XTAL_DEFINE_ID(nobreak)
	XTAL_DEFINE_ID(switch)
	XTAL_DEFINE_ID(case)
	XTAL_DEFINE_ID(default)
	XTAL_DEFINE_ID(singleton)
	XTAL_DEFINE_ID(public)
	XTAL_DEFINE_ID(protected)
	XTAL_DEFINE_ID(private)
	XTAL_DEFINE_ID(keyword_end)

////////////////////////////////////////

	XTAL_DEFINE_ID(op_call)

	XTAL_DEFINE_ID(op_inc)
	XTAL_DEFINE_ID(op_dec)
	XTAL_DEFINE_ID(op_pos)
	XTAL_DEFINE_ID(op_neg)
	XTAL_DEFINE_ID(op_com)

	XTAL_DEFINE_ID(op_at)
	XTAL_DEFINE_ID(op_set_at)
	XTAL_DEFINE_ID(op_range)

	XTAL_DEFINE_ID(op_add)
	XTAL_DEFINE_ID(op_sub)
	XTAL_DEFINE_ID(op_cat)
	XTAL_DEFINE_ID(op_mul)
	XTAL_DEFINE_ID(op_div)
	XTAL_DEFINE_ID(op_mod)
	XTAL_DEFINE_ID(op_and)
	XTAL_DEFINE_ID(op_or)
	XTAL_DEFINE_ID(op_xor)
	XTAL_DEFINE_ID(op_shl)
	XTAL_DEFINE_ID(op_shr)
	XTAL_DEFINE_ID(op_ushr)

	XTAL_DEFINE_ID(op_add_assign)
	XTAL_DEFINE_ID(op_sub_assign)
	XTAL_DEFINE_ID(op_cat_assign)
	XTAL_DEFINE_ID(op_mul_assign)
	XTAL_DEFINE_ID(op_div_assign)
	XTAL_DEFINE_ID(op_mod_assign)
	XTAL_DEFINE_ID(op_and_assign)
	XTAL_DEFINE_ID(op_or_assign)
	XTAL_DEFINE_ID(op_xor_assign)
	XTAL_DEFINE_ID(op_shl_assign)
	XTAL_DEFINE_ID(op_shr_assign)
	XTAL_DEFINE_ID(op_ushr_assign)

	XTAL_DEFINE_ID(op_eq)
	XTAL_DEFINE_ID(op_lt)
	XTAL_DEFINE_ID(op_in)

	//
	XTAL_DEFINE_ID(to_i)
	XTAL_DEFINE_ID(to_f)
	XTAL_DEFINE_ID(to_s)
	XTAL_DEFINE_ID(op_to_array)
	XTAL_DEFINE_ID(op_to_map)
	XTAL_DEFINE_ID(serial_save)
	XTAL_DEFINE_ID(serial_load)
	XTAL_DEFINE_ID(initialize)
	XTAL_DEFINE_ID(p)
	XTAL_DEFINE_ID(filelocal)
	XTAL_DEFINE_ID(toplevel)
	XTAL_DEFINE_ID(debug)
	XTAL_DEFINE_ID(arg)
	XTAL_DEFINE_ID(each)
	XTAL_DEFINE_ID(block_first)
	XTAL_DEFINE_ID(block_next)
	XTAL_DEFINE_ID(block_break)
	XTAL_DEFINE_ID(lib)
	XTAL_DEFINE_ID(global)
	XTAL_DEFINE_ID(stdin)
	XTAL_DEFINE_ID(stdout)
	XTAL_DEFINE_ID(stderr)
	XTAL_DEFINE_ID(builtin)
	XTAL_DEFINE_ID(filesystem)

	XTAL_DEFINE_ID(UNKNOWN)

#ifndef XATL_DEFINE_IDs

	DUMMY_DUM
};};
#else
};
#endif

#define XTAL_DEFINED_ID(x) ::xtal::fetch_defined_id(::xtal::DefinedID::id_##x)

}

#endif


#ifndef XTAL_STRINGSPACE_H_INCLUDE_GUARD
#define XTAL_STRINGSPACE_H_INCLUDE_GUARD

//#pragma once

namespace xtal{

/*
* \brief 文字列を管理するクラス
*/
class StringSpace{
public:

	// デフォルトでは、ノードの数、文字列の長さに16bit制限をかける
	typedef u16 node_t;
	typedef u16 strsize_t;

	struct Node{
		node_t next;
		strsize_t size;
		u16 hash;
		u16 flags;

		void set_pointer(char_t* str){ *(char_t**)XTAL_STRUCT_TAIL(this) = str; }
		char_t* pointer(){ return *(char_t**)XTAL_STRUCT_TAIL(this); }
		char_t* buf(){ return (char_t*)XTAL_STRUCT_TAIL(this); }
		char_t* data() const{ return flags ? *(char_t**)XTAL_STRUCT_TAIL(this) : (char_t*)XTAL_STRUCT_TAIL(this); }
	};

public:

	void initialize();

	void uninitialize();

	const Node* fetch(uint_t n){
		return nodes_[n];
	}

	const char_t* register_string(const char_t* str, uint_t size, uint_t hashcode, bool long_lived);

private:

	enum{
		LIMIT_SHIFT = 9,
		LIMIT_MASK = (1<<LIMIT_SHIFT)-1,
		LIMIT = 1<<(LIMIT_SHIFT+2)
	};

	struct Block{
		uint_t pos;

		void* alloc(int size);
	};

	void add_block();

	void expand_buckets(uint_t n);

	void expand_nodes(uint_t n);

	void* alloc(int size);

private:
	Block** blocks_;
	Block* current_block_;
	uint_t blocks_size_;
	uint_t blocks_capa_;

	node_t* buckets_;
	uint_t buckets_size_;
	uint_t buckets_capa_;

	Node** nodes_;
	uint_t nodes_size_;
	uint_t nodes_capa_;
};

}

#endif // XTAL_STRINGSPACE_H_INCLUDE_GUARD
