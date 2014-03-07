/** \file src/xtal/xtal_bind.h
* \brief src/xtal/xtal_bind.h
*/

#ifndef XTAL_BIND_H_INCLUDE_GUARD
#define XTAL_BIND_H_INCLUDE_GUARD

#pragma once

namespace xtal{

struct FunctorParam{
	Any result;
	const void* fun;
	VMachine* vm;
	
	enum{
		MAX_ARGS = 16
	};

	Any args[MAX_ARGS];
};

template<class R>
struct FunctorParamR : public FunctorParam{
	typedef const R& type;
	void return_result(type r){ vm->return_result(r); }
};

template<class R> 
struct FunctorParamR<R&> : public FunctorParam{
	typedef R& type;
	void return_result(type r){ vm->return_result(&r); }
};

template<class T> 
struct FunctorParamR<SmartPtr<T>&> : public FunctorParam{
	typedef SmartPtr<T>& type;
	void return_result(type r){ vm->return_result(r); }
};

template<class T> 
struct FunctorParamR<const SmartPtr<T>&> : public FunctorParam{
	typedef const SmartPtr<T>& type;
	void return_result(type r){ vm->return_result(r); }
};

template<> 
struct FunctorParamR<void> : public FunctorParam{
	typedef void**** type;
};

#define XTAL_FP_HELPER(Type, XType, InitFun) \
template<> struct FunctorParamR<Type> : public ::xtal::FunctorParam{\
	typedef Type type;\
	void return_result(type r){ result.value_.InitFun((XType::value_type)r); }\
};\
template<> struct FunctorParamR<Type&> : public ::xtal::FunctorParamR<Type>{};\
template<> struct FunctorParamR<const Type&> : public ::xtal::FunctorParamR<Type>{}

XTAL_FP_HELPER(bool, Bool, init_bool);

XTAL_FP_HELPER(char, Int, init_int);
XTAL_FP_HELPER(signed char, Int, init_int);
XTAL_FP_HELPER(unsigned char, Int, init_int);
XTAL_FP_HELPER(short, Int, init_int);
XTAL_FP_HELPER(unsigned short, Int, init_int);
XTAL_FP_HELPER(int, Int, init_int);
XTAL_FP_HELPER(unsigned int, Int, init_int);
XTAL_FP_HELPER(long, Int, init_int);
XTAL_FP_HELPER(unsigned long, Int, init_int);
XTAL_FP_HELPER(float, Float, init_float);
XTAL_FP_HELPER(double, Float, init_float);
XTAL_FP_HELPER(long double, Float, init_float);

//#undef XTAL_FP_HELPER

#define XTAL_BIND_ENUM(enum_type) \
	XTAL_CAST_HELPER(enum_type, Int, XTAL_detail_ivalue);\
	XTAL_FP_HELPER(enum_type, Int, init_int);\
	template<> struct ::xtal::CppClassSymbol<enum_type> : public ::xtal::CppClassSymbol<Int>{}


template<class R>
inline void operator, (FunctorParamR<R>& ret, typename FunctorParamR<R>::type val){
	ret.return_result(val);
}

template<class R>
struct RCast{
	typedef FunctorParamR<R>& type;
	static FunctorParamR<R>& rcast(FunctorParam& a){
		return (FunctorParamR<R>&)a;
	}
};

template<>
struct RCast<void>{
	typedef void type;
	static void rcast(FunctorParam&){}	
};

template<class R>
inline typename RCast<R>::type rcast(FunctorParam& a){
	return RCast<R>::rcast(a);
}

/////////////////////////////////////////

template<class C = Any >
struct param_types_holderM1;

//{REPEAT{{
/*
template<class C #COMMA_REPEAT#class A`i`#>
struct param_types_holder`n`{
	static CppClassSymbolData* const values[`n`+1];
};

template<class C #COMMA_REPEAT#class A`i`#>
CppClassSymbolData* const param_types_holder`n`<C #COMMA_REPEAT#A`i`#>::values[`n`+1] = {
	&CppClassSymbol<C>::value,
	#REPEAT#&CppClassSymbol<A`i`>::value,#
};
*/

template<class C >
struct param_types_holder0{
	static CppClassSymbolData* const values[0+1];
};

template<class C >
CppClassSymbolData* const param_types_holder0<C >::values[0+1] = {
	&CppClassSymbol<C>::value,
	
};

template<class C , class A0>
struct param_types_holder1{
	static CppClassSymbolData* const values[1+1];
};

template<class C , class A0>
CppClassSymbolData* const param_types_holder1<C , A0>::values[1+1] = {
	&CppClassSymbol<C>::value,
	&CppClassSymbol<A0>::value,
};

template<class C , class A0, class A1>
struct param_types_holder2{
	static CppClassSymbolData* const values[2+1];
};

template<class C , class A0, class A1>
CppClassSymbolData* const param_types_holder2<C , A0, A1>::values[2+1] = {
	&CppClassSymbol<C>::value,
	&CppClassSymbol<A0>::value,&CppClassSymbol<A1>::value,
};

template<class C , class A0, class A1, class A2>
struct param_types_holder3{
	static CppClassSymbolData* const values[3+1];
};

template<class C , class A0, class A1, class A2>
CppClassSymbolData* const param_types_holder3<C , A0, A1, A2>::values[3+1] = {
	&CppClassSymbol<C>::value,
	&CppClassSymbol<A0>::value,&CppClassSymbol<A1>::value,&CppClassSymbol<A2>::value,
};

template<class C , class A0, class A1, class A2, class A3>
struct param_types_holder4{
	static CppClassSymbolData* const values[4+1];
};

template<class C , class A0, class A1, class A2, class A3>
CppClassSymbolData* const param_types_holder4<C , A0, A1, A2, A3>::values[4+1] = {
	&CppClassSymbol<C>::value,
	&CppClassSymbol<A0>::value,&CppClassSymbol<A1>::value,&CppClassSymbol<A2>::value,&CppClassSymbol<A3>::value,
};

template<class C , class A0, class A1, class A2, class A3, class A4>
struct param_types_holder5{
	static CppClassSymbolData* const values[5+1];
};

template<class C , class A0, class A1, class A2, class A3, class A4>
CppClassSymbolData* const param_types_holder5<C , A0, A1, A2, A3, A4>::values[5+1] = {
	&CppClassSymbol<C>::value,
	&CppClassSymbol<A0>::value,&CppClassSymbol<A1>::value,&CppClassSymbol<A2>::value,&CppClassSymbol<A3>::value,&CppClassSymbol<A4>::value,
};

template<class C , class A0, class A1, class A2, class A3, class A4, class A5>
struct param_types_holder6{
	static CppClassSymbolData* const values[6+1];
};

template<class C , class A0, class A1, class A2, class A3, class A4, class A5>
CppClassSymbolData* const param_types_holder6<C , A0, A1, A2, A3, A4, A5>::values[6+1] = {
	&CppClassSymbol<C>::value,
	&CppClassSymbol<A0>::value,&CppClassSymbol<A1>::value,&CppClassSymbol<A2>::value,&CppClassSymbol<A3>::value,&CppClassSymbol<A4>::value,&CppClassSymbol<A5>::value,
};

template<class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6>
struct param_types_holder7{
	static CppClassSymbolData* const values[7+1];
};

template<class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6>
CppClassSymbolData* const param_types_holder7<C , A0, A1, A2, A3, A4, A5, A6>::values[7+1] = {
	&CppClassSymbol<C>::value,
	&CppClassSymbol<A0>::value,&CppClassSymbol<A1>::value,&CppClassSymbol<A2>::value,&CppClassSymbol<A3>::value,&CppClassSymbol<A4>::value,&CppClassSymbol<A5>::value,&CppClassSymbol<A6>::value,
};

template<class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7>
struct param_types_holder8{
	static CppClassSymbolData* const values[8+1];
};

template<class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7>
CppClassSymbolData* const param_types_holder8<C , A0, A1, A2, A3, A4, A5, A6, A7>::values[8+1] = {
	&CppClassSymbol<C>::value,
	&CppClassSymbol<A0>::value,&CppClassSymbol<A1>::value,&CppClassSymbol<A2>::value,&CppClassSymbol<A3>::value,&CppClassSymbol<A4>::value,&CppClassSymbol<A5>::value,&CppClassSymbol<A6>::value,&CppClassSymbol<A7>::value,
};

template<class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8>
struct param_types_holder9{
	static CppClassSymbolData* const values[9+1];
};

template<class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8>
CppClassSymbolData* const param_types_holder9<C , A0, A1, A2, A3, A4, A5, A6, A7, A8>::values[9+1] = {
	&CppClassSymbol<C>::value,
	&CppClassSymbol<A0>::value,&CppClassSymbol<A1>::value,&CppClassSymbol<A2>::value,&CppClassSymbol<A3>::value,&CppClassSymbol<A4>::value,&CppClassSymbol<A5>::value,&CppClassSymbol<A6>::value,&CppClassSymbol<A7>::value,&CppClassSymbol<A8>::value,
};

template<class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9>
struct param_types_holder10{
	static CppClassSymbolData* const values[10+1];
};

template<class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9>
CppClassSymbolData* const param_types_holder10<C , A0, A1, A2, A3, A4, A5, A6, A7, A8, A9>::values[10+1] = {
	&CppClassSymbol<C>::value,
	&CppClassSymbol<A0>::value,&CppClassSymbol<A1>::value,&CppClassSymbol<A2>::value,&CppClassSymbol<A3>::value,&CppClassSymbol<A4>::value,&CppClassSymbol<A5>::value,&CppClassSymbol<A6>::value,&CppClassSymbol<A7>::value,&CppClassSymbol<A8>::value,&CppClassSymbol<A9>::value,
};

template<class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10>
struct param_types_holder11{
	static CppClassSymbolData* const values[11+1];
};

template<class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10>
CppClassSymbolData* const param_types_holder11<C , A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10>::values[11+1] = {
	&CppClassSymbol<C>::value,
	&CppClassSymbol<A0>::value,&CppClassSymbol<A1>::value,&CppClassSymbol<A2>::value,&CppClassSymbol<A3>::value,&CppClassSymbol<A4>::value,&CppClassSymbol<A5>::value,&CppClassSymbol<A6>::value,&CppClassSymbol<A7>::value,&CppClassSymbol<A8>::value,&CppClassSymbol<A9>::value,&CppClassSymbol<A10>::value,
};

template<class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11>
struct param_types_holder12{
	static CppClassSymbolData* const values[12+1];
};

template<class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11>
CppClassSymbolData* const param_types_holder12<C , A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11>::values[12+1] = {
	&CppClassSymbol<C>::value,
	&CppClassSymbol<A0>::value,&CppClassSymbol<A1>::value,&CppClassSymbol<A2>::value,&CppClassSymbol<A3>::value,&CppClassSymbol<A4>::value,&CppClassSymbol<A5>::value,&CppClassSymbol<A6>::value,&CppClassSymbol<A7>::value,&CppClassSymbol<A8>::value,&CppClassSymbol<A9>::value,&CppClassSymbol<A10>::value,&CppClassSymbol<A11>::value,
};

template<class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, class A12>
struct param_types_holder13{
	static CppClassSymbolData* const values[13+1];
};

template<class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, class A12>
CppClassSymbolData* const param_types_holder13<C , A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12>::values[13+1] = {
	&CppClassSymbol<C>::value,
	&CppClassSymbol<A0>::value,&CppClassSymbol<A1>::value,&CppClassSymbol<A2>::value,&CppClassSymbol<A3>::value,&CppClassSymbol<A4>::value,&CppClassSymbol<A5>::value,&CppClassSymbol<A6>::value,&CppClassSymbol<A7>::value,&CppClassSymbol<A8>::value,&CppClassSymbol<A9>::value,&CppClassSymbol<A10>::value,&CppClassSymbol<A11>::value,&CppClassSymbol<A12>::value,
};

//}}REPEAT}

struct AM1{};

//{REPEAT{{
/*
template<class TFun, int Method, class R #COMMA_REPEAT#class A`i`#>
struct nfun_base`n`{
	typedef TFun fun_type;
	#REPEAT#typedef typename CppClassSymbol<A`i`>::type AA`i`; #
	typedef typename If<Method,
		param_types_holder`n-1`<#REPEAT_COMMA#AA`i`#>,
		param_types_holder`n`<Any #COMMA_REPEAT#AA`i`#>
		>::type fun_param_holder;
	enum{ extendable = (Method==1 && `n`==1) ? 0 : IsSame<Arguments, typename CppClassSymbol<A`n-1`>::type>::value };
	enum{ ArgBase = 1-Method, method = Method, arity = `n`-Method-extendable, vm = 0 };
	static void call(FunctorParam& fp){ 
		(*(fun_type*)fp.fun)(fp
			#COMMA_REPEAT#unchecked_cast<A`i`>((AnyPtr&)fp.args[ArgBase+`i`])#
		);
	}
};

template<class TFun, int Method, class R>
struct nfun_base<`n`, TFun, Method, R>
	: public nfun_base`n`<TFun, Method, R #COMMA_REPEAT#typename TFun::arg`i+1`_type#>{};

*/

template<class TFun, int Method, class R >
struct nfun_base0{
	typedef TFun fun_type;
	
