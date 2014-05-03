#pragma once

#include "../state/default_user_state.hpp"
#include "../state/eos.hpp"
#include "../state/range_state.hpp"

namespace biscuit
{
	namespace detail
	{
		template< class Parser, class ForwardRange, class UserState >
		bool match_impl(ForwardRange& r, UserState& us)
		{
			typedef typename range_state<ForwardRange>::type state_t;

			state_t s(r);
			if (Parser::parse(s, us))
			{
				return eos(s);
			}
			
			return false;
		}
	} // namespace detail

template< class Parser, class ForwardRange, class UserState >
bool match(ForwardRange& r, UserState& us)
{
	return detail::match_impl<Parser>(r, us);
}

template< class Parser, class ForwardRange, class UserState >
bool match(ForwardRange const& r, UserState& us)
{
	return detail::match_impl<Parser>(r, us);
}

template< class Parser, class ForwardRange>
bool match(ForwardRange& r)
{
	return detail::match_impl<Parser>(r, default_user_state);
}

template< class Parser, class ForwardRange>
bool match(ForwardRange const& r)
{
	return detail::match_impl<Parser>(r, default_user_state);
}

} // namespace biscuit
