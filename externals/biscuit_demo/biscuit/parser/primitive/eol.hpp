#pragma once

#include "../or.hpp"
#include "str.hpp"

namespace biscuit
{

struct eol :
	or_<
		str<0xD>,
		str<0xA>,
		str<0xD,0xA>,
		str<0xA,0xD>
	>
{ };

} // namespace biscuit
