/** \file src/xtal/xtal_serializer.h
* \brief src/xtal/xtal_serializer.h
*/

#ifndef XTAL_SERIALIZER_H_INCLUDE_GUARD
#define XTAL_SERIALIZER_H_INCLUDE_GUARD

#pragma once

namespace xtal{

class Serializer{
public:

	Serializer(const StreamPtr& s);

	~Serializer();

	void serialize(const AnyPtr& v);

	AnyPtr deserialize();

private:

	void inner_serialize(const AnyPtr& v);
	AnyPtr inner_deserialize();
	
	AnyPtr demangle(const AnyPtr& n);

	int_t register_value(const AnyPtr& v);
	int_t append_value(const AnyPtr& v);

	void put_tab(int_t tab);

	void clear();

	void inner_serialize_scope_info(ScopeInfo& info);
	void inner_deserialize_scope_info(ScopeInfo& info);

	AnyPtr inner_deserialize_serial_new();
	AnyPtr inner_deserialize_name();
	StringPtr inner_deserialize_string(int_t charsize, bool intern);
	ArrayPtr inner_deserialize_array();
	ValuesPtr inner_deserialize_values();
	MapPtr inner_deserialize_map();
	CodePtr inner_deserialize_code();

private:

	enum{ 
		SERIAL_NEW,
		NAME,
		REFERENCE,
		
		TNULL, 
		TUNDEFINED, 
		TFALSE, 
		TTRUE,

		TINT32, 
		TINT64, 

		TFLOAT32,
		TFLOAT64,

		TSTRING8,
		TSTRING16,
		TSTRING32,

		TID8,
		TID16,
		TID32,

		TARRAY, 
		TMAP,
		TVALUES
	};

private:

	MapPtr map_;
	xarray values_;

	Stream* stream_;
	StreamPtr pstream_;

	XTAL_DISALLOW_COPY_AND_ASSIGN(Serializer);
};

}

#endif // XTAL_SERIALIZER_H_INCLUDE_GUARD
