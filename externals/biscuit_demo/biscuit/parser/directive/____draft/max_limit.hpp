#pragma once

#include "limit.hpp"

namespace biscuit
{

template<
	class Parser, unsigned int min, unsigned int max = min
>
struct max_limit : limit<0, max>
{ };

} // namespace biscuit
