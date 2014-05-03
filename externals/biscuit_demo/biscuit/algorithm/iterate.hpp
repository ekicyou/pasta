#pragma once

#include <boost/mpl/apply.hpp>
#include <boost/range/result_iterator.hpp>

#include "../state/eos.hpp"
#include "../state/range_state.hpp"
#include "../state/state_iterator.hpp"

namespace biscuit
{
	namespace detail
	{
		template< class Parser, class ForwardRange, class UserState, class Action >
		void iterate_impl(ForwardRange& r, UserState& us, Action action)
		{
			typedef typename range_state<ForwardRange>::type state_t;
			typedef typename state_iterator<state_t>::type iter_t;

			state_t s(r);
			while (!eos(s))
			{
				if (!Parser::parse(s, us))
				{
					iter_t prev = s.cur;
					++s.cur;
					action(prev, s.cur, us);
				}
			}
		}
	} // namespace detail

template< class Parser, class ForwardRange, class UserState, class Action >
void iterate(ForwardRange& r, UserState& us, Action action)
{
	detail::iterate_impl<Parser>(r, us, action);
}

template< class Parser, class ForwardRange, class UserState, class Action >
void iterate(ForwardRange const& r, UserState& us, Action action)
{
	detail::iterate_impl<Parser>(r, us, action);
}

} // namespace biscuit
