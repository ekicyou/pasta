#pragma once

#include <boost/range/begin.hpp>
#include <boost/range/end.hpp>
#include <boost/static_assert.hpp>
// #include <boost/noncopyable.hpp>

namespace biscuit
{

struct the_state_class
{
	typedef the_state_class type;
	
	template< class ForwardIter >
	struct apply // : private boost::noncopyable
	{
		typedef apply type;
		typedef the_state_class state_class_type;
		typedef ForwardIter iterator_type;
		
		iterator_type first;
		iterator_type last;
		iterator_type cur;
		bool actionable;
		
		template< class ForwardIter2 >
		explicit apply(ForwardIter2 f, ForwardIter2 l, ForwardIter2 c, bool a) :
			first(f), last(l), cur(c), actionable(a)
		{ }

		template< class ForwardRange2 >
		explicit apply(ForwardRange2& r, bool a = true) :
			first(boost::begin(r)), last(boost::end(r)), cur(first), actionable(a)
		{ }

		// avoiding C4510 and C4610 warnings
		apply() { }
	};
};

} // namespace biscuit
