#pragma once

#include <iterator>
#include <boost/iterator/iterator_traits.hpp>

#include "../state/state_iterator.hpp"

namespace biscuit
{

template<
	class Parser, unsigned int min, unsigned int max = min
>
struct limit
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		typedef typename state_iterator<State>::type iter_t;
		typedef typename boost::difference<iter_t>::type diff_t;
		
		iter_t const cur0 = s.cur;
		Parser::parse(s, us);
		diff_t d = std::distance(cur0, s.cur);
		if (min <= d & d <= max)
			return true;
		
		s.cur = s.cur0;
		return false;
	}
};

} // namespace biscuit
