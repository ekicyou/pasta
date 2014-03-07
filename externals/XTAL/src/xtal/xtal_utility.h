/** \file src/xtal/xtal_utility.h
* \brief src/xtal/xtal_utility.h
*/

#ifndef XTAL_UTILITY_H_INCLUDE_GUARD
#define XTAL_UTILITY_H_INCLUDE_GUARD

#pragma once

#include <cstddef>

/**
* @brief xtal namespace
*/
namespace xtal{
enum{
	VERSION1 = 1,
	VERSION2 = 1,
	VERSION3 = 0,
	VERSION4 = 0
};
}

#define XTAL_USE_COMPILED_EMB
//#define XTAL_DEBUG_PRINT

//#define XTAL_ENFORCE_64_BIT
//#define XTAL_USE_THREAD_MODEL_2
//#define XTAL_NO_XPEG
//#define XTAL_NO_ASSERT

/**
* \brief 可能な限り整数の範囲を大きくする
*/
//#define XTAL_USE_LARGE_INT

#ifdef _UNICODE
#	ifndef XTAL_USE_WCHAR
#		define XTAL_USE_WCHAR
#	endif
#endif 

#if !defined(XTAL_USE_COMPUTED_GOTO) && (defined(__GNUC__) || defined(__ARMCC_VERSION))
#define XTAL_USE_COMPUTED_GOTO
#endif

#define XTAL_CAT_(x, y) x ## y
#define XTAL_CAT(x, y) XTAL_CAT_(x, y)
#define XTAL_UNIQUE(x) XTAL_CAT(x, __LINE__)

#if (defined(__GNUC__) && (__GNUC__ > 2 || (__GNUC__ == 2 && __GNUC_MINOR__ >= 96))) || defined(__ARMCC_VERSION)
#define XTAL_PREFETCH(x) __builtin_prefetch(x)
#define XTAL_PREFETCHW(x) __builtin_prefetch(x, 2)
#define XTAL_LIKELY(x) __builtin_expect(!!(x), 1)
#define XTAL_UNLIKELY(x) __builtin_expect(!!(x), 0)
#else
#define XTAL_PREFETCH(x) 
#define XTAL_PREFETCHW(x) 
#define XTAL_LIKELY(x) x
#define XTAL_UNLIKELY(x) x
#endif

#define XTAL_STRUCT_TAIL(x) (((char*)(void*)x)+sizeof(*x))

///////////////////////////////////////////////
// Thread

#ifndef XTAL_NO_THREAD

#if defined(_MSC_VER)

// VCにおけるXTAL_TLS_PTRの実装
#define XTAL_TLS_PTR(x) __declspec(thread) x*

#elif defined(XTAL_USE_PTHREAD_TLS)

// pthreadを使ったXTAL_TLS_PTRの実装
#include <pthread.h>
template<class T>
struct TLSPtr{
	TLSPtr(){ pthread_key_create(&key, 0); }
	~TLSPtr(){ pthread_key_delete(key); }
	void operator =(T* p){ pthread_setspecific(key, p); }
	operator T*(){ return static_cast<T*>(pthread_getspecific(key)); }
	T* operator ->(){ return *this; }
private:
	pthread_key_t key;
	TLSPtr(const TLSPtr&);
	void operator=(const TLSPtr&);
};
#define XTAL_TLS_PTR(x) TLSPtr<x>

#elif defined(__GNUC__) && !defined(__CYGWIN__)
			
// gccにおけるXTAL_TLS_PTRの実装
#define XTAL_TLS_PTR(x) __thread x*

#elif defined(_WIN32)

// win32 apiを使ったXTAL_TLS_PTRの実装
#include <windows.h>
template<class T>
struct TLSPtr{
	TLSPtr(){ key = TlsAlloc(); }
	~TLSPtr(){ TlsFree(key); }
	void operator =(T* p){ TlsSetValue(key, p); }
	operator T*(){ return static_cast<T*>(TlsGetValue(key)); }
	T* operator ->(){ return *this; }
private:
	DWORD key;
	TLSPtr(const TLSPtr&);
	void operator=(const TLSPtr&);
};
#define XTAL_TLS_PTR(x) TLSPtr<x>

