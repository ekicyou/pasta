#pragma once

#include <boost/range/iterator_range.hpp>

#include "../algorithm/match.hpp"
#include "../state/state_iterator.hpp"


namespace biscuit
{

template<
	class ParserA, class ParserB
>
struct minus
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		typedef typename state_iterator<State>::type iter_t;
		iter_t const cur0 = s.cur;
		
		if (ParserA::parse(s, us))
		{
			boost::iterator_range<iter_t> ir = boost::make_iterator_range(cur0, s.cur);
			if (!match<ParserB>(ir, us))
			{
				return true;
			}
		}

		s.cur = cur0;
		return false;
	}
};

} // namespace biscuit
