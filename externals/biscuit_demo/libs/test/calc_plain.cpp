
#include <boost/test/unit_test.hpp>
#include <iostream>
#include <string>

#include "../../biscuit/algorithm/match.hpp"
#include "../../biscuit/parser.hpp"
#include "../../biscuit/grammar.hpp"

using namespace biscuit;

struct calculator : grammar< calculator, std::string >
{
	void do_int(iterator_type str, iterator_type end)
	{
		std::string s(str, end);
		std::cout << "PUSH(" << s << ')' << std::endl;
	}

	void do_add(iterator_type, iterator_type)		{ std::cout << "ADD\n"; }
	void do_subt(iterator_type, iterator_type)	{ std::cout << "SUBTRACT\n"; }
	void do_mult(iterator_type, iterator_type)	{ std::cout << "MULTIPLY\n"; }
	void do_div(iterator_type, iterator_type)		{ std::cout << "DIVIDE\n"; }
	void do_neg(iterator_type, iterator_type)		{ std::cout << "NEGATE\n"; }

	struct start;
	struct expression;
	struct term;
	struct factor;

	struct start : identity<expression> { };

	struct expression :
		seq<
			term,
			star<
				or_<
					actor_< seq< str<'+'>, term >, &calculator::do_add >,
					actor_< seq< str<'-'>, term >, &calculator::do_subt >
				>
			>
		>
	{ };
	
	struct term :
		seq<
			factor,
			star<
				or_<
					actor_< seq< str<'*'>, factor >, &calculator::do_mult >,
					actor_< seq< str<'/'>, factor >, &calculator::do_div >
				>
			>
		>
	{ };


	struct factor :
		or_<
			actor_< plus<digit>, &calculator::do_int >,
			seq< str<'('>, expression, str<')'> >,
			actor_< seq< str<'-'>, factor >, &calculator::do_neg >,
			seq< str<'+'>, factor >
		>
	{ };
};

void calc_plain_test()
{
	std::cout << "/////////////////////////////////////////////////////////\n\n";
	std::cout << "\t\tExpression parser...\n\n";
	std::cout << "/////////////////////////////////////////////////////////\n\n";
	std::cout << "Type an expression...or [q or Q] to quit\n\n";

	std::string str;
	while (getline(std::cin, str))
	{
		if (str[0] == 'q' || str[0] == 'Q')
			break;

		calculator calc;
		bool ok
			= match< typename grammar_start<calculator>::type >(str, calc);

		if (ok)
		{
			std::cout << "-------------------------\n";
			std::cout << "Parsing succeeded\n";
			std::cout << "-------------------------\n";
		}
		else
		{
			std::cout << "-------------------------\n";
			std::cout << "Parsing failed\n";
			std::cout << "-------------------------\n";
		}
	}

	std::cout << "Bye... :-) \n\n";
}