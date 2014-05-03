#pragma once

#include "star_until.hpp"
#include "before.hpp"

namespace biscuit
{

template<
	class ParserA, class ParserB
>
struct star_before : star_until< ParserA, before<ParserB> >
{ };

} // namespace biscuit
