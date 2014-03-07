/** \file src/xtal/xtal_base.h
* \brief src/xtal/xtal_base.h
*/

#ifndef XTAL_BASE_H_INCLUDE_GUARD
#define XTAL_BASE_H_INCLUDE_GUARD

#pragma once

namespace xtal{

template<int N, class T>
struct RCBaseClassType{
	enum{ value = 0 };
};

template<class T>
struct RCBaseClassType<INHERITED_RCBASE, T>{
	enum{ value = T::TYPE };
};

struct VirtualMembers{
	CppClassSymbolData* cpp_class_symbol_data;
	u16 rcbase_class_type;
	u16 object_align;
	u16 object_size;
	u16 is_container;

	void (*object_free)(RefCountingBase*);
	void (*object_destroy)(RefCountingBase*);
	void (*visit_members)(RefCountingBase* p, Visitor& m);
	
	void (*rawcall)(RefCountingBase* p, const VMachinePtr& vm);
	const AnyPtr& (*rawmember)(RefCountingBase* p, const IDPtr& primary_key, const AnyPtr& secondary_key, bool inherited_too, int_t& accessibility, bool& nocache);
	void (*def)(RefCountingBase* p, const IDPtr& primary_key, const AnyPtr& value, const AnyPtr& secondary_key, int_t accessibility);
	const ClassPtr& (*object_parent)(RefCountingBase* p);
	void (*set_object_parent)(RefCountingBase* p, const ClassPtr& parent);	
	void (*finalize)(RefCountingBase* p);
	void (*shrink_to_fit)(RefCountingBase* p);
};

struct VirtualMembersN{	
	static void visit_members(RefCountingBase* p, Visitor& m);
	static void rawcall(RefCountingBase* p, const VMachinePtr& vm);
	static const AnyPtr& rawmember(RefCountingBase* p, const IDPtr& primary_key, const AnyPtr& secondary_key, bool inherited_too, int_t& accessibility, bool& nocache);
	static void def(RefCountingBase* p, const IDPtr& primary_key, const AnyPtr& value, const AnyPtr& secondary_key, int_t accessibility);
	static const ClassPtr& object_parent(RefCountingBase* p);
	static void set_object_parent(RefCountingBase* p, const ClassPtr& parent);
	static void finalize(RefCountingBase* p);
	static void shrink_to_fit(RefCountingBase* p);
};

template<class T>
struct VirtualMembersM{
	static T* cast(RefCountingBase* p){ return static_cast<T*>(p); }
	
	static void object_free(RefCountingBase* p){ object_xfree<T>(cast(p)); }
	static void object_destroy(RefCountingBase* p){ cast(p)->~T(); }
	static void visit_members(RefCountingBase* p, Visitor& m){ cast(p)->on_visit_members(m); }
	
	static void rawcall(RefCountingBase* p, const VMachinePtr& vm){ cast(p)->on_rawcall(vm); }
	static const AnyPtr& rawmember(RefCountingBase* p, const IDPtr& primary_key, const AnyPtr& secondary_key, bool inherited_too, int_t& accessibility, bool& nocache){ return cast(p)->on_rawmember(primary_key, secondary_key, inherited_too, accessibility, nocache); }
	static void def(RefCountingBase* p, const IDPtr& primary_key, const AnyPtr& value, const AnyPtr& secondary_key, int_t accessibility){ cast(p)->on_def(primary_key, value, secondary_key, accessibility); }
	static const ClassPtr& object_parent(RefCountingBase* p){ return cast(p)->on_object_parent(); }
	static void set_object_parent(RefCountingBase* p, const ClassPtr& parent){ cast(p)->on_set_object_parent(parent); }
	static void finalize(RefCountingBase* p){ cast(p)->on_finalize(); }
	static void shrink_to_fit(RefCountingBase* p){ cast(p)->on_shrink_to_fit(); }
};


struct VirtualMembersOverwriteCheckerFuncs{
	static NoType rawcall(void (RefCountingBase::*)(const VMachinePtr&));
	template<class U> static YesType rawcall(void (U::*)(const VMachinePtr&));

