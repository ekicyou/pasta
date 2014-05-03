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

std::stringstream out;
std::string s0("<!-- xml comment no.1 -->");
match<xml_comment_action>(s0, out);
BOOST_CHECK( std::string("[<!-- xml comment no.1 -->]") == out.str() );
