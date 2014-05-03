#pragma once

#include <boost/mpl/bool.hpp>

namespace biscuit
{

#ifdef _DEBUG
	struct is_debug : boost::mpl::true_ { };
#else
	struct is_debug : boost::mpl::false_ { };
#endif

} // namespace biscuit

