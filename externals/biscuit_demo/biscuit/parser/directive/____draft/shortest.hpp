#pragma once

#include <algorithm>
#include <iterator>
#include <boost/iterator/iterator_traits.hpp>

#include "nothing.hpp"

// local macro
#define BISCUIT_parser_longest_body_elem(i) \
	if (Parser##i##::parse(s, us)) \
	{ \
		ret = true; \
		d = std::max( std::distance(cur0, s.cur), d ); \
	} \
	s.cur = cur0; \
/**/

namespace biscuit
{

template<
	class Parser0 = nothing, class Parser1 = nothing,
	class Parser2 = nothing, class Parser3 = nothing,
	class Parser4 = nothing, class Parser5 = nothing,
	class Parser6 = nothing, class Parser7 = nothing,
	class Parser8 = nothing, class Parser9 = nothing
>
struct longest
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		typedef typename state_iterator<State>::type iter_t;
		typedef typename boost::iterator_difference<iter_t>::type diff_t;
		
		bool ret = false;
		iter_t const cur0 = s.cur;
		diff_t d = 0;

		BISCUIT_parser_longest_body_elem(0);
		BISCUIT_parser_longest_body_elem(1);
		BISCUIT_parser_longest_body_elem(2);
		BISCUIT_parser_longest_body_elem(3);
		BISCUIT_parser_longest_body_elem(4);
		BISCUIT_parser_longest_body_elem(5);
		BISCUIT_parser_longest_body_elem(6);
		BISCUIT_parser_longest_body_elem(7);
		BISCUIT_parser_longest_body_elem(8);
		BISCUIT_parser_longest_body_elem(9);

		std::advance(s.cur, d);
		return ret;
	}
};

} // namespace biscuit

#undef BISCUIT_parser_longest_body_elem

