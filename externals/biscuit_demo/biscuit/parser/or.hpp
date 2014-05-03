
#pragma once 
#include "detail/na.hpp" 

namespace biscuit
{

	template<
		class P0 = na , class P1 = na , class P2 = na , class P3 = na , class P4 = na , class P5 = na , class P6 = na , class P7 = na , class P8 = na , class P9 = na , class P10 = na , class P11 = na , class P12 = na , class P13 = na , class P14 = na , class P15 = na , class P16 = na , class P17 = na , class P18 = na , class P19 = na
	>
	struct or_
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			return
				P0::parse(s, us) || P1::parse(s, us) || P2::parse(s, us) || P3::parse(s, us) || P4::parse(s, us) || P5::parse(s, us) || P6::parse(s, us) || P7::parse(s, us) || P8::parse(s, us) || P9::parse(s, us) || P10::parse(s, us) || P11::parse(s, us) || P12::parse(s, us) || P13::parse(s, us) || P14::parse(s, us) || P15::parse(s, us) || P16::parse(s, us) || P17::parse(s, us) || P18::parse(s, us) || P19::parse(s, us)
				;
		}
	};

	template<
	>
	struct or_<
	>
	{
		template< class State, class UserState >
			static bool parse(State& , UserState& )
		{
			return false;
		}
	};

	template<
		class P0
	>
	struct or_<
		P0
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			return
				P0::parse(s, us)
				;
		}
	};

	template<
		class P0 , class P1
	>
	struct or_<
		P0 , P1
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			return
				P0::parse(s, us) || P1::parse(s, us)
				;
		}
	};

	template<
		class P0 , class P1 , class P2
	>
	struct or_<
		P0 , P1 , P2
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			return
				P0::parse(s, us) || P1::parse(s, us) || P2::parse(s, us)
				;
		}
	};

	template<
		class P0 , class P1 , class P2 , class P3
	>
	struct or_<
		P0 , P1 , P2 , P3
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			return
				P0::parse(s, us) || P1::parse(s, us) || P2::parse(s, us) || P3::parse(s, us)
				;
		}
	};

	template<
		class P0 , class P1 , class P2 , class P3 , class P4
	>
	struct or_<
		P0 , P1 , P2 , P3 , P4
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			return
				P0::parse(s, us) || P1::parse(s, us) || P2::parse(s, us) || P3::parse(s, us) || P4::parse(s, us)
				;
		}
	};

	template<
		class P0 , class P1 , class P2 , class P3 , class P4 , class P5
	>
	struct or_<
		P0 , P1 , P2 , P3 , P4 , P5
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			return
				P0::parse(s, us) || P1::parse(s, us) || P2::parse(s, us) || P3::parse(s, us) || P4::parse(s, us) || P5::parse(s, us)
				;
		}
	};

	template<
		class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6
	>
	struct or_<
		P0 , P1 , P2 , P3 , P4 , P5 , P6
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			return
				P0::parse(s, us) || P1::parse(s, us) || P2::parse(s, us) || P3::parse(s, us) || P4::parse(s, us) || P5::parse(s, us) || P6::parse(s, us)
				;
		}
	};

	template<
		class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7
	>
	struct or_<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			return
				P0::parse(s, us) || P1::parse(s, us) || P2::parse(s, us) || P3::parse(s, us) || P4::parse(s, us) || P5::parse(s, us) || P6::parse(s, us) || P7::parse(s, us)
				;
		}
	};

	template<
		class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8
	>
	struct or_<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			return
				P0::parse(s, us) || P1::parse(s, us) || P2::parse(s, us) || P3::parse(s, us) || P4::parse(s, us) || P5::parse(s, us) || P6::parse(s, us) || P7::parse(s, us) || P8::parse(s, us)
				;
		}
	};

	template<
		class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9
	>
	struct or_<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			return
				P0::parse(s, us) || P1::parse(s, us) || P2::parse(s, us) || P3::parse(s, us) || P4::parse(s, us) || P5::parse(s, us) || P6::parse(s, us) || P7::parse(s, us) || P8::parse(s, us) || P9::parse(s, us)
				;
		}
	};

	template<
		class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10
	>
	struct or_<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			return
				P0::parse(s, us) || P1::parse(s, us) || P2::parse(s, us) || P3::parse(s, us) || P4::parse(s, us) || P5::parse(s, us) || P6::parse(s, us) || P7::parse(s, us) || P8::parse(s, us) || P9::parse(s, us) || P10::parse(s, us)
				;
		}
	};

	template<
		class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11
	>
	struct or_<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			return
				P0::parse(s, us) || P1::parse(s, us) || P2::parse(s, us) || P3::parse(s, us) || P4::parse(s, us) || P5::parse(s, us) || P6::parse(s, us) || P7::parse(s, us) || P8::parse(s, us) || P9::parse(s, us) || P10::parse(s, us) || P11::parse(s, us)
				;
		}
	};

	template<
		class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12
	>
	struct or_<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			return
				P0::parse(s, us) || P1::parse(s, us) || P2::parse(s, us) || P3::parse(s, us) || P4::parse(s, us) || P5::parse(s, us) || P6::parse(s, us) || P7::parse(s, us) || P8::parse(s, us) || P9::parse(s, us) || P10::parse(s, us) || P11::parse(s, us) || P12::parse(s, us)
				;
		}
	};

	template<
		class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13
	>
	struct or_<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			return
				P0::parse(s, us) || P1::parse(s, us) || P2::parse(s, us) || P3::parse(s, us) || P4::parse(s, us) || P5::parse(s, us) || P6::parse(s, us) || P7::parse(s, us) || P8::parse(s, us) || P9::parse(s, us) || P10::parse(s, us) || P11::parse(s, us) || P12::parse(s, us) || P13::parse(s, us)
				;
		}
	};

	template<
		class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14
	>
	struct or_<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			return
				P0::parse(s, us) || P1::parse(s, us) || P2::parse(s, us) || P3::parse(s, us) || P4::parse(s, us) || P5::parse(s, us) || P6::parse(s, us) || P7::parse(s, us) || P8::parse(s, us) || P9::parse(s, us) || P10::parse(s, us) || P11::parse(s, us) || P12::parse(s, us) || P13::parse(s, us) || P14::parse(s, us)
				;
		}
	};

	template<
		class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15
	>
	struct or_<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			return
				P0::parse(s, us) || P1::parse(s, us) || P2::parse(s, us) || P3::parse(s, us) || P4::parse(s, us) || P5::parse(s, us) || P6::parse(s, us) || P7::parse(s, us) || P8::parse(s, us) || P9::parse(s, us) || P10::parse(s, us) || P11::parse(s, us) || P12::parse(s, us) || P13::parse(s, us) || P14::parse(s, us) || P15::parse(s, us)
				;
		}
	};

	template<
		class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16
	>
	struct or_<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			return
				P0::parse(s, us) || P1::parse(s, us) || P2::parse(s, us) || P3::parse(s, us) || P4::parse(s, us) || P5::parse(s, us) || P6::parse(s, us) || P7::parse(s, us) || P8::parse(s, us) || P9::parse(s, us) || P10::parse(s, us) || P11::parse(s, us) || P12::parse(s, us) || P13::parse(s, us) || P14::parse(s, us) || P15::parse(s, us) || P16::parse(s, us)
				;
		}
	};

	template<
		class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17
	>
	struct or_<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			return
				P0::parse(s, us) || P1::parse(s, us) || P2::parse(s, us) || P3::parse(s, us) || P4::parse(s, us) || P5::parse(s, us) || P6::parse(s, us) || P7::parse(s, us) || P8::parse(s, us) || P9::parse(s, us) || P10::parse(s, us) || P11::parse(s, us) || P12::parse(s, us) || P13::parse(s, us) || P14::parse(s, us) || P15::parse(s, us) || P16::parse(s, us) || P17::parse(s, us)
				;
		}
	};

	template<
		class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18
	>
	struct or_<
		P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18
	>
	{
		template< class State, class UserState >
			static bool parse(State& s, UserState& us)
		{
			return
				P0::parse(s, us) || P1::parse(s, us) || P2::parse(s, us) || P3::parse(s, us) || P4::parse(s, us) || P5::parse(s, us) || P6::parse(s, us) || P7::parse(s, us) || P8::parse(s, us) || P9::parse(s, us) || P10::parse(s, us) || P11::parse(s, us) || P12::parse(s, us) || P13::parse(s, us) || P14::parse(s, us) || P15::parse(s, us) || P16::parse(s, us) || P17::parse(s, us) || P18::parse(s, us)
				;
		}
	};

} 