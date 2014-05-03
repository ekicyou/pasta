#pragma once

namespace biscuit
{

template< class State >
struct state_iterator
{
	typedef typename State::iterator_type type;
};

} // namespace biscuit
