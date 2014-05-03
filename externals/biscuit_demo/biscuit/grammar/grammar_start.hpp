#pragma once

namespace biscuit
{

template< class Grammar >
struct grammar_start
{
	typedef typename Grammar::start type;
};

} // namespace biscuit
