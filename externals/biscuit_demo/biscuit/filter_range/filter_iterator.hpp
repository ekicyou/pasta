#pragma once

#include <boost/iterator/iterator_adaptor.hpp>
#include <boost/range/begin.hpp>
#include <boost/range/end.hpp>
#include <boost/range/empty.hpp>
#include <boost/range/iterator_range.hpp>

#include "../algorithm/search.hpp"

// See: boost/iterator/filter_iterator.hpp

namespace biscuit
{

template< class Parser, class ForwardIter, class UserState >
struct filter_iterator;

namespace detail
{
	template< class Parser, class ForwardIter, class UserState >
	struct filter_iterator_base
	{
		typedef boost::iterator_adaptor<
			filter_iterator<Parser, ForwardIter, UserState>,
			ForwardIter,
			boost::use_default,
			boost::forward_traversal_tag // overwrite!
		> type;
	};
} // namespace detail

template<
	class Parser,
	class ForwardIter,
	class UserState
>
struct filter_iterator :
	detail::filter_iterator_base<Parser, ForwardIter, UserState>::type
{
private:
	typedef typename detail::filter_iterator_base<Parser, ForwardIter, UserState>::type super_t;
	friend class boost::iterator_core_access;

public:
	filter_iterator() { }

	filter_iterator(ForwardIter x, ForwardIter end, UserState *pus) :
		super_t(x), m_submatch_end(x), m_end(end), p_user_state(pus)
	{
		// trigger!
		search_submatch();
	}
	
	/*
	template< class OtherIterator, class OtherParser >
	filter_iterator(
		filter_iterator<OtherParser, OtherIterator, UserState> const& t,
		typename boost::enable_if_convertible<OtherIterator, Iterator>::type* = 0
	) : super_t(t.base()), m_submatch_end(t.submatch_end()), m_end(t.end()), p_user_state(t.user_state())
	{ }
	*/

	ForwardIter submatch_end() const { return m_submatch_end; }
	ForwardIter end() const { return m_end; }
	UserState *user_state() const { return p_user_state; }

private:
	void search_submatch()
	{
		if (base() != m_submatch_end)
		{
			// still on the submatch
			return;
		}
		
		// get a next submatch
		boost::iterator_range<ForwardIter> tr(base(), m_end);
		boost::iterator_range<ForwardIter> sr = ::search<Parser>(tr, *p_user_state);
		
		if (boost::empty(sr)) 
		{
			// not found
			base_reference() = m_end;
			return;
		}

		base_reference() = boost::begin(sr);
		m_submatch_end = boost::end(sr);
	}

	void increment()
	{
		++base_reference();
		search_submatch();
	}

	ForwardIter m_submatch_end;
	ForwardIter m_end;
	UserState *p_user_state;
};

	template< class Parser, class ForwardIter, class UserState >
	filter_iterator<Parser, ForwardIter, UserState>
	make_filter_iterator(ForwardIter x, ForwardIter end, UserState *pus)
	{
		return filter_iterator<Parser, ForwardIter, UserState>(x, end, pus);
	}

} // namespace biscuit
