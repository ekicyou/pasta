#pragma once

#include "../eps.hpp"
#include <algorithm>

namespace biscuit
{

	namespace detail
	{
		template< class Parser >
		struct str_base
		{
			template< class State, class UserState >
			static bool parse(State& s, UserState&)
			{
				typedef typename state_iterator<State>::type iter_t;
				iter_t const cur0 = s.cur;

				char const *it = Parser::characters();
				char const *last = it + Parser::n;

				for (;;)
				{
					if (eos(s))
						break;

					if (it == last)
						return true;
					
					if (*it != *s.cur)
						break;

					++it;
					++s.cur;
				}

				s.cur = cur0;
				return false;
			}
			
			typedef str_base type;
		};
	} // namespace detail

template<
	char ch0 = 0, char ch1 = 0, char ch2 = 0, char ch3 = 0, char ch4 = 0,
	char ch5 = 0, char ch6 = 0, char ch7 = 0, char ch8 = 0, char ch9 = 0
>
struct str;

// 0
template<>
struct str<> :
	eps
{ };

// 1
template<
	char ch0
>
struct str<ch0> :
	detail::str_base<str>::type
{
	static char const* characters()
	{
		static char const chars[] = { ch0 };
		return chars;
	}

	static const int n = 1;
};

// 2
template<
	char ch0, char ch1
>
struct str<ch0, ch1> :
	detail::str_base<str>::type
{
	static char const* characters()
	{
		static char const chars[] = { ch0, ch1 };
		return chars;
	}

	static const int n = 2;
};

// 3
template<
	char ch0, char ch1, char ch2
>
struct str<ch0, ch1, ch2> :
	detail::str_base<str>::type
{
	static char const* characters()
	{
		static char const chars[] = { ch0, ch1, ch2 };
		return chars;
	}

	static const int n = 3;
};

// 4
template<
	char ch0, char ch1, char ch2, char ch3
>
struct str<ch0, ch1, ch2, ch3> :
	detail::str_base<str>::type
{
	static char const* characters()
	{
		static char const chars[] = { ch0, ch1, ch2, ch3 };
		return chars;
	}

	static const int n = 4;
};

// 5
template<
	char ch0, char ch1, char ch2, char ch3, char ch4
>
struct str<ch0, ch1, ch2, ch3, ch4> :
	detail::str_base<str>::type
{
	static char const* characters()
	{
		static char const chars[] = { ch0, ch1, ch2, ch3, ch4 };
		return chars;
	}

	static const int n = 5;
};

// 6
template<
	char ch0, char ch1, char ch2, char ch3, char ch4,
	char ch5
>
struct str<ch0, ch1, ch2, ch3, ch4, ch5> :
	detail::str_base<str>::type
{
	static char const* characters()
	{
		static char const chars[] = { ch0, ch1, ch2, ch3, ch4, ch5 };
		return chars;
	}

	static const int n = 6;
};

// 7
template<
	char ch0, char ch1, char ch2, char ch3, char ch4,
	char ch5, char ch6
>
struct str<ch0, ch1, ch2, ch3, ch4, ch5, ch6> :
	detail::str_base<str>::type
{
	static char const* characters()
	{
		static char const chars[] = { ch0, ch1, ch2, ch3, ch4, ch5, ch6 };
		return chars;
	}

	static const int n = 7;
};

// 8
template<
	char ch0, char ch1, char ch2, char ch3, char ch4,
	char ch5, char ch6, char ch7
>
struct str<ch0, ch1, ch2, ch3, ch4, ch5, ch6, ch7> :
	detail::str_base<str>::type
{
	static char const* characters()
	{
		static char const chars[] = { ch0, ch1, ch2, ch3, ch4, ch5, ch6, ch7 };
		return chars;
	}

	static const int n = 8;
};


// 9
template<
	char ch0, char ch1, char ch2, char ch3, char ch4,
	char ch5, char ch6, char ch7, char ch8
>
struct str<ch0, ch1, ch2, ch3, ch4, ch5, ch6, ch7, ch8> :
	detail::str_base<str>::type
{
	static char const* characters()
	{
		static char const chars[] = { ch0, ch1, ch2, ch3, ch4, ch5, ch6, ch7, ch8 };
		return chars;
	}

	static const int n = 9;
};

// 10
template<
	char ch0, char ch1, char ch2, char ch3, char ch4,
	char ch5, char ch6, char ch7, char ch8, char ch9
>
struct str<ch0, ch1, ch2, ch3, ch4, ch5, ch6, ch7, ch8, ch9> :
	detail::str_base<str>::type
{
	static char const* characters()
	{
		static char const chars[] = { ch0, ch1, ch2, ch3, ch4, ch5, ch6, ch7, ch8, ch9 };
		return chars;
	}

	static const int n = 10;
};

} // namespace biscuit
