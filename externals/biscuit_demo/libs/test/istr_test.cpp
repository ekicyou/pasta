
#include <boost/test/unit_test.hpp>
#include <iostream>
#include <string>

#include "../../biscuit/parser/seq.hpp"
#include "../../biscuit/parser/primitive/ichar.hpp"
#include "../../biscuit/parser/primitive/istr.hpp"
#include "../../biscuit/algorithm/match.hpp"

using boost::unit_test::test_suite;
using namespace biscuit;

void istr_test()
{
	std::cout << "istr_test ing..." << std::endl;

	{
		std::string s0("aBc");
		BOOST_CHECK(( match< seq< ichar<'a'>, ichar<'b'>, ichar<'c'> > >(s0) ));
		BOOST_CHECK(( match< seq< ichar<'A'>, ichar<'B'>, ichar<'C'> > >(s0) ));
		BOOST_CHECK(( match< seq< ichar<'A'>, ichar<'b'>, ichar<'C'> > >(s0) ));
		BOOST_CHECK(( match< seq< ichar<'a'>, ichar<'B'>, ichar<'c'> > >(s0) ));

		BOOST_CHECK(( match< istr<'a','b','c' > >(s0) ));
		BOOST_CHECK(( match< istr<'A','B','C' > >(s0) ));
		BOOST_CHECK(( match< istr<'A','b','C' > >(s0) ));
		BOOST_CHECK(( match< istr<'a','B','c' > >(s0) ));
	}
}
