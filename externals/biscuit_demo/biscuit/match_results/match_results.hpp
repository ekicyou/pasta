#pragma once

#include <boost/range/result_iterator.hpp>

#include "match_iterator.hpp"
#include "../state/default_user_state.hpp"

namespace biscuit
{

template< class Parser, class ForwardRange, class UserState >
struct match_results;

namespace detail
{
	template< class Parser, class ForwardRange, class UserState >
	struct match_results_base
	{
		typedef boost::iterator_range<
			match_iterator<
				Parser,
				typename boost::range_result_iterator<ForwardRange>::type,
				UserState
			>
		> type;
	};
} // namespace detail

template<
	class Parser,
	class ForwardRange,
	class UserState = default_user_state_type const
>
struct match_results : detail::match_results_base<Parser, ForwardRange, UserState>::type
{
private:
	typedef typename detail::match_results_base<Parser, ForwardRange, UserState>::type super_t;

public:
	explicit match_results(ForwardRange& r, UserState& us = default_user_state) :
		super_t(
			make_match_iterator<Parser>(boost::begin(r), boost::end(r), &us),
			make_match_iterator<Parser>(boost::end(r), boost::end(r), &us)
		)
	{ }
};

} // namespace biscuit

