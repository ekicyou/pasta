#pragma once

#include "../state/bos.hpp"

namespace biscuit
{

struct begin
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		return bos(s);
	}
};

} // namespace biscuit
