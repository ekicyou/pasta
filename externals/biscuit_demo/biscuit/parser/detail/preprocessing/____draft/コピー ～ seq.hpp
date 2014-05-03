
#ifndef BOOST_PP_IS_ITERATING

	#ifndef BISCUIT_PARSER_PP_SEQ_INCLUDED
		#define BISCUIT_PARSER_PP_SEQ_INCLUDED

		#include <boost/preprocessor/repetition.hpp>
		#include <boost/preprocessor/iteration/iterate.hpp>
		#include <boost/preprocessor/facilities/identity.hpp>

		#ifndef BISCUIT_PARSER_MAX_ARITY
			#define BISCUIT_PARSER_MAX_ARITY 20
		#endif
		
		#define BISCUIT_print(z, n, data) data
		#define BISCUIT_parser_seq_body(z, n, data) P##n::parse(s, us) &&

		BOOST_PP_IDENTITY(#pragma once)()
		BOOST_PP_IDENTITY(#include "../state/state_iterator.hpp")()
		BOOST_PP_IDENTITY(#include "detail/na.hpp")()
		BOOST_PP_IDENTITY(#pragma warning( push ))()
		BOOST_PP_IDENTITY(#pragma warning( disable : 4127 ))()
		
		namespace biscuit
		{

		// primary
		template<
			BOOST_PP_ENUM_PARAMS_WITH_A_DEFAULT(BISCUIT_PARSER_MAX_ARITY, class P, na)
		>
		struct seq
		{
			template< class State, class UserState >
			static bool parse(State& s, UserState& us)
			{
				typename state_iterator<State>::type cur0 = s.cur;
				
				if (
					BOOST_PP_REPEAT(BISCUIT_PARSER_MAX_ARITY, BISCUIT_parser_seq_body, ~)
					true
				)
				{
					return true;
				}
				else
				{
					s.cur = cur0;
					return false;
				}
			}
		};

		#define BOOST_PP_ITERATION_LIMITS(0, BISCUIT_PARSER_MAX_ARITY-1)
		#define BOOST_PP_FILENAME_1 "seq.hpp"
		#include BOOST_PP_ITERATE()

		} // namespace biscuit

		BOOST_PP_IDENTITY(#pragma warning( pop ))()
		#undef BISCUIT_print
		#undef BISCUIT_parser_seq_body

	#endif // BISCUIT_PARSER_PP_SEQ_INCLUDED

#else // BOOST_PP_IS_ITERATING

	#define n BOOST_PP_ITERATION()

	template<
		BOOST_PP_ENUM_PARAMS(n, class P)
	>
	struct seq<
		BOOST_PP_ENUM_PARAMS(n, P)
	>
	{
		template< class State, class UserState >
		static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State>::type cur0 = s.cur;
			
			if (
				BOOST_PP_REPEAT(n, BISCUIT_parser_seq_body, ~)
				true
			)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	#undef n

#endif // BOOST_PP_IS_ITERATING
