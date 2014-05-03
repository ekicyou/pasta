
#ifndef BOOST_PP_IS_ITERATING

	#ifndef BISCUIT_PARSER_PP_STR_INCLUDED
		#define BISCUIT_PARSER_PP_STR_INCLUDED

		#include <boost/preprocessor/repetition.hpp>
		#include <boost/preprocessor/iteration/iterate.hpp>
		#include <boost/preprocessor/iteration/local.hpp>
		#include <boost/preprocessor/facilities/identity.hpp>

		#ifndef BISCUIT_PARSER_MAX_ARITY
			#define BISCUIT_PARSER_MAX_ARITY 20
		#endif
		
		#define BISCUIT_parser_char_(z, n, data) char_<ch##n>

		BOOST_PP_IDENTITY(#pragma once)()
		BOOST_PP_IDENTITY(#include "../seq.hpp")()
		BOOST_PP_IDENTITY(#include "char.hpp")()

		namespace biscuit
		{

		// primary
		template<
			BOOST_PP_ENUM_PARAMS_WITH_A_DEFAULT(BISCUIT_PARSER_MAX_ARITY, char ch, 0)
		>
		struct str :
			seq<
				BOOST_PP_ENUM(BISCUIT_PARSER_MAX_ARITY, BISCUIT_parser_char_, ~)
			>
		{
		};

		#define BOOST_PP_ITERATION_LIMITS(0, BISCUIT_PARSER_MAX_ARITY-1)
		#define BOOST_PP_FILENAME_1 "str.hpp"
		#include BOOST_PP_ITERATE()

		} // namespace biscuit

		#undef BISCUIT_parser_char_

	#endif // BISCUIT_PARSER_PP_STR_INCLUDED

#else // BOOST_PP_IS_ITERATING

	#define n BOOST_PP_ITERATION()

	template<
		BOOST_PP_ENUM_PARAMS(n, char ch)
	>
	struct str<
		BOOST_PP_ENUM_PARAMS(n, ch)
	> :
		seq<
			BOOST_PP_ENUM(n, BISCUIT_parser_char_, ~)
		>
	{
	};

	#undef n

#endif // BOOST_PP_IS_ITERATING
