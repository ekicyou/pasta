#pragma once

#include "prelude.hpp"

namespace biscuit
{

struct output_action
{
	template< class ForwardIter, class UserState >
	void operator()(ForwardIter first, ForwardIter last, UserState& out)
	{
		for (ForwardIter it = first; it != last; ++it)
		{
			out << *it;
		}
	}
};

} // namespace biscuit
