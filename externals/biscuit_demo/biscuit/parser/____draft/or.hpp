#pragma once

#include "../utility/ignore_unused_variables_warning.hpp"
#include "detail/na.hpp"

namespace biscuit
{

// primary decl.
template<
	class Parser0 = na, class Parser1 = na, class Parser2 = na, class Parser3 = na, class Parser4 = na,
	class Parser5 = na, class Parser6 = na, class Parser7 = na, class Parser8 = na, class Parser9 = na
>
struct or_;

// 0
template<
>
struct or_<>
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		ignore_unused_variables_warning(s, us);
		return false;
	}
};

// 1
template<
	class Parser0
>
struct or_<
	Parser0
>
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		return
			Parser0::parse(s, us)
		;
	}
};

// 2
template<
	class Parser0, class Parser1
>
struct or_<
	Parser0, Parser1
>
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		return
			Parser0::parse(s, us) ||
			Parser1::parse(s, us)
		;
	}
};

// 3
template<
	class Parser0, class Parser1, class Parser2
>
struct or_<
	Parser0, Parser1, Parser2
>
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		return
			Parser0::parse(s, us) ||
			Parser1::parse(s, us) ||
			Parser2::parse(s, us)
		;
	}
};

// 4
template<
	class Parser0, class Parser1, class Parser2, class Parser3
>
struct or_<
	Parser0, Parser1, Parser2, Parser3
>
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		return
			Parser0::parse(s, us) ||
			Parser1::parse(s, us) ||
			Parser2::parse(s, us) ||
			Parser3::parse(s, us)
		;
	}
};

// 5
template<
	class Parser0, class Parser1, class Parser2, class Parser3, class Parser4
>
struct or_<
	Parser0, Parser1, Parser2, Parser3, Parser4
>
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		return
			Parser0::parse(s, us) ||
			Parser1::parse(s, us) ||
			Parser2::parse(s, us) ||
			Parser3::parse(s, us) ||
			Parser4::parse(s, us)
		;
	}
};

// 6
template<
	class Parser0, class Parser1, class Parser2, class Parser3, class Parser4,
	class Parser5
>
struct or_<
	Parser0, Parser1, Parser2, Parser3, Parser4,
	Parser5
>
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		return
			Parser0::parse(s, us) ||
			Parser1::parse(s, us) ||
			Parser2::parse(s, us) ||
			Parser3::parse(s, us) ||
			Parser4::parse(s, us) ||
			Parser5::parse(s, us)
		;
	}
};

// 7
template<
	class Parser0, class Parser1, class Parser2, class Parser3, class Parser4,
	class Parser5 ,class Parser6
>
struct or_<
	Parser0, Parser1, Parser2, Parser3, Parser4,
	Parser5, Parser6
>
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		return
			Parser0::parse(s, us) ||
			Parser1::parse(s, us) ||
			Parser2::parse(s, us) ||
			Parser3::parse(s, us) ||
			Parser4::parse(s, us) ||
			Parser5::parse(s, us) ||
			Parser6::parse(s, us)
		;
	}
};

// 8
template<
	class Parser0, class Parser1, class Parser2, class Parser3, class Parser4,
	class Parser5 ,class Parser6, class Parser7
>
struct or_<
	Parser0, Parser1, Parser2, Parser3, Parser4,
	Parser5, Parser6, Parser7
>
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		return
			Parser0::parse(s, us) ||
			Parser1::parse(s, us) ||
			Parser2::parse(s, us) ||
			Parser3::parse(s, us) ||
			Parser4::parse(s, us) ||
			Parser5::parse(s, us) ||
			Parser6::parse(s, us) ||
			Parser7::parse(s, us)
		;
	}
};

// 9
template<
	class Parser0, class Parser1, class Parser2, class Parser3, class Parser4,
	class Parser5 ,class Parser6, class Parser7, class Parser8
>
struct or_<
	Parser0, Parser1, Parser2, Parser3, Parser4,
	Parser5, Parser6, Parser7, Parser8
>
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		return
			Parser0::parse(s, us) ||
			Parser1::parse(s, us) ||
			Parser2::parse(s, us) ||
			Parser3::parse(s, us) ||
			Parser4::parse(s, us) ||
			Parser5::parse(s, us) ||
			Parser6::parse(s, us) ||
			Parser7::parse(s, us) ||
			Parser8::parse(s, us)
		;
	}
};


// 10, primary
template<
	class Parser0, class Parser1, class Parser2, class Parser3, class Parser4,
	class Parser5 ,class Parser6, class Parser7, class Parser8, class Parser9
>
struct or_
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		return
			Parser0::parse(s, us) ||
			Parser1::parse(s, us) ||
			Parser2::parse(s, us) ||
			Parser3::parse(s, us) ||
			Parser4::parse(s, us) ||
			Parser5::parse(s, us) ||
			Parser6::parse(s, us) ||
			Parser7::parse(s, us) ||
			Parser8::parse(s, us) ||
			Parser9::parse(s, us)
		;
	}
};

} // namespace biscuit

