#pragma once

#include "../state/eos.hpp"

namespace biscuit
{

struct end
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		return eos(s);
	}
	
	typedef end type;
};

} // namespace biscuit