#else

#error // XTAL_NO_THREADを定義しない場合、TLSを実現するためのXTAL_TLS_PTR(x)の実装が必要

#endif

#else

// スレッドを使わない場合、単なるポインタ型にする
#define XTAL_TLS_PTR(x) x*

#endif 

/////////////////////////////////////////////////////

#ifdef XTAL_NO_PARSER
#	define XTAL_USE_COMPILED_EMB
#endif

#if !defined(NDEBUG) && (defined(_DEBUG) || defined(DEBUG))
#	define XTAL_DEBUG
#endif

#if defined(XTAL_DEBUG) && !defined(XTAL_NO_ASSERT)
#	define XTAL_ASSERT(expr) assert(expr)
#else
#	define XTAL_ASSERT(expr) (void)0
#endif

#ifdef XTAL_DEBUG
#	define XTAL_NODEFAULT break; default: XTAL_ASSERT(false); break
#else
#	ifdef _MSC_VER
#		define XTAL_NODEFAULT default: __assume(0)
#	else
#		define XTAL_NODEFAULT
#	endif
#endif

#ifdef XTAL_DEBUG
#	define XTAL_DEBUG_ONLY(x) x
#else
#	define XTAL_DEBUG_ONLY(x) 
#endif

#define XTAL_UNUSED_VAR(x) ((void)&x)

#define XTAL_DISALLOW_COPY_AND_ASSIGN(ClassName) ClassName(const ClassName&); void operator=(const ClassName&)

#define XTAL_STATIC_ASSERT(expr) char XTAL_UNIQUE(static_assert_failure)[(expr)] = {0}; XTAL_UNUSED_VAR(XTAL_UNIQUE(static_assert_failure))

#define XTAL_DEFAULT break; default:
#define XTAL_CASE_N(key) break; key
#define XTAL_CASE(key) XTAL_CASE_N(case key:)
#define XTAL_CASE1(key) XTAL_CASE(key)
#define XTAL_CASE2(key, key2) XTAL_CASE1(key) case key2:
#define XTAL_CASE3(key, key2, key3) XTAL_CASE2(key, key2) case key3:
#define XTAL_CASE4(key, key2, key3, key4) XTAL_CASE3(key, key2, key3) case key4:
#define XTAL_CASE5(key, key2, key3, key4, key5) XTAL_CASE4(key, key2, key3, key4) case key5:
#define XTAL_CASE6(key, key2, key3, key4, key5, key6) XTAL_CASE5(key, key2, key3, key4, key5) case key6:
#define XTAL_CASE7(key, key2, key3, key4, key5, key6, key7) XTAL_CASE6(key, key2, key3, key4, key5, key6) case key7:
#define XTAL_CASE8(key, key2, key3, key4, key5, key6, key7, key8) XTAL_CASE7(key, key2, key3, key4, key5, key6, key7) case key8:

#define XTAL_TYPE_SWITCH(value, N) \
	XTAL_STATIC_ASSERT(N==::xtal::TYPE_MAX);\
	switch(XTAL_detail_type(value))

#ifdef __GNUC__
#	define XTAL_NOINLINE __attribute__((noinline)) 
#else
#	define XTAL_NOINLINE
#endif

#if defined(_MSC_VER) && _MSC_VER>=1400
#	pragma warning(disable: 4819)
#	pragma warning(disable: 4127)
#	pragma warning(disable: 4100)
#endif

#define XTAL_LONG_LIVED_STRING(x) (*(::xtal::LongLivedString*)(x))
#define XTAL_LONG_LIVED_STRINGN(x, N) (*(::xtal::LongLivedStringN<N>*)(x))

