/** \file src/xtal/xtal_basictype.h
* \brief src/xtal/xtal_basictype.h
*/

#ifndef XTAL_BASICTYPE_H_INCLUDE_GUARD
#define XTAL_BASICTYPE_H_INCLUDE_GUARD

#pragma once

namespace xtal{

/**
* \xbind lib::builtin
* \brief ヌル値
*/
class Null : public Any{
public:
	Null(){ 
		value_.init_primitive(TYPE_NULL); 
	} 
};

/**
* \xbind lib::builtin
* \brief 未定義値
*/
class Undefined : public Any{ 
public: 
	Undefined(){ 
		value_.init_primitive(TYPE_UNDEFINED); 
	} 
};

/**
* \xbind lib::builtin
* \brief 整数値
*/
class Int : public Any{
public:
	typedef int_t value_type;

	Int(int_t val = 0){ 
		value_.init_int(val); 
	}

public:
	
	int_t to_i(){
		return XTAL_detail_ivalue(*this);
	}

	float_t to_f(){
		return (float_t)XTAL_detail_ivalue(*this);
	}

	StringPtr to_s(){
		return Any::to_s();
	}

	bool op_in_IntRange(const IntRangePtr& range);

	bool op_in_FloatRange(const FloatRangePtr& range);

	IntRangePtr op_range_Int(int_t right, int_t kind);

	FloatRangePtr op_range_Float(float_t right, int_t kind);

public:

	operator const AnyPtr&() const{
		return ap(*this);
	}
};

/**
* \xbind lib::builtin
* \brief 浮動小数点値
*/
class Float : public Any{
public:
	typedef float_t value_type;

	Float(float_t val = 0){ 
		value_.init_float(val); 
	}

public:

	int_t to_i(){
		return (int_t)XTAL_detail_fvalue(*this);
	}

	float_t to_f(){
		return XTAL_detail_fvalue(*this);
	}

	StringPtr to_s(){
		return Any::to_s();
	}

	bool op_in_IntRange(const IntRangePtr& range);

	bool op_in_FloatRange(const FloatRangePtr& range);

	FloatRangePtr op_range_Int(int_t right, int_t kind);

	FloatRangePtr op_range_Float(float_t right, int_t kind);

public:

	operator const AnyPtr&() const{
		return ap(*this);
	}

};

/**
* \xbind lib::builtin
* \brief 即値
*/
class ImmediateValue : public Any{
public:

	ImmediateValue(int_t value1, int_t value2){
		value_.init_immediate_value(value1, value2);
	}

	ImmediateValue(int_t value1, float_t value2){
		value_.init_immediate_value(value1, value2);
	}

	ImmediateValue(int_t value1, void* value2){
		value_.init_immediate_value(value1, value2);
	}

public:

	int_t first() const{
		return value_.immediate_first_value();
	}

	int_t second_ivalue() const{
		return value_.immediate_second_ivalue();
	}

	float_t second_fvalue() const{
		return value_.immediate_second_fvalue();
	}

	void* second_vpvalue() const{
		return value_.immediate_second_vpvalue();
	}

public:

	operator const AnyPtr&() const{
		return ap(*this);
	}
};

/**
* \xbind lib::builtin
* \brief 真偽値
*/
class Bool : public Any{
public: 
	typedef bool value_type;

	Bool(bool b)
		{ value_.init_bool(b); } 

public:

	operator const AnyPtr&() const{
		return ap(*this);
	}
};

/**
* \xbind lib::builtin
* \brief 区間
*/
class Range : public Base{
public:

	Range(const AnyPtr& left, const AnyPtr& right, int_t kind = RANGE_CLOSED)
		:left_(left), right_(right), kind_(kind){}

	/**
	* \xbind
	* \brief 区間の左の要素を返す
	*/
	const AnyPtr& left(){ return left_; }

	/**
	* \xbind
	* \brief 区間の右の要素を返す
	*/
	const AnyPtr& right(){ return right_; }

	/**
	* \xbind
	* \brief 区間の種類を返す
	*/
	int_t kind(){ return kind_; }

	/**
	* \xbind
	* \brief 左が閉じた区間か返す
	*/
	bool is_left_closed(){ return (kind_&(1<<1))==0; }

	/**
	* \xbind
	* \brief 右が閉じた区間か返す
	*/
	bool is_right_closed(){ return (kind_&(1<<0))==0; }

	/**
	* \brief 区間の種類
	*/
	enum RangeKind{
		/**
		* \brief 閉区間[left, right]
		*/
		CLOSED = RANGE_CLOSED,

		/**
		* \brief 左開右閉区間 [left, right)
		*/
		LEFT_CLOSED_RIGHT_OPEN = RANGE_LEFT_CLOSED_RIGHT_OPEN,

