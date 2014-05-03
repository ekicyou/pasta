#pragma once

#include "../seq.hpp"
#include "ichar.hpp"

namespace biscuit
{

template<
	char ch0 = 0, char ch1 = 0, char ch2 = 0, char ch3 = 0, char ch4 = 0,
	char ch5 = 0, char ch6 = 0, char ch7 = 0, char ch8 = 0, char ch9 = 0
>
struct istr;


template<>
struct istr<> :
	seq<>
{ };


template<
	char ch0
>
struct istr<ch0> :
	seq< ichar<ch0> >
{ };


template<
	char ch0, char ch1
>
struct istr<ch0, ch1> :
	seq< ichar<ch0>, ichar<ch1> >
{ };


template<
	char ch0, char ch1, char ch2
>
struct istr<ch0, ch1, ch2> :
	seq< ichar<ch0>, ichar<ch1>, ichar<ch2> >
{ };


template<
	char ch0, char ch1, char ch2, char ch3
>
struct istr<ch0, ch1, ch2, ch3> :
	seq< ichar<ch0>, ichar<ch1>, ichar<ch2>, ichar<ch3> >
{ };


template<
	char ch0, char ch1, char ch2, char ch3, char ch4
>
struct istr<ch0, ch1, ch2, ch3, ch4> :
	seq< ichar<ch0>, ichar<ch1>, ichar<ch2>, ichar<ch3>, ichar<ch4> >
{ };

template<
	char ch0, char ch1, char ch2, char ch3, char ch4,
	char ch5
>
struct istr<ch0, ch1, ch2, ch3, ch4, ch5> :
	seq< ichar<ch0>, ichar<ch1>, ichar<ch2>, ichar<ch3>, ichar<ch4>, ichar<ch5> >
{ };

template<
	char ch0, char ch1, char ch2, char ch3, char ch4,
	char ch5, char ch6
>
struct istr<ch0, ch1, ch2, ch3, ch4, ch5, ch6> :
	seq< ichar<ch0>, ichar<ch1>, ichar<ch2>, ichar<ch3>, ichar<ch4>, ichar<ch5>, ichar<ch6> >
{ };

template<
	char ch0, char ch1, char ch2, char ch3, char ch4,
	char ch5, char ch6, char ch7
>
struct istr<ch0, ch1, ch2, ch3, ch4, ch5, ch6, ch7> :
	seq< ichar<ch0>, ichar<ch1>, ichar<ch2>, ichar<ch3>, ichar<ch4>, ichar<ch5>, ichar<ch6>, ichar<ch7> >
{ };

template<
	char ch0, char ch1, char ch2, char ch3, char ch4,
	char ch5, char ch6, char ch7, char ch8
>
struct istr<ch0, ch1, ch2, ch3, ch4, ch5, ch6, ch7, ch8> :
	seq< ichar<ch0>, ichar<ch1>, ichar<ch2>, ichar<ch3>, ichar<ch4>, ichar<ch5>, ichar<ch6>, ichar<ch7>, ichar<ch8> >
{ };

template<
	char ch0, char ch1, char ch2, char ch3, char ch4,
	char ch5, char ch6, char ch7, char ch8, char ch9
>
struct istr<ch0, ch1, ch2, ch3, ch4, ch5, ch6, ch7, ch8, ch9> :
	seq< ichar<ch0>, ichar<ch1>, ichar<ch2>, ichar<ch3>, ichar<ch4>, ichar<ch5>, ichar<ch6>, ichar<ch7>, ichar<ch8>, ichar<ch9> >
{ };

} // namespace biscuit
