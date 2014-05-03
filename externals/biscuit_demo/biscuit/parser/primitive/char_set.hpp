#pragma once

#include "../or.hpp"
#include "char.hpp"


namespace biscuit
{

	template<
		char ch0 = 0 , char ch1 = 0 , char ch2 = 0 , char ch3 = 0 , char ch4 = 0 , char ch5 = 0 , char ch6 = 0 , char ch7 = 0 , char ch8 = 0 , char ch9 = 0 , char ch10 = 0 , char ch11 = 0 , char ch12 = 0 , char ch13 = 0 , char ch14 = 0 , char ch15 = 0 , char ch16 = 0 , char ch17 = 0 , char ch18 = 0 , char ch19 = 0
	>
	struct char_set :
		or_<
		char_<ch0> , char_<ch1> , char_<ch2> , char_<ch3> , char_<ch4> , char_<ch5> , char_<ch6> , char_<ch7> , char_<ch8> , char_<ch9> , char_<ch10> , char_<ch11> , char_<ch12> , char_<ch13> , char_<ch14> , char_<ch15> , char_<ch16> , char_<ch17> , char_<ch18> , char_<ch19>
		>
	{
	};

	template<

	>
	struct char_set<

	> :
	or_<

	>
	{
	};

	template<
		char ch0
	>
	struct char_set<
		ch0
	> :
	or_<
		char_<ch0>
	>
	{
	};

	template<
		char ch0 , char ch1
	>
	struct char_set<
		ch0 , ch1
	> :
	or_<
		char_<ch0> , char_<ch1>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2
	>
	struct char_set<
		ch0 , ch1 , ch2
	> :
	or_<
		char_<ch0> , char_<ch1> , char_<ch2>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3
	>
	struct char_set<
		ch0 , ch1 , ch2 , ch3
	> :
	or_<
		char_<ch0> , char_<ch1> , char_<ch2> , char_<ch3>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4
	>
	struct char_set<
		ch0 , ch1 , ch2 , ch3 , ch4
	> :
	or_<
		char_<ch0> , char_<ch1> , char_<ch2> , char_<ch3> , char_<ch4>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5
	>
	struct char_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5
	> :
	or_<
		char_<ch0> , char_<ch1> , char_<ch2> , char_<ch3> , char_<ch4> , char_<ch5>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6
	>
	struct char_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6
	> :
	or_<
		char_<ch0> , char_<ch1> , char_<ch2> , char_<ch3> , char_<ch4> , char_<ch5> , char_<ch6>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7
	>
	struct char_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7
	> :
	or_<
		char_<ch0> , char_<ch1> , char_<ch2> , char_<ch3> , char_<ch4> , char_<ch5> , char_<ch6> , char_<ch7>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7 , char ch8
	>
	struct char_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8
	> :
	or_<
		char_<ch0> , char_<ch1> , char_<ch2> , char_<ch3> , char_<ch4> , char_<ch5> , char_<ch6> , char_<ch7> , char_<ch8>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7 , char ch8 , char ch9
	>
	struct char_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9
	> :
	or_<
		char_<ch0> , char_<ch1> , char_<ch2> , char_<ch3> , char_<ch4> , char_<ch5> , char_<ch6> , char_<ch7> , char_<ch8> , char_<ch9>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7 , char ch8 , char ch9 , char ch10
	>
	struct char_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10
	> :
	or_<
		char_<ch0> , char_<ch1> , char_<ch2> , char_<ch3> , char_<ch4> , char_<ch5> , char_<ch6> , char_<ch7> , char_<ch8> , char_<ch9> , char_<ch10>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7 , char ch8 , char ch9 , char ch10 , char ch11
	>
	struct char_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11
	> :
	or_<
		char_<ch0> , char_<ch1> , char_<ch2> , char_<ch3> , char_<ch4> , char_<ch5> , char_<ch6> , char_<ch7> , char_<ch8> , char_<ch9> , char_<ch10> , char_<ch11>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7 , char ch8 , char ch9 , char ch10 , char ch11 , char ch12
	>
	struct char_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12
	> :
	or_<
		char_<ch0> , char_<ch1> , char_<ch2> , char_<ch3> , char_<ch4> , char_<ch5> , char_<ch6> , char_<ch7> , char_<ch8> , char_<ch9> , char_<ch10> , char_<ch11> , char_<ch12>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7 , char ch8 , char ch9 , char ch10 , char ch11 , char ch12 , char ch13
	>
	struct char_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13
	> :
	or_<
		char_<ch0> , char_<ch1> , char_<ch2> , char_<ch3> , char_<ch4> , char_<ch5> , char_<ch6> , char_<ch7> , char_<ch8> , char_<ch9> , char_<ch10> , char_<ch11> , char_<ch12> , char_<ch13>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7 , char ch8 , char ch9 , char ch10 , char ch11 , char ch12 , char ch13 , char ch14
	>
	struct char_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14
	> :
	or_<
		char_<ch0> , char_<ch1> , char_<ch2> , char_<ch3> , char_<ch4> , char_<ch5> , char_<ch6> , char_<ch7> , char_<ch8> , char_<ch9> , char_<ch10> , char_<ch11> , char_<ch12> , char_<ch13> , char_<ch14>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7 , char ch8 , char ch9 , char ch10 , char ch11 , char ch12 , char ch13 , char ch14 , char ch15
	>
	struct char_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14 , ch15
	> :
	or_<
		char_<ch0> , char_<ch1> , char_<ch2> , char_<ch3> , char_<ch4> , char_<ch5> , char_<ch6> , char_<ch7> , char_<ch8> , char_<ch9> , char_<ch10> , char_<ch11> , char_<ch12> , char_<ch13> , char_<ch14> , char_<ch15>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7 , char ch8 , char ch9 , char ch10 , char ch11 , char ch12 , char ch13 , char ch14 , char ch15 , char ch16
	>
	struct char_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14 , ch15 , ch16
	> :
	or_<
		char_<ch0> , char_<ch1> , char_<ch2> , char_<ch3> , char_<ch4> , char_<ch5> , char_<ch6> , char_<ch7> , char_<ch8> , char_<ch9> , char_<ch10> , char_<ch11> , char_<ch12> , char_<ch13> , char_<ch14> , char_<ch15> , char_<ch16>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7 , char ch8 , char ch9 , char ch10 , char ch11 , char ch12 , char ch13 , char ch14 , char ch15 , char ch16 , char ch17
	>
	struct char_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14 , ch15 , ch16 , ch17
	> :
	or_<
		char_<ch0> , char_<ch1> , char_<ch2> , char_<ch3> , char_<ch4> , char_<ch5> , char_<ch6> , char_<ch7> , char_<ch8> , char_<ch9> , char_<ch10> , char_<ch11> , char_<ch12> , char_<ch13> , char_<ch14> , char_<ch15> , char_<ch16> , char_<ch17>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7 , char ch8 , char ch9 , char ch10 , char ch11 , char ch12 , char ch13 , char ch14 , char ch15 , char ch16 , char ch17 , char ch18
	>
	struct char_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14 , ch15 , ch16 , ch17 , ch18
	> :
	or_<
		char_<ch0> , char_<ch1> , char_<ch2> , char_<ch3> , char_<ch4> , char_<ch5> , char_<ch6> , char_<ch7> , char_<ch8> , char_<ch9> , char_<ch10> , char_<ch11> , char_<ch12> , char_<ch13> , char_<ch14> , char_<ch15> , char_<ch16> , char_<ch17> , char_<ch18>
	>
	{
	};

} // namespace biscuit