	typedef typename If<Method,
		param_types_holderM1<>,
		param_types_holder0<Any >
		>::type fun_param_holder;
	enum{ extendable = (Method==1 && 0==1) ? 0 : IsSame<Arguments, typename CppClassSymbol<AM1>::type>::value };
	enum{ ArgBase = 1-Method, method = Method, arity = 0-Method-extendable, vm = 0 };
	static void call(FunctorParam& fp){ 
		(*(fun_type*)fp.fun)(fp
			
		);
	}
};

template<class TFun, int Method, class R>
struct nfun_base<0, TFun, Method, R>
	: public nfun_base0<TFun, Method, R >{};


template<class TFun, int Method, class R , class A0>
struct nfun_base1{
	typedef TFun fun_type;
	typedef typename CppClassSymbol<A0>::type AA0; 
	typedef typename If<Method,
		param_types_holder0<AA0>,
		param_types_holder1<Any , AA0>
		>::type fun_param_holder;
	enum{ extendable = (Method==1 && 1==1) ? 0 : IsSame<Arguments, typename CppClassSymbol<A0>::type>::value };
	enum{ ArgBase = 1-Method, method = Method, arity = 1-Method-extendable, vm = 0 };
	static void call(FunctorParam& fp){ 
		(*(fun_type*)fp.fun)(fp
			, unchecked_cast<A0>((AnyPtr&)fp.args[ArgBase+0])
		);
	}
};

template<class TFun, int Method, class R>
struct nfun_base<1, TFun, Method, R>
	: public nfun_base1<TFun, Method, R , typename TFun::arg1_type>{};


template<class TFun, int Method, class R , class A0, class A1>
struct nfun_base2{
	typedef TFun fun_type;
	typedef typename CppClassSymbol<A0>::type AA0; typedef typename CppClassSymbol<A1>::type AA1; 
	typedef typename If<Method,
		param_types_holder1<AA0, AA1>,
		param_types_holder2<Any , AA0, AA1>
		>::type fun_param_holder;
	enum{ extendable = (Method==1 && 2==1) ? 0 : IsSame<Arguments, typename CppClassSymbol<A1>::type>::value };
	enum{ ArgBase = 1-Method, method = Method, arity = 2-Method-extendable, vm = 0 };
	static void call(FunctorParam& fp){ 
		(*(fun_type*)fp.fun)(fp
			, unchecked_cast<A0>((AnyPtr&)fp.args[ArgBase+0]), unchecked_cast<A1>((AnyPtr&)fp.args[ArgBase+1])
		);
	}
};

template<class TFun, int Method, class R>
struct nfun_base<2, TFun, Method, R>
	: public nfun_base2<TFun, Method, R , typename TFun::arg1_type, typename TFun::arg2_type>{};


template<class TFun, int Method, class R , class A0, class A1, class A2>
struct nfun_base3{
	typedef TFun fun_type;
	typedef typename CppClassSymbol<A0>::type AA0; typedef typename CppClassSymbol<A1>::type AA1; typedef typename CppClassSymbol<A2>::type AA2; 
	typedef typename If<Method,
		param_types_holder2<AA0, AA1, AA2>,
		param_types_holder3<Any , AA0, AA1, AA2>
		>::type fun_param_holder;
	enum{ extendable = (Method==1 && 3==1) ? 0 : IsSame<Arguments, typename CppClassSymbol<A2>::type>::value };
	enum{ ArgBase = 1-Method, method = Method, arity = 3-Method-extendable, vm = 0 };
	static void call(FunctorParam& fp){ 
		(*(fun_type*)fp.fun)(fp
			, unchecked_cast<A0>((AnyPtr&)fp.args[ArgBase+0]), unchecked_cast<A1>((AnyPtr&)fp.args[ArgBase+1]), unchecked_cast<A2>((AnyPtr&)fp.args[ArgBase+2])
		);
	}
};

template<class TFun, int Method, class R>
struct nfun_base<3, TFun, Method, R>
	: public nfun_base3<TFun, Method, R , typename TFun::arg1_type, typename TFun::arg2_type, typename TFun::arg3_type>{};


template<class TFun, int Method, class R , class A0, class A1, class A2, class A3>
struct nfun_base4{
	typedef TFun fun_type;
	typedef typename CppClassSymbol<A0>::type AA0; typedef typename CppClassSymbol<A1>::type AA1; typedef typename CppClassSymbol<A2>::type AA2; typedef typename CppClassSymbol<A3>::type AA3; 
	typedef typename If<Method,
		param_types_holder3<AA0, AA1, AA2, AA3>,
		param_types_holder4<Any , AA0, AA1, AA2, AA3>
		>::type fun_param_holder;
	enum{ extendable = (Method==1 && 4==1) ? 0 : IsSame<Arguments, typename CppClassSymbol<A3>::type>::value };
	enum{ ArgBase = 1-Method, method = Method, arity = 4-Method-extendable, vm = 0 };
	static void call(FunctorParam& fp){ 
		(*(fun_type*)fp.fun)(fp
			, unchecked_cast<A0>((AnyPtr&)fp.args[ArgBase+0]), unchecked_cast<A1>((AnyPtr&)fp.args[ArgBase+1]), unchecked_cast<A2>((AnyPtr&)fp.args[ArgBase+2]), unchecked_cast<A3>((AnyPtr&)fp.args[ArgBase+3])
		);
	}
};

template<class TFun, int Method, class R>
struct nfun_base<4, TFun, Method, R>
	: public nfun_base4<TFun, Method, R , typename TFun::arg1_type, typename TFun::arg2_type, typename TFun::arg3_type, typename TFun::arg4_type>{};


template<class TFun, int Method, class R , class A0, class A1, class A2, class A3, class A4>
struct nfun_base5{
	typedef TFun fun_type;
	typedef typename CppClassSymbol<A0>::type AA0; typedef typename CppClassSymbol<A1>::type AA1; typedef typename CppClassSymbol<A2>::type AA2; typedef typename CppClassSymbol<A3>::type AA3; typedef typename CppClassSymbol<A4>::type AA4; 
	typedef typename If<Method,
		param_types_holder4<AA0, AA1, AA2, AA3, AA4>,
		param_types_holder5<Any , AA0, AA1, AA2, AA3, AA4>
		>::type fun_param_holder;
	enum{ extendable = (Method==1 && 5==1) ? 0 : IsSame<Arguments, typename CppClassSymbol<A4>::type>::value };
	enum{ ArgBase = 1-Method, method = Method, arity = 5-Method-extendable, vm = 0 };
	static void call(FunctorParam& fp){ 
		(*(fun_type*)fp.fun)(fp
			, unchecked_cast<A0>((AnyPtr&)fp.args[ArgBase+0]), unchecked_cast<A1>((AnyPtr&)fp.args[ArgBase+1]), unchecked_cast<A2>((AnyPtr&)fp.args[ArgBase+2]), unchecked_cast<A3>((AnyPtr&)fp.args[ArgBase+3]), unchecked_cast<A4>((AnyPtr&)fp.args[ArgBase+4])
		);
	}
};

template<class TFun, int Method, class R>
struct nfun_base<5, TFun, Method, R>
	: public nfun_base5<TFun, Method, R , typename TFun::arg1_type, typename TFun::arg2_type, typename TFun::arg3_type, typename TFun::arg4_type, typename TFun::arg5_type>{};


template<class TFun, int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5>
struct nfun_base6{
	typedef TFun fun_type;
	typedef typename CppClassSymbol<A0>::type AA0; typedef typename CppClassSymbol<A1>::type AA1; typedef typename CppClassSymbol<A2>::type AA2; typedef typename CppClassSymbol<A3>::type AA3; typedef typename CppClassSymbol<A4>::type AA4; typedef typename CppClassSymbol<A5>::type AA5; 
	typedef typename If<Method,
		param_types_holder5<AA0, AA1, AA2, AA3, AA4, AA5>,
		param_types_holder6<Any , AA0, AA1, AA2, AA3, AA4, AA5>
		>::type fun_param_holder;
	enum{ extendable = (Method==1 && 6==1) ? 0 : IsSame<Arguments, typename CppClassSymbol<A5>::type>::value };
	enum{ ArgBase = 1-Method, method = Method, arity = 6-Method-extendable, vm = 0 };
	static void call(FunctorParam& fp){ 
		(*(fun_type*)fp.fun)(fp
			, unchecked_cast<A0>((AnyPtr&)fp.args[ArgBase+0]), unchecked_cast<A1>((AnyPtr&)fp.args[ArgBase+1]), unchecked_cast<A2>((AnyPtr&)fp.args[ArgBase+2]), unchecked_cast<A3>((AnyPtr&)fp.args[ArgBase+3]), unchecked_cast<A4>((AnyPtr&)fp.args[ArgBase+4]), unchecked_cast<A5>((AnyPtr&)fp.args[ArgBase+5])
		);
	}
};

template<class TFun, int Method, class R>
struct nfun_base<6, TFun, Method, R>
	: public nfun_base6<TFun, Method, R , typename TFun::arg1_type, typename TFun::arg2_type, typename TFun::arg3_type, typename TFun::arg4_type, typename TFun::arg5_type, typename TFun::arg6_type>{};


template<class TFun, int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6>
struct nfun_base7{
	typedef TFun fun_type;
	typedef typename CppClassSymbol<A0>::type AA0; typedef typename CppClassSymbol<A1>::type AA1; typedef typename CppClassSymbol<A2>::type AA2; typedef typename CppClassSymbol<A3>::type AA3; typedef typename CppClassSymbol<A4>::type AA4; typedef typename CppClassSymbol<A5>::type AA5; typedef typename CppClassSymbol<A6>::type AA6; 
	typedef typename If<Method,
		param_types_holder6<AA0, AA1, AA2, AA3, AA4, AA5, AA6>,
		param_types_holder7<Any , AA0, AA1, AA2, AA3, AA4, AA5, AA6>
		>::type fun_param_holder;
	enum{ extendable = (Method==1 && 7==1) ? 0 : IsSame<Arguments, typename CppClassSymbol<A6>::type>::value };
	enum{ ArgBase = 1-Method, method = Method, arity = 7-Method-extendable, vm = 0 };
	static void call(FunctorParam& fp){ 
		(*(fun_type*)fp.fun)(fp
			, unchecked_cast<A0>((AnyPtr&)fp.args[ArgBase+0]), unchecked_cast<A1>((AnyPtr&)fp.args[ArgBase+1]), unchecked_cast<A2>((AnyPtr&)fp.args[ArgBase+2]), unchecked_cast<A3>((AnyPtr&)fp.args[ArgBase+3]), unchecked_cast<A4>((AnyPtr&)fp.args[ArgBase+4]), unchecked_cast<A5>((AnyPtr&)fp.args[ArgBase+5]), unchecked_cast<A6>((AnyPtr&)fp.args[ArgBase+6])
		);
	}
};

template<class TFun, int Method, class R>
struct nfun_base<7, TFun, Method, R>
	: public nfun_base7<TFun, Method, R , typename TFun::arg1_type, typename TFun::arg2_type, typename TFun::arg3_type, typename TFun::arg4_type, typename TFun::arg5_type, typename TFun::arg6_type, typename TFun::arg7_type>{};


template<class TFun, int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7>
struct nfun_base8{
	typedef TFun fun_type;
	typedef typename CppClassSymbol<A0>::type AA0; typedef typename CppClassSymbol<A1>::type AA1; typedef typename CppClassSymbol<A2>::type AA2; typedef typename CppClassSymbol<A3>::type AA3; typedef typename CppClassSymbol<A4>::type AA4; typedef typename CppClassSymbol<A5>::type AA5; typedef typename CppClassSymbol<A6>::type AA6; typedef typename CppClassSymbol<A7>::type AA7; 
	typedef typename If<Method,
		param_types_holder7<AA0, AA1, AA2, AA3, AA4, AA5, AA6, AA7>,
		param_types_holder8<Any , AA0, AA1, AA2, AA3, AA4, AA5, AA6, AA7>
		>::type fun_param_holder;
	enum{ extendable = (Method==1 && 8==1) ? 0 : IsSame<Arguments, typename CppClassSymbol<A7>::type>::value };
	enum{ ArgBase = 1-Method, method = Method, arity = 8-Method-extendable, vm = 0 };
	static void call(FunctorParam& fp){ 
		(*(fun_type*)fp.fun)(fp
			, unchecked_cast<A0>((AnyPtr&)fp.args[ArgBase+0]), unchecked_cast<A1>((AnyPtr&)fp.args[ArgBase+1]), unchecked_cast<A2>((AnyPtr&)fp.args[ArgBase+2]), unchecked_cast<A3>((AnyPtr&)fp.args[ArgBase+3]), unchecked_cast<A4>((AnyPtr&)fp.args[ArgBase+4]), unchecked_cast<A5>((AnyPtr&)fp.args[ArgBase+5]), unchecked_cast<A6>((AnyPtr&)fp.args[ArgBase+6]), unchecked_cast<A7>((AnyPtr&)fp.args[ArgBase+7])
		);
	}
};

template<class TFun, int Method, class R>
struct nfun_base<8, TFun, Method, R>
	: public nfun_base8<TFun, Method, R , typename TFun::arg1_type, typename TFun::arg2_type, typename TFun::arg3_type, typename TFun::arg4_type, typename TFun::arg5_type, typename TFun::arg6_type, typename TFun::arg7_type, typename TFun::arg8_type>{};


template<class TFun, int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8>
struct nfun_base9{
	typedef TFun fun_type;
	typedef typename CppClassSymbol<A0>::type AA0; typedef typename CppClassSymbol<A1>::type AA1; typedef typename CppClassSymbol<A2>::type AA2; typedef typename CppClassSymbol<A3>::type AA3; typedef typename CppClassSymbol<A4>::type AA4; typedef typename CppClassSymbol<A5>::type AA5; typedef typename CppClassSymbol<A6>::type AA6; typedef typename CppClassSymbol<A7>::type AA7; typedef typename CppClassSymbol<A8>::type AA8; 
	typedef typename If<Method,
		param_types_holder8<AA0, AA1, AA2, AA3, AA4, AA5, AA6, AA7, AA8>,
		param_types_holder9<Any , AA0, AA1, AA2, AA3, AA4, AA5, AA6, AA7, AA8>
		>::type fun_param_holder;
	enum{ extendable = (Method==1 && 9==1) ? 0 : IsSame<Arguments, typename CppClassSymbol<A8>::type>::value };
	enum{ ArgBase = 1-Method, method = Method, arity = 9-Method-extendable, vm = 0 };
	static void call(FunctorParam& fp){ 
		(*(fun_type*)fp.fun)(fp
			, unchecked_cast<A0>((AnyPtr&)fp.args[ArgBase+0]), unchecked_cast<A1>((AnyPtr&)fp.args[ArgBase+1]), unchecked_cast<A2>((AnyPtr&)fp.args[ArgBase+2]), unchecked_cast<A3>((AnyPtr&)fp.args[ArgBase+3]), unchecked_cast<A4>((AnyPtr&)fp.args[ArgBase+4]), unchecked_cast<A5>((AnyPtr&)fp.args[ArgBase+5]), unchecked_cast<A6>((AnyPtr&)fp.args[ArgBase+6]), unchecked_cast<A7>((AnyPtr&)fp.args[ArgBase+7]), unchecked_cast<A8>((AnyPtr&)fp.args[ArgBase+8])
		);
	}
};

template<class TFun, int Method, class R>
struct nfun_base<9, TFun, Method, R>
	: public nfun_base9<TFun, Method, R , typename TFun::arg1_type, typename TFun::arg2_type, typename TFun::arg3_type, typename TFun::arg4_type, typename TFun::arg5_type, typename TFun::arg6_type, typename TFun::arg7_type, typename TFun::arg8_type, typename TFun::arg9_type>{};


template<class TFun, int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9>
struct nfun_base10{
	typedef TFun fun_type;
	typedef typename CppClassSymbol<A0>::type AA0; typedef typename CppClassSymbol<A1>::type AA1; typedef typename CppClassSymbol<A2>::type AA2; typedef typename CppClassSymbol<A3>::type AA3; typedef typename CppClassSymbol<A4>::type AA4; typedef typename CppClassSymbol<A5>::type AA5; typedef typename CppClassSymbol<A6>::type AA6; typedef typename CppClassSymbol<A7>::type AA7; typedef typename CppClassSymbol<A8>::type AA8; typedef typename CppClassSymbol<A9>::type AA9; 
	typedef typename If<Method,
		param_types_holder9<AA0, AA1, AA2, AA3, AA4, AA5, AA6, AA7, AA8, AA9>,
		param_types_holder10<Any , AA0, AA1, AA2, AA3, AA4, AA5, AA6, AA7, AA8, AA9>
		>::type fun_param_holder;
	enum{ extendable = (Method==1 && 10==1) ? 0 : IsSame<Arguments, typename CppClassSymbol<A9>::type>::value };
	enum{ ArgBase = 1-Method, method = Method, arity = 10-Method-extendable, vm = 0 };
	static void call(FunctorParam& fp){ 
		(*(fun_type*)fp.fun)(fp
			, unchecked_cast<A0>((AnyPtr&)fp.args[ArgBase+0]), unchecked_cast<A1>((AnyPtr&)fp.args[ArgBase+1]), unchecked_cast<A2>((AnyPtr&)fp.args[ArgBase+2]), unchecked_cast<A3>((AnyPtr&)fp.args[ArgBase+3]), unchecked_cast<A4>((AnyPtr&)fp.args[ArgBase+4]), unchecked_cast<A5>((AnyPtr&)fp.args[ArgBase+5]), unchecked_cast<A6>((AnyPtr&)fp.args[ArgBase+6]), unchecked_cast<A7>((AnyPtr&)fp.args[ArgBase+7]), unchecked_cast<A8>((AnyPtr&)fp.args[ArgBase+8]), unchecked_cast<A9>((AnyPtr&)fp.args[ArgBase+9])
		);
	}
};

template<class TFun, int Method, class R>
struct nfun_base<10, TFun, Method, R>
	: public nfun_base10<TFun, Method, R , typename TFun::arg1_type, typename TFun::arg2_type, typename TFun::arg3_type, typename TFun::arg4_type, typename TFun::arg5_type, typename TFun::arg6_type, typename TFun::arg7_type, typename TFun::arg8_type, typename TFun::arg9_type, typename TFun::arg10_type>{};


template<class TFun, int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10>
struct nfun_base11{
	typedef TFun fun_type;
	typedef typename CppClassSymbol<A0>::type AA0; typedef typename CppClassSymbol<A1>::type AA1; typedef typename CppClassSymbol<A2>::type AA2; typedef typename CppClassSymbol<A3>::type AA3; typedef typename CppClassSymbol<A4>::type AA4; typedef typename CppClassSymbol<A5>::type AA5; typedef typename CppClassSymbol<A6>::type AA6; typedef typename CppClassSymbol<A7>::type AA7; typedef typename CppClassSymbol<A8>::type AA8; typedef typename CppClassSymbol<A9>::type AA9; typedef typename CppClassSymbol<A10>::type AA10; 
	typedef typename If<Method,
		param_types_holder10<AA0, AA1, AA2, AA3, AA4, AA5, AA6, AA7, AA8, AA9, AA10>,
		param_types_holder11<Any , AA0, AA1, AA2, AA3, AA4, AA5, AA6, AA7, AA8, AA9, AA10>
		>::type fun_param_holder;
	enum{ extendable = (Method==1 && 11==1) ? 0 : IsSame<Arguments, typename CppClassSymbol<A10>::type>::value };
	enum{ ArgBase = 1-Method, method = Method, arity = 11-Method-extendable, vm = 0 };
	static void call(FunctorParam& fp){ 
		(*(fun_type*)fp.fun)(fp
			, unchecked_cast<A0>((AnyPtr&)fp.args[ArgBase+0]), unchecked_cast<A1>((AnyPtr&)fp.args[ArgBase+1]), unchecked_cast<A2>((AnyPtr&)fp.args[ArgBase+2]), unchecked_cast<A3>((AnyPtr&)fp.args[ArgBase+3]), unchecked_cast<A4>((AnyPtr&)fp.args[ArgBase+4]), unchecked_cast<A5>((AnyPtr&)fp.args[ArgBase+5]), unchecked_cast<A6>((AnyPtr&)fp.args[ArgBase+6]), unchecked_cast<A7>((AnyPtr&)fp.args[ArgBase+7]), unchecked_cast<A8>((AnyPtr&)fp.args[ArgBase+8]), unchecked_cast<A9>((AnyPtr&)fp.args[ArgBase+9]), unchecked_cast<A10>((AnyPtr&)fp.args[ArgBase+10])
		);
	}
};

template<class TFun, int Method, class R>
struct nfun_base<11, TFun, Method, R>
	: public nfun_base11<TFun, Method, R , typename TFun::arg1_type, typename TFun::arg2_type, typename TFun::arg3_type, typename TFun::arg4_type, typename TFun::arg5_type, typename TFun::arg6_type, typename TFun::arg7_type, typename TFun::arg8_type, typename TFun::arg9_type, typename TFun::arg10_type, typename TFun::arg11_type>{};


template<class TFun, int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11>
struct nfun_base12{
	typedef TFun fun_type;
	typedef typename CppClassSymbol<A0>::type AA0; typedef typename CppClassSymbol<A1>::type AA1; typedef typename CppClassSymbol<A2>::type AA2; typedef typename CppClassSymbol<A3>::type AA3; typedef typename CppClassSymbol<A4>::type AA4; typedef typename CppClassSymbol<A5>::type AA5; typedef typename CppClassSymbol<A6>::type AA6; typedef typename CppClassSymbol<A7>::type AA7; typedef typename CppClassSymbol<A8>::type AA8; typedef typename CppClassSymbol<A9>::type AA9; typedef typename CppClassSymbol<A10>::type AA10; typedef typename CppClassSymbol<A11>::type AA11; 
	typedef typename If<Method,
		param_types_holder11<AA0, AA1, AA2, AA3, AA4, AA5, AA6, AA7, AA8, AA9, AA10, AA11>,
		param_types_holder12<Any , AA0, AA1, AA2, AA3, AA4, AA5, AA6, AA7, AA8, AA9, AA10, AA11>
		>::type fun_param_holder;
	enum{ extendable = (Method==1 && 12==1) ? 0 : IsSame<Arguments, typename CppClassSymbol<A11>::type>::value };
	enum{ ArgBase = 1-Method, method = Method, arity = 12-Method-extendable, vm = 0 };
	static void call(FunctorParam& fp){ 
		(*(fun_type*)fp.fun)(fp
			, unchecked_cast<A0>((AnyPtr&)fp.args[ArgBase+0]), unchecked_cast<A1>((AnyPtr&)fp.args[ArgBase+1]), unchecked_cast<A2>((AnyPtr&)fp.args[ArgBase+2]), unchecked_cast<A3>((AnyPtr&)fp.args[ArgBase+3]), unchecked_cast<A4>((AnyPtr&)fp.args[ArgBase+4]), unchecked_cast<A5>((AnyPtr&)fp.args[ArgBase+5]), unchecked_cast<A6>((AnyPtr&)fp.args[ArgBase+6]), unchecked_cast<A7>((AnyPtr&)fp.args[ArgBase+7]), unchecked_cast<A8>((AnyPtr&)fp.args[ArgBase+8]), unchecked_cast<A9>((AnyPtr&)fp.args[ArgBase+9]), unchecked_cast<A10>((AnyPtr&)fp.args[ArgBase+10]), unchecked_cast<A11>((AnyPtr&)fp.args[ArgBase+11])
		);
	}
};

template<class TFun, int Method, class R>
struct nfun_base<12, TFun, Method, R>
	: public nfun_base12<TFun, Method, R , typename TFun::arg1_type, typename TFun::arg2_type, typename TFun::arg3_type, typename TFun::arg4_type, typename TFun::arg5_type, typename TFun::arg6_type, typename TFun::arg7_type, typename TFun::arg8_type, typename TFun::arg9_type, typename TFun::arg10_type, typename TFun::arg11_type, typename TFun::arg12_type>{};


template<class TFun, int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, class A12>
struct nfun_base13{
	typedef TFun fun_type;
	typedef typename CppClassSymbol<A0>::type AA0; typedef typename CppClassSymbol<A1>::type AA1; typedef typename CppClassSymbol<A2>::type AA2; typedef typename CppClassSymbol<A3>::type AA3; typedef typename CppClassSymbol<A4>::type AA4; typedef typename CppClassSymbol<A5>::type AA5; typedef typename CppClassSymbol<A6>::type AA6; typedef typename CppClassSymbol<A7>::type AA7; typedef typename CppClassSymbol<A8>::type AA8; typedef typename CppClassSymbol<A9>::type AA9; typedef typename CppClassSymbol<A10>::type AA10; typedef typename CppClassSymbol<A11>::type AA11; typedef typename CppClassSymbol<A12>::type AA12; 
	typedef typename If<Method,
		param_types_holder12<AA0, AA1, AA2, AA3, AA4, AA5, AA6, AA7, AA8, AA9, AA10, AA11, AA12>,
		param_types_holder13<Any , AA0, AA1, AA2, AA3, AA4, AA5, AA6, AA7, AA8, AA9, AA10, AA11, AA12>
		>::type fun_param_holder;
	enum{ extendable = (Method==1 && 13==1) ? 0 : IsSame<Arguments, typename CppClassSymbol<A12>::type>::value };
	enum{ ArgBase = 1-Method, method = Method, arity = 13-Method-extendable, vm = 0 };
	static void call(FunctorParam& fp){ 
		(*(fun_type*)fp.fun)(fp
			, unchecked_cast<A0>((AnyPtr&)fp.args[ArgBase+0]), unchecked_cast<A1>((AnyPtr&)fp.args[ArgBase+1]), unchecked_cast<A2>((AnyPtr&)fp.args[ArgBase+2]), unchecked_cast<A3>((AnyPtr&)fp.args[ArgBase+3]), unchecked_cast<A4>((AnyPtr&)fp.args[ArgBase+4]), unchecked_cast<A5>((AnyPtr&)fp.args[ArgBase+5]), unchecked_cast<A6>((AnyPtr&)fp.args[ArgBase+6]), unchecked_cast<A7>((AnyPtr&)fp.args[ArgBase+7]), unchecked_cast<A8>((AnyPtr&)fp.args[ArgBase+8]), unchecked_cast<A9>((AnyPtr&)fp.args[ArgBase+9]), unchecked_cast<A10>((AnyPtr&)fp.args[ArgBase+10]), unchecked_cast<A11>((AnyPtr&)fp.args[ArgBase+11]), unchecked_cast<A12>((AnyPtr&)fp.args[ArgBase+12])
		);
	}
};

template<class TFun, int Method, class R>
struct nfun_base<13, TFun, Method, R>
	: public nfun_base13<TFun, Method, R , typename TFun::arg1_type, typename TFun::arg2_type, typename TFun::arg3_type, typename TFun::arg4_type, typename TFun::arg5_type, typename TFun::arg6_type, typename TFun::arg7_type, typename TFun::arg8_type, typename TFun::arg9_type, typename TFun::arg10_type, typename TFun::arg11_type, typename TFun::arg12_type, typename TFun::arg13_type>{};


//}}REPEAT}

template<class TFun, class R>
struct nfun_base1<TFun, 0, R, const VMachinePtr&>{
	typedef TFun fun_type;
	typedef param_types_holder1<Any, Any> fun_param_holder;
	enum{ method = 0, arity = 1, extendable = 0, vm = 1 };
	static void call(FunctorParam& fp){ 
		(*(TFun*)fp.fun)(fp,
			to_smartptr(fp.vm)
		);
	}
};

template<class TFun, class R>
struct nfun_base1<TFun, 1, R, const VMachinePtr&>{
	typedef TFun fun_type;
	typedef param_types_holder1<Any, Any> fun_param_holder;
	enum{ method = 1, arity = 1, extendable = 0, vm = 1 };
	static void call(FunctorParam& fp){ 
		(*(TFun*)fp.fun)(fp,
			to_smartptr(fp.vm)
		);
	}
};

template<class TFun, class R, class C>
struct nfun_base2<TFun, 1, R , C, const VMachinePtr&>{
	typedef TFun fun_type;
	typedef param_types_holder1<C, Any> fun_param_holder;
	enum{ method = 1, arity = 1, extendable = 0, vm = 1 };
	static void call(FunctorParam& fp){ 
		(*(TFun*)fp.fun)(fp,
			unchecked_cast<C>((AnyPtr&)fp.args[0]),
			to_smartptr(fp.vm)
		);
	}
};

//{REPEAT{{
/*
template<int Method, class R #COMMA_REPEAT#class A`i`#>
struct nfun<R (*)(#REPEAT_COMMA#A`i`#), Method>
	: public nfun_base<`n`, functor<R (*)(#REPEAT_COMMA#A`i`#)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R #COMMA_REPEAT#class A`i`#>
struct functor<R (*)(#REPEAT_COMMA#A`i`#)>{
	typedef R result_type; #REPEAT#typedef A`i` arg`i+1`_type; #
	typedef R (*fun_type)(#REPEAT_COMMA#A`i`#);
	enum{ arity = `n`};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret #COMMA_REPEAT#A`i` a`i`#) const{
		rcast<R>(ret), fun_(#REPEAT_COMMA#a`i`#);
	}
};

template<int Method, class R, class C #COMMA_REPEAT#class A`i`#>
struct nfun<R (C::*)(#REPEAT_COMMA#A`i`#), Method>
	: public nfun_base<`n+1`, functor<R (C::*)(#REPEAT_COMMA#A`i`#)>, Method, R>{};

// メンバ関数を関数オブジェクト化する
template<class R, class C #COMMA_REPEAT#class A`i`#>
struct functor<R (C::*)(#REPEAT_COMMA#A`i`#)>{
	typedef R result_type; typedef C* arg1_type; #REPEAT#typedef A`i` arg`i+2`_type; #
	typedef R (C::*fun_type)(#REPEAT_COMMA#A`i`#);
	enum{ arity = `n+1`};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self #COMMA_REPEAT#A`i` a`i`#) const{
		rcast<R>(ret), (self->*fun_)(#REPEAT_COMMA#a`i`#);
	}
};

template<int Method, class R, class C #COMMA_REPEAT#class A`i`#>
struct nfun<R (C::*)(#REPEAT_COMMA#A`i`#) const, Method>
	: public nfun_base<`n+1`, functor<R (C::*)(#REPEAT_COMMA#A`i`#) const>, Method, R>{};

// constメンバ関数を関数オブジェクト化する
template<class R, class C #COMMA_REPEAT#class A`i`#>
struct functor<R (C::*)(#REPEAT_COMMA#A`i`#) const>{
	typedef R result_type; typedef C* arg1_type; #REPEAT#typedef A`i` arg`i+2`_type; #
	typedef R (C::*fun_type)(#REPEAT_COMMA#A`i`#) const;
	enum{ arity = `n+1`};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self #COMMA_REPEAT#A`i` a`i`#) const{
		rcast<R>(ret), (self->*fun_)(#REPEAT_COMMA#a`i`#);
	}
};

////

// 関数を関数オブジェクト化する
template<class R #COMMA_REPEAT#class A`i`#, R (*fun_)(#REPEAT_COMMA#A`i`#)>
struct static_functor<R (*)(#REPEAT_COMMA#A`i`#), fun_>{
	typedef R result_type; #REPEAT#typedef A`i` arg`i+1`_type; #
	enum{ arity = `n`};

	void operator()(FunctorParam& ret #COMMA_REPEAT#A`i` a`i`#) const{
		rcast<R>(ret), fun_(#REPEAT_COMMA#a`i`#);
	}
};

// メンバ関数を関数オブジェクト化する
template<class R, class C #COMMA_REPEAT#class A`i`#, R (C::*fun_)(#REPEAT_COMMA#A`i`#)>
struct static_functor<R (C::*)(#REPEAT_COMMA#A`i`#), fun_>{
	typedef R result_type; typedef C* arg1_type; #REPEAT#typedef A`i` arg`i+2`_type; #
	enum{ arity = `n+1`};

	void operator()(FunctorParam& ret, C* self #COMMA_REPEAT#A`i` a`i`#) const{
		rcast<R>(ret), (self->*fun_)(#REPEAT_COMMA#a`i`#);
	}
};

// constメンバ関数を関数オブジェクト化する
template<class R, class C #COMMA_REPEAT#class A`i`#, R (C::*fun_)(#REPEAT_COMMA#A`i`#) const>
struct static_functor<R (C::*)(#REPEAT_COMMA#A`i`#) const, fun_>{
	typedef R result_type; typedef C* arg1_type; #REPEAT#typedef A`i` arg`i+2`_type; #
	enum{ arity = `n+1`};

	void operator()(FunctorParam& ret, C* self #COMMA_REPEAT#A`i` a`i`#) const{
		rcast<R>(ret), (self->*fun_)(#REPEAT_COMMA#a`i`#);
	}
};

// コンストラクタを関数オブジェクト化する
template<class T #COMMA_REPEAT#class A`i`#>
struct ctor_functor<T #COMMA_REPEAT#A`i`#>{
	typedef SmartPtr<T> R;
	typedef R result_type; #REPEAT#typedef A`i` arg`i+1`_type; #
	enum{ arity = `n`};

	void operator()(FunctorParam& ret #COMMA_REPEAT#A`i` a`i`#) const{
		rcast<R>(ret), xnew<T>(#REPEAT_COMMA#a`i`#);
	}
};

// stdcall版
#if defined(_WIN32) && defined(_M_IX86)

template<int Method, class R #COMMA_REPEAT#class A`i`#>
struct nfun<R (__stdcall *)(#REPEAT_COMMA#A`i`#), Method>
	: public nfun_base<`n`, functor<R (__stdcall *)(#REPEAT_COMMA#A`i`#)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R #COMMA_REPEAT#class A`i`#>
struct functor<R (__stdcall *)(#REPEAT_COMMA#A`i`#)>{
	typedef R result_type; #REPEAT#typedef A`i` arg`i+1`_type; #
	typedef R (__stdcall *fun_type)(#REPEAT_COMMA#A`i`#);
	enum{ arity = `n`};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret #COMMA_REPEAT#A`i` a`i`#) const{
		rcast<R>(ret), fun_(#REPEAT_COMMA#a`i`#);
	}
};

// 関数を関数オブジェクト化する
template<class R #COMMA_REPEAT#class A`i`#, R (__stdcall *fun_)(#REPEAT_COMMA#A`i`#)>
struct static_functor<R (__stdcall *)(#REPEAT_COMMA#A`i`#), fun_>{
	typedef R result_type; #REPEAT#typedef A`i` arg`i+1`_type; #
	enum{ arity = `n`};

	void operator()(FunctorParam& ret #COMMA_REPEAT#A`i` a`i`#) const{
		rcast<R>(ret), fun_(#REPEAT_COMMA#a`i`#);
	}
};

#endif

*/

template<int Method, class R >
struct nfun<R (*)(), Method>
	: public nfun_base<0, functor<R (*)()>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R >
struct functor<R (*)()>{
	typedef R result_type; 
	typedef R (*fun_type)();
	enum{ arity = 0};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret ) const{
		rcast<R>(ret), fun_();
	}
};

