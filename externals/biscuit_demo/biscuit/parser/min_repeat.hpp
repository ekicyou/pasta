#pragma once

#include "../state/state_iterator.hpp"

namespace biscuit
{

template<
	class Parser, unsigned int min
>
struct min_repeat // greedy
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		typename state_iterator<State>::type const cur0 = s.cur;

		for (unsigned int i = 0; i < min; ++i)
		{
			if (!Parser::parse(s, us))
			{
				s.cur = cur0;
				return false;
			}
		}
		
		return true;
	}
};

} // namespace biscuit
