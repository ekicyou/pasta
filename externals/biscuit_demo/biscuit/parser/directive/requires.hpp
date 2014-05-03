#pragma once

#include "../state/eos.hpp"
#include "../state/state_iterator.hpp"

namespace biscuit
{

template<
	class Parser,
	class PredType
>
struct requires
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		typedef typename state_iterator<State>::type iter_t;
		iter_t const cur0 = s.cur;
		
		if (Parser::parse(s, us))
		{
			if (PredType()(cur0, s.cur, us))
				return true;
		}

		s.cur = cur0;
		return false;
	}
};

} // namespace biscuit