template<int Method, class R, class C >
struct nfun<R (C::*)(), Method>
	: public nfun_base<1, functor<R (C::*)()>, Method, R>{};

// メンバ関数を関数オブジェクト化する
template<class R, class C >
struct functor<R (C::*)()>{
	typedef R result_type; typedef C* arg1_type; 
	typedef R (C::*fun_type)();
	enum{ arity = 1};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self ) const{
		rcast<R>(ret), (self->*fun_)();
	}
};

template<int Method, class R, class C >
struct nfun<R (C::*)() const, Method>
	: public nfun_base<1, functor<R (C::*)() const>, Method, R>{};

// constメンバ関数を関数オブジェクト化する
template<class R, class C >
struct functor<R (C::*)() const>{
	typedef R result_type; typedef C* arg1_type; 
	typedef R (C::*fun_type)() const;
	enum{ arity = 1};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self ) const{
		rcast<R>(ret), (self->*fun_)();
	}
};

////

// 関数を関数オブジェクト化する
template<class R , R (*fun_)()>
struct static_functor<R (*)(), fun_>{
	typedef R result_type; 
	enum{ arity = 0};

	void operator()(FunctorParam& ret ) const{
		rcast<R>(ret), fun_();
	}
};

// メンバ関数を関数オブジェクト化する
template<class R, class C , R (C::*fun_)()>
struct static_functor<R (C::*)(), fun_>{
	typedef R result_type; typedef C* arg1_type; 
	enum{ arity = 1};

