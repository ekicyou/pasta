#pragma once

#include "char_range.hpp"
#include "../or.hpp"

namespace biscuit
{

struct alpha :
	or_<
		char_range<'a','z'>,
		char_range<'A','Z'>
	>
{ };

} // namespace biscuit
