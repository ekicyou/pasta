#pragma once

#include "../or.hpp"
#include "wchar_t.hpp"


namespace biscuit
{

	template<
		wchar_t ch0 = 0 , wchar_t ch1 = 0 , wchar_t ch2 = 0 , wchar_t ch3 = 0 , wchar_t ch4 = 0 , wchar_t ch5 = 0 , wchar_t ch6 = 0 , wchar_t ch7 = 0 , wchar_t ch8 = 0 , wchar_t ch9 = 0 , wchar_t ch10 = 0 , wchar_t ch11 = 0 , wchar_t ch12 = 0 , wchar_t ch13 = 0 , wchar_t ch14 = 0 , wchar_t ch15 = 0 , wchar_t ch16 = 0 , wchar_t ch17 = 0 , wchar_t ch18 = 0 , wchar_t ch19 = 0
	>
	struct wchar_t_set :
		or_<
		wchar_t_<ch0> , wchar_t_<ch1> , wchar_t_<ch2> , wchar_t_<ch3> , wchar_t_<ch4> , wchar_t_<ch5> , wchar_t_<ch6> , wchar_t_<ch7> , wchar_t_<ch8> , wchar_t_<ch9> , wchar_t_<ch10> , wchar_t_<ch11> , wchar_t_<ch12> , wchar_t_<ch13> , wchar_t_<ch14> , wchar_t_<ch15> , wchar_t_<ch16> , wchar_t_<ch17> , wchar_t_<ch18> , wchar_t_<ch19>
		>
	{
	};

	template<

	>
	struct wchar_t_set<

	> :
	or_<

	>
	{
	};

