/** \file src/xtal/xtal_text.h
* \brief src/xtal/xtal_text.h
*/

#ifndef XTAL_TEXT_H_INCLUDE_GUARD
#define XTAL_TEXT_H_INCLUDE_GUARD

#pragma once

namespace xtal{

class FormatSpecifier{
public:

	FormatSpecifier()
		:type_(0), pos_(0), width_(0), precision_(0){
		buf_[0] = 0;
	}

	enum{ BUF_MAX = 20, REAL_BUF_MAX = BUF_MAX + 2 };
	
	char_t buf_[REAL_BUF_MAX];
	char_t type_;
	int_t pos_, width_, precision_;
	
public:

	enum{ FORMAT_SPECIFIER_MAX = 20 };
	
	int_t type();
	int_t max_buffer_size();

	char_t change_int_type();
	char_t change_float_type();
	bool is_int_type();
	bool is_float_type();

	void make_format_specifier(char_t* dest, char_t type, bool int_type = false);

	uint_t parse_format(const char_t* str, uint_t i, uint_t sz);

private:
	
	uint_t parse_format_inner(const char_t* str, uint_t i, uint_t sz);

	uint_t parse_format_digit(const char_t* str, uint_t i, uint_t sz, int_t& digit);
};

class Text : public Base{
public:
	Text(const IDPtr& key = empty_id);
	void on_rawcall(const VMachinePtr& vm);
	void to_s(const VMachinePtr& vm);
	AnyPtr serial_save();
	void serial_load(const IDPtr& v);
private:	
	IDPtr key_;
};

TextPtr text(const StringPtr& text);
TextPtr text(const LongLivedString& text);

StringPtr format(const StringPtr& text);
StringPtr format(const LongLivedString& text);

void assign_text_map(const AnyPtr& map_iter);
void append_text_map(const AnyPtr& map_iter);

enum{
	SPRINTF_TYPE_f = 0,
	SPRINTF_TYPE_t = 1
};

StringPtr sprintf(int_t type, const LongLivedString& format);
StringPtr sprintf(int_t type, const LongLivedString& format, const LongLivedString& name1, const AnyPtr& value1);
StringPtr sprintf(int_t type, const LongLivedString& format, const LongLivedString& name1, const AnyPtr& value1, const LongLivedString& name2, const AnyPtr& value2);
StringPtr sprintf(int_t type, const LongLivedString& format, const LongLivedString& name1, const AnyPtr& value1, const LongLivedString& name2, const AnyPtr& value2, const LongLivedString& name3, const AnyPtr& value3);
StringPtr sprintf(int_t type, const LongLivedString& format, const LongLivedString& name1, const AnyPtr& value1, const LongLivedString& name2, const AnyPtr& value2, const LongLivedString& name3, const AnyPtr& value3, const LongLivedString& name4, const AnyPtr& value4);

#define XTAL_SPRINTF0(Type, Format) ::xtal::sprintf(SPRINTF_TYPE_##Type, XTAL_STRING(Format))
#define XTAL_SPRINTF1(Type, Format, Name1, Value1) ::xtal::sprintf(SPRINTF_TYPE_##Type, XTAL_STRING(Format), XTAL_STRING(#Name1), Value1)
#define XTAL_SPRINTF2(Type, Format, Name1, Value1, Name2, Value2) ::xtal::sprintf(SPRINTF_TYPE_##Type, XTAL_STRING(Format), XTAL_STRING(#Name1), Value1, XTAL_STRING(#Name2), Value2)
#define XTAL_SPRINTF3(Type, Format, Name1, Value1, Name2, Value2, Name3, Value3) ::xtal::sprintf(SPRINTF_TYPE_##Type, XTAL_STRING(Format), XTAL_STRING(#Name1), Value1, XTAL_STRING(#Name2), Value2, XTAL_STRING(#Name3), Value3)
#define XTAL_SPRINTF4(Type, Format, Name1, Value1, Name2, Value2, Name3, Value3, Name4, Value4) ::xtal::sprintf(SPRINTF_TYPE_##Type, XTAL_STRING(Format), XTAL_STRING(#Name1), Value1, XTAL_STRING(#Name2), Value2, XTAL_STRING(#Name3), Value3, XTAL_STRING(#Name4), Value4)

#define Xf0(Format) XTAL_SPRINTF0(f, Format)
#define Xf1(Format, Name1, Value1) XTAL_SPRINTF1(f, Format, Name1, Value1)
#define Xf2(Format, Name1, Value1, Name2, Value2) XTAL_SPRINTF2(f, Format, Name1, Value1, Name2, Value2)
#define Xf3(Format, Name1, Value1, Name2, Value2, Name3, Value3) XTAL_SPRINTF3(f, Format, Name1, Value1, Name2, Value2, Name3, Value3)
#define Xf4(Format, Name1, Value1, Name2, Value2, Name3, Value3, Name4, Value4)  XTAL_SPRINTF4(f, Format, Name1, Value1, Name2, Value2, Name3, Value3, Name4, Value4)

#define Xt0(Format) XTAL_SPRINTF0(t, Format)
#define Xt1(Format, Name1, Value1) XTAL_SPRINTF1(t, Format, Name1, Value1)
#define Xt2(Format, Name1, Value1, Name2, Value2) XTAL_SPRINTF2(t, Format, Name1, Value1, Name2, Value2)
#define Xt3(Format, Name1, Value1, Name2, Value2, Name3, Value3) XTAL_SPRINTF3(t, Format, Name1, Value1, Name2, Value2, Name3, Value3)
#define Xt4(Format, Name1, Value1, Name2, Value2, Name3, Value3, Name4, Value4)  XTAL_SPRINTF4(t, Format, Name1, Value1, Name2, Value2, Name3, Value3, Name4, Value4)


}

#endif // XTAL_TEXT_H_INCLUDE_GUARD
