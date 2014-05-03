#pragma once

#include <iostream>
#include <iterator>
#include <iosfwd> // streamsize.
#include <boost/iostreams/categories.hpp>

namespace biscuit
{

struct cout_device
{
	typedef char char_type;
	typedef boost::iostreams::sink_tag category;
	std::streamsize write(char_type const *s, std::streamsize n)
	{
		std::vector<char_type> v(s, s+n);
		std::copy(v.begin(), v.end(), std::ostream_iterator<char_type>(std::cout));
		return n;
	}
};


} // namespace biscuit


