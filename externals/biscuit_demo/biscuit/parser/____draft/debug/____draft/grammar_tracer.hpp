#pragma once

#include "../state/state_iterator.hpp"

namespace biscuit
{

template<
	class Grammar, bool on
>
struct grammar_tracer
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		typename state_iterator<State>::type const cur0 = s.cur;

		if (Parser::parse(s, us))
		{
			if (s.actionable)
			{
				Action()(cur0, s.cur, us);
			}
			
			return true;
		}
		
		return false;
	}
	
	typedef actor type;
};

} // namespace biscuit
