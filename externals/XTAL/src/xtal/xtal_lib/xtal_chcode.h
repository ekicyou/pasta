
#pragma once

namespace xtal{

////////////////////////////////////////////////
// sjis

class SJISChCodeLib : public ChCodeLib{
public:
	virtual int_t ch_len(char_t ch){
		u8 c = (u8)ch;
		if(c==0) return 0;
		if((c >= 0x81 && c <= 0x9F) || (c >= 0xE0 && c <= 0xFC)){
			return 2;
		}
		return 1;
	}
};

////////////////////////////////////////////////
// euc

class EUCChCodeLib : public ChCodeLib{
public:
	virtual int_t ch_len(char_t ch){
		u8 c = (u8)ch;
		if(c==0) return 1;
		if(c&0x80){
			return 2;
		}
		return 1;	
	}
};

////////////////////////////////////////////////
// utf8

class UTF8ChCodeLib : public ChCodeLib{
public:
	virtual int_t ch_len(char_t ch){
		u8 c = (u8)ch;
		if(c&0x80) {
			if(c&0x40){
				if(c&0x20){
					if(c&0x10){
						return 4;
					}
					return 3;
				}
				return 2;       
			}
		}
		return 1;
	}
};

////////////////////////////////////////////////
// utf16

class UTF16ChCodeLib : public ChCodeLib{
public:

	bool is_surrogate(unsigned int v){
		return (v & ~((1 << 11) - 1)) == 0xD800;
	}

	virtual int_t ch_len(char_t ch){
		return is_surrogate(ch) ? 2 : 1;
	}
};

////////////////////////////////////////////////
// utf32

class UTF32ChCodeLib : public ChCodeLib{
public:
	virtual int_t ch_len(char_t ch){
		return 1;
	}
};

}
