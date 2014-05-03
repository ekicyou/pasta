#pragma once

#include "../state/eos.hpp"
#include "../utility/ignore_unused_variables_warning.hpp"

namespace biscuit
{

template< class ValueFtorType >
struct value
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		ignore_unused_variables_warning(s, us);

		if (eos(s))
			return false;

		if (ValueFtorType()(us) == *s.cur)
		{
			++s.cur;
			return true;
		}

		return false;
	}
};

} // namespace biscuit