#ifdef XTAL_USE_WCHAR
#	define XTAL_L(x) L##x
#	define XTAL_STRING(x) XTAL_LONG_LIVED_STRINGN(L##x, sizeof(x)-1)
#else 
#	define XTAL_L(x) x
#	define XTAL_STRING(x) XTAL_LONG_LIVED_STRINGN(x, sizeof(x)-1)
#endif

#if defined(_MSC_VER) || defined(__MINGW__) || defined(__MINGW32__)
#	define XTAL_INT_FMT (sizeof(int_t)==8 ? XTAL_L("I64") : XTAL_L(""))
#else
#	define XTAL_INT_FMT (sizeof(int_t)==8 ? XTAL_L("ll") : XTAL_L(""))
#endif

#ifdef XTAL_USE_WCHAR
#	if defined(_MSC_VER) && _MSC_VER>=1400
#		define XTAL_SPRINTF(buffer, data_size, format, value) swprintf_s(buffer, data_size, format, value)
#	else
#		define XTAL_SPRINTF(buffer, data_size, format, value) std::swprintf(buffer, format, value)
#	endif
#else
#	if defined(_MSC_VER) && _MSC_VER>=1400
#		define XTAL_SPRINTF(buffer, data_size, format, value) sprintf_s(buffer, data_size, format, value)
#	else
#		define XTAL_SPRINTF(buffer, data_size, format, value) std::sprintf(buffer, format, value)
#	endif
#endif

namespace xtal{

// 最低限のメタプログラミング下地

template<bool>
struct IfHelper{
	template<class T, class U>
	struct inner{
		typedef T type;
	};
};

template<>
struct IfHelper<false>{
	template<class T, class U>
	struct inner{
		typedef U type;
	};
};

template<bool N, class T, class U>
struct If{
	typedef typename IfHelper<N>::template inner<T, U>::type type;
};

typedef char (&YesType)[2];
typedef char (&NoType)[1];

template<int>
struct TypeIntType{};

template<class T, class U>
struct Convertible{
	static YesType test(U);
	static NoType test(...);
	static T makeT();

	enum{ value = sizeof(test(makeT()))==sizeof(YesType) };
};

template<class U>
struct IsInheritedFuncs{
	static YesType test(const U*);
	static NoType test(...);
};

template<class T, class U>
struct IsInherited{
	static T* makeT();

	enum{ 
		// 完全型チェック
		CHECK = sizeof(T) + sizeof(U),
		
		value = sizeof(IsInheritedFuncs<U>::test(makeT()))==sizeof(YesType) 
	};
};

template<int N>
struct I2T{
	enum{ value = N };
};

template<class T>
struct T2T{
	typedef T type;
};

struct Other{
	template<class T>
	Other(const T&){}
};

template<class T, class U> struct IsSame{ enum{ value = 0 }; };
template<class T> struct IsSame<T, T>{ enum{ value = 1 }; };

template<class T> struct IsVoid{ enum{ value = 0 }; };
template<> struct IsVoid<void>{ enum{ value = 1 }; };

template<class T> struct IsNotVoid{ enum{ value = 1 }; };
template<> struct IsNotVoid<void>{ enum{ value = 0 }; };

template<class T>
struct Ref{
	T& ref;
	Ref(T& ref):ref(ref){};
	operator T&() const{ return ref; }
};

template<class T>
inline Ref<T> ref(T& r){
	return Ref<T>(r);
}

template<class T, class U>
struct NumericCalcResultType{
	typedef typename If<1, T, U>::type type;
};

template<int N>
struct SelectType{
	typedef typename If<sizeof(signed char)==N,
		signed char,
		typename If<sizeof(signed short)==N, 
			signed short int,
			typename If<sizeof(signed int)==N,
				signed int,
				typename If<sizeof(signed long)==N,
					signed long int,
					typename If<sizeof(signed long long)==N,
						signed long long int,
						void
					>::type
				>::type
			>::type
		>::type
	>::type int_t;