	void operator()(FunctorParam& ret, C* self ) const{
		rcast<R>(ret), (self->*fun_)();
	}
};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , R (C::*fun_)() const>
struct static_functor<R (C::*)() const, fun_>{
	typedef R result_type; typedef C* arg1_type; 
	enum{ arity = 1};

	void operator()(FunctorParam& ret, C* self ) const{
		rcast<R>(ret), (self->*fun_)();
	}
};

// コンストラクタを関数オブジェクト化する
template<class T >
struct ctor_functor<T >{
	typedef SmartPtr<T> R;
	typedef R result_type; 
	enum{ arity = 0};

	void operator()(FunctorParam& ret ) const{
		rcast<R>(ret), xnew<T>();
	}
};

// stdcall版
#if defined(_WIN32) && defined(_M_IX86)

template<int Method, class R >
struct nfun<R (__stdcall *)(), Method>
	: public nfun_base<0, functor<R (__stdcall *)()>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R >
struct functor<R (__stdcall *)()>{
	typedef R result_type; 
	typedef R (__stdcall *fun_type)();
	enum{ arity = 0};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret ) const{
		rcast<R>(ret), fun_();
	}
};

// 関数を関数オブジェクト化する
template<class R , R (__stdcall *fun_)()>
struct static_functor<R (__stdcall *)(), fun_>{
	typedef R result_type; 
	enum{ arity = 0};

	void operator()(FunctorParam& ret ) const{
		rcast<R>(ret), fun_();
	}
};

#endif


template<int Method, class R , class A0>
struct nfun<R (*)(A0), Method>
	: public nfun_base<1, functor<R (*)(A0)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0>
struct functor<R (*)(A0)>{
	typedef R result_type; typedef A0 arg1_type; 
	typedef R (*fun_type)(A0);
	enum{ arity = 1};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0) const{
		rcast<R>(ret), fun_(a0);
	}
};

template<int Method, class R, class C , class A0>
struct nfun<R (C::*)(A0), Method>
	: public nfun_base<2, functor<R (C::*)(A0)>, Method, R>{};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0>
