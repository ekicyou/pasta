
#include <boost/test/unit_test.hpp>
#include <iostream>
#include <string>

#include "../../biscuit/parser.hpp"
#include "../../biscuit/algorithm/match.hpp"

using boost::unit_test::test_suite;
using namespace biscuit;

void shortest_longest_test()
{
	std::cout << "shortest_longest_test ing..." << std::endl;

	{
		std::string s0("xxx");
		typedef longest< str<'x'>, str<'x','x','x'>, str<'x','x'> > longp;
		BOOST_CHECK(( match< longp >(s0) ));
	}

	{
		std::string s0("xxx");
		typedef shortest< str<'x'>, str<'x','x','x'>, str<'x','x'> > shortp;
		BOOST_CHECK(( match< seq< shortp, str<'x','x'> > >(s0) ));
	}
}
