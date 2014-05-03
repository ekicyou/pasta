#pragma once

#include "../../state/eos.hpp"

namespace biscuit
{

template< wchar_t MinCh, wchar_t MaxCh>
struct wchar_t_range
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
