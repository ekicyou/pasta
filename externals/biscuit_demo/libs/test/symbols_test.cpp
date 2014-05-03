
#include <boost/test/unit_test.hpp>
#include <iostream>
#include <string>

#include "../../biscuit/parser.hpp"
#include "../../biscuit/parser/primitive/symbols.hpp"
#include "../../biscuit/algorithm/match.hpp"

using boost::unit_test::test_suite;
using namespace biscuit;

void symbols_test()
{
	std::cout << "symbols_test ing..." << std::endl;

	{
		std::string s0("int");
		typedef symbols< str<'i','n','t'>, str<'j','k'>, str<'i','l'> > symbols_p;
		BOOST_CHECK(( match< symbols_p >(s0) ));
	}

	{
		std::string s0("double");
		typedef symbols< str<'d','o'>, str<'d','o','u','b','l','e'>, str<'l','e'> > symbols_p;
		BOOST_CHECK(( !match< symbols_p >(s0) ));
	}

	{
		std::string s0("do");
		typedef symbols< str<'d','o'>, str<'d','o','u','b','l','e'>, str<'l','e'> > symbols_p;
		BOOST_CHECK(( match< symbols_p >(s0) ));
	}

	{
		std::string s0("char");
		typedef symbols< str<'c','n','t'>, str<'c','h','a','r'>, str<'c','h'> > symbols_p;
		BOOST_CHECK(( !match< symbols_p >(s0) ));
	}
	
	{
		std::string s0("ch");
		typedef symbols< str<'c','n','t'>, str<'c','h','a','r'>, str<'c','h'> > symbols_p;
		BOOST_CHECK(( match< symbols_p >(s0) ));
	}

	{
		std::string s0("int");
		typedef symbols< str<'i','n','k'>, str<'i','n','t','e'>, str<'i','n','t','a'> > symbols_p;
		BOOST_CHECK(( !match< symbols_p >(s0) ));
	}
}
