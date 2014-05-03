#pragma once

#include "prelude.hpp"

namespace biscuit
{

struct empty_action
{
	template< class ForwardIter, class UserState >
	void operator()(ForwardIter first, ForwardIter last, UserState& state)
	{
		(first); (last); (state);
	}
};

} // namespace biscuit
