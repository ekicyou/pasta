#pragma once

#include "../../utility/saver.hpp"

namespace biscuit
{

template<
	class Parser
>
struct definitive_actions
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		State tmp(s.first, s.last, s.cur, false);
		if (Parser::parse(tmp, us))
		{
			return Parser::parse(s, us);
		}

		return false;
	}
};

} // namespace biscuit
