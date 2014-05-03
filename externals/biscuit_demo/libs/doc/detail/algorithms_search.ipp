std::string s0("  /* c comment no.1 */x int i; /* c comment no.2 */ i = 1; /* c comment no.3 */ ++i;  ");
boost::sub_range<std::string> sr = search<c_comment>(s0);
BOOST_CHECK( std::string(boost::begin(sr), boost::end(sr)) == "/* c comment no.1 */" );
