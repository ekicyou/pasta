
#ifndef BOOST_PP_IS_ITERATING

	#ifndef BISCUIT_PARSER_PP_OR_INCLUDED
		#define BISCUIT_PARSER_PP_OR_INCLUDED

		#include <boost/preprocessor/arithmetic/sub.hpp>
		#include <boost/preprocessor/cat.hpp>
		#include <boost/preprocessor/comparison/equal.hpp>
		#include <boost/preprocessor/facilities/identity.hpp>
		#include <boost/preprocessor/iteration/iterate.hpp>
		#include <boost/preprocessor/iteration/local.hpp>
		#include <boost/preprocessor/repetition/enum_params.hpp>
		#include <boost/preprocessor/repetition/enum_params_with_a_default.hpp>
		#include <boost/preprocessor/repetition/repeat.hpp>
		#include <boost/preprocessor/stringize.hpp>

		#ifndef BISCUIT_PARSER_MAX_ARITY
			#define BISCUIT_PARSER_MAX_ARITY 20
		#endif
		
		// local macro
		#define BISCUIT_parser_or_body(z, i, to) \
			BOOST_PP_IF(BOOST_PP_EQUAL(i, to), \
				P##i::parse(s, us), \
				P##i::parse(s, us) || \
			) \
		/**/

		// headers
		BOOST_PP_IDENTITY(#pragma once)()
		
		#define BISCUIT_parser_vector_header \
			BOOST_PP_CAT(vector, BISCUIT_PARSER_MAX_ARITY).hpp \
		/**/
		BOOST_PP_IDENTITY(#include BOOST_PP_STRINGIZE(boost/mpl/vector/BISCUIT_parser_vector_header))()
		#undef BISCUIT_parser_vector_header
		
		BOOST_PP_IDENTITY(#include "detail/na.hpp")()


		namespace biscuit
		{

		// primary
		template<
			BOOST_PP_ENUM_PARAMS_WITH_A_DEFAULT(BISCUIT_PARSER_MAX_ARITY, class P, na)
		>
		struct or_ :
			BOOST_PP_CAT(boost::mpl::vector, BISCUIT_PARSER_MAX_ARITY)<
				BOOST_PP_ENUM_PARAMS(BISCUIT_PARSER_MAX_ARITY, P)
			>
		{
			template< class State, class UserState >
			static bool parse(State& s, UserState& us)
			{
				return
					BOOST_PP_REPEAT(BISCUIT_PARSER_MAX_ARITY, BISCUIT_parser_or_body, BOOST_PP_SUB(BISCUIT_PARSER_MAX_ARITY,1))
				;
			}
		};

		// empty
		template<
		>
		struct or_<
		> :
			boost::mpl::vector0<
			>
		{
			template< class State, class UserState >
			static bool parse(State&, UserState&)
			{
				return false;
			}
		};

		#define BOOST_PP_ITERATION_LIMITS(1, BISCUIT_PARSER_MAX_ARITY-1)
		#define BOOST_PP_FILENAME_1 "or.hpp"
		#include BOOST_PP_ITERATE()

		} // namespace biscuit

		#undef BISCUIT_parser_or_body

	#endif // BISCUIT_PARSER_PP_OR_INCLUDED

#else // BOOST_PP_IS_ITERATING

	#define n BOOST_PP_ITERATION()

	template<
		BOOST_PP_ENUM_PARAMS(n, class P)
	>
	struct or_<
		BOOST_PP_ENUM_PARAMS(n, P)
	> :
		BOOST_PP_CAT(boost::mpl::vector, n)<
			BOOST_PP_ENUM_PARAMS(n, P)
		>
	{
		template< class State, class UserState >
		static bool parse(State& s, UserState& us)
		{
			return
				BOOST_PP_REPEAT(n, BISCUIT_parser_or_body, BOOST_PP_SUB(n,1))
			;
		}
	};

	#undef n

#endif // BOOST_PP_IS_ITERATING