	static NoType rawmember(const AnyPtr& (RefCountingBase::*)(const IDPtr&, const AnyPtr&, bool, int_t&, bool&));
	template<class U> static YesType rawmember(const AnyPtr& (U::*)(const IDPtr&, const AnyPtr&, bool, int_t&, bool&));

	static NoType def(void (RefCountingBase::*)(const IDPtr&, const AnyPtr&, const AnyPtr&, int_t));
	template<class U> static YesType def(void (U::*)(const IDPtr&, const AnyPtr&, const AnyPtr&, int_t));

	static NoType object_parent(const ClassPtr& (RefCountingBase::*)());
	template<class U> static YesType object_parent(const ClassPtr& (U::*)());

	static NoType set_object_parent(void (RefCountingBase::*)(const ClassPtr&));
	template<class U> static YesType set_object_parent(void (U::*)(const ClassPtr&));

	static NoType visit_members(void (RefCountingBase::*)(Visitor&));
	template<class U> static YesType visit_members(void (U::*)(Visitor&));

	static NoType finalize(void (RefCountingBase::*)());
	template<class U> static YesType finalize(void (U::*)());

	static NoType shrink_to_fit(void (RefCountingBase::*)());
	template<class U> static YesType shrink_to_fit(void (U::*)());
};

template<class T>
struct VirtualMembersOverwriteChecker{

	enum{
		rawcall_overwrite = sizeof(VirtualMembersOverwriteCheckerFuncs::rawcall(&T::on_rawcall))!=sizeof(NoType),
		rawmember_overwrite = sizeof(VirtualMembersOverwriteCheckerFuncs::rawmember(&T::on_rawmember))!=sizeof(NoType),
		def_overwrite = sizeof(VirtualMembersOverwriteCheckerFuncs::def(&T::on_def))!=sizeof(NoType),
		object_parent_overwrite = sizeof(VirtualMembersOverwriteCheckerFuncs::object_parent(&T::on_object_parent))!=sizeof(NoType),
		set_object_parent_overwrite = sizeof(VirtualMembersOverwriteCheckerFuncs::set_object_parent(&T::on_set_object_parent))!=sizeof(NoType),
		visit_members_overwrite = sizeof(VirtualMembersOverwriteCheckerFuncs::visit_members(&T::on_visit_members))!=sizeof(NoType),
		finaize_overwrite = sizeof(VirtualMembersOverwriteCheckerFuncs::finalize(&T::on_finalize))!=sizeof(NoType),
		shrink_to_fit_overwrite = sizeof(VirtualMembersOverwriteCheckerFuncs::shrink_to_fit(&T::on_shrink_to_fit))!=sizeof(NoType)
	};

/*
	enum{
		value = 1,
		rawcall_overwrite = value,
		rawmember_overwrite = value,
		def_overwrite = value,
		object_parent_overwrite = value,
		set_object_parent_overwrite = value,
		visit_members_overwrite = value,
		finaize_overwrite = value
	};
*/	
};

template<class T>
struct VirtualMembersT{
	static const VirtualMembers value;

	typedef VirtualMembersOverwriteChecker<T> checker;

