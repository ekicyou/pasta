#pragma once

#include "../utility/ignore_unused_variables_warning.hpp"

namespace biscuit
{
	
struct eps
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		ignore_unused_variables_warning(s, us);
		return true;
	}
};

} // namespace biscuit