struct functor<R (C::*)(A0)>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; 
	typedef R (C::*fun_type)(A0);
	enum{ arity = 2};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0) const{
		rcast<R>(ret), (self->*fun_)(a0);
	}
};

template<int Method, class R, class C , class A0>
struct nfun<R (C::*)(A0) const, Method>
	: public nfun_base<2, functor<R (C::*)(A0) const>, Method, R>{};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0>
struct functor<R (C::*)(A0) const>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; 
	typedef R (C::*fun_type)(A0) const;
	enum{ arity = 2};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0) const{
		rcast<R>(ret), (self->*fun_)(a0);
	}
};

////

// 関数を関数オブジェクト化する
template<class R , class A0, R (*fun_)(A0)>
struct static_functor<R (*)(A0), fun_>{
	typedef R result_type; typedef A0 arg1_type; 
	enum{ arity = 1};

	void operator()(FunctorParam& ret , A0 a0) const{
		rcast<R>(ret), fun_(a0);
	}
};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, R (C::*fun_)(A0)>
struct static_functor<R (C::*)(A0), fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; 
	enum{ arity = 2};

	void operator()(FunctorParam& ret, C* self , A0 a0) const{
		rcast<R>(ret), (self->*fun_)(a0);
	}
};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, R (C::*fun_)(A0) const>
struct static_functor<R (C::*)(A0) const, fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; 
	enum{ arity = 2};

	void operator()(FunctorParam& ret, C* self , A0 a0) const{
		rcast<R>(ret), (self->*fun_)(a0);
	}
};

// コンストラクタを関数オブジェクト化する
template<class T , class A0>
struct ctor_functor<T , A0>{
	typedef SmartPtr<T> R;
	typedef R result_type; typedef A0 arg1_type; 
	enum{ arity = 1};

	void operator()(FunctorParam& ret , A0 a0) const{
		rcast<R>(ret), xnew<T>(a0);
	}
};

// stdcall版
#if defined(_WIN32) && defined(_M_IX86)

template<int Method, class R , class A0>
struct nfun<R (__stdcall *)(A0), Method>
	: public nfun_base<1, functor<R (__stdcall *)(A0)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0>
struct functor<R (__stdcall *)(A0)>{
	typedef R result_type; typedef A0 arg1_type; 
	typedef R (__stdcall *fun_type)(A0);
	enum{ arity = 1};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0) const{
		rcast<R>(ret), fun_(a0);
	}
};

// 関数を関数オブジェクト化する
template<class R , class A0, R (__stdcall *fun_)(A0)>
struct static_functor<R (__stdcall *)(A0), fun_>{
	typedef R result_type; typedef A0 arg1_type; 
	enum{ arity = 1};

	void operator()(FunctorParam& ret , A0 a0) const{
		rcast<R>(ret), fun_(a0);
	}
};

#endif


template<int Method, class R , class A0, class A1>
struct nfun<R (*)(A0, A1), Method>
	: public nfun_base<2, functor<R (*)(A0, A1)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1>
struct functor<R (*)(A0, A1)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; 
	typedef R (*fun_type)(A0, A1);
	enum{ arity = 2};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1) const{
		rcast<R>(ret), fun_(a0, a1);
	}
};

template<int Method, class R, class C , class A0, class A1>
struct nfun<R (C::*)(A0, A1), Method>
	: public nfun_base<3, functor<R (C::*)(A0, A1)>, Method, R>{};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1>
struct functor<R (C::*)(A0, A1)>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; 
	typedef R (C::*fun_type)(A0, A1);
	enum{ arity = 3};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1) const{
		rcast<R>(ret), (self->*fun_)(a0, a1);
	}
};

template<int Method, class R, class C , class A0, class A1>
struct nfun<R (C::*)(A0, A1) const, Method>
	: public nfun_base<3, functor<R (C::*)(A0, A1) const>, Method, R>{};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1>
struct functor<R (C::*)(A0, A1) const>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; 
	typedef R (C::*fun_type)(A0, A1) const;
	enum{ arity = 3};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1) const{
		rcast<R>(ret), (self->*fun_)(a0, a1);
	}
};

////

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, R (*fun_)(A0, A1)>
struct static_functor<R (*)(A0, A1), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; 
	enum{ arity = 2};

	void operator()(FunctorParam& ret , A0 a0, A1 a1) const{
		rcast<R>(ret), fun_(a0, a1);
	}
};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, R (C::*fun_)(A0, A1)>
struct static_functor<R (C::*)(A0, A1), fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; 
	enum{ arity = 3};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1) const{
		rcast<R>(ret), (self->*fun_)(a0, a1);
	}
};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, R (C::*fun_)(A0, A1) const>
struct static_functor<R (C::*)(A0, A1) const, fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; 
	enum{ arity = 3};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1) const{
		rcast<R>(ret), (self->*fun_)(a0, a1);
	}
};

// コンストラクタを関数オブジェクト化する
template<class T , class A0, class A1>
struct ctor_functor<T , A0, A1>{
	typedef SmartPtr<T> R;
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; 
	enum{ arity = 2};

	void operator()(FunctorParam& ret , A0 a0, A1 a1) const{
		rcast<R>(ret), xnew<T>(a0, a1);
	}
};

// stdcall版
#if defined(_WIN32) && defined(_M_IX86)

template<int Method, class R , class A0, class A1>
struct nfun<R (__stdcall *)(A0, A1), Method>
	: public nfun_base<2, functor<R (__stdcall *)(A0, A1)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1>
struct functor<R (__stdcall *)(A0, A1)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; 
	typedef R (__stdcall *fun_type)(A0, A1);
	enum{ arity = 2};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1) const{
		rcast<R>(ret), fun_(a0, a1);
	}
};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, R (__stdcall *fun_)(A0, A1)>
struct static_functor<R (__stdcall *)(A0, A1), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; 
	enum{ arity = 2};

	void operator()(FunctorParam& ret , A0 a0, A1 a1) const{
		rcast<R>(ret), fun_(a0, a1);
	}
};

#endif


template<int Method, class R , class A0, class A1, class A2>
struct nfun<R (*)(A0, A1, A2), Method>
	: public nfun_base<3, functor<R (*)(A0, A1, A2)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2>
struct functor<R (*)(A0, A1, A2)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; 
	typedef R (*fun_type)(A0, A1, A2);
	enum{ arity = 3};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2) const{
		rcast<R>(ret), fun_(a0, a1, a2);
	}
};

template<int Method, class R, class C , class A0, class A1, class A2>
struct nfun<R (C::*)(A0, A1, A2), Method>
	: public nfun_base<4, functor<R (C::*)(A0, A1, A2)>, Method, R>{};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2>
struct functor<R (C::*)(A0, A1, A2)>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; 
	typedef R (C::*fun_type)(A0, A1, A2);
	enum{ arity = 4};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2);
	}
};

template<int Method, class R, class C , class A0, class A1, class A2>
struct nfun<R (C::*)(A0, A1, A2) const, Method>
	: public nfun_base<4, functor<R (C::*)(A0, A1, A2) const>, Method, R>{};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2>
struct functor<R (C::*)(A0, A1, A2) const>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; 
	typedef R (C::*fun_type)(A0, A1, A2) const;
	enum{ arity = 4};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2);
	}
};

////

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, R (*fun_)(A0, A1, A2)>
struct static_functor<R (*)(A0, A1, A2), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; 
	enum{ arity = 3};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2) const{
		rcast<R>(ret), fun_(a0, a1, a2);
	}
};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, R (C::*fun_)(A0, A1, A2)>
struct static_functor<R (C::*)(A0, A1, A2), fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; 
	enum{ arity = 4};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2);
	}
};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, R (C::*fun_)(A0, A1, A2) const>
struct static_functor<R (C::*)(A0, A1, A2) const, fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; 
	enum{ arity = 4};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2);
	}
};

// コンストラクタを関数オブジェクト化する
template<class T , class A0, class A1, class A2>
struct ctor_functor<T , A0, A1, A2>{
	typedef SmartPtr<T> R;
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; 
	enum{ arity = 3};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2) const{
		rcast<R>(ret), xnew<T>(a0, a1, a2);
	}
};

// stdcall版
#if defined(_WIN32) && defined(_M_IX86)

template<int Method, class R , class A0, class A1, class A2>
struct nfun<R (__stdcall *)(A0, A1, A2), Method>
	: public nfun_base<3, functor<R (__stdcall *)(A0, A1, A2)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2>
struct functor<R (__stdcall *)(A0, A1, A2)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; 
	typedef R (__stdcall *fun_type)(A0, A1, A2);
	enum{ arity = 3};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2) const{
		rcast<R>(ret), fun_(a0, a1, a2);
	}
};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, R (__stdcall *fun_)(A0, A1, A2)>
struct static_functor<R (__stdcall *)(A0, A1, A2), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; 
	enum{ arity = 3};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2) const{
		rcast<R>(ret), fun_(a0, a1, a2);
	}
};

#endif


template<int Method, class R , class A0, class A1, class A2, class A3>
struct nfun<R (*)(A0, A1, A2, A3), Method>
	: public nfun_base<4, functor<R (*)(A0, A1, A2, A3)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3>
struct functor<R (*)(A0, A1, A2, A3)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; 
	typedef R (*fun_type)(A0, A1, A2, A3);
	enum{ arity = 4};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3);
	}
};

template<int Method, class R, class C , class A0, class A1, class A2, class A3>
struct nfun<R (C::*)(A0, A1, A2, A3), Method>
	: public nfun_base<5, functor<R (C::*)(A0, A1, A2, A3)>, Method, R>{};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3>
struct functor<R (C::*)(A0, A1, A2, A3)>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; 
	typedef R (C::*fun_type)(A0, A1, A2, A3);
	enum{ arity = 5};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3);
	}
};

template<int Method, class R, class C , class A0, class A1, class A2, class A3>
struct nfun<R (C::*)(A0, A1, A2, A3) const, Method>
	: public nfun_base<5, functor<R (C::*)(A0, A1, A2, A3) const>, Method, R>{};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3>
struct functor<R (C::*)(A0, A1, A2, A3) const>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; 
	typedef R (C::*fun_type)(A0, A1, A2, A3) const;
	enum{ arity = 5};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3);
	}
};

////

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, R (*fun_)(A0, A1, A2, A3)>
struct static_functor<R (*)(A0, A1, A2, A3), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; 
	enum{ arity = 4};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3);
	}
};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, R (C::*fun_)(A0, A1, A2, A3)>
struct static_functor<R (C::*)(A0, A1, A2, A3), fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; 
	enum{ arity = 5};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3);
	}
};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, R (C::*fun_)(A0, A1, A2, A3) const>
struct static_functor<R (C::*)(A0, A1, A2, A3) const, fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; 
	enum{ arity = 5};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3);
	}
};

// コンストラクタを関数オブジェクト化する
template<class T , class A0, class A1, class A2, class A3>
struct ctor_functor<T , A0, A1, A2, A3>{
	typedef SmartPtr<T> R;
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; 
	enum{ arity = 4};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3) const{
		rcast<R>(ret), xnew<T>(a0, a1, a2, a3);
	}
};

// stdcall版
#if defined(_WIN32) && defined(_M_IX86)

template<int Method, class R , class A0, class A1, class A2, class A3>
struct nfun<R (__stdcall *)(A0, A1, A2, A3), Method>
	: public nfun_base<4, functor<R (__stdcall *)(A0, A1, A2, A3)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3>
struct functor<R (__stdcall *)(A0, A1, A2, A3)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; 
	typedef R (__stdcall *fun_type)(A0, A1, A2, A3);
	enum{ arity = 4};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3);
	}
};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, R (__stdcall *fun_)(A0, A1, A2, A3)>
struct static_functor<R (__stdcall *)(A0, A1, A2, A3), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; 
	enum{ arity = 4};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3);
	}
};

#endif


template<int Method, class R , class A0, class A1, class A2, class A3, class A4>
struct nfun<R (*)(A0, A1, A2, A3, A4), Method>
	: public nfun_base<5, functor<R (*)(A0, A1, A2, A3, A4)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4>
struct functor<R (*)(A0, A1, A2, A3, A4)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; 
	typedef R (*fun_type)(A0, A1, A2, A3, A4);
	enum{ arity = 5};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4);
	}
};

template<int Method, class R, class C , class A0, class A1, class A2, class A3, class A4>
struct nfun<R (C::*)(A0, A1, A2, A3, A4), Method>
	: public nfun_base<6, functor<R (C::*)(A0, A1, A2, A3, A4)>, Method, R>{};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4>
struct functor<R (C::*)(A0, A1, A2, A3, A4)>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; 
	typedef R (C::*fun_type)(A0, A1, A2, A3, A4);
	enum{ arity = 6};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4);
	}
};

template<int Method, class R, class C , class A0, class A1, class A2, class A3, class A4>
struct nfun<R (C::*)(A0, A1, A2, A3, A4) const, Method>
	: public nfun_base<6, functor<R (C::*)(A0, A1, A2, A3, A4) const>, Method, R>{};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4>
