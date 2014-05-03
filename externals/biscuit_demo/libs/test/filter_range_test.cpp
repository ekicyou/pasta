
#include "../../biscuit/parser.hpp"
#include "../../biscuit/filter_range.hpp"
#include "../../biscuit/algorithm/match.hpp"

#include <boost/test/unit_test.hpp>
#include <boost/type_traits.hpp>
#include <boost/range.hpp>
#include <string>
#include <iostream>

using namespace biscuit;

struct c_comment :
	seq<
		str<'/','*'>,
		star_until< any, str<'*','/'> >
	>
{ };
	
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

struct xml_comment2 :
	seq<
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
	>
{ };

void filter_range_test()
{
	std::cout << "filter_range_test ing..." << std::endl;

	{
		typedef filter_range<c_comment, std::string> fr_t;
    typedef boost::range_const_iterator<fr_t>::type iter_t;

		std::string s0("  /* c comment no.1 */ int i; /* c comment no.2 */ i = 1; /* c comment no.3 */ ++i;  ");
		fr_t fr(s0);
		for (iter_t it = boost::const_begin(fr); it != boost::const_end(fr); ++it)
		{
			std::cout << *it;
		}
		
		std::cout << std::endl;
	}

	{
		typedef filter_range<xml_comment, std::string> fr_t;
    typedef boost::range_result_iterator<fr_t>::type iter_t;
    
		std::string s0(" <hello> <!-- xml comment no.1 -->x biscuit</hello> <wow>biscuit</wow> <!-- xml comment no.2 -->");
		fr_t fr(s0);
		for (iter_t it = boost::begin(fr); it != boost::end(fr); ++it)
		{
			std::cout << *it;
		}
		
		std::cout << std::endl;
	}

	{
		typedef filter_range<xml_comment, std::string> fr_t;
    typedef boost::range_result_iterator<fr_t>::type iter_t;
    
		std::string s0(" <hello> ");
		fr_t fr(s0);
		for (iter_t it = boost::begin(fr); it != boost::end(fr); ++it)
		{
			std::cout << *it;
			BOOST_CHECK( false );
		}
	}

	{
		typedef filter_range<xml_comment, std::string> fr_t;
    typedef boost::range_result_iterator<fr_t>::type iter_t;
    
		std::string s0("<!-- xml comment no.1 --><!-- xml comment no.2 -->");
		fr_t fr(s0);
		for (iter_t it = boost::begin(fr); it != boost::end(fr); ++it)
		{
			std::cout << *it;
		}
	}

	std::cout << std::endl;

	{
		typedef filter_range<not_<space>, std::string const> fr_t;
		std::string s0(" x      x x xx    ");
		fr_t fr(s0);
		BOOST_CHECK(( match< repeat< str<'x'>, 5> >(fr) ));
	}

	{
		typedef filter_range< c_comment, std::string > fr_t;
		std::string s0("  /* c comment no.1 */ int i; /* c comment no.2 */ i = 1; /* c comment no.3 */ ++i;  ");
		fr_t fr(s0);
		BOOST_CHECK(( match< repeat< c_comment, 3> >(fr) ));
	}

	{ // scanner chain
		std::string s0(" <hello> <!-- xml comment no.1 -->x biscuit</hello> <wow>biscuit</wow> <!-- xml comment no.2 -->");
		
		typedef filter_range< xml_comment, std::string > fr0_t;
		fr0_t fr0(s0);
		std::cout << std::string(boost::begin(fr0), boost::end(fr0)) << std::endl;
		BOOST_CHECK(( match< repeat< xml_comment, 2> >(fr0) ));
		BOOST_CHECK(( match< repeat< xml_comment2, 2> >(fr0) ));

		typedef filter_range<str<'-','-',' ','x','m','l'>, fr0_t> fr1_t;
		fr1_t fr1(fr0);
		BOOST_CHECK(( match< repeat< str<'-','-',' ','x','m','l'>, 2> >(fr1) ));

		typedef filter_range<str<'-','-'>, fr1_t > fr2_t;
		fr2_t fr2(fr1);
		BOOST_CHECK(( match< repeat< str<'-','-'>, 2> >(fr2) ));
	}

	{ // scanner chain
		BOOST_CHECK((	match< str<'x','y','z'> >(make_filter_range< not_<space> >("x  y     z")) ));
		
		BOOST_CHECK((
			match< str<'x','y','z'> >(
				make_filter_range< alpha >(
					make_filter_range< not_<space> >(
						make_filter_range< not_<digit> >("x & 4 y . 125 %  z")
					)
				)
			)
		));
	}
}
