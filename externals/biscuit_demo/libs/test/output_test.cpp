
#include <boost/test/unit_test.hpp>
#include <iostream>
#include <string>
#include <sstream>

#include "../../biscuit/algorithm/iterate.hpp"
#include "../../biscuit/parser.hpp"
#include "../../biscuit/action/output_action.hpp"
#include "../../biscuit/action/empty_action.hpp"

using boost::unit_test::test_suite;
using namespace biscuit;

struct c_comment : actor<
	seq<
		str<'/','*'>,
		star_until< any, str<'*','/'> >
	>,
	output_action
>
{ };

struct decorate_action
{
	template< class ForwardIter, class Stream >
	void operator()(ForwardIter first, ForwardIter last, Stream& out)
	{
		out << "[";
		out << std::string(first, last);
		out << "]";
	}
};

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

struct xml_comment_action : actor< xml_comment, decorate_action >
{ };


void output_test()

{
	std::cout << "output_test ing..." << std::endl;
	{
		std::stringstream out;
		std::string s0("  /* c comment no.1 */ int i; /* c comment no.2 */ i = 1; /* c comment no.3 */ ++i;  ");
		iterate<c_comment>(s0, out, empty_action());
		BOOST_CHECK( std::string("/* c comment no.1 *//* c comment no.2 *//* c comment no.3 */") == out.str() );
	}

	{
		std::stringstream out;
		std::string s0("<!-- xml comment no.1 -->");
		match<xml_comment_action>(s0, out);
		BOOST_CHECK( std::string("[<!-- xml comment no.1 -->]") == out.str() );
	}

	{
		std::stringstream out;
		std::string s0(" <hello> <!-- xml comment no.1 --> biscuit</hello> <wow>biscuit</wow> <!-- xml comment no.2 -->  ");
		iterate<xml_comment_action>(s0, out, output_action());
		BOOST_CHECK( std::string(" <hello> [<!-- xml comment no.1 -->] biscuit</hello> <wow>biscuit</wow> [<!-- xml comment no.2 -->]  ") == out.str() );
	}
}