	typedef typename If<sizeof(unsigned char)==N,
		unsigned char,
		typename If<sizeof(unsigned short)==N, 
			unsigned short int,
			typename If<sizeof(unsigned int)==N,
				unsigned int,
				typename If<sizeof(unsigned long)==N,
					unsigned long int,
					typename If<sizeof(unsigned long long)==N,
						unsigned long long int,
						void
					>::type
				>::type
			>::type
		>::type
	>::type uint_t;

	typedef typename If<sizeof(float)==N,
		float,
		typename If<sizeof(double)==N, 
			double,
			void
		>::type
	>::type float_t;
};

/// 1-byte uint
typedef SelectType<1>::uint_t u8;

/// 2-byte uint
typedef SelectType<2>::uint_t u16;
	
/// 4-byte uint
typedef SelectType<4>::uint_t u32;

/// 8-byte uint
typedef SelectType<8>::uint_t u64;

/// 1-byte int
typedef SelectType<1>::int_t i8;

/// 2-byte int
typedef SelectType<2>::int_t i16;
	
/// 4-byte int
typedef SelectType<4>::int_t i32;

/// 8-byte int
typedef SelectType<8>::int_t i64;

/// 4-byte float
typedef SelectType<4>::float_t f32;

/// 8-byte float
typedef SelectType<8>::float_t f64;

#ifdef XTAL_ENFORCE_64_BIT

/// int
typedef SelectType<8>::int_t int_t;
	
/// float
typedef SelectType<8>::float_t float_t;
	
/// uint
typedef SelectType<8>::uint_t uint_t;

#else

/// int
typedef SelectType<sizeof(void*)>::int_t int_t;

/// uint
typedef SelectType<sizeof(void*)>::uint_t uint_t;

/// float
typedef SelectType<sizeof(void*)>::float_t float_t;
	
/// int
typedef SelectType<sizeof(void*)>::int_t iptr_t;

/// uint
typedef SelectType<sizeof(void*)>::uint_t uptr_t;

#endif

/// byte
typedef SelectType<1>::uint_t byte_t;

using std::size_t;
using std::ptrdiff_t;

#ifdef XTAL_USE_WCHAR

typedef wchar_t char_t;

#else

typedef char char_t;

#endif

/// unsigned char_t
typedef SelectType<sizeof(char_t)>::uint_t uchar_t;

template<class T>
struct IsFloat{ enum{ value = 0 }; };

template<>
struct IsFloat<float_t>{ enum{ value = 1 }; };


template<class T>
struct AlignCalc{
	char c;
	T t;

