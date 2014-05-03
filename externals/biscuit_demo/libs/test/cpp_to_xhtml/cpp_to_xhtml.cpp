
#include "stdafx.h"

#include <fstream>

#include "cpp_to_xhtml.hpp"

int _tmain(int argc, _TCHAR* argv[])
{
	(argc);
	std::string filename(argv[1]);
	std::ifstream ifs(filename.c_str());
	std::string text;
	std::getline(ifs, text, '\0');
	tab2space(text);

	std::ofstream ofs( (filename+".xhtml").c_str() );
	cpp_to_xhtml the_grammar(ofs);

	ofs << "<pre class=\"cpp_source\">\n";
	match< typename grammar_start<cpp_to_xhtml>::type >(text, the_grammar);
	ofs << "</pre>\n";
	return 0;
}

