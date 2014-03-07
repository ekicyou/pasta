/** \file src/xtal/xtal_frame.h
* \brief src/xtal/xtal_frame.h
*/

#ifndef XTAL_FRAME_H_INCLUDE_GUARD
#define XTAL_FRAME_H_INCLUDE_GUARD

#pragma once

namespace xtal{

/**
* \xbind lib::builtin
* \xinherit lib::builtin::Any
* \brief フレーム
*/
class Frame : public HaveParentBase{
public:
	
	Frame(const FramePtr& outer, const CodePtr& code, ScopeInfo* info);
	
	Frame();

	~Frame();

public:
	
	/**
	* \xbind
	* \brief 外側のスコープを表すFrameオブジェクトを返す。
	*/
	const FramePtr& outer(){ 
		return outer_; 
	}

	void set_outer(const FramePtr& outer){
		outer_ = outer;
	}

	const CodePtr& code();

	void set_code(const CodePtr& code);

	ScopeInfo* info(){
		return scope_info_;
	}

	void set_info(ScopeInfo* info){
		scope_info_ = info;
	}

	/**
	* \brief リテラル時に定義された要素の数を返す。
	*/
	int_t block_size(){ 
		return scope_info_->variable_size; 
	}

	/**
	* \xbind
	* \brief メンバが格納された、Iteratorを返す
	* ブロックパラメータは(primary_key, secondary_key, value)
	*/
	AnyPtr members();

public:

	/**
	* \brief i番目のメンバーをダイレクトに取得。
	*/
	AnyPtr& member_direct(int_t i){
		return (AnyPtr&)members_.at(i);
	}

	/**
	* \brief i番目のメンバーをダイレクトに設定。
	*/
	void set_member_direct(int_t i, const AnyPtr& value){
		members_.set_at(i, value);
	}

	void resize_member_direct(int_t size){
		members_.resize(size);
	}

	bool replace_member(const IDPtr& primary_key, const AnyPtr& value);

protected:

	void make_members();
	void make_members_force(int_t flags);

protected:

	enum{
		FLAG_ACCESSIBILITY_MASK = 0x3,
		FLAG_NOCACHE = 1<<3,
		FLAG_NODE3 = 1<<4
	};

	struct Node{
		Node* next;
		u16 num;
		u16 flags;

		int_t accessibility() const{
			return flags & FLAG_ACCESSIBILITY_MASK;
		}

		bool nocache() const{
			return (flags&FLAG_NOCACHE)!=0;
		}
	};

	struct Node2 : public Node{
		IDPtr primary_key;
	};

	struct Node3 : public Node2{
		AnyPtr secondary_key;
	};

protected:

	static uint_t hashcode(const IDPtr& primary_key, const AnyPtr& secondary_key){
		return XTAL_detail_rawhash(primary_key) ^ XTAL_detail_rawhash(secondary_key);
	}

	void expand_buckets();

	Node* find_node(const IDPtr& primary_key, const AnyPtr& secondary_key);

	Node* insert_node(const IDPtr& primary_key, const AnyPtr& secondary_key, uint_t num);

protected:
	enum{
		FLAG_NATIVE = 1<<0,
		FLAG_FINAL = 1<<1,
		FLAG_SINGLETON = 1<<2,
		FLAG_PREBINDED = 1<<3,
		FLAG_BINDED = 1<<4,
		FLAG_BINDED2 = 1<<5,
		FLAG_CPP_INHERIT = 1<<6,

		FLAG_LAST_DEFINED_CTOR = 1<<7,
		FLAG_LAST_DEFINED_CTOR2 = 1<<8,

		FLAG_ORPHAN = 1<<9,
		FLAG_INITIALIZED_MEMBERS = 1<<10,

		FLAG_OPTIONS = 1<<11	
	};

	uint_t orphan(){
		return flags_ & FLAG_ORPHAN;
	}

	void set_orphan(){
		flags_ |= FLAG_ORPHAN;
	}
	
	void unset_orphan(){
		flags_ &= ~FLAG_ORPHAN;
	}

	uint_t initialized_members(){
		return flags_ & FLAG_INITIALIZED_MEMBERS;
	}

	void set_initialized_members(){
		flags_ |= FLAG_INITIALIZED_MEMBERS;
	}

public:
	
	const AnyPtr& on_rawmember(const IDPtr& primary_key, const AnyPtr& secondary_key, bool inherited_too, int_t& accessibility, bool& nocache);

	void on_visit_members(Visitor& m);

	void attach(ScopeInfo* info, Code* code, AnyPtr* values, uint_t size){
		scope_info_ = info;
		code_ = code;
		members_.attach(values, size);
	}

	void detach(){
		members_.detach();
		//outer_ = null;
		//code_ = null;
	}

protected:
	BasePtr<Frame> outer_;
	BasePtr<Code> code_;
	ScopeInfo* scope_info_;

	xarray members_;

	Node** buckets_;
	uint_t buckets_capa_;

	u16 object_force_;
	u16 flags_;

private:
	XTAL_DISALLOW_COPY_AND_ASSIGN(Frame);

	friend class MembersIter;
	friend class MembersIter2;
	friend class VMachine;
};

class MembersIter : public Base{
public:

	MembersIter(const FramePtr& a);

	void block_next(const VMachinePtr& vm);

	void move_next();

	void on_visit_members(Visitor& m);

private:
	BasePtr<Frame> frame_;
	Frame::Node* node_;
	uint_t pos_;
};

class MembersIter2 : public Base{
public:

	MembersIter2(const FramePtr& a);

	void block_next(const VMachinePtr& vm);

	void on_visit_members(Visitor& m);

private:
	BasePtr<Frame> frame_;
	int_t it_;
};

}

#endif // XTAL_FRAME_H_INCLUDE_GUARD
