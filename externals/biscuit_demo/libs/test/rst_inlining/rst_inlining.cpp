
#include "stdafx.h"

// <pre class="literal-block">
// example_algorithms_match.ipp
// </pre>

#include <fstream>
#include <cassert>
#include <string>

#include "../cpp_to_xhtml/cpp_to_xhtml.hpp"
#include "../../../biscuit/action.hpp"
#include "../../../biscuit/parser.hpp"


struct on_inline_filename
{
	template< class ForwardIter, class UserState >
	void operator()(ForwardIter first, ForwardIter last, UserState& out)
	{
		std::string filename(first, last);
		std::ifstream ifs(filename.c_str());
		assert( ifs.good() );
		std::string text;
		std::getline(ifs, text, '\0');
		tab2space(text);
		cpp_to_xhtml the_grammar(out);
		if (!match< typename grammar_start<cpp_to_xhtml>::type >(text, the_grammar))
		{
			assert(false);
		}
	}
};

struct on_pre_stag
{
	template< class ForwardIter, class UserState >
	void operator()(ForwardIter first, ForwardIter last, UserState& out)
	{
		(first); (last);
		out << "<pre class=\"cpp_source\">\n";
	}
};

struct on_pre_etag
{
	template< class ForwardIter, class UserState >
	void operator()(ForwardIter first, ForwardIter last, UserState& out)
	{
		(first); (last);
		out << "</pre>\n";
	}
};

struct rst_pre_content :
	star_until< not_< str<'<'> >, str<'.','i','p','p'> >
{ };

struct rst_pre_stag :
	seq<
		str<'<'>, star<space>,
		str<'p','r','e'>, plus<space>,
		str<'c','l','a','s','s'>, star<space>, str<'='>, star<space>,
		seq<
			str<'"'>, str<'l','i','t','e','r','a','l'>, str<'-','b','l','o','c','k'>, str<'"'>, star<space>,
			str<'>'>, star<space>
		>
	>
{ };

struct rst_pre_etag :
	seq< star<space>, str<'<','/','p','r','e','>'> >
{ };


struct rst_pre_element : definitive_actions<
	seq<
		actor< rst_pre_stag,		on_pre_stag >,
		actor< rst_pre_content,	on_inline_filename>,
		actor< rst_pre_etag,		on_pre_etag >
	>
>
{ };

int _tmain(int argc, _TCHAR* argv[])
{
	(argc); (argv);
	if (argc > 1) {
		std::string filename(argv[1]);
		std::ifstream ifs(filename.c_str());
		assert( ifs.good() );
		std::string text;
		std::getline(ifs, text, '\0');

		std::ofstream ofs( (filename + ".html").c_str());
		assert( ofs.good() );

		iterate<rst_pre_element>(text, ofs, output_action());
	}
	return 0;
}

