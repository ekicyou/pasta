#pragma once

#include <boost/range/const_iterator.hpp>

namespace biscuit
{

template< class State >
struct state_class
{
	typedef typename State::state_class_type type;
};

} // namespace biscuit
