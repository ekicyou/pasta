#pragma once

#include "../seq.hpp"
#include "wchar_t.hpp"

namespace biscuit
{

template<
	wchar_t ch0 = 0, wchar_t ch1 = 0, wchar_t ch2 = 0, wchar_t ch3 = 0, wchar_t ch4 = 0,
	wchar_t ch5 = 0, wchar_t ch6 = 0, wchar_t ch7 = 0, wchar_t ch8 = 0, wchar_t ch9 = 0
>
struct wstr;


template<>
struct wstr<> :
	seq<>
{ };


template<
	wchar_t ch0
>
struct wstr<ch0> :
	seq< wchar_t_<ch0> >
{ };


template<
	wchar_t ch0, wchar_t ch1
>
struct wstr<ch0, ch1> :
	seq< wchar_t_<ch0>, wchar_t_<ch1> >
{ };


template<
	wchar_t ch0, wchar_t ch1, wchar_t ch2
>
struct wstr<ch0, ch1, ch2> :
	seq< wchar_t_<ch0>, wchar_t_<ch1>, wchar_t_<ch2> >
{ };


template<
	wchar_t ch0, wchar_t ch1, wchar_t ch2, wchar_t ch3
>
struct wstr<ch0, ch1, ch2, ch3> :
	seq< wchar_t_<ch0>, wchar_t_<ch1>, wchar_t_<ch2>, wchar_t_<ch3> >
{ };


template<
	wchar_t ch0, wchar_t ch1, wchar_t ch2, wchar_t ch3, wchar_t ch4
>
struct wstr<ch0, ch1, ch2, ch3, ch4> :
	seq< wchar_t_<ch0>, wchar_t_<ch1>, wchar_t_<ch2>, wchar_t_<ch3>, wchar_t_<ch4> >
{ };

template<
	wchar_t ch0, wchar_t ch1, wchar_t ch2, wchar_t ch3, wchar_t ch4,
	wchar_t ch5
>
struct wstr<ch0, ch1, ch2, ch3, ch4, ch5> :
	seq< wchar_t_<ch0>, wchar_t_<ch1>, wchar_t_<ch2>, wchar_t_<ch3>, wchar_t_<ch4>, wchar_t_<ch5> >
{ };

template<
	wchar_t ch0, wchar_t ch1, wchar_t ch2, wchar_t ch3, wchar_t ch4,
	wchar_t ch5, wchar_t ch6
>
struct wstr<ch0, ch1, ch2, ch3, ch4, ch5, ch6> :
	seq< wchar_t_<ch0>, wchar_t_<ch1>, wchar_t_<ch2>, wchar_t_<ch3>, wchar_t_<ch4>, wchar_t_<ch5>, wchar_t_<ch6> >
{ };

template<
	wchar_t ch0, wchar_t ch1, wchar_t ch2, wchar_t ch3, wchar_t ch4,
	wchar_t ch5, wchar_t ch6, wchar_t ch7
>
struct wstr<ch0, ch1, ch2, ch3, ch4, ch5, ch6, ch7> :
	seq< wchar_t_<ch0>, wchar_t_<ch1>, wchar_t_<ch2>, wchar_t_<ch3>, wchar_t_<ch4>, wchar_t_<ch5>, wchar_t_<ch6>, wchar_t_<ch7> >
{ };

template<
	wchar_t ch0, wchar_t ch1, wchar_t ch2, wchar_t ch3, wchar_t ch4,
	wchar_t ch5, wchar_t ch6, wchar_t ch7, wchar_t ch8
>
struct wstr<ch0, ch1, ch2, ch3, ch4, ch5, ch6, ch7, ch8> :
	seq< wchar_t_<ch0>, wchar_t_<ch1>, wchar_t_<ch2>, wchar_t_<ch3>, wchar_t_<ch4>, wchar_t_<ch5>, wchar_t_<ch6>, wchar_t_<ch7>, wchar_t_<ch8> >
{ };

template<
	wchar_t ch0, wchar_t ch1, wchar_t ch2, wchar_t ch3, wchar_t ch4,
	wchar_t ch5, wchar_t ch6, wchar_t ch7, wchar_t ch8, wchar_t ch9
>
struct wstr<ch0, ch1, ch2, ch3, ch4, ch5, ch6, ch7, ch8, ch9> :
	seq< wchar_t_<ch0>, wchar_t_<ch1>, wchar_t_<ch2>, wchar_t_<ch3>, wchar_t_<ch4>, wchar_t_<ch5>, wchar_t_<ch6>, wchar_t_<ch7>, wchar_t_<ch8>, wchar_t_<ch9> >
{ };

} // namespace biscuit
