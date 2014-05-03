#pragma once

#include <iterator>
#include <functional>
#include <boost/call_traits.hpp>
#include <boost/mpl/apply.hpp>
#include <boost/iterator/transform_iterator.hpp>

#include "../../state/state_class.hpp"
#include "../../state/state_iterator.hpp"

namespace biscuit
{

template<
	class Parser, class FtorType
>
struct transform
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		typedef typename state_iterator<State>::type iter_t;
    typedef boost::transform_iterator<FtorType, iter_t> tr_iter_t;

		FtorType ftor;
		tr_iter_t f(s.first, ftor);
		tr_iter_t l(s.last, ftor);
		tr_iter_t c(s.cur, ftor);
		tr_iter_t tmp = c;

		typedef typename state_class<State>::type state_class_t;
		typedef typename boost::mpl::apply<state_class_t, tr_iter_t>::type tr_state_t;
		
		tr_state_t tr_s(f, l, c, s.actionable);
		bool ret = Parser::parse(tr_s, us);
		std::advance(s.cur, std::distance(tmp, tr_s.cur));
		return ret;
	}
};

} // namespace biscuit
