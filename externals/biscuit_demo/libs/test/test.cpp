
#include <boost/test/unit_test.hpp>
using boost::unit_test::test_suite;

#define BOOST_LIB_NAME boost_unit_test_framework
#include <boost/config/auto_link.hpp>

void match_results_test();
void match_test();
void repeat_test();
void output_test();
void xml_grammar_match_test();
void as_lower_test();
void grm_value_test();
void search_test();
void filter_range_test();
void introduction_test();
void calc_plain_test();
void calc_debug_test();
void shortest_longest_test();
void istr_test();
void requires_test();
//void symbols_test();

test_suite*
init_unit_test_suite(int, char* [])
{
	test_suite *test = BOOST_TEST_SUITE("biscuit test");

	test->add( BOOST_TEST_CASE(&match_results_test) );
	test->add( BOOST_TEST_CASE(&match_test) );
	test->add( BOOST_TEST_CASE(&repeat_test) );
	test->add( BOOST_TEST_CASE(&output_test) );
	test->add( BOOST_TEST_CASE(&as_lower_test) );
	test->add( BOOST_TEST_CASE(&grm_value_test) );
	test->add( BOOST_TEST_CASE(&search_test) );
	test->add( BOOST_TEST_CASE(&xml_grammar_match_test) );
	test->add( BOOST_TEST_CASE(&filter_range_test) );
	test->add( BOOST_TEST_CASE(&introduction_test) );
	test->add( BOOST_TEST_CASE(&shortest_longest_test) );
	test->add( BOOST_TEST_CASE(&istr_test) );
	test->add( BOOST_TEST_CASE(&requires_test) );
	//test->add( BOOST_TEST_CASE(&symbols_test) );

	//test->add( BOOST_TEST_CASE(&calc_plain_test) );
	test->add( BOOST_TEST_CASE(&calc_debug_test) );
	return test;
}
