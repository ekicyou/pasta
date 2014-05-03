#pragma once

#include <boost/iterator/iterator_traits.hpp>
#include <boost/range/result_iterator.hpp>

#include "filter_iterator.hpp"
#include "../state/default_user_state.hpp"

namespace biscuit
{

template< class Parser, class ForwardRange, class UserState >
struct filter_range;

namespace detail
{
	template< class Parser, class ForwardRange, class UserState >
	struct filter_range_base
	{
		typedef boost::iterator_range<
			filter_iterator<
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
struct filter_range : detail::filter_range_base<Parser, ForwardRange, UserState>::type
{
private:
	typedef typename detail::filter_range_base<Parser, ForwardRange, UserState>::type super_t;

public:
	explicit filter_range(ForwardRange& r, UserState& us = default_user_state) :
		super_t(
			make_filter_iterator<Parser>(boost::begin(r), boost::end(r), &us),
			make_filter_iterator<Parser>(boost::end(r), boost::end(r), &us)
		)
	{ }
};

template< class Parser, class ForwardRange, class UserState >
filter_range<Parser, ForwardRange, UserState>
make_filter_range(ForwardRange& r, UserState& us)
{
	return filter_range<Parser, ForwardRange, UserState>(r, us);
}

template< class Parser, class ForwardRange >
filter_range<Parser, ForwardRange>
make_filter_range(ForwardRange& r)
{
	return filter_range<Parser, ForwardRange>(r);
}

template< class Parser, class ForwardRange, class UserState >
filter_range<Parser, ForwardRange const, UserState>
make_filter_range(ForwardRange const& r, UserState& us)
{
	return filter_range<Parser, ForwardRange const, UserState>(r, us);
}

template< class Parser, class ForwardRange >
filter_range<Parser, ForwardRange const>
make_filter_range(ForwardRange const& r)
{
	return filter_range<Parser, ForwardRange const>(r);
}


} // namespace biscuit
