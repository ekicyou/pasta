#pragma once

#include <boost/mpl/void.hpp>

namespace biscuit
{

template<
	class Parser,
	class Arg0 = boost::mpl::void_,
	class Arg1 = boost::mpl::void_,
	class Arg2 = boost::mpl::void_,
	class Arg3 = boost::mpl::void_,
	class Arg4 = boost::mpl::void_,
	class Arg5 = boost::mpl::void_,
	class Arg6 = boost::mpl::void_,
	class Arg7 = boost::mpl::void_,
	class Arg8 = boost::mpl::void_,
	class Arg9 = boost::mpl::void_
>
struct directive;

} // namespace biscuit
