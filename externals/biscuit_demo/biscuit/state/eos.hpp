#pragma once

namespace biscuit
{

template< class State >
bool eos(State &state)
{
	return state.cur == state.last;
}

} // namespace biscuit
