
#include <boost/test/unit_test.hpp>
#include <iostream>
#include <string>

#include "../../biscuit/parser.hpp"
#include "../../biscuit/algorithm/match.hpp"

using boost::unit_test::test_suite;
using namespace biscuit;

void as_lower_test()
{
	std::cout << "as_lower_test ing..." << std::endl;
	{
		std::string s0("XxXxX");
		BOOST_CHECK(( match< as_lower< star< str<'x'> > > >(s0) ));
		BOOST_CHECK(( !match< as_lower< star< str<'X'> > > >(s0) ));
	
		BOOST_CHECK(( match< as_lower< str<'x','x','x','x','x'> > >(s0) ));
	}
	{
		std::string s0("aBcDe");
		BOOST_CHECK(( match< as_lower< str<'a','b','c','d','e'> > >(s0) ));
	}

	{
		std::string s0("aBcDefgH");
		BOOST_CHECK(( match< seq< as_lower< str<'a','b','c','d','e'> >, str<'f','g','H'> > >(s0) ));
		BOOST_CHECK(( !match< seq< as_lower< str<'a','b','C','d','e'> >, str<'f','g','H'> > >(s0) ));
		BOOST_CHECK(( !match< seq< as_lower< str<'a','b','C','d','e'> >, str<'F','g','H'> > >(s0) ));
	}
}