		/**
		* \brief 左開右閉区間 (left, right]
		*/
		LEFT_OPEN_RIGHT_CLOSED = RANGE_LEFT_OPEN_RIGHT_CLOSED,

		/**
		* \brief 開区間 (left, right)
		*/
		OPEN = RANGE_OPEN
	};

	void on_visit_members(Visitor& m){
		Base::on_visit_members(m);
		m & left_ & right_;
	}

protected:

	AnyPtr left_;
	AnyPtr right_;
	int_t kind_;
};

/**
* \xbind lib::builtin
* \brief 区間
*/
class IntRange : public Range{
public:

	IntRange(int_t left, int_t right, int_t kind = RANGE_CLOSED)
		:Range(left, right, kind){}

public:

	/**
	* \xbind
	* \brief 範囲の開始を返す
	*
	* beginは左閉右開区間 [begin, end) で得ることが出来る 
	*/
	int_t begin(){ return is_left_closed() ? left() : left()+1; }

	/**
	* \xbind
	* \brief 範囲の終端を返す
	*
	* endは左閉右開区間 [begin, end) で得ることが出来る 
	*/
	int_t end(){ return is_right_closed() ? right()+1 : right(); }

	/**
	* \xbind
	* \brief 区間の左の要素を返す
	*/
	int_t left(){ return XTAL_detail_ivalue(left_); }

	/**
	* \xbind
	* \brief 区間の左の要素を返す
	*/
	int_t right(){ return XTAL_detail_ivalue(right_); }

	AnyPtr each();
};

class IntRangeIter : public Base{
public:

	IntRangeIter(const IntRangePtr& range)
		:it_(range->begin()), end_(range->end()){
	}

	void block_next(const VMachinePtr& vm);

private:
	int_t it_, end_;
};

/**
* \xbind lib::builtin
* \brief 区間
*/
class FloatRange : public Range{
public:

	FloatRange(float_t left, float_t right, int_t kind = RANGE_CLOSED)
		:Range(left, right, kind){}

public:

	/**
	* \xbind
	* \brief 区間の左の要素を返す
	*/
	float_t left(){ return XTAL_detail_fvalue(left_); }

	/**
	* \xbind
	* \brief 区間の左の要素を返す
	*/
	float_t right(){ return XTAL_detail_fvalue(right_); }

	AnyPtr each();
};

/**
* \brief 多値
*/
class Values : public RefCountingBase{
public:
	enum{ TYPE = TYPE_VALUES };

	Values(const AnyPtr& head);

	Values(const AnyPtr& head, const ValuesPtr& tail);

	~Values();

	const AnyPtr& head(){
		return head_;
	}

	const ValuesPtr& tail(){
		return tail_;
	}

	void block_next(const VMachinePtr& vm);

	int_t size();

	const AnyPtr& at(int_t i);

	const AnyPtr& op_at(int_t i){
		return at(i);
	}

	/**
	* \xbind
	* \brief 値が等しいか調べる
	*/
	bool op_eq(const ValuesPtr& other);

	StringPtr to_s();

	void on_visit_members(Visitor& m){
		m & head_ & tail_;
	}

public:

	void set(const AnyPtr& head, const ValuesPtr& tail = null);
	
private:
	AnyPtr head_;
	ValuesPtr tail_;
};

ValuesPtr mv(const AnyPtr& v1, const AnyPtr& v2);

class HaveParent{
public:

	HaveParent(){}

	const ClassPtr& object_parent();

	void set_object_parent(const ClassPtr& parent);

	void orphan(){
		parent_ = null;
	}

private:
	BasePtr<Class> parent_;

private:
	XTAL_DISALLOW_COPY_AND_ASSIGN(HaveParent);
};

/**
* \brief スコープ的な親を認識するクラス
*/
class HaveParentBase : public Base{
public:

	const ClassPtr& on_object_parent(){
		return have_parent_.object_parent();
	}

	void on_set_object_parent(const ClassPtr& parent){
		have_parent_.set_object_parent(parent);
	}

	void on_visit_members(Visitor& m){
		Base::on_visit_members(m);
		m & have_parent_.object_parent();
	}

	void object_orphan(){
		have_parent_.orphan();
	}

protected:
	HaveParent have_parent_;
};

/**
* \brief スコープ的な親を認識するクラス
*/
class HaveParentRefCountingBase : public RefCountingBase{
public:

	const ClassPtr& on_object_parent(){
		return have_parent_.object_parent();
	}

	void on_set_object_parent(const ClassPtr& parent){
		have_parent_.set_object_parent(parent);
	}

	void on_visit_members(Visitor& m){
		m & have_parent_.object_parent();
	}

	void object_orphan(){
		have_parent_.orphan();
	}

protected:
	HaveParent have_parent_;
};

}

#endif // XTAL_BASICTYPE_H_INCLUDE_GUARD
