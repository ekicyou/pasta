
#include <boost/test/unit_test.hpp>
#include <boost/ref.hpp>
#include <iostream>
#include <fstream>
#include <string>

#include "../../biscuit/algorithm/match.hpp"
#include "../../biscuit/grammar.hpp"
#include "../../biscuit/parser.hpp"

using namespace biscuit;

template< class ForwardRange >
struct xml_grammar : grammar<xml_grammar, ForwardRange>
{
  struct Char :
		or_<
			ucs4<0x9>, ucs4<0xA>, ucs4<0xD>,
			ucs4_range<0x20,0xD7FF>,
			ucs4_range<0xE000,0xFFFD>,
			ucs4_range<0x10000,0x10FFFF>
		>
	{ };

	struct CharData :
		minus<
			star< not_< char_set<'<','&'> > >,
			seq<
				star_until< not_< char_set<'<','&'> >, str<']',']','>'> >,
				star< not_< char_set<'<','&'> > >
			>
		>
	{ };

	// semantic action
	void comment_action(iterator_type first, iterator_type last)
	{
		std::wcout << L"XML Comment: " << std::wstring(first, last) << std::endl;
	}

	struct Comment : actor_<
		seq<
			str<'<','!','-','-'>,
			star<
				or_<
					minus< Char, str<'-'> >,
					seq<
						str<'-'>,
						minus< Char, str<'-'> >
					>
				>
			>,
			str<'-','-','>'>
		>,
		&xml_grammar::comment_action
	>
	{ };

	struct S :
		plus< char_set<0x20,0x9,0xD,0xA> >
	{ };

	struct Digit :
		or_<
			ucs4_range<0x0030,0x0039>, ucs4_range<0x0660,0x0669>
			// too many
		>
	{ };

	struct BaseChar :
		or_<
			ucs4_range<0x0041,0x005A>, ucs4_range<0x0061,0x007A>
		>
	{ };

	struct Ideographic :
		or_<
			ucs4_range<0x4E00,0x9FA5>, ucs4<0x3007>,
			ucs4_range<0x3021,0x3029>
		>
	{ };

	struct CombiningChar :
		or_<
			ucs4_range<0x0300,0x0345>, ucs4_range<0x0360,0x0361>
			// too many
		>
	{ };

	struct Extender :
		or_<
			ucs4<0x00B7>, ucs4<0x02D0>, ucs4<0x02D1>, ucs4<0x0387>
			// many
		>
	{ };

	struct Letter :
		or_< BaseChar, Ideographic >
	{ };

	struct NameChar :
		or_<
			Letter, Digit, str<'.'>, str<'-'>, str<'_'>, str<':'>,
			CombiningChar, Extender
		>
	{ };

	struct Name :
		seq<
			or_< Letter, str<'_'>, str<':'> >,
			star< NameChar >
		>
	{ };

	struct Eq :
		seq< opt<S>, str<'='>, opt<S> >
	{ };

	struct PITarget :
		minus<
			Name,
			seq<
				or_< str<'X'>, str<'x'> >,
				or_< str<'M'>, str<'m'> >,
				or_< str<'L'>, str<'l'> >
			>
		>
	{ };
		
	struct PI :
		seq<
			str<'<','?'>,
			PITarget,
			opt<
				seq< S, star_before< Char, str<'?','>'> > >
			>,
			str<'?','>'>
		>
	{ };

	struct CDStart :
		str<'<','!','[','C','D','A','T','A','['>
	{ };

	struct CData :
		star_before< Char, str<']',']','>'> >
	{ };

	struct CDEnd :
		str<']',']','>'>
	{ };

	struct CDSect :
		seq< CDStart, CData, CDEnd >
	{ };

	struct doctypedecl :
		seq<
			str<'<','!','D','O','C','T','Y','P','E'>,
			star_until< Char, str<'>'> >
		>
	{ };

	struct XMLDecl :
		seq<
			str<'<','?','x','m','l'>,
			star_until< Char, str<'?','>'> >
		>
	{ };

	struct Misc :
		or_< Comment, PI, S >
	{ };

	struct prolog :
		seq<
			opt<XMLDecl>,
			star<Misc>,
			opt< seq< doctypedecl, star<Misc> > >
		>
	{ };

	struct CharRef :
		or_<
			seq<
				str<'&','#'>,
				plus< char_range<'0','9'> >,
				str<';'>
			>,
			seq<
				str<'&','#','x'>,
				plus< or_< char_range<'0','9'>, char_range<'a','f'>, char_range<'A','F'> > >,
				str<';'>
			>
		>
	{ };

	struct EntityRef :
		seq< str<'&'>, Name, str<';'> >
	{ };

	struct Reference :
		or_<EntityRef, CharRef>
	{ };

	struct AttValue :
		or_<
			seq<
				str<'"'>,
				star<
					or_<
						not_< char_set<'<','&','"'> >,
						Reference
					>
				>,
				str<'"'>
			>,
			seq<
				str<'\''>,
				star<
					or_<
						not_< char_set<'<','&','\''> >,
						Reference
					>
				>,
				str<'\''>
			>
		>
	{ };

	struct Attribute :
		seq< Name, Eq, AttValue >
	{ };

	struct EmptyElemTag :
		seq<
			str<'<'>,
			Name,
			star< seq<S,Attribute> >,
			opt<S>,
			str<'/','>'>
		>
	{ };

	struct STag :
		seq<
			str<'<'>,
			Name,
			star< seq<S,Attribute> >,
			opt<S>,
			str<'>'>
		>
	{ };

	struct ETag :
		seq<
			str<'<','/'>,
			Name,
			opt<S>,
			str<'>'>
		>
	{ };
	
	struct element;

	struct content :
		seq<
			opt<CharData>,
			star<
				seq<
					or_<
						element, // the magical recursion!
						Reference,
						CDSect,
						PI,
						Comment
					>,
					opt<CharData>
				>
			>
		>
	{ };

	struct element :
		or_<
			EmptyElemTag, 
			seq<STag, content, ETag>
		>
	{ };

	struct document :
		seq<
			prolog,
			element,
			star<Misc>
		>
	{ };
	
	struct start : document
	{ };
};

void xml_grammar_match_test()
{
	std::cout << "xml_grammar_match_test ing..." << std::endl;
	std::wifstream fin("xml_grammar_match_test.xml");	// put the file to your project directory
	BOOST_REQUIRE( fin.good() );
	std::wstring text;
	std::getline(fin, text, wchar_t(0xFFFF));					// hack :-)

	typedef xml_grammar<std::wstring> grammar_type;
	grammar_type the_grammar;
	BOOST_CHECK( match< typename grammar_start<grammar_type>::type >(text, the_grammar) );
	// BOOST_CHECK( the_grammar.match(text) );
}
