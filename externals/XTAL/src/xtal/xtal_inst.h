/** \file src/xtal/xtal_inst.h
* \brief src/xtal/xtal_inst.h
*/

#ifndef XTAL_INST_H_INCLUDE_GUARD
#define XTAL_INST_H_INCLUDE_GUARD

#pragma once

namespace xtal{

typedef u16 inst_t;
typedef i16 address16;

StringPtr make_inst_string(const LongLivedString& InstName);

StringPtr make_inst_string(const LongLivedString& InstName, 
						const LongLivedString& MemberName1, int_t MemberValue1);

StringPtr make_inst_string(const LongLivedString& InstName, 
						const LongLivedString& MemberName1, int_t MemberValue1,
						const LongLivedString& MemberName2, int_t MemberValue2);

StringPtr make_inst_string(const LongLivedString& InstName, 
						const LongLivedString& MemberName1, int_t MemberValue1,
						const LongLivedString& MemberName2, int_t MemberValue2,
						const LongLivedString& MemberName3, int_t MemberValue3);
						
StringPtr make_inst_string(const LongLivedString& InstName, 
						const LongLivedString& MemberName1, int_t MemberValue1,
						const LongLivedString& MemberName2, int_t MemberValue2,
						const LongLivedString& MemberName3, int_t MemberValue3,
						const LongLivedString& MemberName4, int_t MemberValue4);

StringPtr make_inst_string(const LongLivedString& InstName, 
						const LongLivedString& MemberName1, int_t MemberValue1,
						const LongLivedString& MemberName2, int_t MemberValue2,
						const LongLivedString& MemberName3, int_t MemberValue3,
						const LongLivedString& MemberName4, int_t MemberValue4,
						const LongLivedString& MemberName5, int_t MemberValue5);

StringPtr inspect_range(const CodePtr& code, const inst_t* start, const inst_t* end);

int_t inst_size(uint_t no);

inline int_t inst_inspect_i8(int value, const inst_t*, const CodePtr&){ return (int_t)value; }
inline int_t inst_inspect_u8(int value, const inst_t*, const CodePtr&){ return (int_t)value; }
inline int_t inst_inspect_i16(int value, const inst_t*, const CodePtr&){ return (int_t)value; }
inline int_t inst_inspect_u16(int value, const inst_t*, const CodePtr&){ return (int_t)value; }
int_t inst_inspect_address16(int value, const inst_t*, const CodePtr&);


#define XTAL_ALIGN(V,N) ((V+(N-1))&~(N-1))

#define XTAL_op_u16(A,B,x) (int)((*((x)+A)))
#define XTAL_op_i16(A,B,x) (int)((i16)(u16)(XTAL_op_u16(A,B,x)))
#define XTAL_op_address16(A,B,x) XTAL_op_i16(A,B,x)
#define XTAL_op_u8(A,B,x) (int)((u8)((XTAL_op_u16(A,B,x))>>(B*8)))
#define XTAL_op_i8(A,B,x) (int)((i8)(u8)XTAL_op_u8(A,B,x))

#define XTAL_op(Type, A, B, x) XTAL_op_##Type(A,B,x) 

#define XTAL_opc(x) ((*(x)) & 0xff) 

inline void XTAL_set_op_u16(int A, int B, inst_t* x, int value){
	x[A] = (u16)value;
	if((int)(XTAL_op_u16(A, B, x))!=value){ 
		*x = 0xff; 
	}
}

inline void XTAL_set_op_i16(int A, int B, inst_t* x, int value){
	x[A] = (u16)(i16)value;
	if((int)(XTAL_op_i16(A, B, x))!=value){ 
		*x = 0xff; 
	}
}

inline void XTAL_set_op_address16(int A, int B, inst_t* x, int value){
	XTAL_set_op_i16(A, B, x, value);
}

inline void XTAL_set_op_u8(int A, int B, inst_t* x, int value){
	x[A] = (x[A]&(~(0xff<<(B*8)))) | (((u8)value)<<(B*8));
	if((int)(XTAL_op_u8(A, B, x))!=value){ 
		*x = 0xff; 
	}
}

inline void XTAL_set_op_i8(int A, int B, inst_t* x, int value){
	x[A] = (x[A]&(~(0xff<<(B*8)))) | (((u8)(i8)value)<<(B*8));
	if((int)(XTAL_op_i8(A, B, x))!=value){ 
		*x = 0xff; 
	}
}

#define XTAL_set_op(Type, A, B, x, value) XTAL_set_op_##Type(A, B, x, value) 

#define XTAL_DEF_INST_0(N, InstName) \
	struct InstName{\
		enum{ \
			NUMBER = N,\
			BSIZE = XTAL_ALIGN(0+sizeof(u8), sizeof(inst_t)),\
			ISIZE = BSIZE/sizeof(inst_t),\
		};\
\
		static void set(inst_t* x){\
			*x = N;\
		}\
\
		static StringPtr inspect(const inst_t* x, const CodePtr& code){\
			return make_inst_string(\
					XTAL_STRING(#InstName));\
		}\
	}

#define XTAL_DEF_INST_1(N, InstName, Type1, Name1) \
	struct InstName{\
		enum{ \
			NUMBER = N,\
			OFFSET_##Name1 = XTAL_ALIGN(0+sizeof(u8), sizeof(Type1)), A1 = OFFSET_##Name1/sizeof(inst_t), B1 = OFFSET_##Name1%sizeof(inst_t),\
			BSIZE = XTAL_ALIGN(OFFSET_##Name1+sizeof(Type1), sizeof(inst_t)),\
			ISIZE = BSIZE/sizeof(inst_t),\
		};\
\
		static void set(inst_t* x, int m1){\
			*x = N;\
			XTAL_set_op(Type1, A1, B1, x, m1);\
		}\
\
		static int Name1(const inst_t* x){ return XTAL_op(Type1, A1, B1, x); } static void set_##Name1(inst_t* x, int v){ XTAL_set_op(Type1, A1, B1, x, v); }\
\
		static StringPtr inspect(const inst_t* x, const CodePtr& code){\
			return make_inst_string(\
					XTAL_STRING(#InstName),\
					XTAL_STRING(#Name1), inst_inspect_##Type1(Name1(x), x, code));\
		}\
	}

#define XTAL_DEF_INST_2(N, InstName, Type1, Name1, Type2, Name2) \
	struct InstName{\
		enum{ \
			NUMBER = N,\
			OFFSET_##Name1 = XTAL_ALIGN(0+sizeof(u8), sizeof(Type1)), A1 = OFFSET_##Name1/sizeof(inst_t), B1 = OFFSET_##Name1%sizeof(inst_t),\
			OFFSET_##Name2 = XTAL_ALIGN(OFFSET_##Name1+sizeof(Type1), sizeof(Type2)), A2 = OFFSET_##Name2/sizeof(inst_t), B2 = OFFSET_##Name2%sizeof(inst_t),\
			BSIZE = XTAL_ALIGN(OFFSET_##Name2+sizeof(Type2), sizeof(inst_t)),\
			ISIZE = BSIZE/sizeof(inst_t),\
		};\
\
		static void set(inst_t* x, int m1, int m2){\
			*x = N;\
			XTAL_set_op(Type1, A1, B1, x, m1);\
			XTAL_set_op(Type2, A2, B2, x, m2);\
		}\
\
		static int Name1(const inst_t* x){ return XTAL_op(Type1, A1, B1, x); } static void set_##Name1(inst_t* x, int v){ XTAL_set_op(Type1, A1, B1, x, v); }\
		static int Name2(const inst_t* x){ return XTAL_op(Type2, A2, B2, x); } static void set_##Name2(inst_t* x, int v){ XTAL_set_op(Type2, A2, B2, x, v); }\
\
		static StringPtr inspect(const inst_t* x, const CodePtr& code){\
			return make_inst_string(\
					XTAL_STRING(#InstName),\
					XTAL_STRING(#Name1), inst_inspect_##Type1(Name1(x), x, code),\
					XTAL_STRING(#Name2), inst_inspect_##Type2(Name2(x), x, code));\
		}\
	}

#define XTAL_DEF_INST_3(N, InstName, Type1, Name1, Type2, Name2, Type3, Name3) \
	struct InstName{\
		enum{ \
			NUMBER = N,\
			OFFSET_##Name1 = XTAL_ALIGN(0+sizeof(u8), sizeof(Type1)), A1 = OFFSET_##Name1/sizeof(inst_t), B1 = OFFSET_##Name1%sizeof(inst_t),\
			OFFSET_##Name2 = XTAL_ALIGN(OFFSET_##Name1+sizeof(Type1), sizeof(Type2)), A2 = OFFSET_##Name2/sizeof(inst_t), B2 = OFFSET_##Name2%sizeof(inst_t),\
			OFFSET_##Name3 = XTAL_ALIGN(OFFSET_##Name2+sizeof(Type2), sizeof(Type3)), A3 = OFFSET_##Name3/sizeof(inst_t), B3 = OFFSET_##Name3%sizeof(inst_t),\
			BSIZE = XTAL_ALIGN(OFFSET_##Name3+sizeof(Type3), sizeof(inst_t)),\
			ISIZE = BSIZE/sizeof(inst_t),\
		};\
\
		static void set(inst_t* x, int m1, int m2, int m3){\
			*x = N;\
			XTAL_set_op(Type1, A1, B1, x, m1);\
			XTAL_set_op(Type2, A2, B2, x, m2);\
			XTAL_set_op(Type3, A3, B3, x, m3);\
		}\
\
		static int Name1(const inst_t* x){ return XTAL_op(Type1, A1, B1, x); } static void set_##Name1(inst_t* x, int v){ XTAL_set_op(Type1, A1, B1, x, v); }\
		static int Name2(const inst_t* x){ return XTAL_op(Type2, A2, B2, x); } static void set_##Name2(inst_t* x, int v){ XTAL_set_op(Type2, A2, B2, x, v); }\
		static int Name3(const inst_t* x){ return XTAL_op(Type3, A3, B3, x); } static void set_##Name3(inst_t* x, int v){ XTAL_set_op(Type3, A3, B3, x, v); }\
\
		static StringPtr inspect(const inst_t* x, const CodePtr& code){\
			return make_inst_string(\
					XTAL_STRING(#InstName),\
					XTAL_STRING(#Name1), inst_inspect_##Type1(Name1(x), x, code),\
					XTAL_STRING(#Name2), inst_inspect_##Type2(Name2(x), x, code),\
					XTAL_STRING(#Name3), inst_inspect_##Type3(Name3(x), x, code));\
		}\
	}

#define XTAL_DEF_INST_4(N, InstName, Type1, Name1, Type2, Name2, Type3, Name3, Type4, Name4) \
	struct InstName{\
		enum{ \
			NUMBER = N,\
			OFFSET_##Name1 = XTAL_ALIGN(0+sizeof(u8), sizeof(Type1)), A1 = OFFSET_##Name1/sizeof(inst_t), B1 = OFFSET_##Name1%sizeof(inst_t),\
			OFFSET_##Name2 = XTAL_ALIGN(OFFSET_##Name1+sizeof(Type1), sizeof(Type2)), A2 = OFFSET_##Name2/sizeof(inst_t), B2 = OFFSET_##Name2%sizeof(inst_t),\
			OFFSET_##Name3 = XTAL_ALIGN(OFFSET_##Name2+sizeof(Type2), sizeof(Type3)), A3 = OFFSET_##Name3/sizeof(inst_t), B3 = OFFSET_##Name3%sizeof(inst_t),\
			OFFSET_##Name4 = XTAL_ALIGN(OFFSET_##Name3+sizeof(Type3), sizeof(Type4)), A4 = OFFSET_##Name4/sizeof(inst_t), B4 = OFFSET_##Name4%sizeof(inst_t),\
			BSIZE = XTAL_ALIGN(OFFSET_##Name4+sizeof(Type4), sizeof(inst_t)),\
			ISIZE = BSIZE/sizeof(inst_t),\
		};\
\
		static void set(inst_t* x, int m1, int m2, int m3, int m4){\
			*x = N;\
			XTAL_set_op(Type1, A1, B1, x, m1);\
			XTAL_set_op(Type2, A2, B2, x, m2);\
			XTAL_set_op(Type3, A3, B3, x, m3);\
			XTAL_set_op(Type4, A4, B4, x, m4);\
		}\
\
		static int Name1(const inst_t* x){ return XTAL_op(Type1, A1, B1, x); } static void set_##Name1(inst_t* x, int v){ XTAL_set_op(Type1, A1, B1, x, v); }\
		static int Name2(const inst_t* x){ return XTAL_op(Type2, A2, B2, x); } static void set_##Name2(inst_t* x, int v){ XTAL_set_op(Type2, A2, B2, x, v); }\
		static int Name3(const inst_t* x){ return XTAL_op(Type3, A3, B3, x); } static void set_##Name3(inst_t* x, int v){ XTAL_set_op(Type3, A3, B3, x, v); }\
		static int Name4(const inst_t* x){ return XTAL_op(Type4, A4, B4, x); } static void set_##Name4(inst_t* x, int v){ XTAL_set_op(Type4, A4, B4, x, v); }\
\
		static StringPtr inspect(const inst_t* x, const CodePtr& code){\
			return make_inst_string(\
					XTAL_STRING(#InstName),\
					XTAL_STRING(#Name1), inst_inspect_##Type1(Name1(x), x, code),\
					XTAL_STRING(#Name2), inst_inspect_##Type2(Name2(x), x, code),\
					XTAL_STRING(#Name3), inst_inspect_##Type3(Name3(x), x, code),\
					XTAL_STRING(#Name4), inst_inspect_##Type4(Name4(x), x, code));\
		}\
	}

#define XTAL_DEF_INST_5(N, InstName, Type1, Name1, Type2, Name2, Type3, Name3, Type4, Name4, Type5, Name5) \
	struct InstName{\
		enum{ \
			NUMBER = N,\
			OFFSET_##Name1 = XTAL_ALIGN(0+sizeof(u8), sizeof(Type1)), A1 = OFFSET_##Name1/sizeof(inst_t), B1 = OFFSET_##Name1%sizeof(inst_t),\
			OFFSET_##Name2 = XTAL_ALIGN(OFFSET_##Name1+sizeof(Type1), sizeof(Type2)), A2 = OFFSET_##Name2/sizeof(inst_t), B2 = OFFSET_##Name2%sizeof(inst_t),\
			OFFSET_##Name3 = XTAL_ALIGN(OFFSET_##Name2+sizeof(Type2), sizeof(Type3)), A3 = OFFSET_##Name3/sizeof(inst_t), B3 = OFFSET_##Name3%sizeof(inst_t),\
			OFFSET_##Name4 = XTAL_ALIGN(OFFSET_##Name3+sizeof(Type3), sizeof(Type4)), A4 = OFFSET_##Name4/sizeof(inst_t), B4 = OFFSET_##Name4%sizeof(inst_t),\
			OFFSET_##Name5 = XTAL_ALIGN(OFFSET_##Name4+sizeof(Type4), sizeof(Type5)), A5 = OFFSET_##Name5/sizeof(inst_t), B5 = OFFSET_##Name5%sizeof(inst_t),\
			BSIZE = XTAL_ALIGN(OFFSET_##Name5+sizeof(Type5), sizeof(inst_t)),\
			ISIZE = BSIZE/sizeof(inst_t),\
		};\
\
		static void set(inst_t* x, int m1, int m2, int m3, int m4, int m5){\
			*x = N;\
			XTAL_set_op(Type1, A1, B1, x, m1);\
			XTAL_set_op(Type2, A2, B2, x, m2);\
			XTAL_set_op(Type3, A3, B3, x, m3);\
			XTAL_set_op(Type4, A4, B4, x, m4);\
			XTAL_set_op(Type5, A5, B5, x, m5);\
		}\
\
		static int Name1(const inst_t* x){ return XTAL_op(Type1, A1, B1, x); } static void set_##Name1(inst_t* x, int v){ XTAL_set_op(Type1, A1, B1, x, v); }\
		static int Name2(const inst_t* x){ return XTAL_op(Type2, A2, B2, x); } static void set_##Name2(inst_t* x, int v){ XTAL_set_op(Type2, A2, B2, x, v); }\
		static int Name3(const inst_t* x){ return XTAL_op(Type3, A3, B3, x); } static void set_##Name3(inst_t* x, int v){ XTAL_set_op(Type3, A3, B3, x, v); }\
		static int Name4(const inst_t* x){ return XTAL_op(Type4, A4, B4, x); } static void set_##Name4(inst_t* x, int v){ XTAL_set_op(Type4, A4, B4, x, v); }\
		static int Name5(const inst_t* x){ return XTAL_op(Type5, A5, B5, x); } static void set_##Name5(inst_t* x, int v){ XTAL_set_op(Type5, A5, B5, x, v); }\
\
		static StringPtr inspect(const inst_t* x, const CodePtr& code){\
			return make_inst_string(\
					XTAL_STRING(#InstName),\
					XTAL_STRING(#Name1), inst_inspect_##Type1(Name1(x), x, code),\
					XTAL_STRING(#Name2), inst_inspect_##Type2(Name2(x), x, code),\
					XTAL_STRING(#Name3), inst_inspect_##Type3(Name3(x), x, code),\
					XTAL_STRING(#Name4), inst_inspect_##Type4(Name4(x), x, code),\
					XTAL_STRING(#Name5), inst_inspect_##Type5(Name5(x), x, code));\
		}\
	}

#define XTAL_DEF_INST_6(N, InstName, Type1, Name1, Type2, Name2, Type3, Name3, Type4, Name4, Type5, Name5, Type6, Name6) \
	struct InstName{\
		enum{ \
			NUMBER = N,\
			OFFSET_##Name1 = XTAL_ALIGN(0+sizeof(u8), sizeof(Type1)), A1 = OFFSET_##Name1/sizeof(inst_t), B1 = OFFSET_##Name1%sizeof(inst_t),\
			OFFSET_##Name2 = XTAL_ALIGN(OFFSET_##Name1+sizeof(Type1), sizeof(Type2)), A2 = OFFSET_##Name2/sizeof(inst_t), B2 = OFFSET_##Name2%sizeof(inst_t),\
			OFFSET_##Name3 = XTAL_ALIGN(OFFSET_##Name2+sizeof(Type2), sizeof(Type3)), A3 = OFFSET_##Name3/sizeof(inst_t), B3 = OFFSET_##Name3%sizeof(inst_t),\
			OFFSET_##Name4 = XTAL_ALIGN(OFFSET_##Name3+sizeof(Type3), sizeof(Type4)), A4 = OFFSET_##Name4/sizeof(inst_t), B4 = OFFSET_##Name4%sizeof(inst_t),\
			OFFSET_##Name5 = XTAL_ALIGN(OFFSET_##Name4+sizeof(Type4), sizeof(Type5)), A5 = OFFSET_##Name5/sizeof(inst_t), B5 = OFFSET_##Name5%sizeof(inst_t),\
			OFFSET_##Name6 = XTAL_ALIGN(OFFSET_##Name5+sizeof(Type5), sizeof(Type6)), A6 = OFFSET_##Name6/sizeof(inst_t), B6 = OFFSET_##Name6%sizeof(inst_t),\
			BSIZE = XTAL_ALIGN(OFFSET_##Name6+sizeof(Type6), sizeof(inst_t)),\
			ISIZE = BSIZE/sizeof(inst_t),\
		};\
\
		static void set(inst_t* x, int m1, int m2, int m3, int m4, int m5, int m6){\
			*x = N;\
			XTAL_set_op(Type1, A1, B1, x, m1);\
			XTAL_set_op(Type2, A2, B2, x, m2);\
			XTAL_set_op(Type3, A3, B3, x, m3);\
			XTAL_set_op(Type4, A4, B4, x, m4);\
			XTAL_set_op(Type5, A5, B5, x, m5);\
			XTAL_set_op(Type6, A6, B6, x, m6);\
		}\
\
		static int Name1(const inst_t* x){ return XTAL_op(Type1, A1, B1, x); } static void set_##Name1(inst_t* x, int v){ XTAL_set_op(Type1, A1, B1, x, v); }\
		static int Name2(const inst_t* x){ return XTAL_op(Type2, A2, B2, x); } static void set_##Name2(inst_t* x, int v){ XTAL_set_op(Type2, A2, B2, x, v); }\
		static int Name3(const inst_t* x){ return XTAL_op(Type3, A3, B3, x); } static void set_##Name3(inst_t* x, int v){ XTAL_set_op(Type3, A3, B3, x, v); }\
		static int Name4(const inst_t* x){ return XTAL_op(Type4, A4, B4, x); } static void set_##Name4(inst_t* x, int v){ XTAL_set_op(Type4, A4, B4, x, v); }\
		static int Name5(const inst_t* x){ return XTAL_op(Type5, A5, B5, x); } static void set_##Name5(inst_t* x, int v){ XTAL_set_op(Type5, A5, B5, x, v); }\
		static int Name6(const inst_t* x){ return XTAL_op(Type6, A6, B6, x); } static void set_##Name6(inst_t* x, int v){ XTAL_set_op(Type6, A6, B6, x, v); }\
\
		static StringPtr inspect(const inst_t* x, const CodePtr& code){\
			return make_inst_string(\
					XTAL_STRING(#InstName),\
					XTAL_STRING(#Name1), inst_inspect_##Type1(Name1(x), x, code),\
					XTAL_STRING(#Name2), inst_inspect_##Type2(Name2(x), x, code),\
					XTAL_STRING(#Name3), inst_inspect_##Type3(Name3(x), x, code),\
					XTAL_STRING(#Name4), inst_inspect_##Type4(Name4(x), x, code),\
					XTAL_STRING(#Name5), inst_inspect_##Type5(Name5(x), x, code));\
		}\
	}

#define XTAL_DEF_INST_7(N, InstName, Type1, Name1, Type2, Name2, Type3, Name3, Type4, Name4, Type5, Name5, Type6, Name6, Type7, Name7) \
	struct InstName{\
		enum{ \
			NUMBER = N,\
			OFFSET_##Name1 = XTAL_ALIGN(0+sizeof(u8), sizeof(Type1)), A1 = OFFSET_##Name1/sizeof(inst_t), B1 = OFFSET_##Name1%sizeof(inst_t),\
			OFFSET_##Name2 = XTAL_ALIGN(OFFSET_##Name1+sizeof(Type1), sizeof(Type2)), A2 = OFFSET_##Name2/sizeof(inst_t), B2 = OFFSET_##Name2%sizeof(inst_t),\
			OFFSET_##Name3 = XTAL_ALIGN(OFFSET_##Name2+sizeof(Type2), sizeof(Type3)), A3 = OFFSET_##Name3/sizeof(inst_t), B3 = OFFSET_##Name3%sizeof(inst_t),\
			OFFSET_##Name4 = XTAL_ALIGN(OFFSET_##Name3+sizeof(Type3), sizeof(Type4)), A4 = OFFSET_##Name4/sizeof(inst_t), B4 = OFFSET_##Name4%sizeof(inst_t),\
			OFFSET_##Name5 = XTAL_ALIGN(OFFSET_##Name4+sizeof(Type4), sizeof(Type5)), A5 = OFFSET_##Name5/sizeof(inst_t), B5 = OFFSET_##Name5%sizeof(inst_t),\
			OFFSET_##Name6 = XTAL_ALIGN(OFFSET_##Name5+sizeof(Type5), sizeof(Type6)), A6 = OFFSET_##Name6/sizeof(inst_t), B6 = OFFSET_##Name6%sizeof(inst_t),\
			OFFSET_##Name7 = XTAL_ALIGN(OFFSET_##Name6+sizeof(Type6), sizeof(Type7)), A7 = OFFSET_##Name7/sizeof(inst_t), B7 = OFFSET_##Name7%sizeof(inst_t),\
			BSIZE = XTAL_ALIGN(OFFSET_##Name7+sizeof(Type7), sizeof(inst_t)),\
			ISIZE = BSIZE/sizeof(inst_t),\
		};\
\
		static void set(inst_t* x, int m1, int m2, int m3, int m4, int m5, int m6, int m7){\
			*x = N;\
			XTAL_set_op(Type1, A1, B1, x, m1);\
			XTAL_set_op(Type2, A2, B2, x, m2);\
			XTAL_set_op(Type3, A3, B3, x, m3);\
			XTAL_set_op(Type4, A4, B4, x, m4);\
			XTAL_set_op(Type5, A5, B5, x, m5);\
			XTAL_set_op(Type6, A6, B6, x, m6);\
			XTAL_set_op(Type7, A7, B7, x, m7);\
		}\
\
		static int Name1(const inst_t* x){ return XTAL_op(Type1, A1, B1, x); } static void set_##Name1(inst_t* x, int v){ XTAL_set_op(Type1, A1, B1, x, v); }\
		static int Name2(const inst_t* x){ return XTAL_op(Type2, A2, B2, x); } static void set_##Name2(inst_t* x, int v){ XTAL_set_op(Type2, A2, B2, x, v); }\
		static int Name3(const inst_t* x){ return XTAL_op(Type3, A3, B3, x); } static void set_##Name3(inst_t* x, int v){ XTAL_set_op(Type3, A3, B3, x, v); }\
		static int Name4(const inst_t* x){ return XTAL_op(Type4, A4, B4, x); } static void set_##Name4(inst_t* x, int v){ XTAL_set_op(Type4, A4, B4, x, v); }\
		static int Name5(const inst_t* x){ return XTAL_op(Type5, A5, B5, x); } static void set_##Name5(inst_t* x, int v){ XTAL_set_op(Type5, A5, B5, x, v); }\
		static int Name6(const inst_t* x){ return XTAL_op(Type6, A6, B6, x); } static void set_##Name6(inst_t* x, int v){ XTAL_set_op(Type6, A6, B6, x, v); }\
		static int Name7(const inst_t* x){ return XTAL_op(Type7, A7, B7, x); } static void set_##Name7(inst_t* x, int v){ XTAL_set_op(Type7, A7, B7, x, v); }\
\
		static StringPtr inspect(const inst_t* x, const CodePtr& code){\
			return make_inst_string(\
					XTAL_STRING(#InstName),\
					XTAL_STRING(#Name1), inst_inspect_##Type1(Name1(x), x, code),\
					XTAL_STRING(#Name2), inst_inspect_##Type2(Name2(x), x, code),\
					XTAL_STRING(#Name3), inst_inspect_##Type3(Name3(x), x, code),\
					XTAL_STRING(#Name4), inst_inspect_##Type4(Name4(x), x, code),\
					XTAL_STRING(#Name5), inst_inspect_##Type5(Name5(x), x, code));\
		}\
	}

#define XTAL_DEF_INST_8(N, InstName, Type1, Name1, Type2, Name2, Type3, Name3, Type4, Name4, Type5, Name5, Type6, Name6, Type7, Name7, Type8, Name8) \
	struct InstName{\
		enum{ \
			NUMBER = N,\
			OFFSET_##Name1 = XTAL_ALIGN(0+sizeof(u8), sizeof(Type1)), A1 = OFFSET_##Name1/sizeof(inst_t), B1 = OFFSET_##Name1%sizeof(inst_t),\
			OFFSET_##Name2 = XTAL_ALIGN(OFFSET_##Name1+sizeof(Type1), sizeof(Type2)), A2 = OFFSET_##Name2/sizeof(inst_t), B2 = OFFSET_##Name2%sizeof(inst_t),\
			OFFSET_##Name3 = XTAL_ALIGN(OFFSET_##Name2+sizeof(Type2), sizeof(Type3)), A3 = OFFSET_##Name3/sizeof(inst_t), B3 = OFFSET_##Name3%sizeof(inst_t),\
			OFFSET_##Name4 = XTAL_ALIGN(OFFSET_##Name3+sizeof(Type3), sizeof(Type4)), A4 = OFFSET_##Name4/sizeof(inst_t), B4 = OFFSET_##Name4%sizeof(inst_t),\
			OFFSET_##Name5 = XTAL_ALIGN(OFFSET_##Name4+sizeof(Type4), sizeof(Type5)), A5 = OFFSET_##Name5/sizeof(inst_t), B5 = OFFSET_##Name5%sizeof(inst_t),\
			OFFSET_##Name6 = XTAL_ALIGN(OFFSET_##Name5+sizeof(Type5), sizeof(Type6)), A6 = OFFSET_##Name6/sizeof(inst_t), B6 = OFFSET_##Name6%sizeof(inst_t),\
			OFFSET_##Name7 = XTAL_ALIGN(OFFSET_##Name6+sizeof(Type6), sizeof(Type7)), A7 = OFFSET_##Name7/sizeof(inst_t), B7 = OFFSET_##Name7%sizeof(inst_t),\
			OFFSET_##Name8 = XTAL_ALIGN(OFFSET_##Name7+sizeof(Type7), sizeof(Type8)), A8 = OFFSET_##Name8/sizeof(inst_t), B8 = OFFSET_##Name8%sizeof(inst_t),\
			BSIZE = XTAL_ALIGN(OFFSET_##Name8+sizeof(Type8), sizeof(inst_t)),\
			ISIZE = BSIZE/sizeof(inst_t),\
		};\
\
		static void set(inst_t* x, int m1, int m2, int m3, int m4, int m5, int m6, int m7, int m8){\
			*x = N;\
			XTAL_set_op(Type1, A1, B1, x, m1);\
			XTAL_set_op(Type2, A2, B2, x, m2);\
			XTAL_set_op(Type3, A3, B3, x, m3);\
			XTAL_set_op(Type4, A4, B4, x, m4);\
			XTAL_set_op(Type5, A5, B5, x, m5);\
			XTAL_set_op(Type6, A6, B6, x, m6);\
			XTAL_set_op(Type7, A7, B7, x, m7);\
			XTAL_set_op(Type8, A8, B8, x, m8);\
		}\
\
		static int Name1(const inst_t* x){ return XTAL_op(Type1, A1, B1, x); } static void set_##Name1(inst_t* x, int v){ XTAL_set_op(Type1, A1, B1, x, v); }\
		static int Name2(const inst_t* x){ return XTAL_op(Type2, A2, B2, x); } static void set_##Name2(inst_t* x, int v){ XTAL_set_op(Type2, A2, B2, x, v); }\
		static int Name3(const inst_t* x){ return XTAL_op(Type3, A3, B3, x); } static void set_##Name3(inst_t* x, int v){ XTAL_set_op(Type3, A3, B3, x, v); }\
		static int Name4(const inst_t* x){ return XTAL_op(Type4, A4, B4, x); } static void set_##Name4(inst_t* x, int v){ XTAL_set_op(Type4, A4, B4, x, v); }\
		static int Name5(const inst_t* x){ return XTAL_op(Type5, A5, B5, x); } static void set_##Name5(inst_t* x, int v){ XTAL_set_op(Type5, A5, B5, x, v); }\
		static int Name6(const inst_t* x){ return XTAL_op(Type6, A6, B6, x); } static void set_##Name6(inst_t* x, int v){ XTAL_set_op(Type6, A6, B6, x, v); }\
		static int Name7(const inst_t* x){ return XTAL_op(Type7, A7, B7, x); } static void set_##Name7(inst_t* x, int v){ XTAL_set_op(Type7, A7, B7, x, v); }\
		static int Name8(const inst_t* x){ return XTAL_op(Type8, A8, B8, x); } static void set_##Name8(inst_t* x, int v){ XTAL_set_op(Type8, A8, B8, x, v); }\
\
		static StringPtr inspect(const inst_t* x, const CodePtr& code){\
			return make_inst_string(\
					XTAL_STRING(#InstName),\
					XTAL_STRING(#Name1), inst_inspect_##Type1(Name1(x), x, code),\
					XTAL_STRING(#Name2), inst_inspect_##Type2(Name2(x), x, code),\
					XTAL_STRING(#Name3), inst_inspect_##Type3(Name3(x), x, code),\
					XTAL_STRING(#Name4), inst_inspect_##Type4(Name4(x), x, code),\
					XTAL_STRING(#Name5), inst_inspect_##Type5(Name5(x), x, code));\
		}\
	}

#define XTAL_DEF_INST_9(N, InstName, Type1, Name1, Type2, Name2, Type3, Name3, Type4, Name4, Type5, Name5, Type6, Name6, Type7, Name7, Type8, Name8, Type9, Name9) \
	struct InstName{\
		enum{ \
			NUMBER = N,\
			OFFSET_##Name1 = XTAL_ALIGN(0+sizeof(u8), sizeof(Type1)), A1 = OFFSET_##Name1/sizeof(inst_t), B1 = OFFSET_##Name1%sizeof(inst_t),\
			OFFSET_##Name2 = XTAL_ALIGN(OFFSET_##Name1+sizeof(Type1), sizeof(Type2)), A2 = OFFSET_##Name2/sizeof(inst_t), B2 = OFFSET_##Name2%sizeof(inst_t),\
			OFFSET_##Name3 = XTAL_ALIGN(OFFSET_##Name2+sizeof(Type2), sizeof(Type3)), A3 = OFFSET_##Name3/sizeof(inst_t), B3 = OFFSET_##Name3%sizeof(inst_t),\
			OFFSET_##Name4 = XTAL_ALIGN(OFFSET_##Name3+sizeof(Type3), sizeof(Type4)), A4 = OFFSET_##Name4/sizeof(inst_t), B4 = OFFSET_##Name4%sizeof(inst_t),\
			OFFSET_##Name5 = XTAL_ALIGN(OFFSET_##Name4+sizeof(Type4), sizeof(Type5)), A5 = OFFSET_##Name5/sizeof(inst_t), B5 = OFFSET_##Name5%sizeof(inst_t),\
			OFFSET_##Name6 = XTAL_ALIGN(OFFSET_##Name5+sizeof(Type5), sizeof(Type6)), A6 = OFFSET_##Name6/sizeof(inst_t), B6 = OFFSET_##Name6%sizeof(inst_t),\
			OFFSET_##Name7 = XTAL_ALIGN(OFFSET_##Name6+sizeof(Type6), sizeof(Type7)), A7 = OFFSET_##Name7/sizeof(inst_t), B7 = OFFSET_##Name7%sizeof(inst_t),\
			OFFSET_##Name8 = XTAL_ALIGN(OFFSET_##Name7+sizeof(Type7), sizeof(Type8)), A8 = OFFSET_##Name8/sizeof(inst_t), B8 = OFFSET_##Name8%sizeof(inst_t),\
			OFFSET_##Name9 = XTAL_ALIGN(OFFSET_##Name8+sizeof(Type8), sizeof(Type9)), A9 = OFFSET_##Name9/sizeof(inst_t), B9 = OFFSET_##Name9%sizeof(inst_t),\
			BSIZE = XTAL_ALIGN(OFFSET_##Name9+sizeof(Type9), sizeof(inst_t)),\
			ISIZE = BSIZE/sizeof(inst_t),\
		};\
\
		static void set(inst_t* x, int m1, int m2, int m3, int m4, int m5, int m6, int m7, int m8, int m9){\
			*x = N;\
			XTAL_set_op(Type1, A1, B1, x, m1);\
			XTAL_set_op(Type2, A2, B2, x, m2);\
			XTAL_set_op(Type3, A3, B3, x, m3);\
			XTAL_set_op(Type4, A4, B4, x, m4);\
			XTAL_set_op(Type5, A5, B5, x, m5);\
			XTAL_set_op(Type6, A6, B6, x, m6);\
			XTAL_set_op(Type7, A7, B7, x, m7);\
			XTAL_set_op(Type8, A8, B8, x, m8);\
			XTAL_set_op(Type9, A9, B9, x, m9);\
		}\
\
		static int Name1(const inst_t* x){ return XTAL_op(Type1, A1, B1, x); } static void set_##Name1(inst_t* x, int v){ XTAL_set_op(Type1, A1, B1, x, v); }\
		static int Name2(const inst_t* x){ return XTAL_op(Type2, A2, B2, x); } static void set_##Name2(inst_t* x, int v){ XTAL_set_op(Type2, A2, B2, x, v); }\
		static int Name3(const inst_t* x){ return XTAL_op(Type3, A3, B3, x); } static void set_##Name3(inst_t* x, int v){ XTAL_set_op(Type3, A3, B3, x, v); }\
		static int Name4(const inst_t* x){ return XTAL_op(Type4, A4, B4, x); } static void set_##Name4(inst_t* x, int v){ XTAL_set_op(Type4, A4, B4, x, v); }\
		static int Name5(const inst_t* x){ return XTAL_op(Type5, A5, B5, x); } static void set_##Name5(inst_t* x, int v){ XTAL_set_op(Type5, A5, B5, x, v); }\
		static int Name6(const inst_t* x){ return XTAL_op(Type6, A6, B6, x); } static void set_##Name6(inst_t* x, int v){ XTAL_set_op(Type6, A6, B6, x, v); }\
		static int Name7(const inst_t* x){ return XTAL_op(Type7, A7, B7, x); } static void set_##Name7(inst_t* x, int v){ XTAL_set_op(Type7, A7, B7, x, v); }\
		static int Name8(const inst_t* x){ return XTAL_op(Type8, A8, B8, x); } static void set_##Name8(inst_t* x, int v){ XTAL_set_op(Type8, A8, B8, x, v); }\
		static int Name9(const inst_t* x){ return XTAL_op(Type9, A9, B9, x); } static void set_##Name9(inst_t* x, int v){ XTAL_set_op(Type9, A9, B9, x, v); }\
\
		static StringPtr inspect(const inst_t* x, const CodePtr& code){\
			return make_inst_string(\
					XTAL_STRING(#InstName),\
					XTAL_STRING(#Name1), inst_inspect_##Type1(Name1(x), x, code),\
					XTAL_STRING(#Name2), inst_inspect_##Type2(Name2(x), x, code),\
					XTAL_STRING(#Name3), inst_inspect_##Type3(Name3(x), x, code),\
					XTAL_STRING(#Name4), inst_inspect_##Type4(Name4(x), x, code),\
					XTAL_STRING(#Name5), inst_inspect_##Type5(Name5(x), x, code));\
		}\
	}


///////////////////////////////////////////////////////////////////////////////////////////////

/**
* \internal
* \brief 行が変わるごとに挟まれる命令
*
*/
XTAL_DEF_INST_0(0, InstLine);

enum{
	LOAD_NULL,
	LOAD_UNDEFINED,
	LOAD_FALSE,
	LOAD_TRUE
};

/**
* \internal
* \brief nullかundefinedかfalseかtrue値を取り出す
*
*/
XTAL_DEF_INST_2(1, InstLoadValue,
	i8, result, // 値を代入するローカル変数番号
	u8, value // nullかundefinedかfalseかtrue番号
);

/**
* \internal
* \brief 値テーブルから値を取り出す
*
*/
XTAL_DEF_INST_2(2, InstLoadConstant,
	i8, result, // 値を代入するローカル変数番号
    u16, value_number // 値を取り出すテーブル番号
);

/**
* \internal
* \brief ローカル変数にint値をプッシュする。
*
*/
XTAL_DEF_INST_2(3, InstLoadInt1Byte,
	i8, result,  // 値を代入するローカル変数番号
    i8, value
);

/**
* \internal
* \brief ローカル変数にfloat値をプッシュする。
*
*/
XTAL_DEF_INST_2(4, InstLoadFloat1Byte,
	i8, result,  // 値を代入するローカル変数番号
    i8, value
);

/**
* \internal
* \brief ローカル変数に現在実行中の関数をプッシュする。
*
*/
XTAL_DEF_INST_1(5, InstLoadCallee,
	i8, result  // 値を代入するローカル変数番号
);

/**
* \internal
* \brief ローカル変数に現在のthisを入れる。
*
*/
XTAL_DEF_INST_1(6, InstLoadThis,
	i8, result  // 値を代入するローカル変数番号
);

/**
* \internal
* \brief コピーする
*
*/
XTAL_DEF_INST_2(7, InstCopy,
	i8, result,  // 値を代入するローカル変数番号
	i8, target // 値を取り出すローカル変数番号
);

XTAL_DEF_INST_3(8, InstInc,
	i8, result,  // 値を代入するローカル変数番号
	i8, target, // 値を取り出すローカル変数番号
	i8, stack_base
);

XTAL_DEF_INST_3(9, InstDec,
	i8, result,  // 値を代入するローカル変数番号
	i8, target, // 値を取り出すローカル変数番号
	i8, stack_base
);

XTAL_DEF_INST_3(10, InstPos,
	i8, result,  // 値を代入するローカル変数番号
	i8, target, // 値を取り出すローカル変数番号
	i8, stack_base
);

XTAL_DEF_INST_3(11, InstNeg,
	i8, result,  // 値を代入するローカル変数番号
	i8, target, // 値を取り出すローカル変数番号
	i8, stack_base
);

XTAL_DEF_INST_3(12, InstCom,
	i8, result,  // 値を代入するローカル変数番号
	i8, target, // 値を取り出すローカル変数番号
	i8, stack_base
);

XTAL_DEF_INST_5(13, InstAdd,
	i8, result,  // 値を代入するローカル変数番号
	i8, lhs,
	i8, rhs,
	i8, stack_base,
	u8, assign
);

XTAL_DEF_INST_5(14, InstSub,
	i8, result,  // 値を代入するローカル変数番号
	i8, lhs,
	i8, rhs,
	i8, stack_base,
	u8, assign
);

XTAL_DEF_INST_5(15, InstCat,
	i8, result,  // 値を代入するローカル変数番号
	i8, lhs,
	i8, rhs,
	i8, stack_base,
	u8, assign
);

XTAL_DEF_INST_5(16, InstMul,
	i8, result,  // 値を代入するローカル変数番号
	i8, lhs,
	i8, rhs,
	i8, stack_base,
	u8, assign
);

XTAL_DEF_INST_5(17, InstDiv,
	i8, result,  // 値を代入するローカル変数番号
	i8, lhs,
	i8, rhs,
	i8, stack_base,
	u8, assign
);

XTAL_DEF_INST_5(18, InstMod,
	i8, result,  // 値を代入するローカル変数番号
	i8, lhs,
	i8, rhs,
	i8, stack_base,
	u8, assign
);

XTAL_DEF_INST_5(19, InstAnd,
	i8, result,  // 値を代入するローカル変数番号
	i8, lhs,
	i8, rhs,
	i8, stack_base,
	u8, assign
);

XTAL_DEF_INST_5(20, InstOr,
	i8, result,  // 値を代入するローカル変数番号
	i8, lhs,
	i8, rhs,
	i8, stack_base,
	u8, assign
	);

XTAL_DEF_INST_5(21, InstXor,
	i8, result,  // 値を代入するローカル変数番号
	i8, lhs,
	i8, rhs,
	i8, stack_base,
	u8, assign
);

XTAL_DEF_INST_5(22, InstShl,
	i8, result,  // 値を代入するローカル変数番号
	i8, lhs,
	i8, rhs,
	i8, stack_base,
	u8, assign
);

XTAL_DEF_INST_5(23, InstShr,
	i8, result,  // 値を代入するローカル変数番号
	i8, lhs,
	i8, rhs,
	i8, stack_base,
	u8, assign
);

XTAL_DEF_INST_5(24, InstUshr,
	i8, result,  // 値を代入するローカル変数番号
	i8, lhs,
	i8, rhs,
	i8, stack_base,
	u8, assign
);

XTAL_DEF_INST_4(25, InstAt,
	i8, result,  // 値を代入するローカル変数番号
	i8, target, // 値を取り出すローカル変数番号
	i8, index,
	i8, stack_base
);

XTAL_DEF_INST_4(26, InstSetAt,
	i8, target, // 値を取り出すローカル変数番号
	i8, index,
	i8, value,
	i8, stack_base
);

/**
* \internal
* \brief 無条件分岐
*
*/
XTAL_DEF_INST_1(27, InstGoto,
        address16, address
);

XTAL_DEF_INST_2(28, InstNot,
	i8, result,  // 値を代入するローカル変数番号
	i8, target);

/**
* \internal
* \brief 条件分岐
*
*/
XTAL_DEF_INST_3(29, InstIf,
		i8, target, // 値を取り出すローカル変数番号
        address16, address_true,
		address16, address_false
);

XTAL_DEF_INST_3(30, InstIfEq,
	i8, lhs,
	i8, rhs,
	i8, stack_base
);

XTAL_DEF_INST_3(31, InstIfLt,
	i8, lhs,
	i8, rhs,
	i8, stack_base
);

XTAL_DEF_INST_3(32, InstIfRawEq,
	i8, lhs,
	i8, rhs,
	i8, stack_base
);

XTAL_DEF_INST_3(33, InstIfIs,
	i8, lhs,
	i8, rhs,
	i8, stack_base
);

XTAL_DEF_INST_3(34, InstIfIn,
	i8, lhs,
	i8, rhs,
	i8, stack_base
);

XTAL_DEF_INST_3(35, InstIfUndefined,
	i8, target, // 値を取り出すローカル変数番号
    address16, address_true,
	address16, address_false
);

XTAL_DEF_INST_1(36, InstIfDebug,
	address16, address
);


XTAL_DEF_INST_1(37, InstPush,
	i8, target // 値を取り出すローカル変数番号
);

XTAL_DEF_INST_1(38, InstPop,
	i8, result
);

/**
* \internal
* \brief 値の数を調整する
*
*/
XTAL_DEF_INST_3(39, InstAdjustValues,
	u8, stack_base,
    u8, result_count,
    u8, need_result_count
);

/**
* \internal
* \brief ローカル変数を取り出す。
*
*/
XTAL_DEF_INST_3(40, InstLocalVariable,
	i8, result,  // 値を代入するローカル変数番号
    u16, number,
	u8, depth
);

/**
* \internal
* \brief ローカル変数に値を設定する。
*
*/
XTAL_DEF_INST_3(41, InstSetLocalVariable,
	i8, target,
	u16, number,
	u8, depth
);

/**
* \internal
* \brief インスタンス変数を取り出す。
*
*/
XTAL_DEF_INST_3(42, InstInstanceVariable,
	i8, result,  // 値を代入するローカル変数番号
    u16, info_number,
    u8, number
);

/**
* \internal
* \brief インスタンス変数に値を設定する。
*
*/
XTAL_DEF_INST_3(43, InstSetInstanceVariable,
	i8, value,
    u16, info_number,
    u8, number
);

/**
* \internal
* \brief インスタンス変数を取り出す。
*
*/
XTAL_DEF_INST_2(44, InstInstanceVariableByName,
	i8, result,  // 値を代入するローカル変数番号
    u16, identifier_number
);

/**
* \internal
* \brief インスタンス変数に値を設定する。
*
*/
XTAL_DEF_INST_2(45, InstSetInstanceVariableByName,
	i8, value,
    u16, identifier_number
);

/**
* \internal
* \brief ファイルローカル変数を取り出す。
*
*/
XTAL_DEF_INST_2(46, InstFilelocalVariable,
	i8, result, // 値を代入するローカル変数番号
    u16, value_number
);

/**
* \internal
* \brief ファイルローカル変数に値を設定する。
*
*/
XTAL_DEF_INST_2(47, InstSetFilelocalVariable,
	i8, value,
    u16, value_number
);

/**
* \internal
* \brief ファイルローカル変数を取り出す。
*
*/
XTAL_DEF_INST_2(48, InstFilelocalVariableByName,
		i8, result,  // 値を代入するローカル変数番号
        u16, identifier_number
);

/**
* \internal
* \brief ファイルローカル変数に値を設定する。
*
*/
XTAL_DEF_INST_2(49, InstSetFilelocalVariableByName,
	i8, value,
    u16, identifier_number
);

enum{
	CALL_FLAG_ARGS = 0,
	CALL_FLAG_THIS = 1,

	CALL_FLAG_ARGS_BIT = 1<<CALL_FLAG_ARGS,
	CALL_FLAG_THIS_BIT = 1<<CALL_FLAG_THIS
};

enum{
	MEMBER_FLAG_P = 2,
	MEMBER_FLAG_S = 3,
	MEMBER_FLAG_Q = 4,

	MEMBER_FLAG_P_BIT = 1<<MEMBER_FLAG_P,
	MEMBER_FLAG_S_BIT = 1<<MEMBER_FLAG_S,
	MEMBER_FLAG_Q_BIT = 1<<MEMBER_FLAG_Q
};

/**
* \internal
* \brief オブジェクトのメンバを取り出す。
*
*/
XTAL_DEF_INST_3(50, InstMember,
	i8, result,  // 値を代入するローカル変数番号
	i8, target, // 値を取り出すローカル変数番号
	i16, primary
);

XTAL_DEF_INST_5(51, InstMemberEx,
	i8, result,  // 値を代入するローカル変数番号
	i8, target, // 値を取り出すローカル変数番号
	i16, primary,
	i8, secondary,
	u8, flags
);

XTAL_DEF_INST_5(52, InstCall,
	i8, result,  // 値を代入するローカル変数番号
    u8, need_result,
	i8, target, // 値を取り出すローカル変数番号
	i8, stack_base,
    u8, ordered
);

XTAL_DEF_INST_8(53, InstCallEx,
	i8, result,  // 値を代入するローカル変数番号
    u8, need_result,
	i8, target, // 値を取り出すローカル変数番号
	i8, self,
	i8, stack_base,
    u8, ordered,
    u8, named,
    u8, flags
);

XTAL_DEF_INST_7(54, InstSend,
	i8, result,  // 値を代入するローカル変数番号
    u8, need_result,
	i8, target, // 値を取り出すローカル変数番号
	i16, primary,
	i8, secondary,
	i8, stack_base,
    u8, ordered
);

XTAL_DEF_INST_9(55, InstSendEx,
	i8, result,  // 値を代入するローカル変数番号
    u8, need_result,
	i8, target, // 値を取り出すローカル変数番号
	i16, primary,
	i8, secondary,
	i8, stack_base,
    u8, ordered,
    u8, named,
    u8, flags
);

XTAL_DEF_INST_4(56, InstProperty,
	i8, result,  // 値を代入するローカル変数番号
    i16, primary,
	i8, target, // 値を取り出すローカル変数番号
	i8, stack_base
);

XTAL_DEF_INST_3(57, InstSetProperty,
    i16, primary,
	i8, target, // 値を取り出すローカル変数番号
	i8, stack_base
);

XTAL_DEF_INST_1(58, InstScopeBegin,
    u16, info_number
);

XTAL_DEF_INST_0(59, InstScopeEnd);

/**
* \internal
* \brief 関数から抜ける
*
*/
XTAL_DEF_INST_2(60, InstReturn,
	i8, base,
    u8, result_count
);

/**
* \internal
* \brief fiberの実行を一時中断する
*
*/
XTAL_DEF_INST_4(61, InstYield,
	i8, result,  // 値を代入するローカル変数番号
	u8, need_result_count,
	u8, base,
    i8, result_count
);

/**
* \internal
* \brief 仮想マシンのループから脱出する。
*
*/
XTAL_DEF_INST_0(62, InstExit);

XTAL_DEF_INST_5(63, InstRange,
	i8, result,  // 値を代入するローカル変数番号
	u8, kind,
	i8, lhs,
	i8, rhs,
	i8, stack_base
);

XTAL_DEF_INST_3(64, InstOnce,
	i8, result,  // 値を代入するローカル変数番号
    address16, address,
    u16, value_number
);

XTAL_DEF_INST_2(65, InstSetOnce,
	i8, target, // 値を取り出すローカル変数番号
    u16, value_number
);

XTAL_DEF_INST_1(66, InstMakeArray,
	i8, result  // 値を代入するローカル変数番号
);

XTAL_DEF_INST_2(67, InstArrayAppend,
	i8, target, // 値を取り出すローカル変数番号
	i8, value
);

XTAL_DEF_INST_1(68, InstMakeMap,
	i8, result  // 値を代入するローカル変数番号
);

XTAL_DEF_INST_3(69, InstMapInsert,
	i8, target, // 値を取り出すローカル変数番号
	i8, key,
	i8, value
);

XTAL_DEF_INST_2(70, InstMapSetDefault,
	i8, target, // 値を取り出すローカル変数番号
	i8, value
);

XTAL_DEF_INST_2(71, InstClassBegin,
	i8, mixin_base,
    u16, info_number
);

XTAL_DEF_INST_1(72, InstClassEnd,
	i8, result  // 値を代入するローカル変数番号
);

XTAL_DEF_INST_5(73, InstDefineClassMember,
    u16, number,
	u16, primary,
	i8, secondary,
	u8, accessibility,
	i8, value
);

/**
* \internal
* \brief オブジェクトのメンバを定義する。
*
*/
XTAL_DEF_INST_5(74, InstDefineMember,
	i8, target,
	i16, primary,
	i8, secondary,
	u8, flags,
	i8, value
);

XTAL_DEF_INST_3(75, InstMakeFun,
	i8, result,  // 値を代入するローカル変数番号
    u16, info_number,
	address16, address
);

XTAL_DEF_INST_4(76, InstMakeInstanceVariableAccessor,
	i8, result,  // 値を代入するローカル変数番号
	u8, type,
	u8, number,
    u16, info_number
);

XTAL_DEF_INST_1(77, InstTryBegin,
    u16, info_number
);

XTAL_DEF_INST_0(78, InstTryEnd);

XTAL_DEF_INST_1(79, InstPushGoto,
    address16, address
);

XTAL_DEF_INST_0(80, InstPopGoto);

XTAL_DEF_INST_0(81, InstThrow);

XTAL_DEF_INST_1(82, InstAssert,
	i8, message);

XTAL_DEF_INST_0(83, InstBreakPoint);

XTAL_DEF_INST_0(84, InstMAX);

}

#endif // XTAL_INST_H_INCLUDE_GUARD
