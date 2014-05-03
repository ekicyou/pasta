#pragma once

#include <boost/noncopyable.hpp>

namespace biscuit
{

	template< class T >
	struct saver : private boost::noncopyable
	{
		T& src;
		T const prev;
		explicit saver(T& x) : src(x), prev(x) { }
		~saver() { src = prev; }
	};

} // namespace biscuit
