
#include "../../biscuit/parser.hpp"
#include "../../biscuit/match_results.hpp"

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

void match_results_test()
{
	std::cout << "match_results_test ing..." << std::endl;

	{
		typedef match_results<c_comment, std::string> mrs_t;
    typedef boost::range_const_iterator<mrs_t>::type iter_t;

		std::string s0("  /* c comment no.1 */int i; /* c comment no.2 */i = 1; /* c comment no.3 */ ++i;  ");
		mrs_t mrs(s0);
		for (iter_t it = boost::const_begin(mrs); it != boost::const_end(mrs); ++it)
		{
			std::cout << std::string(boost::begin(*it), boost::end(*it)) << std::endl;
		}
	}

	{
		typedef match_results<xml_comment, std::string> mrs_t;
    typedef boost::range_result_iterator<mrs_t>::type iter_t;
    
		std::string s0(" <hello> <!-- xml comment no.1 --> biscuit</hello> <wow>biscuit</wow> <!-- xml comment no.2 -->");
		mrs_t mrs(s0);
		for (iter_t it = boost::begin(mrs); it != boost::end(mrs); ++it)
		{
			boost::sub_range<std::string> sr = *it; // required for detering const-qualified.
			std::string::iterator p = boost::begin(sr);
			*p = '['; // mischief :-)
			std::cout << std::string(boost::begin(*it), boost::end(*it)) << std::endl;
		}
	}

	{
		typedef match_results<xml_comment, std::string> mrs_t;
    typedef boost::range_result_iterator<mrs_t>::type iter_t;
    
		std::string s0(" <hello> <!-- xml comment no.2 -- -->");
		mrs_t mrs(s0);
		for (iter_t it = boost::begin(mrs); it != boost::end(mrs); ++it)
		{
			BOOST_CHECK( false );
		}
	}
}
