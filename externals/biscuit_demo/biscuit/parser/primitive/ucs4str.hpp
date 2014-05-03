#pragma once

#include "../seq.hpp"
#include "ucs4.hpp"

namespace biscuit
{

	template<
		wchar_t ch0 = 0 , wchar_t ch1 = 0 , wchar_t ch2 = 0 , wchar_t ch3 = 0 , wchar_t ch4 = 0 , wchar_t ch5 = 0 , wchar_t ch6 = 0 , wchar_t ch7 = 0 , wchar_t ch8 = 0 , wchar_t ch9 = 0 , wchar_t ch10 = 0 , wchar_t ch11 = 0 , wchar_t ch12 = 0 , wchar_t ch13 = 0 , wchar_t ch14 = 0 , wchar_t ch15 = 0 , wchar_t ch16 = 0 , wchar_t ch17 = 0 , wchar_t ch18 = 0 , wchar_t ch19 = 0
	>
	struct ucs4str :
		seq<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8> , ucs4<ch9> , ucs4<ch10> , ucs4<ch11> , ucs4<ch12> , ucs4<ch13> , ucs4<ch14> , ucs4<ch15> , ucs4<ch16> , ucs4<ch17> , ucs4<ch18> , ucs4<ch19>
		>
	{
	};

	template<

	>
	struct ucs4str<

	> :
	seq<

	>
	{
	};

	template<
		wchar_t ch0
	>
	struct ucs4str<
		ch0
	> :
	seq<
		ucs4<ch0>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1
	>
	struct ucs4str<
		ch0 , ch1
	> :
	seq<
		ucs4<ch0> , ucs4<ch1>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2
	>
	struct ucs4str<
		ch0 , ch1 , ch2
	> :
	seq<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3
	>
	struct ucs4str<
		ch0 , ch1 , ch2 , ch3
	> :
	seq<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4
	>
	struct ucs4str<
		ch0 , ch1 , ch2 , ch3 , ch4
	> :
	seq<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5
	>
	struct ucs4str<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5
	> :
	seq<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6
	>
	struct ucs4str<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6
	> :
	seq<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7
	>
	struct ucs4str<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7
	> :
	seq<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7 , wchar_t ch8
	>
	struct ucs4str<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8
	> :
	seq<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7 , wchar_t ch8 , wchar_t ch9
	>
	struct ucs4str<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9
	> :
	seq<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8> , ucs4<ch9>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7 , wchar_t ch8 , wchar_t ch9 , wchar_t ch10
	>
	struct ucs4str<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10
	> :
	seq<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8> , ucs4<ch9> , ucs4<ch10>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7 , wchar_t ch8 , wchar_t ch9 , wchar_t ch10 , wchar_t ch11
	>
	struct ucs4str<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11
	> :
	seq<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8> , ucs4<ch9> , ucs4<ch10> , ucs4<ch11>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7 , wchar_t ch8 , wchar_t ch9 , wchar_t ch10 , wchar_t ch11 , wchar_t ch12
	>
	struct ucs4str<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12
	> :
	seq<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8> , ucs4<ch9> , ucs4<ch10> , ucs4<ch11> , ucs4<ch12>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7 , wchar_t ch8 , wchar_t ch9 , wchar_t ch10 , wchar_t ch11 , wchar_t ch12 , wchar_t ch13
	>
	struct ucs4str<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13
	> :
	seq<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8> , ucs4<ch9> , ucs4<ch10> , ucs4<ch11> , ucs4<ch12> , ucs4<ch13>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7 , wchar_t ch8 , wchar_t ch9 , wchar_t ch10 , wchar_t ch11 , wchar_t ch12 , wchar_t ch13 , wchar_t ch14
	>
	struct ucs4str<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14
	> :
	seq<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8> , ucs4<ch9> , ucs4<ch10> , ucs4<ch11> , ucs4<ch12> , ucs4<ch13> , ucs4<ch14>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7 , wchar_t ch8 , wchar_t ch9 , wchar_t ch10 , wchar_t ch11 , wchar_t ch12 , wchar_t ch13 , wchar_t ch14 , wchar_t ch15
	>
	struct ucs4str<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14 , ch15
	> :
	seq<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8> , ucs4<ch9> , ucs4<ch10> , ucs4<ch11> , ucs4<ch12> , ucs4<ch13> , ucs4<ch14> , ucs4<ch15>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7 , wchar_t ch8 , wchar_t ch9 , wchar_t ch10 , wchar_t ch11 , wchar_t ch12 , wchar_t ch13 , wchar_t ch14 , wchar_t ch15 , wchar_t ch16
	>
	struct ucs4str<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14 , ch15 , ch16
	> :
	seq<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8> , ucs4<ch9> , ucs4<ch10> , ucs4<ch11> , ucs4<ch12> , ucs4<ch13> , ucs4<ch14> , ucs4<ch15> , ucs4<ch16>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7 , wchar_t ch8 , wchar_t ch9 , wchar_t ch10 , wchar_t ch11 , wchar_t ch12 , wchar_t ch13 , wchar_t ch14 , wchar_t ch15 , wchar_t ch16 , wchar_t ch17
	>
	struct ucs4str<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14 , ch15 , ch16 , ch17
	> :
	seq<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8> , ucs4<ch9> , ucs4<ch10> , ucs4<ch11> , ucs4<ch12> , ucs4<ch13> , ucs4<ch14> , ucs4<ch15> , ucs4<ch16> , ucs4<ch17>
	>
	{
	};

	template<
		wchar_t ch0 , wchar_t ch1 , wchar_t ch2 , wchar_t ch3 , wchar_t ch4 , wchar_t ch5 , wchar_t ch6 , wchar_t ch7 , wchar_t ch8 , wchar_t ch9 , wchar_t ch10 , wchar_t ch11 , wchar_t ch12 , wchar_t ch13 , wchar_t ch14 , wchar_t ch15 , wchar_t ch16 , wchar_t ch17 , wchar_t ch18
	>
	struct ucs4str<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14 , ch15 , ch16 , ch17 , ch18
	> :
	seq<
		ucs4<ch0> , ucs4<ch1> , ucs4<ch2> , ucs4<ch3> , ucs4<ch4> , ucs4<ch5> , ucs4<ch6> , ucs4<ch7> , ucs4<ch8> , ucs4<ch9> , ucs4<ch10> , ucs4<ch11> , ucs4<ch12> , ucs4<ch13> , ucs4<ch14> , ucs4<ch15> , ucs4<ch16> , ucs4<ch17> , ucs4<ch18>
	>
	{
	};

} // namespace biscuit
