#include "xtal.h"
#include "xtal_macro.h"

namespace xtal{

void assign_text_map(const AnyPtr& map){
	text_map()->assign(map);
}

void append_text_map(const AnyPtr& map){
	text_map()->append(map);
}

TextPtr text(const StringPtr& text){
	return xnew<Text>(text->intern());
}

TextPtr text(const LongLivedString& text){
	return xnew<Text>(text);
}

StringPtr format(const StringPtr& text){
	return text;
}

StringPtr format(const LongLivedString& text){
	return text;
}

bool FormatSpecifier::is_int_type(){
	switch(type_){
	case 'i': case 'd': case 'x': case 'X':
		return true;
	
	default:
		return false;
	}
}

bool FormatSpecifier::is_float_type(){
	switch(type_){
	case 'e': case 'E': case 'g':
	case 'G': case 'f':
		return true;
	
	default:
		return false;
	}
}

char_t FormatSpecifier::change_int_type(){
	if(is_int_type()){
		return type_;
	}
	return 'd';
}

char_t FormatSpecifier::change_float_type(){
	if(is_float_type()){
		return type_;
	}
	return 'g';
}
	
int_t FormatSpecifier::type(){
	return type_;
}

int_t FormatSpecifier::max_buffer_size(){
	return width_ + precision_;
}

void FormatSpecifier::make_format_specifier(char_t* dest, char_t type, bool int_type){
	*dest++ = '%';

	{ // 書式指定子を埋め込む
		const char_t* src = buf_;
		while((*dest++ = *src++)!=0){}
		--dest;
	}

	if(int_type){ // int_t が64-bitの時に特別な書式指定子を埋め込む
		const char_t* src = XTAL_INT_FMT;
		while((*dest++ = *src++)!=0){}
		--dest;
	}

	*dest++ = type;
	*dest++ = 0;
}

uint_t FormatSpecifier::parse_format(const char_t* const str, uint_t i, uint_t sz){
	width_ = 0;
	precision_ = 0;
	pos_ = 0;		
	i = parse_format_inner(str, i, sz);
	buf_[pos_] = '\0';
	return i;
}

uint_t FormatSpecifier::parse_format_digit(const char_t* const str, uint_t i, uint_t sz, int_t& digit){
	digit = 0;
	while(i!=sz){
		if(str[i]>='0' && str[i]<= '9'){
			digit *= 10;
			digit += str[i]-'0';
			buf_[pos_++] = str[i++]; 
			if(pos_ == BUF_MAX){
				break;
			}
		}
		else{
			break;
		}
	}

	return i;
}

uint_t FormatSpecifier::parse_format_inner(const char_t* const str, uint_t i, uint_t sz){

	while(i!=sz){
		switch(str[i]){
		case '-': case '+': case '0': case ' ': case '#':
			buf_[pos_++] = str[i++];
			if(pos_ == BUF_MAX){
				return i;
			}
			continue;
		}
		break;
	}
	
	i = parse_format_digit(str, i, sz, width_);
	if(pos_ == BUF_MAX || i==sz){ return i; }

	if(str[i]=='.'){	
		buf_[pos_++] = str[i++]; 
		if(pos_ == BUF_MAX || i==sz){ return i; }
		
		i = parse_format_digit(str, i, sz, precision_);
		if(pos_ == BUF_MAX || i==sz){ return i; }
	}

	char_t type = str[i];

	switch(type){
	case 'i': case 'd': case 'x': case 'X':
	case 'e': case 'E': case 'g':
	case 'G': case 'f':
	case 's':
		i++;
		type_ = type;
		break;
	
	default:
		type_ = 's';
		break;
	}

	return i;
}

void String::on_rawcall(const VMachinePtr& vm){
	MemoryStreamPtr ms = xnew<MemoryStream>();
	enum{
		LIMIT = 255
	};

	char_t cbuf[LIMIT+1];
	char_t spec[FormatSpecifier::FORMAT_SPECIFIER_MAX];

	const char_t* const str = data();
	const char_t* begin = str;
	int_t n = 0;		
	
	AnyPtr value;
	FormatSpecifier fs;

	uint_t i = 0;
	uint_t sz = data_size();

	while(true){
		if(i==sz){
			ms->put_s(begin, str+i);
			break; 
		}

		if(str[i]=='%'){
			ms->put_s(begin, str+i);
			i++;
			if(i==sz){ break; }
			
			if(str[i]=='%'){
				begin = str+i;
				i++;
			}
			else{
				if(str[i]=='('){
					i++;
					if(i==sz){ break; }

					begin = str+i;
					
					bool number = false;
					if('0'<=str[i] && str[i]<='9'){
						number = true;
					}

					while(str[i]!=')'){
						i++;
						if(i==sz){ break; }
					}

					const IDPtr& arg_id = xtal::intern(begin, (str+i)-begin);

					if(number){
						int_t arg_i = arg_id->to_i();

						if(arg_i>0 && arg_i<=vm->ordered_arg_count()){
							value = vm->arg(arg_i-1);
						}
						else{
							value = vm->arg(arg_id);
						}
					}
					else{
						value = vm->arg(arg_id);
					}

					if(str[i]==')'){
						i++;
					}
				}
				else{

					if(n<vm->ordered_arg_count()){
						value = vm->arg(n);
					}
					else{
						XTAL_SPRINTF(cbuf, LIMIT, XTAL_L("%d"), n);
						value = vm->arg(xtal::intern(cbuf));
					}
					++n;
				}

				i = fs.parse_format(str, i, sz);
				begin = str + i;

				if(fs.is_int_type()){
					fs.make_format_specifier(spec, fs.change_int_type(), true);
					XTAL_SPRINTF(cbuf, LIMIT, spec, value->to_i());
					ms->put_s(cbuf);
				}
				else if(fs.is_float_type()){
					fs.make_format_specifier(spec, fs.change_float_type());
					XTAL_SPRINTF(cbuf, LIMIT, spec, value->to_f());
					ms->put_s(cbuf);
				}
				else{
					ms->put_s(value);
				}
			}
		}
		else{
			i++;
		}
	}

	vm->return_result(ms->to_s());
}

Text::Text(const IDPtr& key)
	:key_(key){}

void Text::on_rawcall(const VMachinePtr& vm){
	if(MapPtr m = text_map()){
		if(const AnyPtr& value = m->at(key_)){
			value->to_s()->rawcall(vm);
			return;
		}
	}
	
	key_->rawcall(vm);
}

void Text::to_s(const VMachinePtr& vm){
	on_rawcall(vm);
}

AnyPtr Text::serial_save(){
	return key_;
}

void Text::serial_load(const IDPtr& v){
	key_ = v;
}

AnyPtr format_or_text(int_t type,  const LongLivedString& format_string){
	if(type==SPRINTF_TYPE_f){
		return format(format_string);
	}
	else{
		return text(format_string);
	}
}

StringPtr sprintf(int_t type, const LongLivedString& format){
	return format_or_text(type, format)->call()->to_s();
}

StringPtr sprintf(int_t type, const LongLivedString& format, const LongLivedString& name1, const AnyPtr& value1){
	return format_or_text(type, format)->call(Named(name1, value1))->to_s();
}

StringPtr sprintf(int_t type, const LongLivedString& format, const LongLivedString& name1, const AnyPtr& value1, const LongLivedString& name2, const AnyPtr& value2){
	return format_or_text(type, format)->call(Named(name1, value1), Named(name2, value2))->to_s();
}

StringPtr sprintf(int_t type, const LongLivedString& format, const LongLivedString& name1, const AnyPtr& value1, const LongLivedString& name2, const AnyPtr& value2, const LongLivedString& name3, const AnyPtr& value3){
	return format_or_text(type, format)->call(Named(name1, value1), Named(name2, value2), Named(name3, value3))->to_s();
}

StringPtr sprintf(int_t type, const LongLivedString& format, const LongLivedString& name1, const AnyPtr& value1, const LongLivedString& name2, const AnyPtr& value2, const LongLivedString& name3, const AnyPtr& value3, const LongLivedString& name4, const AnyPtr& value4){
	return format_or_text(type, format)->call(Named(name1, value1), Named(name2, value2), Named(name3, value3), Named(name4, value4))->to_s();
}

}
