/** \file src/xtal/xtal_macro.h
* \brief src/xtal/xtal_macro.h
*/

#ifndef XTAL_MACRO_H_INCLUDE_GUARD
#define XTAL_MACRO_H_INCLUDE_GUARD

#pragma once

/**
* \hideinitializer
* \brief foreachを簡単に記述するためのマクロ
*
* \code
* Xfor(value, array){
*   // use value
* }
* \endcode
*/
#define Xfor(var, target) \
	if(bool not_end = true)\
	for(::xtal::BlockValueHolder1 block_value_holder(target, not_end); not_end; not_end=false)\
	for(const ::xtal::AnyPtr& var = block_value_holder.values[0]; not_end; not_end=false)\
	for(bool first_step=not_end=::xtal::block_next(block_value_holder, true); XTAL_UNUSED_VAR(var), XTAL_UNUSED_VAR(first_step), not_end; not_end=::xtal::block_next(block_value_holder, false), first_step=false)

/**
* \hideinitializer
* \brief foreachを簡単に記述するためのマクロ
*
* \code
* Xfor2(key, value, map){
*   // use key and value
* }
* \endcode
*/
#define Xfor2(var1, var2, target) \
	if(bool not_end = true)\
	for(::xtal::BlockValueHolder2 block_value_holder(target, not_end); not_end; not_end=false)\
	for(const ::xtal::AnyPtr& var1 = block_value_holder.values[0], &var2 = block_value_holder.values[1]; not_end; not_end=false)\
	for(bool first_step=not_end=::xtal::block_next(block_value_holder, true); XTAL_UNUSED_VAR(var1), XTAL_UNUSED_VAR(var2), XTAL_UNUSED_VAR(first_step), not_end; not_end=::xtal::block_next(block_value_holder, false), first_step=false)

/**
* \hideinitializer
* \brief foreachを簡単に記述するためのマクロ
*
* \code
* Xfor3(v1, v2, v3, hoge.send("each3")){
*   // use v1, v2 and v3
* }
* \endcode
*/
#define Xfor3(var1, var2, var3, target) \
	if(bool not_end = true)\
	for(::xtal::BlockValueHolder3 block_value_holder(target, not_end); not_end; not_end=false)\
	for(const ::xtal::AnyPtr& var1 = block_value_holder.values[0], &var2 = block_value_holder.values[1], &var3 = block_value_holder.values[2]; not_end; not_end=false)\
	for(bool first_step=not_end=::xtal::block_next(block_value_holder, true); XTAL_UNUSED_VAR(var1), XTAL_UNUSED_VAR(var2), XTAL_UNUSED_VAR(var3), XTAL_UNUSED_VAR(first_step), not_end; not_end=::xtal::block_next(block_value_holder, false), first_step=false)


/**
* \hideinitializer
* \brief foreachを簡単に記述するためのマクロ
*
* 各要素を受け取る変数に型をつけることが出来る。
* \code
* Xfor_cast(StringPtr value, array){
*   // valueはStringPtrにキャストされている
* }
* else{
*   // elseをつけると、キャストに失敗したら実行される
* }
* \endcode
*/
#define Xfor_cast(var, target) \
	if(bool not_end = true)\
	for(::xtal::BlockValueHolder1 block_value_holder(target, not_end); not_end; not_end=false)\
	for(bool first_step=not_end=::xtal::block_next(block_value_holder, true); XTAL_UNUSED_VAR(first_step), not_end; not_end=::xtal::block_next(block_value_holder, false), first_step=false)\
	if(var = ::xtal::tricky_cast(block_value_holder.values[0], (void (*)(var##e))0))

/**
* \hideinitializer
* \brief foreachを簡単に記述するためのマクロ
*
* 各要素を受け取る変数に型をつけることが出来る。
* \code
* Xfor_unchecked_cast(StringPtr value, array){
*   // valueはStringPtrにキャストされている
* }
* else{
*   // elseをつけると、キャストに失敗したら実行される
* }
* \endcode
*/
#define Xfor_unchecked_cast(var, target) \
	if(bool not_end = true)\
	for(::xtal::BlockValueHolder1 block_value_holder(target, not_end); not_end; not_end=false)\
	for(bool first_step=not_end=::xtal::block_next(block_value_holder, true); XTAL_UNUSED_VAR(var), XTAL_UNUSED_VAR(first_step), not_end; not_end=::xtal::block_next(block_value_holder, false), first_step=false)\
	if(var = ::xtal::tricky_unchecked_cast(block_value_holder.values[0], (void (*)(var##e))0))

/**
* \hideinitializer
* \brief textを簡単に記述するためのマクロ
*
* \code
* TextPtr text = Xt("Text");
* \endcode
*/
#define Xt(txt) ::xtal::text(XTAL_STRING(txt)) 

/**
* \brief formatを簡単に記述するためのマクロ
*
* \hideinitializer
* \code
* FormatPtr fmt = Xf("Text %d %s");
* \endcode
*/
#define Xf(txt) ::xtal::format(XTAL_STRING(txt)) 

#ifndef XTAL_NO_PARSER

/**
* \hideinitializer
* \brief Xtalのソースを簡単に記述するためのマクロ
*
* \code
* AnyPtr src = Xsrc((
*   return [0, 2, 3, 4];
* ));
* \endcode
*/
#define Xsrc(text) ::xtal::source(XTAL_L(#text)+1, sizeof(XTAL_L(#text))/sizeof(::xtal::char_t)-3)

#endif

#ifdef XTAL_USE_COMPILED_EMB

#define Xemb(text, compiled_text) ::xtal::exec_compiled_source(compiled_text, sizeof(compiled_text)-1)

#else

#define Xemb(text, compiled_text) ::xtal::exec_source(XTAL_L(#text)+1, sizeof(XTAL_L(#text))/sizeof(::xtal::char_t)-3)

#endif

#if defined(XTAL_DEBUG)

/*
* \hideinitializer
* \brief インターンされた文字列を簡単に記述するためのマクロ
*
* \code
* IDPtr id = Xid(test);
* \endcode
*/
#define Xid(x) ::xtal::intern(XTAL_STRING(#x))

#else

#define Xid(x) ::xtal::intern(XTAL_STRING(#x))

#endif

#define Xid2(x) XTAL_L(#x)

/*
* \hideinitializer
* \brief 文字列リテラル
*
* \code
* StringPtr str = Xs("This is a pen.");
* \endcode
*/
#define Xs(x) XTAL_STRING(x)


#define Xnamed(name, value) ::xtal::Named(Xid(name), value)


#endif // XTAL_MACRO_H_INCLUDE_GUARD
