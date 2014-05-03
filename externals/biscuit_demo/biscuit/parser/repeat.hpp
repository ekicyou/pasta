#pragma once

#include "../state/state_iterator.hpp"

namespace biscuit
{

template<
	class Parser, unsigned int min, unsigned int max = min
>
struct repeat // greedy
{
	BOOST_STATIC_ASSERT( min <= max );

	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		typename state_iterator<State>::type const cur0 = s.cur;

		for (unsigned int i = 0; i < max; ++i)
		{
			if (!Parser::parse(s, us))
			{
				if (i < min) // not enough
				{
					s.cur = cur0;
					return false;
				}
			}
		}
		
		return true;
	}
};

} // namespace biscuit
