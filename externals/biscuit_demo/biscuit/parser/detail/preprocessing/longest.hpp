
#ifndef BOOST_PP_IS_ITERATING

	#ifndef BISCUIT_PARSER_PP_LONGEST_INCLUDED
		#define BISCUIT_PARSER_PP_LONGEST_INCLUDED

		#include <boost/preprocessor/repetition.hpp>
		#include <boost/preprocessor/iteration/iterate.hpp>
		#include <boost/preprocessor/iteration/local.hpp>
		#include <boost/preprocessor/facilities/identity.hpp>

		#ifndef BISCUIT_PARSER_MAX_ARITY
			#define BISCUIT_PARSER_MAX_ARITY 20
		#endif
		
		// local macros
		#define BISCUIT_parser_longest_body_header() \
			typedef typename state_iterator<State>::type iter_t; \
			typedef typename boost::iterator_difference<iter_t>::type diff_t; \
			bool ret = false; \
			iter_t const cur0 = s.cur; \
			diff_t d = 0; \
		/**/

		#define BISCUIT_parser_longest_body_elem(z, n, data) \
			if (P##n::parse(s, us)) \
			{ \
				ret = true; \
				d = std::max( std::distance(cur0, s.cur), d ); \
			} \
			s.cur = cur0; \
		/**/

		#define BISCUIT_parser_longest_body_footer() \
			if (ret) \
			{ \
				std::advance(s.cur, d); \
			} \
			return ret; \
		/**/

		BOOST_PP_IDENTITY(#pragma once)()
		BOOST_PP_IDENTITY(#include <algorithm>)()
		BOOST_PP_IDENTITY(#include <iterator>)()
		BOOST_PP_IDENTITY(#include <boost/iterator/iterator_traits.hpp>)()
		BOOST_PP_IDENTITY(#include "../../state/state_iterator.hpp")()
		BOOST_PP_IDENTITY(#include "../detail/na.hpp")()

		namespace biscuit
		{

		// primary
		template<
			BOOST_PP_ENUM_PARAMS_WITH_A_DEFAULT(BISCUIT_PARSER_MAX_ARITY, class P, na)
		>
		struct longest
		{
			template< class State, class UserState >
			static bool parse(State& s, UserState& us)
			{
				BISCUIT_parser_longest_body_header()
				
				#define BOOST_PP_LOCAL_MACRO(n)   BISCUIT_parser_longest_body_elem(~, n, ~)
				#define BOOST_PP_LOCAL_LIMITS     (0, BISCUIT_PARSER_MAX_ARITY-1)
				#include BOOST_PP_LOCAL_ITERATE()
				#undef BOOST_PP_LOCAL_MACRO

				BISCUIT_parser_longest_body_footer()
			}
		};
		
		// empty
		template<
		>
		struct longest<
		>
		{
			template< class State, class UserState >
			static bool parse(State&, UserState&)
			{
				return false;
			}
		};

		#define BOOST_PP_ITERATION_LIMITS(1, BISCUIT_PARSER_MAX_ARITY-1)
		#define BOOST_PP_FILENAME_1 "longest.hpp"
		#include BOOST_PP_ITERATE()

		} // namespace biscuit
		#undef BISCUIT_parser_longest_body_header
		#undef BISCUIT_parser_longest_body_elem
		#undef BISCUIT_parser_longest_body_footer

	#endif // BISCUIT_PARSER_PP_LONGEST_INCLUDED

#else // BOOST_PP_IS_ITERATING

	#define n BOOST_PP_ITERATION()

	template<
		BOOST_PP_ENUM_PARAMS(n, class P)
	>
	struct longest<
		BOOST_PP_ENUM_PARAMS(n, P)
	>
	{
		template< class State, class UserState >
		static bool parse(State& s, UserState& us)
		{
			BISCUIT_parser_longest_body_header()
			
			#define BOOST_PP_LOCAL_MACRO(n)   BISCUIT_parser_longest_body_elem(~, n, ~)
			#define BOOST_PP_LOCAL_LIMITS     (0, n-1)
			#include BOOST_PP_LOCAL_ITERATE()
			#undef BOOST_PP_LOCAL_MACRO

			BISCUIT_parser_longest_body_footer()
		}
	};

	#undef n

#endif // BOOST_PP_IS_ITERATING
