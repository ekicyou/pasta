#pragma once

#include <boost/preprocessor/iteration/local.hpp>
#include <boost/preprocessor/repetition.hpp>
#include <boost/preprocessor/arithmetic.hpp>
#include <boost/preprocessor/punctuation/comma_if.hpp>
#include <boost/preprocessor/iteration/iterate.hpp>
#include <boost/preprocessor/iteration/local.hpp>

#ifndef BISCUIT_PARSER_MAX_ARITY
	#define BISCUIT_PARSER_MAX_ARITY 20
#endif

#define BISCUIT_parser_shortest_body_statement(z, n, data) \
	if (Parser##n::parse(s, us)) \
	{ \
		ret = true; \
		d = std::min( std::distance(cur0, s.cur), d ); \
	} \
	s.cur = cur0; \
/**/

#define BISCUIT_parser_shortest_spec(z, n, unused) \
	template< \
		BOOST_PP_ENUM_PARAMS(n, class P) \
	> \
	struct shortest \
	{ \
		template< class State, class UserState > \
		static bool parse(State& s, UserState& us) \
		{ \
			typedef typename state_iterator<State>::type iter_t; \
			typedef typename boost::iterator_difference<iter_t>::type diff_t; \
			bool ret = false; \
			iter_t const cur0 = s.cur; \
			diff_t d = std::numeric_limits<diff_t>::max(); \


			if (ret) \
				std::advance(s.cur, d); \
			return ret; \
		} \
	}; \
/**/

namespace biscuit
{
	
	#define BOOST_PP_LOCAL_MACRO(n) BISCUIT_parser_shortest_spec(~, n, ~)
	#define BOOST_PP_LOCAL_LIMITS (0, BISCUIT_PARSER_MAX_ARITY-1)
	#include BOOST_PP_LOCAL_ITERATE()
} // namespace biscuit