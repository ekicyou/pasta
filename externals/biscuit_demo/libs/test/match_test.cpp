
#include <boost/test/unit_test.hpp>
#include <iostream>
#include <string>

#include "../../biscuit/algorithm/match.hpp"
#include "../../biscuit/parser.hpp"

using namespace biscuit;

struct xml_comment :
	seq<
		str<'<','!','-','-'>,
		star<
			or_<
				minus< any, str<'-'> >,
				seq<
					str<'-'>,
					minus< any, str<'-'> >
				>
			>
		>,
		str<'-','-','>'>
	>
{ };

void match_test()
{
	std::cout << "match_test ing..." << std::endl;
	{
		typedef seq<
			str<'/','*'>,
			star_until< any, str<'*','/'> >
		> c_comment;

		BOOST_CHECK( match<c_comment>("/* hello, c comment */") );
	}
	
	{
		BOOST_CHECK( match<xml_comment>("<!-- hello, xml comment -->") );
		BOOST_CHECK( !match<xml_comment>("<!-- not well-formed comment -- -->") );
	}
}
