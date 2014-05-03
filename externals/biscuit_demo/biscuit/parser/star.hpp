#pragma once

#include "../state/eos.hpp"
#include "begin.hpp"
#include "end.hpp"
#include "eps.hpp"

namespace biscuit
{

template<
	class Parser
>
struct star
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		while (Parser::parse(s, us))
		{
			// Note: It can't be determined whether parser's width is 0 or not
			//       on compile-time. So you should check it.
			if (eos(s)) // check it!
				return true;
		}
		return true;
	}
};

// meaningless
template<>
struct star<begin>;

template<>
struct star<end>;

template<>
struct star<eps>;

} // namespace biscuit