	AlignCalc();
};

template<class T>
struct AlignOf{
	enum{
		TSIZE = sizeof(T),
		ASIZE = sizeof(AlignCalc<T>) - TSIZE,
		value = ASIZE < TSIZE ? ASIZE : TSIZE
	};
};

#if defined(_MSC_VER)

#define XTAL_ALIGN_VAR(type, name, A) __declspec(align(A)) type name

#elif defined(__GNUC__)

#define XTAL_ALIGN_VAR(type, name, A) type __attribute__((aligned(A))) name;

#endif

struct align16_t{ XTAL_ALIGN_VAR(int, v, 16); };
struct align32_t{ XTAL_ALIGN_VAR(int, v, 32); };
struct align64_t{ XTAL_ALIGN_VAR(int, v, 64); };
struct align128_t{ XTAL_ALIGN_VAR(int, v, 128); };

template<int N>
struct SelectAlignType{
	typedef typename If<sizeof(unsigned char)==N,
		unsigned char,
		typename If<sizeof(unsigned short)==N, 
			unsigned short int,
			typename If<sizeof(unsigned int)==N,
				unsigned int,
				typename If<sizeof(unsigned long)==N,
					unsigned long int,
					typename If<sizeof(unsigned long long)==N,
						unsigned long long int,
						typename If<sizeof(double)==N,
							double,
							typename If<sizeof(long double)==N,
								long double,
								typename If<16==N,
									align16_t,
									void
								>::type
							>::type
						>::type
					>::type
				>::type
			>::type
		>::type
	>::type type;	
};

template<class T>
struct AlignBuffer{
	union{
		typename SelectAlignType<AlignOf<T>::value>::type a;
		char buffer[sizeof(T)];
	};
};

//////////////////////////////////////////////////

/**
* \brief 整数値のアライメント調整
*/
template<class T>
inline T align(T v, int N){
	return (v+(N-1)) & ~(T)(N-1);
}

/**
* \brief ポインタ値のアライメント調整
*/
template<class T>
inline T* align_p(T* v, int N){
	return (T*)align((uptr_t)v, N);
}

/**
* \brief 整数値の2の累乗のアライメント調整
*/
template<class T>
inline T align_2(T v){
	v-=1;
	v|=v>>1;
	v|=v>>2;
	v|=v>>4;
	v|=v>>8;
	v|=v>>16;
//	v|=v>>32;
	return v+1;
}

/**
* \brief 静的にビットの数を数えるメタ関数
*/
template<int N>
class static_count_bits{
	enum{
		N1 = (N & 0x55555555) + (N >> 1 & 0x55555555),
		N2 = (N1 & 0x33333333) + (N1 >> 2 & 0x33333333),
		N3 = (N2 & 0x0f0f0f0f) + (N2 >> 4 & 0x0f0f0f0f),
		N4 = (N3 & 0x00ff00ff) + (N3 >> 8 & 0x00ff00ff)
	};
public:	
	enum{
		value = (N4 & 0x0000ffff) + (N4>>16 & 0x0000ffff)	
	};
};

/**
* \brief 静的にNumber of Training Zero (NTZ)を計算するメタ関数
*/
template<int N>
struct static_ntz{
	enum{
		value = static_count_bits<((N&(-N))-1)>::value
	};
};

//////////////////////////////////////////////////
//


class Any;

template<class T>
class SmartPtr;

template<class T>
class BasePtr;

template<> class SmartPtr<Any>;
typedef SmartPtr<Any> AnyPtr;

class Environment;

class Null;
class Undefined;
class Array;
class Map;
class Set;
class Stream;
class MemoryStream;
class PointerStream;
class StringStream;
class FileStream;
class CompressEncoder;
class CompressDecoder;
class Fun;
class Method;
class Fiber;
class InstanceVariableGetter;
class InstanceVariableSetter;
class Lambda;
class String;
class ID;
class Code;
class Arguments;
class VMachine;
class NativeMethod;
class Frame;
class Class;
class Lib;
class Thread;
class Mutex;
class IntRange;
class FloatRange;
class ChRange;
class DoubleDispatchMethod;
class DoubleDispatchFun;
class Values;
class Exception;
class Text;
class TreeNode;

typedef SmartPtr<Null> NullPtr;
typedef SmartPtr<Undefined> UndefinedPtr;
typedef SmartPtr<Array> ArrayPtr;
typedef SmartPtr<Map> MapPtr;
typedef SmartPtr<Set> SetPtr;
typedef SmartPtr<Stream> StreamPtr;
typedef SmartPtr<MemoryStream> MemoryStreamPtr;
typedef SmartPtr<PointerStream> PointerStreamPtr;
typedef SmartPtr<StringStream> StringStreamPtr;
typedef SmartPtr<FileStream> FileStreamPtr;
typedef SmartPtr<CompressEncoder> CompressEncoderPtr;
typedef SmartPtr<CompressDecoder> CompressDecoderPtr;
typedef SmartPtr<Fun> FunPtr;
typedef SmartPtr<Method> MethodPtr;
typedef SmartPtr<Fiber> FiberPtr;
typedef SmartPtr<InstanceVariableGetter> InstanceVariableGetterPtr;
typedef SmartPtr<InstanceVariableSetter> InstanceVariableSetterPtr;
typedef SmartPtr<String> StringPtr;
typedef SmartPtr<ID> IDPtr;
typedef SmartPtr<Code> CodePtr;
typedef SmartPtr<Arguments> ArgumentsPtr;
typedef SmartPtr<VMachine> VMachinePtr;
typedef SmartPtr<NativeMethod> NativeFunPtr;
typedef SmartPtr<Frame> FramePtr;
typedef SmartPtr<Class> ClassPtr;
typedef SmartPtr<Lib> LibPtr;
typedef SmartPtr<Thread> ThreadPtr;
typedef SmartPtr<Mutex> MutexPtr;
typedef SmartPtr<IntRange> IntRangePtr;
typedef SmartPtr<FloatRange> FloatRangePtr;
typedef SmartPtr<ChRange> ChRangePtr;
typedef SmartPtr<DoubleDispatchMethod> DoubleDispatchMethodPtr;
typedef SmartPtr<DoubleDispatchFun> DoubleDispatchFunPtr;
typedef SmartPtr<Values> ValuesPtr;
typedef SmartPtr<Exception> ExceptionPtr;
typedef SmartPtr<Text> TextPtr;
typedef SmartPtr<TreeNode> TreeNodePtr;

class Base;
class RefCountingBase;
class Any;

class Int;
class Float;
class Undefined;
class Bool;

class Visitor;
class InstanceVariables;

namespace xpeg{

class MatchResult;
typedef SmartPtr<MatchResult> MatchResultPtr;

class Executor;
typedef SmartPtr<Executor> ExecutorPtr;

struct Element;
typedef SmartPtr<Element> ElementPtr;

struct Trans;
typedef SmartPtr<Trans> TransPtr;

struct NFA;
typedef SmartPtr<NFA> NFAPtr;

}

struct CppClassSymbolData;

template<class T>
struct CppClassSymbol;

struct param_types_holder_n;
struct VirtualMembers;

enum InheritedEnum{
	INHERITED_BASE,
	INHERITED_RCBASE,
	INHERITED_ANY,
	INHERITED_ANYPTR,
	INHERITED_OTHER
};

template<class T>
struct InheritedN{
	enum{
		value = 
			IsInherited<T, Base>::value ? INHERITED_BASE : 
			IsInherited<T, RefCountingBase>::value ? INHERITED_RCBASE : 
			IsInherited<T, AnyPtr>::value ? INHERITED_ANYPTR :
			IsInherited<T, Any>::value ? INHERITED_ANY : INHERITED_OTHER
	};
};

template<class T> struct InheritedN<T&> : public InheritedN<T>{};
template<class T> struct InheritedN<T*> : public InheritedN<T>{};
template<class T> struct InheritedN<const T> : public InheritedN<T>{};
template<class T> struct InheritedN<volatile T> : public InheritedN<T>{};


template<class T, int N = -1>
struct InheritedCast;

template<class T>
struct InheritedCast<T, -1> : public InheritedCast<T, InheritedN<T>::value>{};

template<class T>
struct InheritedCast<T, INHERITED_RCBASE>{
	static const RefCountingBase* cast(const T* p){ return p; }
};

template<class T>
struct InheritedCast<T, INHERITED_BASE>{
	static const Base* cast(const T* p){ return p; }
};

template<class T>
struct InheritedCast<T, INHERITED_ANY>{
	static const Any* cast(const T* p){ return p; }
};

template<class T>
struct InheritedCast<T, INHERITED_ANYPTR>{
	static const AnyPtr* cast(const T* p){ return p; }
};

template<class T>
struct InheritedCast<T, INHERITED_OTHER>{
	static const T* cast(const T* p){ return p; }
};

////////////////////////////////////////////////

const uint_t hash_m = 0x5bd1e995;
const uint_t hash_seed = 0xdeadbeef;
const uint_t hash_r = 24;

uint_t string_length(const char_t* str);
uint_t string_data_size(const char_t* str);
uint_t string_hashcode(const char_t* str, uint_t size);
void string_data_size_and_hashcode(const char_t* str, uint_t& size, uint_t& hash);
void string_copy(char_t* a, const char_t* b, uint_t size);
bool string_is_ch(const char_t* str, uint_t size);
int_t string_compare(const char_t* a, uint_t asize, const char_t* b, uint_t bsize);

struct LongLivedString{
	const char_t* str() const{ return (char_t*)this; }
	uint_t size() const{ return string_data_size((char_t*)this); }
};

template<int N>
struct LongLivedStringN : public LongLivedString{
	uint_t size() const{ return N; }	
};

template<class T>
inline const SmartPtr<T>&
unchecked_ptr_cast(const AnyPtr& a);

//
////////////////////////////////////////////

/**
* \brief プリミティブな型の種類
*/
enum PrimitiveType{
	TYPE_NULL,
	TYPE_UNDEFINED,

