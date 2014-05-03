#pragma once

#include "repeat.hpp"

namespace biscuit
{

template<
	class Parser, unsigned int max
>
struct max_repeat : repeat<Parser, 0, max>
{ };

} // namespace biscuit