	template<
		wchar_t ch0
	>
	struct wchar_t_set<
		ch0
	> :
	or_<
		wchar_t_<ch0>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1
	>
	struct wchar_t_set<
		ch0 , ch1
	> :
	or_<
		wchar_t_<ch0> , wchar_t_<ch1>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2
	>
	struct wchar_t_set<
		ch0 , ch1 , ch2
	> :
	or_<
		wchar_t_<ch0> , wchar_t_<ch1> , wchar_t_<ch2>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3
	>
	struct wchar_t_set<
		ch0 , ch1 , ch2 , ch3
	> :
	or_<
		wchar_t_<ch0> , wchar_t_<ch1> , wchar_t_<ch2> , wchar_t_<ch3>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4
	>
	struct wchar_t_set<
		ch0 , ch1 , ch2 , ch3 , ch4
	> :
	or_<
		wchar_t_<ch0> , wchar_t_<ch1> , wchar_t_<ch2> , wchar_t_<ch3> , wchar_t_<ch4>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5
	>
	struct wchar_t_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5
	> :
	or_<
		wchar_t_<ch0> , wchar_t_<ch1> , wchar_t_<ch2> , wchar_t_<ch3> , wchar_t_<ch4> , wchar_t_<ch5>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6
	>
	struct wchar_t_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6
	> :
	or_<
		wchar_t_<ch0> , wchar_t_<ch1> , wchar_t_<ch2> , wchar_t_<ch3> , wchar_t_<ch4> , wchar_t_<ch5> , wchar_t_<ch6>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7
	>
	struct wchar_t_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7
	> :
	or_<
		wchar_t_<ch0> , wchar_t_<ch1> , wchar_t_<ch2> , wchar_t_<ch3> , wchar_t_<ch4> , wchar_t_<ch5> , wchar_t_<ch6> , wchar_t_<ch7>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7 , wchar_t ch8
	>
	struct wchar_t_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8
	> :
	or_<
		wchar_t_<ch0> , wchar_t_<ch1> , wchar_t_<ch2> , wchar_t_<ch3> , wchar_t_<ch4> , wchar_t_<ch5> , wchar_t_<ch6> , wchar_t_<ch7> , wchar_t_<ch8>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7 , wchar_t ch8 , wchar_t ch9
	>
	struct wchar_t_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9
	> :
	or_<
		wchar_t_<ch0> , wchar_t_<ch1> , wchar_t_<ch2> , wchar_t_<ch3> , wchar_t_<ch4> , wchar_t_<ch5> , wchar_t_<ch6> , wchar_t_<ch7> , wchar_t_<ch8> , wchar_t_<ch9>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7 , wchar_t ch8 , wchar_t ch9 , wchar_t ch10
	>
	struct wchar_t_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10
	> :
	or_<
		wchar_t_<ch0> , wchar_t_<ch1> , wchar_t_<ch2> , wchar_t_<ch3> , wchar_t_<ch4> , wchar_t_<ch5> , wchar_t_<ch6> , wchar_t_<ch7> , wchar_t_<ch8> , wchar_t_<ch9> , wchar_t_<ch10>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7 , wchar_t ch8 , wchar_t ch9 , wchar_t ch10 , wchar_t ch11
	>
	struct wchar_t_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11
	> :
	or_<
		wchar_t_<ch0> , wchar_t_<ch1> , wchar_t_<ch2> , wchar_t_<ch3> , wchar_t_<ch4> , wchar_t_<ch5> , wchar_t_<ch6> , wchar_t_<ch7> , wchar_t_<ch8> , wchar_t_<ch9> , wchar_t_<ch10> , wchar_t_<ch11>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7 , wchar_t ch8 , wchar_t ch9 , wchar_t ch10 , wchar_t ch11 , wchar_t ch12
	>
	struct wchar_t_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12
	> :
	or_<
		wchar_t_<ch0> , wchar_t_<ch1> , wchar_t_<ch2> , wchar_t_<ch3> , wchar_t_<ch4> , wchar_t_<ch5> , wchar_t_<ch6> , wchar_t_<ch7> , wchar_t_<ch8> , wchar_t_<ch9> , wchar_t_<ch10> , wchar_t_<ch11> , wchar_t_<ch12>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7 , wchar_t ch8 , wchar_t ch9 , wchar_t ch10 , wchar_t ch11 , wchar_t ch12 , wchar_t ch13
	>
	struct wchar_t_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13
	> :
	or_<
		wchar_t_<ch0> , wchar_t_<ch1> , wchar_t_<ch2> , wchar_t_<ch3> , wchar_t_<ch4> , wchar_t_<ch5> , wchar_t_<ch6> , wchar_t_<ch7> , wchar_t_<ch8> , wchar_t_<ch9> , wchar_t_<ch10> , wchar_t_<ch11> , wchar_t_<ch12> , wchar_t_<ch13>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7 , wchar_t ch8 , wchar_t ch9 , wchar_t ch10 , wchar_t ch11 , wchar_t ch12 , wchar_t ch13 , wchar_t ch14
	>
	struct wchar_t_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14
	> :
	or_<
		wchar_t_<ch0> , wchar_t_<ch1> , wchar_t_<ch2> , wchar_t_<ch3> , wchar_t_<ch4> , wchar_t_<ch5> , wchar_t_<ch6> , wchar_t_<ch7> , wchar_t_<ch8> , wchar_t_<ch9> , wchar_t_<ch10> , wchar_t_<ch11> , wchar_t_<ch12> , wchar_t_<ch13> , wchar_t_<ch14>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7 , wchar_t ch8 , wchar_t ch9 , wchar_t ch10 , wchar_t ch11 , wchar_t ch12 , wchar_t ch13 , wchar_t ch14 , wchar_t ch15
	>
	struct wchar_t_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14 , ch15
	> :
	or_<
		wchar_t_<ch0> , wchar_t_<ch1> , wchar_t_<ch2> , wchar_t_<ch3> , wchar_t_<ch4> , wchar_t_<ch5> , wchar_t_<ch6> , wchar_t_<ch7> , wchar_t_<ch8> , wchar_t_<ch9> , wchar_t_<ch10> , wchar_t_<ch11> , wchar_t_<ch12> , wchar_t_<ch13> , wchar_t_<ch14> , wchar_t_<ch15>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7 , wchar_t ch8 , wchar_t ch9 , wchar_t ch10 , wchar_t ch11 , wchar_t ch12 , wchar_t ch13 , wchar_t ch14 , wchar_t ch15 , wchar_t ch16
	>
	struct wchar_t_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14 , ch15 , ch16
	> :
	or_<
		wchar_t_<ch0> , wchar_t_<ch1> , wchar_t_<ch2> , wchar_t_<ch3> , wchar_t_<ch4> , wchar_t_<ch5> , wchar_t_<ch6> , wchar_t_<ch7> , wchar_t_<ch8> , wchar_t_<ch9> , wchar_t_<ch10> , wchar_t_<ch11> , wchar_t_<ch12> , wchar_t_<ch13> , wchar_t_<ch14> , wchar_t_<ch15> , wchar_t_<ch16>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7 , wchar_t ch8 , wchar_t ch9 , wchar_t ch10 , wchar_t ch11 , wchar_t ch12 , wchar_t ch13 , wchar_t ch14 , wchar_t ch15 , wchar_t ch16 , wchar_t ch17
	>
	struct wchar_t_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14 , ch15 , ch16 , ch17
	> :
	or_<
		wchar_t_<ch0> , wchar_t_<ch1> , wchar_t_<ch2> , wchar_t_<ch3> , wchar_t_<ch4> , wchar_t_<ch5> , wchar_t_<ch6> , wchar_t_<ch7> , wchar_t_<ch8> , wchar_t_<ch9> , wchar_t_<ch10> , wchar_t_<ch11> , wchar_t_<ch12> , wchar_t_<ch13> , wchar_t_<ch14> , wchar_t_<ch15> , wchar_t_<ch16> , wchar_t_<ch17>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7 , wchar_t ch8 , wchar_t ch9 , wchar_t ch10 , wchar_t ch11 , wchar_t ch12 , wchar_t ch13 , wchar_t ch14 , wchar_t ch15 , wchar_t ch16 , wchar_t ch17 , wchar_t ch18
	>
	struct wchar_t_set<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14 , ch15 , ch16 , ch17 , ch18
	> :
	or_<
		wchar_t_<ch0> , wchar_t_<ch1> , wchar_t_<ch2> , wchar_t_<ch3> , wchar_t_<ch4> , wchar_t_<ch5> , wchar_t_<ch6> , wchar_t_<ch7> , wchar_t_<ch8> , wchar_t_<ch9> , wchar_t_<ch10> , wchar_t_<ch11> , wchar_t_<ch12> , wchar_t_<ch13> , wchar_t_<ch14> , wchar_t_<ch15> , wchar_t_<ch16> , wchar_t_<ch17> , wchar_t_<ch18>
	>
	{
	};

} // namespace biscuit
