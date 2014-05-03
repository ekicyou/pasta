#pragma once

#include "../seq.hpp"
#include "ichar.hpp"


namespace biscuit
{

	template<
		char ch0 = 0 , char ch1 = 0 , char ch2 = 0 , char ch3 = 0 , char ch4 = 0 , char ch5 = 0 , char ch6 = 0 , char ch7 = 0 , char ch8 = 0 , char ch9 = 0 , char ch10 = 0 , char ch11 = 0 , char ch12 = 0 , char ch13 = 0 , char ch14 = 0 , char ch15 = 0 , char ch16 = 0 , char ch17 = 0 , char ch18 = 0 , char ch19 = 0
	>
	struct istr :
		seq<
		ichar<ch0> , ichar<ch1> , ichar<ch2> , ichar<ch3> , ichar<ch4> , ichar<ch5> , ichar<ch6> , ichar<ch7> , ichar<ch8> , ichar<ch9> , ichar<ch10> , ichar<ch11> , ichar<ch12> , ichar<ch13> , ichar<ch14> , ichar<ch15> , ichar<ch16> , ichar<ch17> , ichar<ch18> , ichar<ch19>
		>
	{
	};

	template<

	>
	struct istr<

	> :
	seq<

	>
	{
	};

	template<
		char ch0
	>
	struct istr<
		ch0
	> :
	seq<
		ichar<ch0>
	>
	{
	};

