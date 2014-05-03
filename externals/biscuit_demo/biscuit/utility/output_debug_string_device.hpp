#pragma once

#include <vector>
#include <iosfwd> // streamsize.
#include <boost/iostreams/categories.hpp>

#include <Windows.h>
#include <tchar.h>

namespace biscuit
{

struct output_debug_string_device
{
	typedef _TCHAR char_type;
	typedef boost::iostreams::sink_tag category;
	std::streamsize write(char_type const *s, std::streamsize n)
	{
		std::vector<char_type> v(s, s+n);
		v.push_back(0);
		OutputDebugString(&v[0]);
		return n;
	}
};


} // namespace biscuit


