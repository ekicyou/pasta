#pragma once

namespace biscuit
{

template< class State >
bool bos(State &state)
{
	return state.cur == state.first;
}

} // namespace biscuit