	TYPE_FALSE,
	// ここから上は、ifなどで偽と評価される

	// ここから下は、値によらず真と評価される
	TYPE_TRUE,
	
	TYPE_INT,
	TYPE_FLOAT,

	TYPE_IMMEDIATE_VALUE,

	TYPE_POINTER,
	TYPE_STATELESS_NATIVE_METHOD,

	TYPE_IVAR_GETTER,
	TYPE_IVAR_SETTER,
	
	TYPE_SMALL_STRING,
	TYPE_LONG_LIVED_STRING,
	TYPE_INTERNED_STRING,

	TYPE_PADDING_0, // 14
	TYPE_PADDING_1, // 15

	// ここから上はimmutableな値型である
};

enum PrimitiveTypeRef{
	TYPE_REF_SHIFT = 4,

	// ここから下は参照型である
	TYPE_BASE = 16,

	TYPE_STRING,
	
	TYPE_ARRAY,
	TYPE_VALUES,
	TYPE_TREE_NODE,

	TYPE_NATIVE_METHOD,
	TYPE_NATIVE_FUN,

	TYPE_METHOD,
	TYPE_FUN,
	TYPE_LAMBDA,
	TYPE_FIBER,

	/*
	TYPE_FRAME,
	TYPE_CLASS,
	*/