	template<
		char ch0 , char ch1
	>
	struct istr<
		ch0 , ch1
	> :
	seq<
		ichar<ch0> , ichar<ch1>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2
	>
	struct istr<
		ch0 , ch1 , ch2
	> :
	seq<
		ichar<ch0> , ichar<ch1> , ichar<ch2>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3
	>
	struct istr<
		ch0 , ch1 , ch2 , ch3
	> :
	seq<
		ichar<ch0> , ichar<ch1> , ichar<ch2> , ichar<ch3>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4
	>
	struct istr<
		ch0 , ch1 , ch2 , ch3 , ch4
	> :
	seq<
		ichar<ch0> , ichar<ch1> , ichar<ch2> , ichar<ch3> , ichar<ch4>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5
	>
	struct istr<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5
	> :
	seq<
		ichar<ch0> , ichar<ch1> , ichar<ch2> , ichar<ch3> , ichar<ch4> , ichar<ch5>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6
	>
	struct istr<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6
	> :
	seq<
		ichar<ch0> , ichar<ch1> , ichar<ch2> , ichar<ch3> , ichar<ch4> , ichar<ch5> , ichar<ch6>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7
	>
	struct istr<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7
	> :
	seq<
		ichar<ch0> , ichar<ch1> , ichar<ch2> , ichar<ch3> , ichar<ch4> , ichar<ch5> , ichar<ch6> , ichar<ch7>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7 , char ch8
	>
	struct istr<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8
	> :
	seq<
		ichar<ch0> , ichar<ch1> , ichar<ch2> , ichar<ch3> , ichar<ch4> , ichar<ch5> , ichar<ch6> , ichar<ch7> , ichar<ch8>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7 , char ch8 , char ch9
	>
	struct istr<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9
	> :
	seq<
		ichar<ch0> , ichar<ch1> , ichar<ch2> , ichar<ch3> , ichar<ch4> , ichar<ch5> , ichar<ch6> , ichar<ch7> , ichar<ch8> , ichar<ch9>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7 , char ch8 , char ch9 , char ch10
	>
	struct istr<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10
	> :
	seq<
		ichar<ch0> , ichar<ch1> , ichar<ch2> , ichar<ch3> , ichar<ch4> , ichar<ch5> , ichar<ch6> , ichar<ch7> , ichar<ch8> , ichar<ch9> , ichar<ch10>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7 , char ch8 , char ch9 , char ch10 , char ch11
	>
	struct istr<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11
	> :
	seq<
		ichar<ch0> , ichar<ch1> , ichar<ch2> , ichar<ch3> , ichar<ch4> , ichar<ch5> , ichar<ch6> , ichar<ch7> , ichar<ch8> , ichar<ch9> , ichar<ch10> , ichar<ch11>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7 , char ch8 , char ch9 , char ch10 , char ch11 , char ch12
	>
	struct istr<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12
	> :
	seq<
		ichar<ch0> , ichar<ch1> , ichar<ch2> , ichar<ch3> , ichar<ch4> , ichar<ch5> , ichar<ch6> , ichar<ch7> , ichar<ch8> , ichar<ch9> , ichar<ch10> , ichar<ch11> , ichar<ch12>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7 , char ch8 , char ch9 , char ch10 , char ch11 , char ch12 , char ch13
	>
	struct istr<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13
	> :
	seq<
		ichar<ch0> , ichar<ch1> , ichar<ch2> , ichar<ch3> , ichar<ch4> , ichar<ch5> , ichar<ch6> , ichar<ch7> , ichar<ch8> , ichar<ch9> , ichar<ch10> , ichar<ch11> , ichar<ch12> , ichar<ch13>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7 , char ch8 , char ch9 , char ch10 , char ch11 , char ch12 , char ch13 , char ch14
	>
	struct istr<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14
	> :
	seq<
		ichar<ch0> , ichar<ch1> , ichar<ch2> , ichar<ch3> , ichar<ch4> , ichar<ch5> , ichar<ch6> , ichar<ch7> , ichar<ch8> , ichar<ch9> , ichar<ch10> , ichar<ch11> , ichar<ch12> , ichar<ch13> , ichar<ch14>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7 , char ch8 , char ch9 , char ch10 , char ch11 , char ch12 , char ch13 , char ch14 , char ch15
	>
	struct istr<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14 , ch15
	> :
	seq<
		ichar<ch0> , ichar<ch1> , ichar<ch2> , ichar<ch3> , ichar<ch4> , ichar<ch5> , ichar<ch6> , ichar<ch7> , ichar<ch8> , ichar<ch9> , ichar<ch10> , ichar<ch11> , ichar<ch12> , ichar<ch13> , ichar<ch14> , ichar<ch15>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7 , char ch8 , char ch9 , char ch10 , char ch11 , char ch12 , char ch13 , char ch14 , char ch15 , char ch16
	>
	struct istr<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14 , ch15 , ch16
	> :
	seq<
		ichar<ch0> , ichar<ch1> , ichar<ch2> , ichar<ch3> , ichar<ch4> , ichar<ch5> , ichar<ch6> , ichar<ch7> , ichar<ch8> , ichar<ch9> , ichar<ch10> , ichar<ch11> , ichar<ch12> , ichar<ch13> , ichar<ch14> , ichar<ch15> , ichar<ch16>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7 , char ch8 , char ch9 , char ch10 , char ch11 , char ch12 , char ch13 , char ch14 , char ch15 , char ch16 , char ch17
	>
	struct istr<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14 , ch15 , ch16 , ch17
	> :
	seq<
		ichar<ch0> , ichar<ch1> , ichar<ch2> , ichar<ch3> , ichar<ch4> , ichar<ch5> , ichar<ch6> , ichar<ch7> , ichar<ch8> , ichar<ch9> , ichar<ch10> , ichar<ch11> , ichar<ch12> , ichar<ch13> , ichar<ch14> , ichar<ch15> , ichar<ch16> , ichar<ch17>
	>
	{
	};

	template<
		char ch0 , char ch1 , char ch2 , char ch3 , char ch4 , char ch5 , char ch6 , char ch7 , char ch8 , char ch9 , char ch10 , char ch11 , char ch12 , char ch13 , char ch14 , char ch15 , char ch16 , char ch17 , char ch18
	>
	struct istr<
		ch0 , ch1 , ch2 , ch3 , ch4 , ch5 , ch6 , ch7 , ch8 , ch9 , ch10 , ch11 , ch12 , ch13 , ch14 , ch15 , ch16 , ch17 , ch18
	> :
	seq<
		ichar<ch0> , ichar<ch1> , ichar<ch2> , ichar<ch3> , ichar<ch4> , ichar<ch5> , ichar<ch6> , ichar<ch7> , ichar<ch8> , ichar<ch9> , ichar<ch10> , ichar<ch11> , ichar<ch12> , ichar<ch13> , ichar<ch14> , ichar<ch15> , ichar<ch16> , ichar<ch17> , ichar<ch18>
	>
	{
	};

} // namespace biscuit
