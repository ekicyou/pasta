
#include <boost/test/unit_test.hpp>
#include <iostream>
#include <string>

#include "../../biscuit/algorithm/match.hpp"
#include "../../biscuit/parser.hpp"
#include "../../biscuit/filter_range.hpp"

using namespace biscuit;

struct integer : seq< opt< or_<str<'+'>,str<'-'> > >, plus<digit> > { };

struct expression ; // magic!
struct group      : seq< str<'('>, expression, str<')'> > { };
struct factor     : or_< integer, group > { };
struct term       : seq< factor, star< or_< seq< str<'*'>, factor >, seq< str<'/'>, factor > > > > { };
struct expression : seq< term, star< or_< seq< str<'+'>, term >, seq< str<'-'>, term > > > > { };

struct skip_space : not_<space> { };

void introduction_test()
{
	std::cout << "introduction_test ing..." << std::endl;
	{
		BOOST_CHECK( match<expression>( make_filter_range<skip_space>("12345") ) );
		BOOST_CHECK( match<expression>( make_filter_range<skip_space>("-12345") ) );
		BOOST_CHECK( match<expression>( make_filter_range<skip_space>("+12345") ) );
		BOOST_CHECK( match<expression>( make_filter_range<skip_space>("1 + 2") ) );
		BOOST_CHECK( match<expression>( make_filter_range<skip_space>("1 * 2") ) );
		BOOST_CHECK( match<expression>( make_filter_range<skip_space>("1/2 + 3/4") ) );
		BOOST_CHECK( match<expression>( make_filter_range<skip_space>("1 + 2 + 3 + 4") ) );
		BOOST_CHECK( match<expression>( make_filter_range<skip_space>("1 * 2 * 3 * 4") ) );
		BOOST_CHECK( match<expression>( make_filter_range<skip_space>("(1 + 2) * (3 + 4)") ) );
		BOOST_CHECK( match<expression>( make_filter_range<skip_space>("(-1 + 2) * (3 + -4)") ) );
		BOOST_CHECK( match<expression>( make_filter_range<skip_space>("1 + ((6 * 200) - 20) / 6") ) );
		BOOST_CHECK( match<expression>( make_filter_range<skip_space>("(1 + (2 + (3 + (4 + 5))))") ) );
		BOOST_CHECK( !match<expression>( make_filter_range<skip_space>("(1 + (2 + (3 + (4 + 5)))") ) );	
	}
}