	TYPE_MAX,

	TYPE_SHIFT = 5,
	TYPE_MASK = (1<<TYPE_SHIFT)-1
};

enum{
	SMALL_STRING_MAX = sizeof(void*) / sizeof(char_t)
};

/**
* \brief ブロックの種類
*/
enum BlockKind{
	KIND_BLOCK,
	KIND_CLASS,
	KIND_SINGLETON,

	KIND_FUN,
	KIND_LAMBDA,
	KIND_METHOD,
	KIND_FIBER
};

/**
* \brief 文字列の種類
*/
enum StringKind{
	KIND_STRING,
	KIND_TEXT,
	KIND_FORMAT
};

/**
* \brief 可触性の種類
*/
enum AccessibilityKind{
	/**
	* \biref public
	*/
	KIND_PUBLIC = 0,

	/**
	* \brief protected
	*/
	KIND_PROTECTED = 1<<0,

	/**
	* \brief private
	*/
	KIND_PRIVATE = 1<<1,

	
	KIND_DEFAULT = -1,
};

/**
* \brief ブレークポイントの種類
*/
enum BreakPointKind{
	/**
	* \brief ブレークポイント
	*/
	BREAKPOINT,
	BREAKPOINT_INNER_LINE, // 内部で使用する用
	BREAKPOINT_INNER_RETURN, // 内部で使用する用
	BREAKPOINT_INNER_CALL, // 内部で使用する用

	/**
	* \brief ラインごとのブレークポイント
	*/
	BREAKPOINT_LINE,

	/**
	* \brief 関数リターン時ブレークポイント
	*/
	BREAKPOINT_RETURN,

	/**
	* \brief 関数呼び出し時ブレークポイント
	*/
	BREAKPOINT_CALL,

	/**
	* \brief 例外創出時ブレークポイント
	*/
	BREAKPOINT_THROW,

	/**
	* \brief 表明時ブレークポイント
	*/
	BREAKPOINT_ASSERT,

