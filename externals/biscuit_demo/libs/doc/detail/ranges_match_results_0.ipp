typedef match_results<c_comment, std::string> mrs_t;
typedef boost::range_const_iterator<mrs_t>::type iter_t;

std::string s0("  /* c comment no.1 */int i; /* c comment no.2 */i = 1; /* c comment no.3 */ ++i;  ");
mrs_t mrs(s0);
for (iter_t it = boost::const_begin(mrs); it != boost::const_end(mrs); ++it)
{
	std::cout << std::string(boost::begin(*it), boost::end(*it)) << std::endl;
}
