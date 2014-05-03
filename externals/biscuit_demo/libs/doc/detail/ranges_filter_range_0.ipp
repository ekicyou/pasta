typedef filter_range< c_comment, std::string > fr_t;
std::string s0("  /* c comment no.1 */ int i; /* c comment no.2 */ i = 1; /* c comment no.3 */ ++i;  ");
fr_t fr(s0);
BOOST_CHECK(( match< repeat< c_comment, 3 > >(fr) ));