struct functor<R (C::*)(A0, A1, A2, A3, A4) const>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; 
	typedef R (C::*fun_type)(A0, A1, A2, A3, A4) const;
	enum{ arity = 6};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4);
	}
};

////

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, R (*fun_)(A0, A1, A2, A3, A4)>
struct static_functor<R (*)(A0, A1, A2, A3, A4), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; 
	enum{ arity = 5};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4);
	}
};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, R (C::*fun_)(A0, A1, A2, A3, A4)>
struct static_functor<R (C::*)(A0, A1, A2, A3, A4), fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; 
	enum{ arity = 6};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4);
	}
};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, R (C::*fun_)(A0, A1, A2, A3, A4) const>
struct static_functor<R (C::*)(A0, A1, A2, A3, A4) const, fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; 
	enum{ arity = 6};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4);
	}
};

// コンストラクタを関数オブジェクト化する
template<class T , class A0, class A1, class A2, class A3, class A4>
struct ctor_functor<T , A0, A1, A2, A3, A4>{
	typedef SmartPtr<T> R;
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; 
	enum{ arity = 5};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4) const{
		rcast<R>(ret), xnew<T>(a0, a1, a2, a3, a4);
	}
};

// stdcall版
#if defined(_WIN32) && defined(_M_IX86)

template<int Method, class R , class A0, class A1, class A2, class A3, class A4>
struct nfun<R (__stdcall *)(A0, A1, A2, A3, A4), Method>
	: public nfun_base<5, functor<R (__stdcall *)(A0, A1, A2, A3, A4)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4>
struct functor<R (__stdcall *)(A0, A1, A2, A3, A4)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; 
	typedef R (__stdcall *fun_type)(A0, A1, A2, A3, A4);
	enum{ arity = 5};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4);
	}
};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, R (__stdcall *fun_)(A0, A1, A2, A3, A4)>
struct static_functor<R (__stdcall *)(A0, A1, A2, A3, A4), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; 
	enum{ arity = 5};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4);
	}
};

#endif


template<int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5>
struct nfun<R (*)(A0, A1, A2, A3, A4, A5), Method>
	: public nfun_base<6, functor<R (*)(A0, A1, A2, A3, A4, A5)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5>
struct functor<R (*)(A0, A1, A2, A3, A4, A5)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; 
	typedef R (*fun_type)(A0, A1, A2, A3, A4, A5);
	enum{ arity = 6};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5);
	}
};

template<int Method, class R, class C , class A0, class A1, class A2, class A3, class A4, class A5>
struct nfun<R (C::*)(A0, A1, A2, A3, A4, A5), Method>
	: public nfun_base<7, functor<R (C::*)(A0, A1, A2, A3, A4, A5)>, Method, R>{};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5>
struct functor<R (C::*)(A0, A1, A2, A3, A4, A5)>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; 
	typedef R (C::*fun_type)(A0, A1, A2, A3, A4, A5);
	enum{ arity = 7};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5);
	}
};

template<int Method, class R, class C , class A0, class A1, class A2, class A3, class A4, class A5>
struct nfun<R (C::*)(A0, A1, A2, A3, A4, A5) const, Method>
	: public nfun_base<7, functor<R (C::*)(A0, A1, A2, A3, A4, A5) const>, Method, R>{};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5>
struct functor<R (C::*)(A0, A1, A2, A3, A4, A5) const>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; 
	typedef R (C::*fun_type)(A0, A1, A2, A3, A4, A5) const;
	enum{ arity = 7};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5);
	}
};

////

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, R (*fun_)(A0, A1, A2, A3, A4, A5)>
struct static_functor<R (*)(A0, A1, A2, A3, A4, A5), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; 
	enum{ arity = 6};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5);
	}
};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, R (C::*fun_)(A0, A1, A2, A3, A4, A5)>
struct static_functor<R (C::*)(A0, A1, A2, A3, A4, A5), fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; 
	enum{ arity = 7};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5);
	}
};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, R (C::*fun_)(A0, A1, A2, A3, A4, A5) const>
struct static_functor<R (C::*)(A0, A1, A2, A3, A4, A5) const, fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; 
	enum{ arity = 7};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5);
	}
};

// コンストラクタを関数オブジェクト化する
template<class T , class A0, class A1, class A2, class A3, class A4, class A5>
struct ctor_functor<T , A0, A1, A2, A3, A4, A5>{
	typedef SmartPtr<T> R;
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; 
	enum{ arity = 6};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5) const{
		rcast<R>(ret), xnew<T>(a0, a1, a2, a3, a4, a5);
	}
};

// stdcall版
#if defined(_WIN32) && defined(_M_IX86)

template<int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5>
struct nfun<R (__stdcall *)(A0, A1, A2, A3, A4, A5), Method>
	: public nfun_base<6, functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5>
struct functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; 
	typedef R (__stdcall *fun_type)(A0, A1, A2, A3, A4, A5);
	enum{ arity = 6};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5);
	}
};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, R (__stdcall *fun_)(A0, A1, A2, A3, A4, A5)>
struct static_functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; 
	enum{ arity = 6};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5);
	}
};

#endif


template<int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6>
struct nfun<R (*)(A0, A1, A2, A3, A4, A5, A6), Method>
	: public nfun_base<7, functor<R (*)(A0, A1, A2, A3, A4, A5, A6)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6>
struct functor<R (*)(A0, A1, A2, A3, A4, A5, A6)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; 
	typedef R (*fun_type)(A0, A1, A2, A3, A4, A5, A6);
	enum{ arity = 7};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6);
	}
};

template<int Method, class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6>
struct nfun<R (C::*)(A0, A1, A2, A3, A4, A5, A6), Method>
	: public nfun_base<8, functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6)>, Method, R>{};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6>
struct functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6)>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; 
	typedef R (C::*fun_type)(A0, A1, A2, A3, A4, A5, A6);
	enum{ arity = 8};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6);
	}
};

template<int Method, class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6>
struct nfun<R (C::*)(A0, A1, A2, A3, A4, A5, A6) const, Method>
	: public nfun_base<8, functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6) const>, Method, R>{};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6>
struct functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6) const>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; 
	typedef R (C::*fun_type)(A0, A1, A2, A3, A4, A5, A6) const;
	enum{ arity = 8};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6);
	}
};

////

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, R (*fun_)(A0, A1, A2, A3, A4, A5, A6)>
struct static_functor<R (*)(A0, A1, A2, A3, A4, A5, A6), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; 
	enum{ arity = 7};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6);
	}
};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, R (C::*fun_)(A0, A1, A2, A3, A4, A5, A6)>
struct static_functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6), fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; 
	enum{ arity = 8};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6);
	}
};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, R (C::*fun_)(A0, A1, A2, A3, A4, A5, A6) const>
struct static_functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6) const, fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; 
	enum{ arity = 8};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6);
	}
};

// コンストラクタを関数オブジェクト化する
template<class T , class A0, class A1, class A2, class A3, class A4, class A5, class A6>
struct ctor_functor<T , A0, A1, A2, A3, A4, A5, A6>{
	typedef SmartPtr<T> R;
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; 
	enum{ arity = 7};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6) const{
		rcast<R>(ret), xnew<T>(a0, a1, a2, a3, a4, a5, a6);
	}
};

// stdcall版
#if defined(_WIN32) && defined(_M_IX86)

template<int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6>
struct nfun<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6), Method>
	: public nfun_base<7, functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6>
struct functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; 
	typedef R (__stdcall *fun_type)(A0, A1, A2, A3, A4, A5, A6);
	enum{ arity = 7};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6);
	}
};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, R (__stdcall *fun_)(A0, A1, A2, A3, A4, A5, A6)>
struct static_functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; 
	enum{ arity = 7};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6);
	}
};

#endif


template<int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7>
struct nfun<R (*)(A0, A1, A2, A3, A4, A5, A6, A7), Method>
	: public nfun_base<8, functor<R (*)(A0, A1, A2, A3, A4, A5, A6, A7)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7>
struct functor<R (*)(A0, A1, A2, A3, A4, A5, A6, A7)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; 
	typedef R (*fun_type)(A0, A1, A2, A3, A4, A5, A6, A7);
	enum{ arity = 8};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7);
	}
};

template<int Method, class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7>
struct nfun<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7), Method>
	: public nfun_base<9, functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7)>, Method, R>{};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7>
struct functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7)>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; 
	typedef R (C::*fun_type)(A0, A1, A2, A3, A4, A5, A6, A7);
	enum{ arity = 9};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7);
	}
};

template<int Method, class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7>
struct nfun<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7) const, Method>
	: public nfun_base<9, functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7) const>, Method, R>{};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7>
struct functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7) const>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; 
	typedef R (C::*fun_type)(A0, A1, A2, A3, A4, A5, A6, A7) const;
	enum{ arity = 9};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7);
	}
};

////

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, R (*fun_)(A0, A1, A2, A3, A4, A5, A6, A7)>
struct static_functor<R (*)(A0, A1, A2, A3, A4, A5, A6, A7), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; 
	enum{ arity = 8};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7);
	}
};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, R (C::*fun_)(A0, A1, A2, A3, A4, A5, A6, A7)>
struct static_functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7), fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; 
	enum{ arity = 9};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7);
	}
};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, R (C::*fun_)(A0, A1, A2, A3, A4, A5, A6, A7) const>
struct static_functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7) const, fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; 
	enum{ arity = 9};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7);
	}
};

// コンストラクタを関数オブジェクト化する
template<class T , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7>
struct ctor_functor<T , A0, A1, A2, A3, A4, A5, A6, A7>{
	typedef SmartPtr<T> R;
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; 
	enum{ arity = 8};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7) const{
		rcast<R>(ret), xnew<T>(a0, a1, a2, a3, a4, a5, a6, a7);
	}
};

// stdcall版
#if defined(_WIN32) && defined(_M_IX86)

template<int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7>
struct nfun<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7), Method>
	: public nfun_base<8, functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7>
struct functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; 
	typedef R (__stdcall *fun_type)(A0, A1, A2, A3, A4, A5, A6, A7);
	enum{ arity = 8};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7);
	}
};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, R (__stdcall *fun_)(A0, A1, A2, A3, A4, A5, A6, A7)>
struct static_functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; 
	enum{ arity = 8};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7);
	}
};

#endif


template<int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8>
struct nfun<R (*)(A0, A1, A2, A3, A4, A5, A6, A7, A8), Method>
	: public nfun_base<9, functor<R (*)(A0, A1, A2, A3, A4, A5, A6, A7, A8)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8>
struct functor<R (*)(A0, A1, A2, A3, A4, A5, A6, A7, A8)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; 
	typedef R (*fun_type)(A0, A1, A2, A3, A4, A5, A6, A7, A8);
	enum{ arity = 9};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7, a8);
	}
};

template<int Method, class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8>
struct nfun<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8), Method>
	: public nfun_base<10, functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8)>, Method, R>{};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8>
struct functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8)>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; typedef A8 arg10_type; 
	typedef R (C::*fun_type)(A0, A1, A2, A3, A4, A5, A6, A7, A8);
	enum{ arity = 10};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7, a8);
	}
};

template<int Method, class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8>
struct nfun<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8) const, Method>
	: public nfun_base<10, functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8) const>, Method, R>{};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8>
struct functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8) const>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; typedef A8 arg10_type; 
	typedef R (C::*fun_type)(A0, A1, A2, A3, A4, A5, A6, A7, A8) const;
	enum{ arity = 10};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7, a8);
	}
};

////

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, R (*fun_)(A0, A1, A2, A3, A4, A5, A6, A7, A8)>
struct static_functor<R (*)(A0, A1, A2, A3, A4, A5, A6, A7, A8), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; 
	enum{ arity = 9};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7, a8);
	}
};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, R (C::*fun_)(A0, A1, A2, A3, A4, A5, A6, A7, A8)>
struct static_functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8), fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; typedef A8 arg10_type; 
	enum{ arity = 10};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7, a8);
	}
};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, R (C::*fun_)(A0, A1, A2, A3, A4, A5, A6, A7, A8) const>
struct static_functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8) const, fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; typedef A8 arg10_type; 
	enum{ arity = 10};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7, a8);
	}
};

// コンストラクタを関数オブジェクト化する
template<class T , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8>
struct ctor_functor<T , A0, A1, A2, A3, A4, A5, A6, A7, A8>{
	typedef SmartPtr<T> R;
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; 
	enum{ arity = 9};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8) const{
		rcast<R>(ret), xnew<T>(a0, a1, a2, a3, a4, a5, a6, a7, a8);
	}
};

// stdcall版
#if defined(_WIN32) && defined(_M_IX86)

template<int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8>
struct nfun<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7, A8), Method>
	: public nfun_base<9, functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7, A8)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8>
struct functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7, A8)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; 
	typedef R (__stdcall *fun_type)(A0, A1, A2, A3, A4, A5, A6, A7, A8);
	enum{ arity = 9};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7, a8);
	}
};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, R (__stdcall *fun_)(A0, A1, A2, A3, A4, A5, A6, A7, A8)>
struct static_functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7, A8), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; 
	enum{ arity = 9};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7, a8);
	}
};

