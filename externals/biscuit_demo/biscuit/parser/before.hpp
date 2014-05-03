#pragma once

#include "../state/state_iterator.hpp"
#include "directive/no_actions.hpp"

namespace biscuit
{

template<
	class Parser
>
struct before
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		typename state_iterator<State>::type const cur0 = s.cur;

		if (no_actions<Parser>::parse(s, us))
		{
			s.cur = cur0;
			return true;
		}

		return false;
	}
};

} // namespace biscuit