	typedef typename If<checker::rawcall_overwrite, VirtualMembersM<T>, VirtualMembersN>::type rawcall;
	typedef typename If<checker::rawmember_overwrite, VirtualMembersM<T>, VirtualMembersN>::type rawmember;
	typedef typename If<checker::def_overwrite, VirtualMembersM<T>, VirtualMembersN>::type def;
	typedef typename If<checker::object_parent_overwrite, VirtualMembersM<T>, VirtualMembersN>::type object_parent;
	typedef typename If<checker::set_object_parent_overwrite, VirtualMembersM<T>, VirtualMembersN>::type set_object_parent;
	typedef typename If<checker::visit_members_overwrite, VirtualMembersM<T>, VirtualMembersN>::type visit_members;
	typedef typename If<checker::finaize_overwrite, VirtualMembersM<T>, VirtualMembersN>::type finalize;
	typedef typename If<checker::shrink_to_fit_overwrite, VirtualMembersM<T>, VirtualMembersN>::type shrink_to_fit;
};

template<class T>
const VirtualMembers VirtualMembersT<T>::value = {
	&CppClassSymbol<T>::value,
	(u16)RCBaseClassType<InheritedN<T>::value, T>::value,
	(u16)AlignOf<T>::value,
	(u16)sizeof(T),
	(u16)checker::visit_members_overwrite,

	&VirtualMembersM<T>::object_free,
	&VirtualMembersM<T>::object_destroy,
	&visit_members::visit_members,
	
	&rawcall::rawcall,
	&rawmember::rawmember,
	&def::def,
	&object_parent::object_parent,
	&set_object_parent::set_object_parent,
	&finalize::finalize,
	&shrink_to_fit::shrink_to_fit,
};

/**
* \brief 参照カウンタの機能を有するクラス
*/
class RefCountingBase : public Any{
public:

	/**
	* \brief 関数オブジェクトとみなし、関数呼び出しをする。
	* 引数や戻り値はvmを通してやり取りする。
	*/
	void rawcall(const VMachinePtr& vm){
		virtual_members()->rawcall(this, vm);
	}

	/**
	* \brief nameメンバを初期値valueで定義する。
	*/
	void def(const IDPtr& primary_key, const AnyPtr& value, const AnyPtr& secondary_key, int_t accessibility){
		virtual_members()->def(this, primary_key, value, secondary_key, accessibility);
	}

	/**
	* \brief nameメンバを取得する。
	* \retval undefined そのメンバは存在しない
	* \retval 非undefined nameに対応したメンバ  
	*/
	const AnyPtr& rawmember(const IDPtr& primary_key, const AnyPtr& secondary_key, bool inherited_too, int_t& accessibility, bool& nocache){
		return virtual_members()->rawmember(this, primary_key, secondary_key, inherited_too, accessibility, nocache);
	}

	/**
	* \brief このオブジェクトがメンバとなっている親のクラスを返す。
	*/
	const ClassPtr& object_parent(){
		return virtual_members()->object_parent(this);
	}
	
	/**
	* \brief このオブジェクトに親を設定する。
	* 親を持てないオブジェクトや、前に付けられた親の方が強い場合無視される。
	* \param parent 親
	*/
	void set_object_parent(const ClassPtr& parent){
		virtual_members()->set_object_parent(this, parent);
	}

public:

	void finalize(){
		if(!object_destroyed()){
			++ref_count_;
			virtual_members()->finalize(this);
			--ref_count_;
		}
	}

	void visit_members(Visitor& m){
		if(!object_destroyed()){
			virtual_members()->visit_members(this, m);
		}
	}

	void shrink_to_fit(){
		virtual_members()->shrink_to_fit(this);
	}

public:

	void on_rawcall(const VMachinePtr& vm);
	
	void on_def(const IDPtr& /*primary_key*/, const AnyPtr& /*value*/, const AnyPtr& /*secondary_key*/, int_t /*accessibility*/){}
	
	const AnyPtr& on_rawmember(const IDPtr& /*primary_key*/, const AnyPtr& /*secondary_key*/, bool /*inherited_too*/, int_t& /*accessibility*/, bool& /*nocache*/);
	
	const ClassPtr& on_object_parent();
	
	void on_set_object_parent(const ClassPtr& /*parent*/){}
	
	void on_finalize(){}
	
	void on_visit_members(Visitor&){}

