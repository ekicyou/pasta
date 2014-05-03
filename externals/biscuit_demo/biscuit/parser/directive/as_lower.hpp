#pragma once

#include <iterator>
#include <functional>
#include <boost/call_traits.hpp>


#include "../../state/state_iterator.hpp"
#include "transform.hpp"

namespace biscuit
{

	namespace detail
	{
		template< class Iterator >
		struct tolower :
			std::unary_function<
				typename boost::call_traits< typename boost::iterator_value<Iterator>::type >::param_type,
				typename boost::iterator_value<Iterator>::type
			>
		{
			result_type operator()(argument_type ch) const
			{
				if ('A' <= ch && ch <= 'Z')
					return ch-'A'+'a';
				
				return ch;
			}
		};
	} // namespace detail


template<
	class Parser
>
struct as_lower
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		typedef typename state_iterator<State>::type iter_t;
    return transform< Parser, detail::tolower<iter_t> >::parse(s, us);
	}
};

} // namespace biscuit