#endif


template<int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9>
struct nfun<R (*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9), Method>
	: public nfun_base<10, functor<R (*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9>
struct functor<R (*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; typedef A9 arg10_type; 
	typedef R (*fun_type)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9);
	enum{ arity = 10};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9);
	}
};

template<int Method, class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9>
struct nfun<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9), Method>
	: public nfun_base<11, functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9)>, Method, R>{};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9>
struct functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9)>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; typedef A8 arg10_type; typedef A9 arg11_type; 
	typedef R (C::*fun_type)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9);
	enum{ arity = 11};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9);
	}
};

template<int Method, class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9>
struct nfun<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9) const, Method>
	: public nfun_base<11, functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9) const>, Method, R>{};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9>
struct functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9) const>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; typedef A8 arg10_type; typedef A9 arg11_type; 
	typedef R (C::*fun_type)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9) const;
	enum{ arity = 11};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9);
	}
};

////

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, R (*fun_)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9)>
struct static_functor<R (*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; typedef A9 arg10_type; 
	enum{ arity = 10};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9);
	}
};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, R (C::*fun_)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9)>
struct static_functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9), fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; typedef A8 arg10_type; typedef A9 arg11_type; 
	enum{ arity = 11};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9);
	}
};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, R (C::*fun_)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9) const>
struct static_functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9) const, fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; typedef A8 arg10_type; typedef A9 arg11_type; 
	enum{ arity = 11};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9);
	}
};

// コンストラクタを関数オブジェクト化する
template<class T , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9>
struct ctor_functor<T , A0, A1, A2, A3, A4, A5, A6, A7, A8, A9>{
	typedef SmartPtr<T> R;
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; typedef A9 arg10_type; 
	enum{ arity = 10};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9) const{
		rcast<R>(ret), xnew<T>(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9);
	}
};

// stdcall版
#if defined(_WIN32) && defined(_M_IX86)

template<int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9>
struct nfun<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9), Method>
	: public nfun_base<10, functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9>
struct functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; typedef A9 arg10_type; 
	typedef R (__stdcall *fun_type)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9);
	enum{ arity = 10};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9);
	}
};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, R (__stdcall *fun_)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9)>
struct static_functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; typedef A9 arg10_type; 
	enum{ arity = 10};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9);
	}
};

#endif


template<int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10>
struct nfun<R (*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10), Method>
	: public nfun_base<11, functor<R (*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10>
struct functor<R (*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; typedef A9 arg10_type; typedef A10 arg11_type; 
	typedef R (*fun_type)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10);
	enum{ arity = 11};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10);
	}
};

template<int Method, class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10>
struct nfun<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10), Method>
	: public nfun_base<12, functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10)>, Method, R>{};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10>
struct functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10)>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; typedef A8 arg10_type; typedef A9 arg11_type; typedef A10 arg12_type; 
	typedef R (C::*fun_type)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10);
	enum{ arity = 12};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10);
	}
};

template<int Method, class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10>
struct nfun<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10) const, Method>
	: public nfun_base<12, functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10) const>, Method, R>{};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10>
struct functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10) const>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; typedef A8 arg10_type; typedef A9 arg11_type; typedef A10 arg12_type; 
	typedef R (C::*fun_type)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10) const;
	enum{ arity = 12};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10);
	}
};

////

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, R (*fun_)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10)>
struct static_functor<R (*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; typedef A9 arg10_type; typedef A10 arg11_type; 
	enum{ arity = 11};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10);
	}
};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, R (C::*fun_)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10)>
struct static_functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10), fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; typedef A8 arg10_type; typedef A9 arg11_type; typedef A10 arg12_type; 
	enum{ arity = 12};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10);
	}
};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, R (C::*fun_)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10) const>
struct static_functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10) const, fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; typedef A8 arg10_type; typedef A9 arg11_type; typedef A10 arg12_type; 
	enum{ arity = 12};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10);
	}
};

// コンストラクタを関数オブジェクト化する
template<class T , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10>
struct ctor_functor<T , A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10>{
	typedef SmartPtr<T> R;
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; typedef A9 arg10_type; typedef A10 arg11_type; 
	enum{ arity = 11};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10) const{
		rcast<R>(ret), xnew<T>(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10);
	}
};

// stdcall版
#if defined(_WIN32) && defined(_M_IX86)

template<int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10>
struct nfun<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10), Method>
	: public nfun_base<11, functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10>
struct functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; typedef A9 arg10_type; typedef A10 arg11_type; 
	typedef R (__stdcall *fun_type)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10);
	enum{ arity = 11};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10);
	}
};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, R (__stdcall *fun_)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10)>
struct static_functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; typedef A9 arg10_type; typedef A10 arg11_type; 
	enum{ arity = 11};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10);
	}
};

#endif


template<int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11>
struct nfun<R (*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11), Method>
	: public nfun_base<12, functor<R (*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11>
struct functor<R (*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; typedef A9 arg10_type; typedef A10 arg11_type; typedef A11 arg12_type; 
	typedef R (*fun_type)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11);
	enum{ arity = 12};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10, A11 a11) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11);
	}
};

template<int Method, class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11>
struct nfun<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11), Method>
	: public nfun_base<13, functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)>, Method, R>{};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11>
struct functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; typedef A8 arg10_type; typedef A9 arg11_type; typedef A10 arg12_type; typedef A11 arg13_type; 
	typedef R (C::*fun_type)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11);
	enum{ arity = 13};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10, A11 a11) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11);
	}
};

template<int Method, class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11>
struct nfun<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11) const, Method>
	: public nfun_base<13, functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11) const>, Method, R>{};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11>
struct functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11) const>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; typedef A8 arg10_type; typedef A9 arg11_type; typedef A10 arg12_type; typedef A11 arg13_type; 
	typedef R (C::*fun_type)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11) const;
	enum{ arity = 13};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10, A11 a11) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11);
	}
};

////

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, R (*fun_)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)>
struct static_functor<R (*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; typedef A9 arg10_type; typedef A10 arg11_type; typedef A11 arg12_type; 
	enum{ arity = 12};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10, A11 a11) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11);
	}
};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, R (C::*fun_)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)>
struct static_functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11), fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; typedef A8 arg10_type; typedef A9 arg11_type; typedef A10 arg12_type; typedef A11 arg13_type; 
	enum{ arity = 13};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10, A11 a11) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11);
	}
};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, R (C::*fun_)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11) const>
struct static_functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11) const, fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; typedef A8 arg10_type; typedef A9 arg11_type; typedef A10 arg12_type; typedef A11 arg13_type; 
	enum{ arity = 13};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10, A11 a11) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11);
	}
};

// コンストラクタを関数オブジェクト化する
template<class T , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11>
struct ctor_functor<T , A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11>{
	typedef SmartPtr<T> R;
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; typedef A9 arg10_type; typedef A10 arg11_type; typedef A11 arg12_type; 
	enum{ arity = 12};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10, A11 a11) const{
		rcast<R>(ret), xnew<T>(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11);
	}
};

// stdcall版
#if defined(_WIN32) && defined(_M_IX86)

template<int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11>
struct nfun<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11), Method>
	: public nfun_base<12, functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11>
struct functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; typedef A9 arg10_type; typedef A10 arg11_type; typedef A11 arg12_type; 
	typedef R (__stdcall *fun_type)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11);
	enum{ arity = 12};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10, A11 a11) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11);
	}
};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, R (__stdcall *fun_)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11)>
struct static_functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; typedef A9 arg10_type; typedef A10 arg11_type; typedef A11 arg12_type; 
	enum{ arity = 12};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10, A11 a11) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11);
	}
};

#endif


template<int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, class A12>
struct nfun<R (*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12), Method>
	: public nfun_base<13, functor<R (*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, class A12>
struct functor<R (*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; typedef A9 arg10_type; typedef A10 arg11_type; typedef A11 arg12_type; typedef A12 arg13_type; 
	typedef R (*fun_type)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12);
	enum{ arity = 13};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10, A11 a11, A12 a12) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12);
	}
};

template<int Method, class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, class A12>
struct nfun<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12), Method>
	: public nfun_base<14, functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)>, Method, R>{};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, class A12>
struct functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; typedef A8 arg10_type; typedef A9 arg11_type; typedef A10 arg12_type; typedef A11 arg13_type; typedef A12 arg14_type; 
	typedef R (C::*fun_type)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12);
	enum{ arity = 14};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10, A11 a11, A12 a12) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12);
	}
};

template<int Method, class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, class A12>
struct nfun<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12) const, Method>
	: public nfun_base<14, functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12) const>, Method, R>{};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, class A12>
struct functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12) const>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; typedef A8 arg10_type; typedef A9 arg11_type; typedef A10 arg12_type; typedef A11 arg13_type; typedef A12 arg14_type; 
	typedef R (C::*fun_type)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12) const;
	enum{ arity = 14};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10, A11 a11, A12 a12) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12);
	}
};

////

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, class A12, R (*fun_)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)>
struct static_functor<R (*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; typedef A9 arg10_type; typedef A10 arg11_type; typedef A11 arg12_type; typedef A12 arg13_type; 
	enum{ arity = 13};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10, A11 a11, A12 a12) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12);
	}
};

// メンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, class A12, R (C::*fun_)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)>
struct static_functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12), fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; typedef A8 arg10_type; typedef A9 arg11_type; typedef A10 arg12_type; typedef A11 arg13_type; typedef A12 arg14_type; 
	enum{ arity = 14};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10, A11 a11, A12 a12) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12);
	}
};

// constメンバ関数を関数オブジェクト化する
template<class R, class C , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, class A12, R (C::*fun_)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12) const>
struct static_functor<R (C::*)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12) const, fun_>{
	typedef R result_type; typedef C* arg1_type; typedef A0 arg2_type; typedef A1 arg3_type; typedef A2 arg4_type; typedef A3 arg5_type; typedef A4 arg6_type; typedef A5 arg7_type; typedef A6 arg8_type; typedef A7 arg9_type; typedef A8 arg10_type; typedef A9 arg11_type; typedef A10 arg12_type; typedef A11 arg13_type; typedef A12 arg14_type; 
	enum{ arity = 14};

	void operator()(FunctorParam& ret, C* self , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10, A11 a11, A12 a12) const{
		rcast<R>(ret), (self->*fun_)(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12);
	}
};

// コンストラクタを関数オブジェクト化する
template<class T , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, class A12>
struct ctor_functor<T , A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12>{
	typedef SmartPtr<T> R;
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; typedef A9 arg10_type; typedef A10 arg11_type; typedef A11 arg12_type; typedef A12 arg13_type; 
	enum{ arity = 13};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10, A11 a11, A12 a12) const{
		rcast<R>(ret), xnew<T>(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12);
	}
};

// stdcall版
#if defined(_WIN32) && defined(_M_IX86)

template<int Method, class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, class A12>
struct nfun<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12), Method>
	: public nfun_base<13, functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)>, Method, R>{};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, class A12>
struct functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; typedef A9 arg10_type; typedef A10 arg11_type; typedef A11 arg12_type; typedef A12 arg13_type; 
	typedef R (__stdcall *fun_type)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12);
	enum{ arity = 13};

	fun_type fun_;
	functor(fun_type fun):fun_(fun){}

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10, A11 a11, A12 a12) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12);
	}
};

// 関数を関数オブジェクト化する
template<class R , class A0, class A1, class A2, class A3, class A4, class A5, class A6, class A7, class A8, class A9, class A10, class A11, class A12, R (__stdcall *fun_)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12)>
struct static_functor<R (__stdcall *)(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12), fun_>{
	typedef R result_type; typedef A0 arg1_type; typedef A1 arg2_type; typedef A2 arg3_type; typedef A3 arg4_type; typedef A4 arg5_type; typedef A5 arg6_type; typedef A6 arg7_type; typedef A7 arg8_type; typedef A8 arg9_type; typedef A9 arg10_type; typedef A10 arg11_type; typedef A11 arg12_type; typedef A12 arg13_type; 
	enum{ arity = 13};

	void operator()(FunctorParam& ret , A0 a0, A1 a1, A2 a2, A3 a3, A4 a4, A5 a5, A6 a6, A7 a7, A8 a8, A9 a9, A10 a10, A11 a11, A12 a12) const{
		rcast<R>(ret), fun_(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12);
	}
};

#endif


//}}REPEAT}


///////////////////////////////////////////////////////

template<class C, class T>
struct getter_functor{
	typedef T& R;
	typedef R result_type; typedef C* arg1_type;
	enum{ arity = 1 };
	typedef T C::* fun_t;

	fun_t fun_;
	getter_functor(fun_t fun): fun_(fun){}

	void operator()(FunctorParam& ret, C* a0){
		T& t = a0->*fun_;
		rcast<R>(ret), t;
	}
};

template<class C, class T>
struct setter_functor{
	typedef T& R;
	typedef R result_type; typedef C* arg1_type; typedef const T& arg2_type;
	enum{ arity = 2 };
	typedef T C::* fun_t;

