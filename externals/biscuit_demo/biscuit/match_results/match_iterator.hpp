#pragma once

#include <boost/iterator/iterator_facade.hpp>
#include <boost/mpl/apply.hpp>
#include <boost/range/result_iterator.hpp>
#include <boost/range/iterator_range.hpp>
#include <cassert>

#include "../state/state_iterator.hpp"
#include "../algorithm/search.hpp"

namespace biscuit
{

template< class Parser, class ForwardIter, class UserState >
struct match_iterator;

namespace detail
{
	template< class Parser, class ForwardIter, class UserState >
	struct match_iterator_base
	{
		typedef boost::iterator_facade<
			match_iterator<Parser, ForwardIter, UserState>,
			boost::iterator_range<ForwardIter>,
			boost::forward_traversal_tag,
			boost::iterator_range<ForwardIter> // not reference!
		> type;
	};
} // namespace detail

template<
	class Parser,
	class ForwardIter,
	class UserState
>
struct match_iterator :
	detail::match_iterator_base<Parser, ForwardIter, UserState>::type
{
private:
	typedef typename detail::match_iterator_base<Parser, ForwardIter, UserState>::type super_t;
	friend class boost::iterator_core_access;

public:
	match_iterator() { }

	match_iterator(ForwardIter x, ForwardIter end, UserState *pus) :
		m_submatch(x, x), m_end(end), p_user_state(pus)
	{
		search_submatch(); // trigger!
	}

private:
	void search_submatch()
	{
		// get a submatch
		boost::iterator_range<ForwardIter> tr(boost::end(m_submatch), m_end);
		m_submatch = ::search<Parser>(tr, *p_user_state);
		// if not found, m_submatch becomes [m_end, m_end)
	}

	void increment()
	{
		search_submatch();
	}

	bool equal(match_iterator const& other) const
	{
		assert( m_end == other.m_end && p_user_state == other.p_user_state);
		return m_submatch == other.m_submatch;
	}

	boost::iterator_range<ForwardIter> dereference() const
	{
		return m_submatch;
	}

	boost::iterator_range<ForwardIter> m_submatch;
	ForwardIter m_end;
	UserState *p_user_state;
};

	template< class Parser, class ForwardIter, class UserState >
	match_iterator<Parser, ForwardIter, UserState>
	make_match_iterator(ForwardIter x, ForwardIter end, UserState *pus)
	{
		return match_iterator<Parser, ForwardIter, UserState>(x, end, pus);
	}

} // namespace biscuit
