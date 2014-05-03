#pragma once

#include "../../state/eos.hpp"
#include "../../utility/ignore_unused_variables_warning.hpp"

namespace biscuit
{

template< ucs4_t MinCh, ucs4_t MaxCh>
struct ucs4_range
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		ignore_unused_variables_warning(s, us);

		if (eos(s))
			return false;

		if (MinCh <= *s.cur && *s.cur <= MaxCh)
		{
			++s.cur;
			return true;
		}

		return false;
	}
};

} // namespace biscuit
