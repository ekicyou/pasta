#pragma once

#include <boost/mpl/apply.hpp>
#include <boost/range/result_iterator.hpp>

#include "the_state_class.hpp"

namespace biscuit
{

template< class ForwardRange >
struct range_state
{
	typedef typename boost::mpl::apply<
		the_state_class,
		typename boost::range_result_iterator<ForwardRange>::type
	>::type type;
};

} // namespace biscuit
