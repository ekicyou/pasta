#pragma once

#include "../or.hpp"
#include "alpha.hpp"
#include "digit.hpp"

namespace biscuit
{

struct alnum :
	or_<
		alpha,
		digit
	>
{ };

} // namespace biscuit
