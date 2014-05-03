#pragma once

#include "../or.hpp"
#include "ucs4_t.hpp"
#include "ucs4.hpp"


namespace biscuit
{

	template<
		ucs4_t ch0 = 0 , ucs4_t ch1 = 0 , ucs4_t ch2 = 0 , ucs4_t ch3 = 0 , ucs4_t ch4 = 0 , ucs4_t ch5 = 0 , ucs4_t ch6 = 0 , ucs4_t ch7 = 0 , ucs4_t ch8 = 0 , ucs4_t ch9 = 0 , ucs4_t ch10 = 0 , ucs4_t ch11 = 0 , ucs4_t ch12 = 0 , ucs4_t ch13 = 0 , ucs4_t ch14 = 0 , ucs4_t ch15 = 0 , ucs4_t ch16 = 0 , ucs4_t ch17 = 0 , ucs4_t ch18 = 0 , ucs4_t ch19 = 0
	>
	struct ucs4_set :
		or_<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8> , ucs4<ch9> , ucs4<ch10> , ucs4<ch11> , ucs4<ch12> , ucs4<ch13> , ucs4<ch14> , ucs4<ch15> , ucs4<ch16> , ucs4<ch17> , ucs4<ch18> , ucs4<ch19>
		>
	{
	};

	template<

	>
	struct ucs4_set<

	> :
	or_<

	>
	{
	};

	template<
		ucs4_t ch0
	>
	struct ucs4_set<
		ch0
	> :
	or_<
		ucs4<ch0>
	>
	{
	};

	template<
		ucs4_t ch0 , ucs4_t ch1
	>
	struct ucs4_set<
		ch0 , ch1
	> :
	or_<
		ucs4<ch0> , ucs4<ch1>
	>
	{
	};

	template<
		ucs4_t ch0 , ucs4_t ch1 , ucs4_t ch2
	>
	struct ucs4_set<
		ch0 , ch1 , ch2
	> :
	or_<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2>
	>
	{
	};

	template<
		ucs4_t ch0 , ucs4_t ch1 , ucs4_t ch2 , ucs4_t ch3
	>
	struct ucs4_set<
		ch0 , ch1 , ch2 , ch3
	> :
	or_<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3>
	>
	{
	};

	template<
		ucs4_t ch0 , ucs4_t ch1 , ucs4_t ch2 , ucs4_t ch3 , ucs4_t ch4
	>
	struct ucs4_set<
		ch0 , ch1 , ch2 , ch3 , ch4
	> :
	or_<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4>
	>
	{
	};

	template<
		ucs4_t ch0 , ucs4_t ch1 , ucs4_t ch2 , ucs4_t ch3 , ucs4_t ch4 , ucs4_t ch5
	>
	struct ucs4_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5
	> :
	or_<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5>
	>
	{
	};

	template<
		ucs4_t ch0 , ucs4_t ch1 , ucs4_t ch2 , ucs4_t ch3 , ucs4_t ch4 , ucs4_t ch5 , ucs4_t ch6
	>
	struct ucs4_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6
	> :
	or_<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6>
	>
	{
	};

	template<
		ucs4_t ch0 , ucs4_t ch1 , ucs4_t ch2 , ucs4_t ch3 , ucs4_t ch4 , ucs4_t ch5 , ucs4_t ch6 , ucs4_t ch7
	>
	struct ucs4_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7
	> :
	or_<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7>
	>
	{
	};

	template<
		ucs4_t ch0 , ucs4_t ch1 , ucs4_t ch2 , ucs4_t ch3 , ucs4_t ch4 , ucs4_t ch5 , ucs4_t ch6 , ucs4_t ch7 , ucs4_t ch8
	>
	struct ucs4_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8
	> :
	or_<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8>
	>
	{
	};

	template<
		ucs4_t ch0 , ucs4_t ch1 , ucs4_t ch2 , ucs4_t ch3 , ucs4_t ch4 , ucs4_t ch5 , ucs4_t ch6 , ucs4_t ch7 , ucs4_t ch8 , ucs4_t ch9
	>
	struct ucs4_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9
	> :
	or_<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8> , ucs4<ch9>
	>
	{
	};

	template<
		ucs4_t ch0 , ucs4_t ch1 , ucs4_t ch2 , ucs4_t ch3 , ucs4_t ch4 , ucs4_t ch5 , ucs4_t ch6 , ucs4_t ch7 , ucs4_t ch8 , ucs4_t ch9 , ucs4_t ch10
	>
	struct ucs4_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10
	> :
	or_<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8> , ucs4<ch9> , ucs4<ch10>
	>
	{
	};

	template<
		ucs4_t ch0 , ucs4_t ch1 , ucs4_t ch2 , ucs4_t ch3 , ucs4_t ch4 , ucs4_t ch5 , ucs4_t ch6 , ucs4_t ch7 , ucs4_t ch8 , ucs4_t ch9 , ucs4_t ch10 , ucs4_t ch11
	>
	struct ucs4_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11
	> :
	or_<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8> , ucs4<ch9> , ucs4<ch10> , ucs4<ch11>
	>
	{
	};

