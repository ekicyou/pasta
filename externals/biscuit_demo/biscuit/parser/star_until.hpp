#pragma once

#include "../state/state_iterator.hpp"
#include "begin.hpp"
#include "end.hpp"
#include "eps.hpp"

namespace biscuit
{

template<
	class ParserA, class ParserB
>
struct star_until // -*A >> B
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		typename state_iterator<State>::type const cur0 = s.cur;

		while (!ParserB::parse(s, us))
		{
			if (!ParserA::parse(s, us))
			{
				s.cur = cur0;
				return false;
			}
		}

		return true;
	}
	
	typedef star_until type;
};

// meaningless
template< class ParserB >
struct star_until<begin, ParserB>;

template< class ParserB >
struct star_until<end, ParserB>;

template< class ParserB >
struct star_until<eps, ParserB>;

} // namespace biscuit
