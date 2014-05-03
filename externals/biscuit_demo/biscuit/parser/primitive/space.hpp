#pragma once

#include "char_set.hpp"

namespace biscuit
{

struct space :
	char_set<0x20,0x9,0xD,0xA>
{ };

} // namespace biscuit
