
#include <boost/preprocessor/repetition.hpp>
#include <boost/preprocessor/arithmetic.hpp>
#include <boost/preprocessor/punctuation/comma_if.hpp>
#include <boost/preprocessor/iteration/iterate.hpp>

#define BISCUIT_PARSER_MAX_ARITY 50
#define BISCUIT_print(z, n, data) data
#define BISCUIT_parser_seq_body(z, n, data) P##n::parse(s, us) &&

	namespace biscuit
	{

	// primary
	template< BOOST_PP_ENUM_PARAMS_WITH_A_DEFAULT(BISCUIT_PARSER_MAX_ARITY, class P, eps) >
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

