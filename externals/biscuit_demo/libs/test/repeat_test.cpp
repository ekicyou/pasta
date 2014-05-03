
#include <boost/test/unit_test.hpp>
#include <iostream>
#include <string>

#include "../../biscuit/algorithm/match.hpp"
#include "../../biscuit/parser.hpp"

using boost::unit_test::test_suite;
using namespace biscuit;

void repeat_test()
{
	std::cout << "repeat_test ing..." << std::endl;
	{
		std::string s0("xxxxx");
		BOOST_CHECK(( match< repeat< str<'x'>, 4, 6 > >(s0) ));
		BOOST_CHECK(( match< repeat< str<'x'>, 5, 5 > >(s0) ));
		BOOST_CHECK(( match< repeat< str<'x'>, 5 > >(s0) ));
		BOOST_CHECK(( match< repeat< str<'x'>, 0, 5 > >(s0) ));
		BOOST_CHECK(( match< repeat< str<'x'>, 1, 200 > >(s0) ));
		BOOST_CHECK(( match< repeat< str<'x'>, 5, 500 > >(s0) ));

		BOOST_CHECK(( !match< repeat< str<'x'>, 2, 4 > >(s0) ));
		BOOST_CHECK(( !match< repeat< str<'x'>, 0, 4 > >(s0) ));
		BOOST_CHECK(( !match< repeat< str<'x'>, 4 > >(s0) ));
		BOOST_CHECK(( !match< repeat< str<'x'>, 6, 100 > >(s0) ));
		BOOST_CHECK(( !match< repeat< str<'x'>, 40, 200 > >(s0) ));

		BOOST_CHECK(( !match< max_repeat< str<'x'>, 4 > >(s0) ));
		BOOST_CHECK(( match< max_repeat< str<'x'>, 5 > >(s0) ));
		BOOST_CHECK(( match< max_repeat< str<'x'>, 6 > >(s0) ));
		BOOST_CHECK(( match< max_repeat< str<'x'>, 7 > >(s0) ));
	}

	typedef seq<
		str<'/','*'>,
		star_until< any, str<'*','/'> >
	> c_comment;

	{
		std::string s0("/* c comment no.1 *//* c comment no.2 *//* c comment no.3 */");
		BOOST_CHECK(( match< repeat< c_comment, 1, 6 > >(s0) ));
		BOOST_CHECK(( match< repeat< c_comment, 1, 3 > >(s0) ));
		BOOST_CHECK(( match< repeat< c_comment, 3 > >(s0) ));
		BOOST_CHECK(( match< repeat< c_comment, 3, 600 > >(s0) ));
		BOOST_CHECK(( match< repeat< c_comment, 2, 50 > >(s0) ));


		BOOST_CHECK(( !match< repeat< c_comment, 2, 2 > >(s0) ));
		BOOST_CHECK(( !match< repeat< c_comment, 1, 2 > >(s0) ));
		BOOST_CHECK(( !match< repeat< c_comment, 100, 200 > >(s0) ));
		BOOST_CHECK(( !match< repeat< c_comment, 0 > >(s0) ));
		BOOST_CHECK(( !match< repeat< c_comment, 4, 200 > >(s0) ));

		BOOST_CHECK(( match< min_repeat< c_comment, 3 > >(s0) ));
		BOOST_CHECK(( !match< min_repeat< c_comment, 4 > >(s0) ));
	}

	{
		std::string s0("/* c comment no.1 *//* c comment no.2 *//* c comment no.3 */xxxxx");
		BOOST_CHECK(( match< seq< repeat< c_comment, 1, 6 >, repeat< str<'x'>, 3, 5 > > >(s0) ));
		BOOST_CHECK(( match< seq< repeat< c_comment, 1, 60 >, repeat< str<'x'>, 0, 10 > > >(s0) ));
		BOOST_CHECK(( match< seq< repeat< c_comment, 0, 30 >, repeat< str<'x'>, 3, 50 > > >(s0) ));
		BOOST_CHECK(( match< seq< repeat< c_comment, 3 >, repeat< str<'x'>, 5 > > >(s0) ));
		BOOST_CHECK(( match< seq< repeat< c_comment, 3, 60 >, repeat< str<'x'>, 5, 50 > > >(s0) ));

		BOOST_CHECK(( !match< seq< repeat< c_comment, 1, 2 >, repeat< str<'x'>, 3, 5 > > >(s0) ));
		BOOST_CHECK(( !match< seq< repeat< c_comment, 6, 6 >, repeat< str<'x'>, 0, 10 > > >(s0) ));
		BOOST_CHECK(( !match< seq< repeat< c_comment, 3 >, repeat< str<'x'>, 3 > > >(s0) ));
		BOOST_CHECK(( !match< seq< repeat< c_comment, 0 >, repeat< str<'x'>, 5 > > >(s0) ));
		BOOST_CHECK(( !match< seq< repeat< c_comment, 0, 60 >, repeat< str<'x'>, 6, 50 > > >(s0) ));
	}

	typedef seq<
		str<'<','!','-','-'>,
		star_until<
			or_<
				minus< any, str<'-'> >,
				seq<
					str<'-'>,
					minus< any, str<'-'> >
				>
			>,
			str<'-','-','>'>
		>		
	> xml_comment;

	typedef seq<
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
	> xml_comment2;

	std::string s0("<!-- xml comment no.1 --><!-- xml comment no.2 -->");
	BOOST_CHECK(( match< repeat< xml_comment, 2, 2 > > (s0) ));
	BOOST_CHECK(( match< repeat< xml_comment, 2, 60 > >(s0) ));
	BOOST_CHECK(( match< repeat< xml_comment, 0, 30 > >(s0) ));
	BOOST_CHECK(( match< repeat< xml_comment2, 2 > >(s0) ));
	BOOST_CHECK(( match< repeat< xml_comment, 1, 60 > >(s0) ));

	BOOST_CHECK(( !match< repeat< xml_comment, 1, 1 > >(s0) ));
	BOOST_CHECK(( !match< repeat< xml_comment, 6, 6 > >(s0) ));
	BOOST_CHECK(( !match< repeat< xml_comment, 3 > >(s0) ));
	BOOST_CHECK(( !match< repeat< xml_comment2, 50 > >(s0) ));
	BOOST_CHECK(( !match< repeat< xml_comment, 0, 0 > >(s0) ));
}
