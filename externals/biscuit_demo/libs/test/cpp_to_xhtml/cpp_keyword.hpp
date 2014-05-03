#pragma once

#include "../../../biscuit/parser/or.hpp"
#include "../../../biscuit/parser/seq.hpp"
#include "../../../biscuit/parser/primitive/str.hpp"

namespace {
	
	using namespace biscuit;

	struct cpp_keyword0 :
		or_<
			str<'a','n','d'>,
			str<'a','n','d','_','e','q'>,
			str<'b','i','t','a','n','d'>,
			str<'b','i','t','o','r'>,
			str<'c','o','m','p','l'>,
			str<'n','o','t'>,
			str<'o','r'>,
			str<'n','o','t','_','e','q'>,
			str<'c','a','s','e'>,
			str<'d','e','f','a','u','l','t'>
		>
	{ };

	struct cpp_keyword1 :
		or_<
			str<'i','f'>,
			str<'e','l','s','e'>,
			str<'s','w','i','t','c','h'>,
			str<'c','h','a','r'>,
			str<'w','c','h','a','r','_','t'>,
			str<'b','o','o','l'>,
			str<'s','h','o','r','t'>,
			str<'i','n','t'>,
			str<'l','o','n','g'>,
			str<'s','i','g','n','e','d'>
		>
	{ };

	struct cpp_keyword2 :
		or_<
			str<'u','n','s','i','g','n','e','d'>,
			str<'c','l','a','s','s'>,
			str<'s','t','r','u','c','t'>,
			str<'u','n','i','o','n'>,
			str<'p','r','i','v','a','t','e'>,
			str<'p','r','o','t','e','c','t','e','d'>,
			str<'p','u','b','l','i','c'>,
			str<'o','p','e','r','a','t','o','r'>,
			str<'e','n','u','m'>,
			str<'n','a','m','e','s','p','a','c','e'>
		>
	{ };

	struct cpp_keyword3 :
		or_<
			str<'u','s','i','n','g'>,
			str<'a','s','m'>,
			str<'c','o','n','s','t'>,
			str<'v','o','l','a','t','i','l','e'>,
			str<'e','x','p','o','r','t'>,
			str<'t','e','m','p','l','a','t','e'>,
			str<'f','a','l','s','e'>,
			str<'t','r','u','e'>,
			str<'f','r','i','e','n','d'>,
			str<'t','y','p','e','d','e','f'>
		>
	{ };
	
	struct cpp_keyword4 :
		or_<
			str<'a','u','t','o'>,
			str<'r','e','g','i','s','t','e','r'>,
			str<'s','t','a','t','i','c'>,
			str<'e','x','t','e','r','n'>,
			str<'m','u','t','a','b','l','e'>,
			str<'i','n','l','i','n','e'>,
			str<'t','h','i','s'>,
			seq< str<'d','y','n','a','m','i','c','_','c','a'>, str<'s','t'> >,
			seq< str<'s','t','a','t','i','c','_','c','a','s'>, str<'t'> >,
			seq< str<'r','e','i','n','t','e','r','p','r','e'>, str<'t','_','c','a','s','t'> >
		>
	{ };
	
	struct cpp_keyword5 :
		or_<
			str<'c','o','n','s','t','_','c','a','s','t'>,
			str<'t','r','y'>,
			str<'c','a','t','c','h'>,
			str<'t','h','r','o','w'>,
			str<'t','y','p','e','i','d'>,
			str<'t','y','p','e','n','a','m','e'>,
			str<'s','i','z','e','o','f'>,
			str<'n','e','w'>,
			str<'d','e','l','e','t','e'>,
			str<'u','n','s','i','g','n','e','d'>
		>
	{ };
	
	struct cpp_keyword6 :
		or_<
			str<'f','l','o','a','t'>,
			str<'d','o','u','b','l','e'>,
			str<'v','o','i','d'>,
			str<'v','i','r','t','u','a','l'>,
			str<'e','x','p','l','i','c','i','t'>,
			str<'w','h','i','l','e'>,
			str<'d','o'>,
			str<'f','o','r'>,
			str<'b','r','e','a','k'>,
			str<'c','o','n','t','i','n','u','e'>
		>
	{ };

	struct cpp_keyword7 :
		or_<
			str<'r','e','t','u','r','n'>,
			str<'g','o','t','o'>,
			str<'x','o','r'>,
			str<'o','r','_','e','q'>,
			str<'x','o','r','_','e','q'>	
		>
	{ };

	struct cpp_keyword :
		or_<
			cpp_keyword0,
			cpp_keyword1,
			cpp_keyword2,
			cpp_keyword3,
			cpp_keyword4,
			cpp_keyword5,
			cpp_keyword6,
			cpp_keyword7
		>
	{ };

}