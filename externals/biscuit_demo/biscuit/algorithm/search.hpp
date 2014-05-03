#pragma once

#include <boost/range/end.hpp>
#include <boost/range/iterator_range.hpp>
#include <boost/range/const_iterator.hpp>
#include <boost/range/result_iterator.hpp>

#include "../state/default_user_state.hpp"
#include "../state/eos.hpp"
#include "../state/range_state.hpp"
#include "../state/state_iterator.hpp"

namespace biscuit
{
	namespace detail
	{
		template< class Parser, class ForwardRange, class UserState >
		boost::iterator_range< typename boost::range_result_iterator<ForwardRange>::type >
		search_impl(ForwardRange& r, UserState& us)
		{
			typedef typename range_state<ForwardRange>::type state_t;
			typedef typename state_iterator<state_t>::type iter_t;

			state_t s(r);
			for (;;)
			{
				iter_t const cur0 = s.cur;
				if (Parser::parse(s, us))
				{
					return boost::make_iterator_range(cur0, s.cur);
				}
				
				if (eos(s))
				{
					break;
				}
				
				++s.cur;
			}
			
			return boost::make_iterator_range(boost::end(r), boost::end(r));
		}
	} // namespace detail

template< class Parser, class ForwardRange, class UserState >
boost::iterator_range< typename boost::range_result_iterator<ForwardRange>::type >
search(ForwardRange& r, UserState& us)
{
	return detail::search_impl<Parser>(r, us);
}

template< class Parser, class ForwardRange, class UserState >
boost::iterator_range< typename boost::range_const_iterator<ForwardRange>::type >
search(ForwardRange const& r, UserState& us)
{
	return detail::search_impl<Parser>(r, us);
}

template< class Parser, class ForwardRange>
boost::iterator_range< typename boost::range_result_iterator<ForwardRange>::type >
search(ForwardRange& r)
{
	return detail::search_impl<Parser>(r, default_user_state);
}

template< class Parser, class ForwardRange>
boost::iterator_range< typename boost::range_const_iterator<ForwardRange>::type >
search(ForwardRange const& r)
{
	return detail::search_impl<Parser>(r, default_user_state);
}

} // namespace biscuit
