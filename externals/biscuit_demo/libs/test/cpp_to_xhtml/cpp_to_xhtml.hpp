#pragma once

// Emulation of legacy Spirit's sample from
// http://spirit.sourceforge.net/distrib/spirit_1_6_1/libs/spirit/example/application/cpp_to_html/cpp_to_html.cpp
// Same bugs stand!

#include <iostream>
#include <string>
#include <boost/range/iterator_range.hpp>
#include <boost/noncopyable.hpp>

#include "../../../biscuit/grammar.hpp"
#include "../../../biscuit/parser.hpp"
#include "../../../biscuit/algorithm.hpp"
#include "../../../biscuit/action.hpp"
#include "cpp_keyword.hpp"

using namespace biscuit;

struct cpp_to_xhtml : grammar<cpp_to_xhtml, std::string>, private boost::noncopyable
{
	std::ostream& out;
	cpp_to_xhtml(std::ostream& out_) : out(out_) { }
	
	struct program;
	struct preprocessor;
	struct comment;
	struct keyword;
	struct special;
	struct string_;
	struct literal;
	struct number;
	struct identifier;
	typedef program start;

	void escape0_action(iterator_type, iterator_type) { out << "&amp;"; }
	void escape1_action(iterator_type, iterator_type) { out << "&lt;"; }
	void escape2_action(iterator_type, iterator_type) { out << "&gt;"; }
	void escape3_action(iterator_type, iterator_type) { out << "&quot;"; }

	struct output_
	{
		void operator()(iterator_type first, iterator_type last, cpp_to_xhtml& self)
		{
			self.out << std::string(first, last);
		}
	};

	struct xhtml_special :
		or_<
			actor_< str<'&'>, &escape0_action >,
			actor_< str<'<'>, &escape1_action >,
			actor_< str<'>'>, &escape2_action >,
			actor_< str<'"'>, &escape3_action >
		>
	{ };

	void escape_xhtml(iterator_type first, iterator_type last)
	{
		iterate<xhtml_special>(boost::make_iterator_range(first, last), *this, output_());
	}

	// actions
	void on_keyword(iterator_type first, iterator_type last)
	{
		out << "<span class=\"cpp_keyword\">";
		escape_xhtml(first, last);
		out << "</span>";
	}
	
	void on_preprocessor(iterator_type first, iterator_type last)
	{
		out << "<span class=\"cpp_pp_directive\">";
		escape_xhtml(first, last);
		out << "</span>";
	}
	
	void on_string_(iterator_type first, iterator_type last)
	{
		out << "<span class=\"cpp_string_literal\">";
		escape_xhtml(first, last);
		out << "</span>";
	}
	
	void on_unexpected_char(iterator_type first, iterator_type last)
	{
		escape_xhtml(first, last);
	}
	
	void on_comment(iterator_type first, iterator_type last)
	{
		out << "<span class=\"cpp_comment\">";
		escape_xhtml(first, last);
		out << "</span>";
	}

	void on_identifier(iterator_type first, iterator_type last)
	{
		escape_xhtml(first, last);
	}

	void on_special(iterator_type first, iterator_type last)
	{
		escape_xhtml(first, last);
	}

	void on_literal(iterator_type first, iterator_type last)
	{
		out << "<span class=\"cpp_string_literal\">";
		escape_xhtml(first, last);
		out << "</span>";
	}

	void on_number(iterator_type first, iterator_type last)
	{
		out << "<span class=\"cpp_number_literal\">";
		escape_xhtml(first, last);
		out << "</span>";
	}

	struct program :
		seq<
			star<space>,
			star<
				or_<
					actor_< preprocessor,	&on_preprocessor >,
					actor_< comment,			&on_comment >,
					actor_< keyword, 			&on_keyword >,
					actor_< identifier,		&on_identifier >,
					actor_< special,			&on_special >,
					actor_< string_,			&on_string_ >,
					actor_< literal,			&on_literal >,
					actor_< number,				&on_number >,
					actor_< any,					&on_unexpected_char >
				>
			>
		>
	{ };
	
	struct preprocessor :
		seq<
			str<'#'>,
			seq< or_< alpha, str<'_'> >, star< or_<alnum, str<'_'> > > >,
			star<space>
		>
	{ };
	
	struct cpp_comment_p :
		seq< str<'/','/'>, star_before< any, eol > >
	{ };
	
	struct c_comment_p :
		seq< str<'/','*'>, star_until< any, str<'*','/'> > >
	{ };
	
	struct comment :
		seq<
			plus< or_< c_comment_p, cpp_comment_p > >,
			star<space>
		>
	{ };

	struct keyword :
		seq<
			cpp_keyword,
			not_< before< or_< alnum, str<'_'> > > >,
			star<space>
		>
	{ };
	
	struct special :
		seq<
			or_<
				char_set<'~','!','%','^','&','*','(',')','+','='>,
				char_set<'{','[','}',']',':',';',',','<','.','>'>,
				char_set<'?','/','|','\\','-'>
			>,
			star<space>
		>
	{ };
	
	struct string_ :
		seq<
			opt< char_set<'l','L'> >,
			seq<
				str<'"'>,
				star<
					or_<
						seq< str<'\\'>, any >,
						not_< str<'"'> >
					>
				>
			>,
			str<'"'>
		>
	{ };

	struct literal :
		seq<
			opt< char_set<'l','L'> >,
			seq<
				str<'\''>,
				star<
					or_<
						seq< str<'\\'>, any >,
						not_< str<'\''> >
					>
				>,
				str<'\''>
			>
		>
	{ };


	struct real_p :
		seq<
			opt< char_set<'+','-'> >,
			or_<
				seq< plus<digit>, str<'.'>, plus<digit> >,
				seq< plus<digit>, str<'.'> >,
				seq< str<'.'>, plus<digit> >,
				plus<digit>
			>,
			opt<
				seq<
					char_set<'e','E'>,
					opt< char_set<'+','-'> >,
					plus<digit>
				>
			>
		>
	{ };
	
	struct hex_p :
		plus<
			or_< char_range<'0','9'>, char_range<'a','f'>, char_range<'A','F'> >
		>
	{ };
	
	struct oct_p :
		plus<
			char_range<'0','7'>
		>
	{ };

	struct number :
		seq<
			or_<
				real_p,
				seq< as_lower< str<'0','x'> >, hex_p >,
				seq< str<'0'>, oct_p >
			>,
			star< as_lower< char_set<'l','d','f','u'> > >,
			star<space>
		>
	{ };
	
	struct identifier :
		seq<
			seq<
				or_< alpha, str<'_'> >,
				star< or_< alnum, str<'_'> > >
			>,
			star<space>
		>
	{ };
	
};


// thanks to http://cham.ne.jp/piro/p_cp.html
inline std::string& tab2space(std::string& str_, int tab_ = 2) {
	std::string::size_type t_, n_, i_, nb_=0;
	while ((nb_ = t_ = str_.find('\t',nb_)) != std::string::npos) {
		n_ = str_.rfind('\n',t_) + 1;
		if (n_ == std::string::npos) n_ = 0;
		i_ = tab_ - (t_ - n_)%tab_;
		str_.replace(t_, 1, i_, ' ');
	}
	return str_;
}