	void on_shrink_to_fit(){}

public:
	RefCountingBase(){ ref_count_ = 0; }

public:
	enum{
		DESTROYED_FLAG_SHIFT = USER_FLAG_SHIFT,
		DESTROYED_FLAG_BIT = 1<<DESTROYED_FLAG_SHIFT,

		HAVE_FINALIZER_FLAG_SHIFT = DESTROYED_FLAG_SHIFT+1,
		HAVE_FINALIZER_FLAG_BIT = 1<<HAVE_FINALIZER_FLAG_SHIFT,

		//REF_COUNT_SHIFT = HAVE_FINALIZER_FLAG_SHIFT+1,
		//REF_COUNT_MASK = ~((1<<REF_COUNT_SHIFT)-1),
		//REF_COUNT_NUMBER = 1<<REF_COUNT_SHIFT,
	};

	int_t ref_count(){ return ref_count_; }
	int_t alive_ref_count() const{ return ref_count_; }
	void add_ref_count(int_t n){ ref_count_ += n; }
	void inc_ref_count(){ ++ref_count_; }
	void dec_ref_count(){ if(XTAL_UNLIKELY(!--ref_count_)){ object_destroy(); } }
	//void dec_ref_count(){ --ref_count_; }

	void atomic_inc_ref_count(){ ++ref_count_; }
	int_t atomic_dec_ref_count(){ return --ref_count_; }

	uint_t have_finalizer() const{ return (XTAL_detail_user_flags(*this) & HAVE_FINALIZER_FLAG_BIT); }
	void set_finalizer_flag(){ XTAL_detail_user_flags(*this) |= HAVE_FINALIZER_FLAG_BIT; }
	void unset_finalizer_flag(){ XTAL_detail_user_flags(*this) &= ~HAVE_FINALIZER_FLAG_BIT; }

	uint_t object_destroyed() const{ return (XTAL_detail_user_flags(*this) & DESTROYED_FLAG_BIT); }
	void set_object_destroyed_flag(){ XTAL_detail_user_flags(*this) |= DESTROYED_FLAG_BIT; }

	void object_destroy();
	void object_free(){ virtual_members()->object_free(this); }

public:
	void special_initialize(const VirtualMembers* vmembers);

	const VirtualMembers* virtual_members() const{
		return vmembers_;
	}

protected:
	const VirtualMembers* vmembers_;
	uint_t ref_count_;
	
private:

#ifdef XTAL_DEBUG // VCなど、仮想関数を持つクラスはデバッグがしやすくなることがあるので
	//virtual void virtual_function(){}
#endif

private:
	XTAL_DISALLOW_COPY_AND_ASSIGN(RefCountingBase);

};

inline void XTAL_detail_inc_ref_count(const Any& v){ 
	if(XTAL_detail_is_rcpvalue(v)){
		XTAL_detail_rcpvalue(v)->inc_ref_count();
	}
}

inline void XTAL_detail_dec_ref_count(const Any& v){ 
	if(XTAL_detail_is_rcpvalue(v)){
		XTAL_detail_rcpvalue(v)->dec_ref_count();
	}
}

/**
* @brief 基底クラス
*/
class Base : public RefCountingBase{
public:

	/**
	* \brief このオブジェクトが所属するクラスを返す。
	*/
	const ClassPtr& get_class();

public:
	
	Base(){}
			
public:

	void special_initialize();
	void special_initialize(const VirtualMembers* vmembers);
	void special_uninitialize();

	InstanceVariables* instance_variables(){ return instance_variables_; }

	void init_instance_variables(ClassInfo* info);

	void set_class(const ClassPtr& c);

	void on_visit_members(Visitor& m);
	
private:

	// インスタンス変数テーブル
	InstanceVariables* instance_variables_;

	// 所属クラス
	Class* class_;

private:
	XTAL_DISALLOW_COPY_AND_ASSIGN(Base);
};

}


#endif // XTAL_BASE_H_INCLUDE_GUARD