	template<
		ucs4_t ch0 , ucs4_t ch1 , ucs4_t ch2 , ucs4_t ch3 , ucs4_t ch4 , ucs4_t ch5 , ucs4_t ch6 , ucs4_t ch7 , ucs4_t ch8 , ucs4_t ch9 , ucs4_t ch10 , ucs4_t ch11 , ucs4_t ch12
	>
	struct ucs4_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12
	> :
	or_<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8> , ucs4<ch9> , ucs4<ch10> , ucs4<ch11> , ucs4<ch12>
	>
	{
	};

	template<
		ucs4_t ch0 , ucs4_t ch1 , ucs4_t ch2 , ucs4_t ch3 , ucs4_t ch4 , ucs4_t ch5 , ucs4_t ch6 , ucs4_t ch7 , ucs4_t ch8 , ucs4_t ch9 , ucs4_t ch10 , ucs4_t ch11 , ucs4_t ch12 , ucs4_t ch13
	>
	struct ucs4_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13
	> :
	or_<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8> , ucs4<ch9> , ucs4<ch10> , ucs4<ch11> , ucs4<ch12> , ucs4<ch13>
	>
	{
	};

	template<
		ucs4_t ch0 , ucs4_t ch1 , ucs4_t ch2 , ucs4_t ch3 , ucs4_t ch4 , ucs4_t ch5 , ucs4_t ch6 , ucs4_t ch7 , ucs4_t ch8 , ucs4_t ch9 , ucs4_t ch10 , ucs4_t ch11 , ucs4_t ch12 , ucs4_t ch13 , ucs4_t ch14
	>
	struct ucs4_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14
	> :
	or_<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8> , ucs4<ch9> , ucs4<ch10> , ucs4<ch11> , ucs4<ch12> , ucs4<ch13> , ucs4<ch14>
	>
	{
	};

	template<
		ucs4_t ch0 , ucs4_t ch1 , ucs4_t ch2 , ucs4_t ch3 , ucs4_t ch4 , ucs4_t ch5 , ucs4_t ch6 , ucs4_t ch7 , ucs4_t ch8 , ucs4_t ch9 , ucs4_t ch10 , ucs4_t ch11 , ucs4_t ch12 , ucs4_t ch13 , ucs4_t ch14 , ucs4_t ch15
	>
	struct ucs4_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14 , ch15
	> :
	or_<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8> , ucs4<ch9> , ucs4<ch10> , ucs4<ch11> , ucs4<ch12> , ucs4<ch13> , ucs4<ch14> , ucs4<ch15>
	>
	{
	};

	template<
		ucs4_t ch0 , ucs4_t ch1 , ucs4_t ch2 , ucs4_t ch3 , ucs4_t ch4 , ucs4_t ch5 , ucs4_t ch6 , ucs4_t ch7 , ucs4_t ch8 , ucs4_t ch9 , ucs4_t ch10 , ucs4_t ch11 , ucs4_t ch12 , ucs4_t ch13 , ucs4_t ch14 , ucs4_t ch15 , ucs4_t ch16
	>
	struct ucs4_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14 , ch15 , ch16
	> :
	or_<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8> , ucs4<ch9> , ucs4<ch10> , ucs4<ch11> , ucs4<ch12> , ucs4<ch13> , ucs4<ch14> , ucs4<ch15> , ucs4<ch16>
	>
	{
	};

	template<
		ucs4_t ch0 , ucs4_t ch1 , ucs4_t ch2 , ucs4_t ch3 , ucs4_t ch4 , ucs4_t ch5 , ucs4_t ch6 , ucs4_t ch7 , ucs4_t ch8 , ucs4_t ch9 , ucs4_t ch10 , ucs4_t ch11 , ucs4_t ch12 , ucs4_t ch13 , ucs4_t ch14 , ucs4_t ch15 , ucs4_t ch16 , ucs4_t ch17
	>
	struct ucs4_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14 , ch15 , ch16 , ch17
	> :
	or_<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8> , ucs4<ch9> , ucs4<ch10> , ucs4<ch11> , ucs4<ch12> , ucs4<ch13> , ucs4<ch14> , ucs4<ch15> , ucs4<ch16> , ucs4<ch17>
	>
	{
	};

	template<
		ucs4_t ch0 , ucs4_t ch1 , ucs4_t ch2 , ucs4_t ch3 , ucs4_t ch4 , ucs4_t ch5 , ucs4_t ch6 , ucs4_t ch7 , ucs4_t ch8 , ucs4_t ch9 , ucs4_t ch10 , ucs4_t ch11 , ucs4_t ch12 , ucs4_t ch13 , ucs4_t ch14 , ucs4_t ch15 , ucs4_t ch16 , ucs4_t ch17 , ucs4_t ch18
	>
	struct ucs4_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14 , ch15 , ch16 , ch17 , ch18
	> :
	or_<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8> , ucs4<ch9> , ucs4<ch10> , ucs4<ch11> , ucs4<ch12> , ucs4<ch13> , ucs4<ch14> , ucs4<ch15> , ucs4<ch16> , ucs4<ch17> , ucs4<ch18>
	>
	{
	};

} // namespace biscuit
