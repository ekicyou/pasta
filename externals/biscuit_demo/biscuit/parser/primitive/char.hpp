#pragma once

#include <boost/mpl/integral_c.hpp>

#include "../../state/eos.hpp"
#include "../../utility/ignore_unused_variables_warning.hpp"

namespace biscuit
{

template< char ch >
struct char_ :
	boost::mpl::integral_c<char, ch>
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		ignore_unused_variables_warning(s, us);

		if (eos(s))
			return false;

		if (ch == *s.cur)
		{
			++s.cur;
			return true;
		}

		return false;
	}
};

} // namespace biscuit