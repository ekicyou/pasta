#ifndef XTALSRC_H
#define XTALSRC_H

#include "../src/xtal/xtal.h"
#include "../src/xtal/xtal_macro.h"
#include "../src/xtal/xtal_debug.h"

using namespace xtal;

inline StringPtr qstr2xstr(const QString& str){
	return str.toLocal8Bit().constData();
}

inline StringPtr qstr2xid(const QString& str){
	return intern(str.toLocal8Bit().constData());
}

inline QString xstr2qstr(const StringPtr& str){
	return str->c_str();
}

inline CodePtr eval_compile(const QString& str){
	CodePtr ret = eval_compile(qstr2xstr(str));
	XTAL_CATCH_EXCEPT(e){}
	return ret;
}

#endif // XTALSRC_H