	fun_t fun_;
	setter_functor(fun_t fun): fun_(fun){}

	void operator()(FunctorParam& ret, C* a0, const T& a1){
		T& t = a0->*fun_;
		t = a1;
		rcast<R>(ret), t;
	}
};

template<class TFun, TFun fun_>
struct static_getter_functor;

template<class TFun, TFun fun_>
struct static_setter_functor;

template<class C, class T, T C::* fun_>
struct static_getter_functor<T C::*, fun_>{
	typedef T& R;
	typedef R result_type; typedef C* arg1_type;
	enum{ arity = 1 };

	void operator()(FunctorParam& ret, C* a0){
		T& t = a0->*fun_;
		rcast<R>(ret), t;
	}
};

template<class C, class T, T C::* fun_>
struct static_setter_functor<T C::*, fun_>{
	typedef T& R;
	typedef R result_type; typedef C* arg1_type; typedef const T& arg2_type;
	enum{ arity = 2 };

	void operator()(FunctorParam& ret, C* a0, const T& a1){
		T& t = a0->*fun_;
		t = a1;
		rcast<R>(ret), t;
	}
};

//////////////////////////////////////////

#if defined(_MSC_VER) && _MSC_VER<=1300

#define Xdef_method(fun) it->def_method(XTAL_STRING(#fun), fun)
#define Xdef_method2(fun, secondary_key) it->def_method(XTAL_STRING(#fun), secondary_key, &Self::fun)
#define Xdef_method_alias(fun, impl) it->def_method(XTAL_STRING(#fun), impl)
#define Xdef_method_alias2(fun, impl, secondary_key) it->def_method(XTAL_STRING(#fun), secondary_key, impl)

#define Xdef_fun(fun) it->def_fun(XTAL_STRING(#fun), &Self::fun)
#define Xdef_fun2(fun, secondary_key) it->def_fun(XTAL_STRING(#fun), secondary_key, &Self::fun)
#define Xdef_fun_alias(fun, impl) it->def_fun(XTAL_STRING(#fun), impl)
#define Xdef_fun_alias2(fun, impl, secondary_key) it->def_fun(XTAL_STRING(#fun), secondary_key, impl)

#define Xdef_getter(fun) it->def_getter(XTAL_STRING(#fun), &Self::fun)
#define Xdef_setter(fun) it->def_setter(XTAL_LONG_LIVED_STRING(XTAL_L("set_") XTAL_L(#fun), &Self::fun)

#else

template<class TFun> struct static_cfun_meker{ 
	template<TFun fun> static const param_types_holder_n& inner(){ 
		return fun_param_holder<dfun<static_functor<TFun, fun> > >::value; 
	} 
};

template<class TFun> inline static_cfun_meker<TFun> generate_static_cfun(TFun){ 
	return static_cfun_meker<TFun>(); 
}

template<class TFun> struct static_cmemfun_meker{ 
	template<TFun fun> static const param_types_holder_n& inner(){ 
		return fun_param_holder<dmemfun<static_functor<TFun, fun> > >::value;
	} 
};

template<class TFun> inline static_cmemfun_meker<TFun> generate_static_cmemfun(TFun){ 
	return static_cmemfun_meker<TFun>(); 
}


template<class TFun> struct static_getter_meker{ 
	template<TFun fun> static const param_types_holder_n& inner(){ 
		return fun_param_holder<dmemfun<static_getter_functor<TFun, fun> > >::value;
	} 
};

template<class TFun> inline static_getter_meker<TFun> generate_static_getter(TFun){ 
	return static_getter_meker<TFun>(); 
}

template<class TFun> struct static_setter_meker{ 
	template<TFun fun> static const param_types_holder_n& inner(){ 
		return fun_param_holder<dmemfun<static_setter_functor<TFun, fun> > >::value;
	} 
};

template<class TFun> inline static_setter_meker<TFun> generate_static_setter(TFun){ 
	return static_setter_meker<TFun>(); 
}


#define XTAL_FUN(fun) ::xtal::generate_static_cfun(fun).inner<fun>()
#define XTAL_METHOD(fun) ::xtal::generate_static_cmemfun(fun).inner<fun>()
#define XTAL_GETTER(fun) ::xtal::generate_static_getter(fun).inner<fun>()
#define XTAL_SETTER(fun) ::xtal::generate_static_setter(fun).inner<fun>()

/**
* \brief メソッドをバインドする
* XTAL_BINDの中で使用する。
* \code
* XTAL_BIND(Vector){
*   Xdef_method(length); // Vector::lengthメソッドをバインドする
* }
* \endcode
*/
#define Xdef_method(fun) it->define(XTAL_STRING(#fun), XTAL_METHOD(&Self::fun))

/**
* \brief メソッドをセカンダリーキー付でバインドする
* XTAL_BINDの中で使用する。
* \code
* XTAL_BIND(Vector){
*   Xdef_method2(length, Vector); // Vector::lengthメソッドをセカンダリーキーをVectorクラスでバインドする
* }
* \endcode
*/
#define Xdef_method2(fun, secondary_key) it->define(XTAL_STRING(#fun), ::xtal::cpp_class< secondary_key >(), XTAL_METHOD(&Self::fun))

/**
* \brief メソッドを別名でバインドする
* XTAL_BINDの中で使用する。
* \code
* XTAL_BIND(Vector){
*   Xdef_method_alias(size, &Vector::length); // Vector::lengthメソッドをsizeという名前でバインドする
* }
* \endcode
*/
#define Xdef_method_alias(fun, impl) it->define(XTAL_STRING(#fun), XTAL_METHOD(impl))

/**
* \brief メソッドを別名でセカンダリーキー付でバインドする
* XTAL_BINDの中で使用する。
* \code
* XTAL_BIND(Vector){
*   Xdef_method_alias2(size, &Vector::length, Vector); // Vector::lengthメソッドをsizeという名前でセカンダリーキーをVectorクラスでバインドする
* }
* \endcode
*/
#define Xdef_method_alias2(fun, impl, secondary_key) it->define(XTAL_STRING(#fun), ::xtal::cpp_class< secondary_key >(), XTAL_METHOD(impl))

/**
* \brief 関数をバインドする
* XTAL_BINDの中で使用する。
* \code
* XTAL_BIND(Test){
*   Xdef_fun(fuge); // Test::fuge関数をバインドする
* }
* \endcode
*/
#define Xdef_fun(fun) it->define(XTAL_STRING(#fun), XTAL_FUN(&Self::fun))

/**
* \brief 関数をセカンダリーキー付でバインドする
* XTAL_BINDの中で使用する。
* \code
* XTAL_BIND(Test){
*   Xdef_fun2(fuge, Hoge); // Test::fuge関数をセカンダリーキーをHogeクラスでバインドする
* }
* \endcode
*/
#define Xdef_fun2(fun, secondary_key) it->define(XTAL_STRING(#fun), ::xtal::cpp_class< secondary_key >(), XTAL_FUN(&Self::fun))

/**
* \brief 関数を別名でバインドする
* XTAL_BINDの中で使用する。
* \code
* XTAL_BIND(Test){
*   Xdef_fun_alias(fuge, &hogehogen); // hogehogen関数をfugaという名前でバインドする
* }
* \endcode
*/
#define Xdef_fun_alias(fun, impl) it->define(XTAL_STRING(#fun), XTAL_FUN(impl))

/**
* \brief メソッドを別名でセカンダリーキー付でバインドする
* XTAL_BINDの中で使用する。
* \code
* XTAL_BIND(Test){
*   Xdef_method_alias2(fuge, &hogehogen, Huga); // hogehogen関数をfugaという名前でセカンダリーキーをHugaクラスでバインドする
* }
* \endcode
*/
#define Xdef_fun_alias2(fun, impl, secondary_key) it->define(XTAL_STRING(#fun), ::xtal::cpp_class< secondary_key >(), XTAL_FUN(impl))

/**
* \brief メンバ変数のゲッターをバインドする
*/
#define Xdef_getter(var) it->define(XTAL_STRING(#var), XTAL_GETTER(&Self::var))

/**
* \brief メンバ変数のセッターをバインドする
*/
#define Xdef_setter(var) it->define(XTAL_LONG_LIVED_STRING(XTAL_L("set_") XTAL_L(#var)), XTAL_SETTER(&Self::var))

#endif

#define Xdef_double_dispatch_method(fun) it->def_double_dispatch_method(XTAL_STRING(#fun))

/**
* \brief メンバ変数のゲッターとセッターをバインドする
*/
#define Xdef_var(var) Xdef_getter(var), Xdef_setter(var)

/**
* \brief Xdef_methodやXdef_funなどで定義したメソッドや関数のオプショナル引数を定義する
*/
#define Xparam(name, default_value) it->define_param(XTAL_STRING(#name), default_value)

/**
* \brief メンバを定義する
*/
#define Xdef(name, value) it->define(XTAL_STRING(#name), value)

/**
* \brief メンバをセカンダリキー付きで定義する
*/
#define Xdef2(name, value, secondary_key) it->define(XTAL_STRING(#name), value, ::xtal::cpp_class< secondary_key >())

/**
* \brief クラス定数を定義する
*/
#define Xdef_const(name) it->define(XTAL_STRING(#name), (int_t)Self::name)

/**
* \brief クラス定数をセカンダリキー付きで定義する
*/
#define Xdef_const2(name, secondary_key) it->define(XTAL_STRING(#name), (int_t)Self::name, ::xtal::cpp_class< secondary_key >())

/**
* \brief クラスを他のクラスに登録する
*/
#define Xregister(parent_class) ::xtal::cpp_class< parent_class >()->define(XTAL_LONG_LIVED_STRING(::xtal::CppClassSymbol<Self>::value.name), it)

/**
* \brief クラスを他のクラスに名前を変えて登録する
*/
#define Xregister_alias(parent_class, alias) ::xtal::cpp_class< parent_class >()->define(XTAL_STRING(#alias), it)

#define Xinherit(super_class) it->inherit(::xtal::cpp_class< super_class >())

/**
* \brief バインドするクラスをXtalのクラスから継承できなくする
* Xtalのクラスからはinheritできなくなるが、C++からはinheritできる。
*/
#define Xfinal() it->set_final()

/**
* \brief クラスをC++で定義されたクラスではなく、Xtalで定義されたクラスとみなすようにする
* C++のクラスは二個以上inheritできない、などの制限があるため、
* 名前空間的に使うだけのクラスなどはこれを使ってXtalで定義したクラスとみなすようにすることができる。
*/
#define Xxtal_class() it->set_xtal_class()

/**
* \brief シングルトンマークをつける
*/
#define Xsingleton() it->set_singleton()

/**
* \brief 次から定義するメンバをpublicとする
*/
#define Xpublic() it->set_accessibility(::xtal::KIND_PUBLIC)

/**
* \brief 次から定義するメンバをpublicとする
*/
#define Xprotected() it->set_accessibility(::xtal::KIND_PROTECTED)

/**
* \brief 次から定義するメンバをpublicとする
*/
#define Xprivate() it->set_accessibility(::xtal::KIND_PRIVATE)

//{REPEAT{{
/*
#define Xdef_ctor`n`(#REPEAT_COMMA#A`i`#) it->def_ctor`n`<Self #COMMA_REPEAT#A`i`#>()
*/

#define Xdef_ctor0() it->def_ctor0<Self >()

#define Xdef_ctor1(A0) it->def_ctor1<Self , A0>()

#define Xdef_ctor2(A0, A1) it->def_ctor2<Self , A0, A1>()

#define Xdef_ctor3(A0, A1, A2) it->def_ctor3<Self , A0, A1, A2>()

#define Xdef_ctor4(A0, A1, A2, A3) it->def_ctor4<Self , A0, A1, A2, A3>()

#define Xdef_ctor5(A0, A1, A2, A3, A4) it->def_ctor5<Self , A0, A1, A2, A3, A4>()

#define Xdef_ctor6(A0, A1, A2, A3, A4, A5) it->def_ctor6<Self , A0, A1, A2, A3, A4, A5>()

#define Xdef_ctor7(A0, A1, A2, A3, A4, A5, A6) it->def_ctor7<Self , A0, A1, A2, A3, A4, A5, A6>()

#define Xdef_ctor8(A0, A1, A2, A3, A4, A5, A6, A7) it->def_ctor8<Self , A0, A1, A2, A3, A4, A5, A6, A7>()

#define Xdef_ctor9(A0, A1, A2, A3, A4, A5, A6, A7, A8) it->def_ctor9<Self , A0, A1, A2, A3, A4, A5, A6, A7, A8>()

#define Xdef_ctor10(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9) it->def_ctor10<Self , A0, A1, A2, A3, A4, A5, A6, A7, A8, A9>()

#define Xdef_ctor11(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10) it->def_ctor11<Self , A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10>()

#define Xdef_ctor12(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11) it->def_ctor12<Self , A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11>()

#define Xdef_ctor13(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12) it->def_ctor13<Self , A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12>()

//}}REPEAT}

#define Xdef_serial_ctor() it->def_serial_ctor(ctor<Self>())

}

#endif
