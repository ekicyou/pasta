#pragma once

#include "../state/state_iterator.hpp"
#include "detail/na.hpp"

#pragma warning( push )
#pragma warning( disable : 4127 )

namespace biscuit
{

	template< class P0 = na , class P1 = na , class P2 = na , class P3 = na , class P4 = na , class P5 = na , class P6 = na , class P7 = na , class P8 = na , class P9 = na , class P10 = na , class P11 = na , class P12 = na , class P13 = na , class P14 = na , class P15 = na , class P16 = na , class P17 = na , class P18 = na , class P19 = na , class P20 = na , class P21 = na , class P22 = na , class P23 = na , class P24 = na , class P25 = na , class P26 = na , class P27 = na , class P28 = na , class P29 = na , class P30 = na , class P31 = na , class P32 = na , class P33 = na , class P34 = na , class P35 = na , class P36 = na , class P37 = na , class P38 = na , class P39 = na , class P40 = na , class P41 = na , class P42 = na , class P43 = na , class P44 = na , class P45 = na , class P46 = na , class P47 = na , class P48 = na , class P49 = na >
	struct seq
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) && P29::parse(s, us) && P30::parse(s, us) && P31::parse(s, us) && P32::parse(s, us) && P33::parse(s, us) && P34::parse(s, us) && P35::parse(s, us) && P36::parse(s, us) && P37::parse(s, us) && P38::parse(s, us) && P39::parse(s, us) && P40::parse(s, us) && P41::parse(s, us) && P42::parse(s, us) && P43::parse(s, us) && P44::parse(s, us) && P45::parse(s, us) && P46::parse(s, us) && P47::parse(s, us) && P48::parse(s, us) && P49::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< >
	struct seq<

		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (

				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 >
	struct seq<
		P0 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 >
	struct seq<
		P0 , P1 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 >
	struct seq<
		P0 , P1 , P2 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 >
	struct seq<
		P0 , P1 , P2 , P3 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 , class P27 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 , P27 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 , class P27 , class P28 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 , P27 , P28 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 , class P27 , class P28 , class P29 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 , P27 , P28 , P29 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) && P29::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 , class P27 , class P28 , class P29 , class P30 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 , P27 , P28 , P29 , P30 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) && P29::parse(s, us) && P30::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 , class P27 , class P28 , class P29 , class P30 , class P31 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 , P27 , P28 , P29 , P30 , P31 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) && P29::parse(s, us) && P30::parse(s, us) && P31::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 , class P27 , class P28 , class P29 , class P30 , class P31 , class P32 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 , P27 , P28 , P29 , P30 , P31 , P32 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) && P29::parse(s, us) && P30::parse(s, us) && P31::parse(s, us) && P32::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 , class P27 , class P28 , class P29 , class P30 , class P31 , class P32 , class P33 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 , P27 , P28 , P29 , P30 , P31 , P32 , P33 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) && P29::parse(s, us) && P30::parse(s, us) && P31::parse(s, us) && P32::parse(s, us) && P33::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 , class P27 , class P28 , class P29 , class P30 , class P31 , class P32 , class P33 , class P34 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 , P27 , P28 , P29 , P30 , P31 , P32 , P33 , P34 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) && P29::parse(s, us) && P30::parse(s, us) && P31::parse(s, us) && P32::parse(s, us) && P33::parse(s, us) && P34::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 , class P27 , class P28 , class P29 , class P30 , class P31 , class P32 , class P33 , class P34 , class P35 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 , P27 , P28 , P29 , P30 , P31 , P32 , P33 , P34 , P35 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) && P29::parse(s, us) && P30::parse(s, us) && P31::parse(s, us) && P32::parse(s, us) && P33::parse(s, us) && P34::parse(s, us) && P35::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 , class P27 , class P28 , class P29 , class P30 , class P31 , class P32 , class P33 , class P34 , class P35 , class P36 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 , P27 , P28 , P29 , P30 , P31 , P32 , P33 , P34 , P35 , P36 ,
		na , na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) && P29::parse(s, us) && P30::parse(s, us) && P31::parse(s, us) && P32::parse(s, us) && P33::parse(s, us) && P34::parse(s, us) && P35::parse(s, us) && P36::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 , class P27 , class P28 , class P29 , class P30 , class P31 , class P32 , class P33 , class P34 , class P35 , class P36 , class P37 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 , P27 , P28 , P29 , P30 , P31 , P32 , P33 , P34 , P35 , P36 , P37 ,
		na , na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) && P29::parse(s, us) && P30::parse(s, us) && P31::parse(s, us) && P32::parse(s, us) && P33::parse(s, us) && P34::parse(s, us) && P35::parse(s, us) && P36::parse(s, us) && P37::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 , class P27 , class P28 , class P29 , class P30 , class P31 , class P32 , class P33 , class P34 , class P35 , class P36 , class P37 , class P38 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 , P27 , P28 , P29 , P30 , P31 , P32 , P33 , P34 , P35 , P36 , P37 , P38 ,
		na , na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) && P29::parse(s, us) && P30::parse(s, us) && P31::parse(s, us) && P32::parse(s, us) && P33::parse(s, us) && P34::parse(s, us) && P35::parse(s, us) && P36::parse(s, us) && P37::parse(s, us) && P38::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 , class P27 , class P28 , class P29 , class P30 , class P31 , class P32 , class P33 , class P34 , class P35 , class P36 , class P37 , class P38 , class P39 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 , P27 , P28 , P29 , P30 , P31 , P32 , P33 , P34 , P35 , P36 , P37 , P38 , P39 ,
		na , na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) && P29::parse(s, us) && P30::parse(s, us) && P31::parse(s, us) && P32::parse(s, us) && P33::parse(s, us) && P34::parse(s, us) && P35::parse(s, us) && P36::parse(s, us) && P37::parse(s, us) && P38::parse(s, us) && P39::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 , class P27 , class P28 , class P29 , class P30 , class P31 , class P32 , class P33 , class P34 , class P35 , class P36 , class P37 , class P38 , class P39 , class P40 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 , P27 , P28 , P29 , P30 , P31 , P32 , P33 , P34 , P35 , P36 , P37 , P38 , P39 , P40 ,
		na , na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) && P29::parse(s, us) && P30::parse(s, us) && P31::parse(s, us) && P32::parse(s, us) && P33::parse(s, us) && P34::parse(s, us) && P35::parse(s, us) && P36::parse(s, us) && P37::parse(s, us) && P38::parse(s, us) && P39::parse(s, us) && P40::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 , class P27 , class P28 , class P29 , class P30 , class P31 , class P32 , class P33 , class P34 , class P35 , class P36 , class P37 , class P38 , class P39 , class P40 , class P41 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 , P27 , P28 , P29 , P30 , P31 , P32 , P33 , P34 , P35 , P36 , P37 , P38 , P39 , P40 , P41 ,
		na , na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) && P29::parse(s, us) && P30::parse(s, us) && P31::parse(s, us) && P32::parse(s, us) && P33::parse(s, us) && P34::parse(s, us) && P35::parse(s, us) && P36::parse(s, us) && P37::parse(s, us) && P38::parse(s, us) && P39::parse(s, us) && P40::parse(s, us) && P41::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 , class P27 , class P28 , class P29 , class P30 , class P31 , class P32 , class P33 , class P34 , class P35 , class P36 , class P37 , class P38 , class P39 , class P40 , class P41 , class P42 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 , P27 , P28 , P29 , P30 , P31 , P32 , P33 , P34 , P35 , P36 , P37 , P38 , P39 , P40 , P41 , P42 ,
		na , na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) && P29::parse(s, us) && P30::parse(s, us) && P31::parse(s, us) && P32::parse(s, us) && P33::parse(s, us) && P34::parse(s, us) && P35::parse(s, us) && P36::parse(s, us) && P37::parse(s, us) && P38::parse(s, us) && P39::parse(s, us) && P40::parse(s, us) && P41::parse(s, us) && P42::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 , class P27 , class P28 , class P29 , class P30 , class P31 , class P32 , class P33 , class P34 , class P35 , class P36 , class P37 , class P38 , class P39 , class P40 , class P41 , class P42 , class P43 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 , P27 , P28 , P29 , P30 , P31 , P32 , P33 , P34 , P35 , P36 , P37 , P38 , P39 , P40 , P41 , P42 , P43 ,
		na , na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) && P29::parse(s, us) && P30::parse(s, us) && P31::parse(s, us) && P32::parse(s, us) && P33::parse(s, us) && P34::parse(s, us) && P35::parse(s, us) && P36::parse(s, us) && P37::parse(s, us) && P38::parse(s, us) && P39::parse(s, us) && P40::parse(s, us) && P41::parse(s, us) && P42::parse(s, us) && P43::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 , class P27 , class P28 , class P29 , class P30 , class P31 , class P32 , class P33 , class P34 , class P35 , class P36 , class P37 , class P38 , class P39 , class P40 , class P41 , class P42 , class P43 , class P44 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 , P27 , P28 , P29 , P30 , P31 , P32 , P33 , P34 , P35 , P36 , P37 , P38 , P39 , P40 , P41 , P42 , P43 , P44 ,
		na , na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) && P29::parse(s, us) && P30::parse(s, us) && P31::parse(s, us) && P32::parse(s, us) && P33::parse(s, us) && P34::parse(s, us) && P35::parse(s, us) && P36::parse(s, us) && P37::parse(s, us) && P38::parse(s, us) && P39::parse(s, us) && P40::parse(s, us) && P41::parse(s, us) && P42::parse(s, us) && P43::parse(s, us) && P44::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 , class P27 , class P28 , class P29 , class P30 , class P31 , class P32 , class P33 , class P34 , class P35 , class P36 , class P37 , class P38 , class P39 , class P40 , class P41 , class P42 , class P43 , class P44 , class P45 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 , P27 , P28 , P29 , P30 , P31 , P32 , P33 , P34 , P35 , P36 , P37 , P38 , P39 , P40 , P41 , P42 , P43 , P44 , P45 ,
		na , na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) && P29::parse(s, us) && P30::parse(s, us) && P31::parse(s, us) && P32::parse(s, us) && P33::parse(s, us) && P34::parse(s, us) && P35::parse(s, us) && P36::parse(s, us) && P37::parse(s, us) && P38::parse(s, us) && P39::parse(s, us) && P40::parse(s, us) && P41::parse(s, us) && P42::parse(s, us) && P43::parse(s, us) && P44::parse(s, us) && P45::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 , class P27 , class P28 , class P29 , class P30 , class P31 , class P32 , class P33 , class P34 , class P35 , class P36 , class P37 , class P38 , class P39 , class P40 , class P41 , class P42 , class P43 , class P44 , class P45 , class P46 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 , P27 , P28 , P29 , P30 , P31 , P32 , P33 , P34 , P35 , P36 , P37 , P38 , P39 , P40 , P41 , P42 , P43 , P44 , P45 , P46 ,
		na , na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) && P29::parse(s, us) && P30::parse(s, us) && P31::parse(s, us) && P32::parse(s, us) && P33::parse(s, us) && P34::parse(s, us) && P35::parse(s, us) && P36::parse(s, us) && P37::parse(s, us) && P38::parse(s, us) && P39::parse(s, us) && P40::parse(s, us) && P41::parse(s, us) && P42::parse(s, us) && P43::parse(s, us) && P44::parse(s, us) && P45::parse(s, us) && P46::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 , class P27 , class P28 , class P29 , class P30 , class P31 , class P32 , class P33 , class P34 , class P35 , class P36 , class P37 , class P38 , class P39 , class P40 , class P41 , class P42 , class P43 , class P44 , class P45 , class P46 , class P47 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 , P27 , P28 , P29 , P30 , P31 , P32 , P33 , P34 , P35 , P36 , P37 , P38 , P39 , P40 , P41 , P42 , P43 , P44 , P45 , P46 , P47 ,
		na , na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) && P29::parse(s, us) && P30::parse(s, us) && P31::parse(s, us) && P32::parse(s, us) && P33::parse(s, us) && P34::parse(s, us) && P35::parse(s, us) && P36::parse(s, us) && P37::parse(s, us) && P38::parse(s, us) && P39::parse(s, us) && P40::parse(s, us) && P41::parse(s, us) && P42::parse(s, us) && P43::parse(s, us) && P44::parse(s, us) && P45::parse(s, us) && P46::parse(s, us) && P47::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

	template< class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18 , class P19 , class P20 , class P21 , class P22 , class P23 , class P24 , class P25 , class P26 , class P27 , class P28 , class P29 , class P30 , class P31 , class P32 , class P33 , class P34 , class P35 , class P36 , class P37 , class P38 , class P39 , class P40 , class P41 , class P42 , class P43 , class P44 , class P45 , class P46 , class P47 , class P48 >
	struct seq<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18 , P19 , P20 , P21 , P22 , P23 , P24 , P25 , P26 , P27 , P28 , P29 , P30 , P31 , P32 , P33 , P34 , P35 , P36 , P37 , P38 , P39 , P40 , P41 , P42 , P43 , P44 , P45 , P46 , P47 , P48 ,
		na
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			typename state_iterator<State> ::type cur0 = s.cur;

			if (
				P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) && P29::parse(s, us) && P30::parse(s, us) && P31::parse(s, us) && P32::parse(s, us) && P33::parse(s, us) && P34::parse(s, us) && P35::parse(s, us) && P36::parse(s, us) && P37::parse(s, us) && P38::parse(s, us) && P39::parse(s, us) && P40::parse(s, us) && P41::parse(s, us) && P42::parse(s, us) && P43::parse(s, us) && P44::parse(s, us) && P45::parse(s, us) && P46::parse(s, us) && P47::parse(s, us) && P48::parse(s, us) &&
				true
				)
			{
				return true;
			}
			else
			{
				s.cur = cur0;
				return false;
			}
		}
	};

} // namespace biscuit

#pragma warning( pop )
