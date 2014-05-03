
#include <boost/test/unit_test.hpp>
#include <iostream>
#include <string>
#include <sstream>
#include <boost/range/sub_range.hpp>

#include "../../biscuit/algorithm/search.hpp"
#include "../../biscuit/parser.hpp"

using boost::unit_test::test_suite;
using namespace biscuit;

struct c_comment :
	seq<
		str<'/','*'>,
		star_until< any, str<'*','/'> >
	>
{ };


void search_test()

{
	std::cout << "search_test ing..." << std::endl;
	{
		std::string s0("  /* c comment no.1 */x int i; /* c comment no.2 */ i = 1; /* c comment no.3 */ ++i;  ");
		boost::sub_range<std::string> sr = search<c_comment>(s0);
		BOOST_CHECK( std::string(boost::begin(sr), boost::end(sr)) == "/* c comment no.1 */" );
	}
	{
		std::string s0("// not c comment, but c++ comment ");
		boost::sub_range<std::string> sr = search<c_comment>(s0);
		BOOST_CHECK( std::string(boost::begin(sr), boost::end(sr)) == "" );
	}
}
