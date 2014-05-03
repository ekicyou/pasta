#pragma once

namespace biscuit
{

template<
	class Parser
>
struct opt
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		Parser::parse(s, us);
		return true;
	}
};

} // namespace biscuit
