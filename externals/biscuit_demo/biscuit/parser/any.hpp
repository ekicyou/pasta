#pragma once

#include "../state/eos.hpp"
#include "../utility/ignore_unused_variables_warning.hpp"

namespace biscuit
{

struct any // _
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		ignore_unused_variables_warning(s, us);

		if (eos(s))
			return false;
		
		++s.cur;
		return true;
	}
	
	typedef any type;
};

} // namespace biscuit
