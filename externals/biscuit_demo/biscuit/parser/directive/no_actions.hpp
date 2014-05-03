#pragma once

#include "../../utility/saver.hpp"

namespace biscuit
{

template<
	class Parser
>
struct no_actions
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		saver<bool> sv(s.actionable);
		s.actionable = false;
		return Parser::parse(s, us);
	}
};

} // namespace biscuit
