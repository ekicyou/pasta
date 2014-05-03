#pragma once

#include "../or.hpp"
#include "char.hpp"

namespace biscuit
{

template<
	char ch0 = 0, char ch1 = 0, char ch2 = 0, char ch3 = 0, char ch4 = 0,
	char ch5 = 0, char ch6 = 0, char ch7 = 0, char ch8 = 0, char ch9 = 0
>
struct char_set;


template<>
struct char_set<> :
	or_<>
{ };


template<
	char ch0
>
struct char_set<ch0> :
	or_< char_<ch0> >
{ };


template<
	char ch0, char ch1
>
struct char_set<ch0, ch1> :
	or_< char_<ch0>, char_<ch1> >
{ };


template<
	char ch0, char ch1, char ch2
>
struct char_set<ch0, ch1, ch2> :
	or_< char_<ch0>, char_<ch1>, char_<ch2> >
{ };


template<
	char ch0, char ch1, char ch2, char ch3
>
struct char_set<ch0, ch1, ch2, ch3> :
	or_< char_<ch0>, char_<ch1>, char_<ch2>, char_<ch3> >
{ };


template<
	char ch0, char ch1, char ch2, char ch3, char ch4
>
struct char_set<ch0, ch1, ch2, ch3, ch4> :
	or_< char_<ch0>, char_<ch1>, char_<ch2>, char_<ch3>, char_<ch4> >
{ };

template<
	char ch0, char ch1, char ch2, char ch3, char ch4,
	char ch5
>
struct char_set<ch0, ch1, ch2, ch3, ch4, ch5> :
	or_< char_<ch0>, char_<ch1>, char_<ch2>, char_<ch3>, char_<ch4>, char_<ch5> >
{ };

template<
	char ch0, char ch1, char ch2, char ch3, char ch4,
	char ch5, char ch6
>
struct char_set<ch0, ch1, ch2, ch3, ch4, ch5, ch6> :
	or_< char_<ch0>, char_<ch1>, char_<ch2>, char_<ch3>, char_<ch4>, char_<ch5>, char_<ch6> >
{ };

template<
	char ch0, char ch1, char ch2, char ch3, char ch4,
	char ch5, char ch6, char ch7
>
struct char_set<ch0, ch1, ch2, ch3, ch4, ch5, ch6, ch7> :
	or_< char_<ch0>, char_<ch1>, char_<ch2>, char_<ch3>, char_<ch4>, char_<ch5>, char_<ch6>, char_<ch7> >
{ };

template<
	char ch0, char ch1, char ch2, char ch3, char ch4,
	char ch5, char ch6, char ch7, char ch8
>
struct char_set<ch0, ch1, ch2, ch3, ch4, ch5, ch6, ch7, ch8> :
	or_< char_<ch0>, char_<ch1>, char_<ch2>, char_<ch3>, char_<ch4>, char_<ch5>, char_<ch6>, char_<ch7>, char_<ch8> >
{ };

template<
	char ch0, char ch1, char ch2, char ch3, char ch4,
	char ch5, char ch6, char ch7, char ch8, char ch9
>
struct char_set<ch0, ch1, ch2, ch3, ch4, ch5, ch6, ch7, ch8, ch9> :
	or_< char_<ch0>, char_<ch1>, char_<ch2>, char_<ch3>, char_<ch4>, char_<ch5>, char_<ch6>, char_<ch7>, char_<ch8>, char_<ch9> >
{ };

} // namespace biscuit
