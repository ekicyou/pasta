
#include <boost/test/unit_test.hpp>
#include <iostream>
#include <vector>
#include <string>

#include "../../biscuit/algorithm/match.hpp"
#include "../../biscuit/parser.hpp"
#include "../../biscuit/grammar.hpp"

using namespace biscuit;

struct my_grammar : grammar< my_grammar, std::vector<std::string> >
{
	std::string text0() { return "hello"; };
	std::string text1() { return "grammar"; };
	std::string text2() { return "value"; };

	struct start :
		seq<
			value_<&my_grammar::text0>,
			value_<&my_grammar::text1>,
			value_<&my_grammar::text2>
		>
	{ };
};

void grm_value_test()
{
	std::cout << "grm_value_test ing..." << std::endl;

	std::vector<std::string> texts;
	texts.push_back(std::string("hello"));
	texts.push_back(std::string("grammar"));
	texts.push_back(std::string("value"));
	
	my_grammar the_grammar;
	BOOST_CHECK( match< typename grammar_start<my_grammar>::type >(texts, the_grammar) );
}
