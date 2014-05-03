#pragma once

#include "star.hpp"

namespace biscuit
{

template< class Parser >
struct plus
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		if (Parser::parse(s, us))
		{
			return star<Parser>::parse(s, us);
		}
		
		return false;
	}
};

} // namespace biscuit