	/**
	* \brief ラインごとの軽量ブレークポイント
	*/
	BREAKPOINT_LINE_LIGHT_WEIGHT,

	/**
	* \brief 関数リターン時軽量ブレークポイント
	*/
	BREAKPOINT_RETURN_LIGHT_WEIGHT,

	/**
	* \brief 関数呼び出し時軽量ブレークポイント
	*/
	BREAKPOINT_CALL_LIGHT_WEIGHT,

	/**
	* \brief ラインごとのプロファイル用軽量ブレークポイント
	*/
	BREAKPOINT_LINE_PROFILE,

	/**
	* \brief 関数リターン時プロファイル用軽量ブレークポイント
	*/
	BREAKPOINT_RETURN_PROFILE,

	/**
	* \brief 関数呼び出しプロファイル用時軽量ブレークポイント
	*/
	BREAKPOINT_CALL_PROFILE,

	BREAKPOINT_MAX
};

/**
* \brief 区間の種類
*/
enum RangeKind{
	/**
	* \brief 閉区間[left, right]
	*/
	RANGE_CLOSED = (0<<1) | (0<<0),

	/**
	* \brief 左開右閉区間 [left, right)
	*/
	RANGE_LEFT_CLOSED_RIGHT_OPEN = (0<<1) | (1<<0),

	/**
	* \brief 左開右閉区間 (left, right]
	*/
	RANGE_LEFT_OPEN_RIGHT_CLOSED = (1<<1) | (0<<0),

	/**
	* \brief 開区間 (left, right)
	*/
	RANGE_OPEN = (1<<1) | (1<<0)
};

/**
* \brief スコープ情報
*/
struct ScopeInfo{
	ScopeInfo()
		:pc(0), kind(0), flags(0), variable_identifier_offset(0), variable_size(0){}

	u32 pc;
	u8 kind;
	u8 flags;
	u16 variable_identifier_offset;
	u16 variable_size;
	u16 variable_offset;

	enum{
		FLAG_SCOPE_CHAIN = 1<<0,
		FLAG_USED_BIT = 1
	};
};

/**
* \brief クラススコープ情報
*/
struct ClassInfo : public ScopeInfo{
	ClassInfo(u16 size = 0, u16 offset = 0)
		:instance_variable_size(size), instance_variable_identifier_offset(offset), name_number(0), mixins(0){}

	u16 instance_variable_size;
	u16 instance_variable_identifier_offset;
	u16 name_number;
	u8 mixins;
};

/**
* \brief 関数スコープ情報
*/
struct FunInfo : public ScopeInfo{
	FunInfo()
		:max_stack(256), max_variable(0), name_number(0), min_param_count(0), max_param_count(0){}

	u16 max_stack;
	u16 max_variable;
	u16 name_number;
	u8 min_param_count;
	u8 max_param_count;

	enum{
		FLAG_EXTENDABLE_PARAM = 1<<(ScopeInfo::FLAG_USED_BIT+0),

		FLAG_USED_BIT = ScopeInfo::FLAG_USED_BIT+1
	};
};

/**
* \brief 例外スコープ情報
*/
struct ExceptInfo{
	ExceptInfo()
		:catch_pc(0), finally_pc(0), end_pc(0){}

	u32 catch_pc;
	u32 finally_pc;
	u32 end_pc;
};

extern ScopeInfo empty_scope_info;
extern ClassInfo empty_class_info;
extern FunInfo empty_xfun_info;
extern ExceptInfo empty_except_info;
extern InstanceVariables empty_instance_variables;

//////////////////////////////////////////

/**
* \brief null
*/
extern NullPtr null;

/**
* \brief 未定義値
*/
extern UndefinedPtr undefined;

/**
* \brief 空文字列
*/
extern IDPtr empty_id;

/**
* \brief 空文字列
*/
extern StringPtr empty_string;
	
//////////////////////////////////////////////

}

#endif // XTAL_UTILITY_H_INCLUDE_GUARD
