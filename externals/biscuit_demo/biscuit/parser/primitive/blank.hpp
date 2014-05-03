#pragma once

#include "char_set.hpp"

namespace biscuit
{

struct blank :
	char_set<0x20,0x9>
{ };

} // namespace biscuit
