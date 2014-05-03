#include <boost/test/unit_test.hpp>
#include <functional>
#include <boost/noncopyable.hpp>
#include <iostream>
#include <string>
#include <stack>

#pragma warning( push, 3 )
#pragma warning( disable : 4701 )
#include <boost/lexical_cast.hpp>
#pragma warning( pop )

// See: http://www.boost.org/libs/spirit/example/fundamental/calc_debug.cpp
// Note: I did changed name calculator to calculator_debug, for unknown troubles.

#include "../../biscuit/algorithm/match.hpp"
#include "../../biscuit/parser.hpp"
#include "../../biscuit/grammar.hpp"
#include "../../biscuit/debug.hpp"
#include "../../biscuit/filter_range.hpp"

using namespace biscuit;

struct integer_tag { };
struct factor_tag { };
struct term_tag { };
struct expression_tag { };
struct start_tag { };

template< class ForwardRange >
struct calculator_debug :
	grammar< calculator_debug, ForwardRange >,
	private boost::noncopyable
{
	calculator_debug(std::stack<long>& eval_) : eval(eval_) { }

	void push_int(iterator_type first, iterator_type last)
	{
		std::string s(first, last);
		long n = boost::lexical_cast<long>(s); // std::strtol(str, 0, 10);
		eval.push(n);
		std::cout << "push\t" << long(n) << std::endl;
	}

	template< class Function >
	void do_op(Function op)
	{
		long rhs = eval.top();
		eval.pop();
		long lhs = eval.top();
		eval.pop();

		std::cout << "popped " << lhs << " and " << rhs << " from the stack. ";
		std::cout << "pushing " << op(lhs, rhs) << " onto the stack.\n";
		eval.push(op(lhs, rhs));
	}

	void do_add(iterator_type, iterator_type)		{ do_op(std::plus<long>()); }
	void do_subt(iterator_type, iterator_type)	{ do_op(std::minus<long>()); }
	void do_mult(iterator_type, iterator_type)	{ do_op(std::multiplies<long>()); }
	void do_div(iterator_type, iterator_type)		{ do_op(std::divides<long>()); }

	void do_negate(iterator_type, iterator_type)
	{
			long lhs = eval.top();
			eval.pop();

			std::cout << "popped " << lhs << " from the stack. ";
			std::cout << "pushing " << -lhs << " onto the stack.\n";
			eval.push(-lhs);
	}

	struct integer;
	struct factor;
	struct term;
	struct expression;

	struct start : debugger<start_tag, // debugger<start, also ok!
		expression
	>
	{ };

	struct integer : debugger<integer_tag,
		actor_< plus<digit>, &calculator_debug::push_int >
	>
	{ };

	struct factor : debugger<factor_tag,
		or_<
			integer,
			seq< str<'('>, expression, str<')'> >,
			actor_< seq< str<'-'>, factor >, &calculator_debug::do_negate >,
			seq< str<'+'>, factor >
		>
	>
	{ };

	struct expression : debugger<expression_tag,
		seq<
			term,
			star<
				or_<
					actor_< seq< str<'+'>, term >, &calculator_debug::do_add >,
					actor_< seq< str<'-'>, term >, &calculator_debug::do_subt >
				>
			>
		>
	>
	{ };
	
	struct term : debugger<term_tag,
		seq<
			factor,
			star<
				or_<
					actor_< seq< str<'*'>, factor >, &calculator_debug::do_mult >,
					actor_< seq< str<'/'>, factor >, &calculator_debug::do_div >
				>
			>
		>
	>
	{ };

	std::stack<long>& eval;
};

void calc_debug_test()
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

		typedef filter_range< not_<space>, std::string > range_t;
		range_t fr(str);
		typedef calculator_debug<range_t> calculator_t;

		std::stack<long> eval;
		calculator_t calc(eval);
		bool ok = match< typename grammar_start<calculator_t>::type >(fr, calc);

		if (ok)
		{
			std::cout << "-------------------------\n";
			std::cout << "Parsing succeeded\n";
			std::cout << "result = " << calc.eval.top() << std::endl;
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