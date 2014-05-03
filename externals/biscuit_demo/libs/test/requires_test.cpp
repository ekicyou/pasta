
#include <boost/test/unit_test.hpp>
#include <iostream>
#include <string>

#include "../../biscuit/parser.hpp"
#include "../../biscuit/algorithm/match.hpp"

using boost::unit_test::test_suite;
using namespace biscuit;

struct is_abc
{
	template< class Iter, class UserState >
	bool operator()(Iter f, Iter l, UserState&)
	{
		return std::string(f, l) == "abc";
	}
};

void requires_test()
{
	std::cout << "requires_test ing..." << std::endl;
	{
		std::string s0("abc");
		BOOST_CHECK(( match< requires< repeat<any, 3>, is_abc > >(s0) ));
	}
	{
		std::string s0("abz");
		BOOST_CHECK(( !match< requires< repeat<any, 3>, is_abc > >(s0) ));
	}
}
