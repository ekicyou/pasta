#line 42 "longest.hpp"
#pragma once 
#include <algorithm> 
#include <iterator> 
#include <boost/iterator/iterator_traits.hpp> 
#include "../../state/state_iterator.hpp" 
#include "../detail/na.hpp" 

namespace biscuit
{
#line 53 "longest.hpp"
template<
class P0 = na , class P1 = na , class P2 = na , class P3 = na , class P4 = na , class P5 = na , class P6 = na , class P7 = na , class P8 = na , class P9 = na , class P10 = na , class P11 = na , class P12 = na , class P13 = na , class P14 = na , class P15 = na , class P16 = na , class P17 = na , class P18 = na , class P19 = na
>
struct longest
{
template< class State, class UserState >
static bool parse(State& s, UserState& us)
{
typedef typename state_iterator<State> ::type iter_t; typedef typename boost::iterator_difference<iter_t> ::type diff_t; bool ret = false; iter_t const cur0 = s.cur; diff_t d = 0;
#line 34 "D:\\Application\\boost_1_32_0\\boost\\preprocessor\\iteration\\detail\\local.hpp"
if (P0::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P1::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P2::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P3::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P4::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P5::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P6::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P7::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P8::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P9::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P10::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P11::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P12::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P13::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P14::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P15::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P16::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P17::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P18::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P19::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
#line 68 "longest.hpp"
if (ret) { std::advance(s.cur, d); } return ret;
}
};
#line 73 "longest.hpp"
template<
>
struct longest<
>
{
template< class State, class UserState >
static bool parse(State& , UserState& )
{
return false;
}
};
#line 100 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
template<
class P0
>
struct longest<
P0
>
{
template< class State, class UserState >
static bool parse(State& s, UserState& us)
{
typedef typename state_iterator<State> ::type iter_t; typedef typename boost::iterator_difference<iter_t> ::type diff_t; bool ret = false; iter_t const cur0 = s.cur; diff_t d = 0;
#line 34 "D:\\Application\\boost_1_32_0\\boost\\preprocessor\\iteration\\detail\\local.hpp"
if (P0::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
#line 117 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
if (ret) { std::advance(s.cur, d); } return ret;
}
};
#line 100 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
template<
class P0 , class P1
>
struct longest<
P0 , P1
>
{
template< class State, class UserState >
static bool parse(State& s, UserState& us)
{
typedef typename state_iterator<State> ::type iter_t; typedef typename boost::iterator_difference<iter_t> ::type diff_t; bool ret = false; iter_t const cur0 = s.cur; diff_t d = 0;
#line 34 "D:\\Application\\boost_1_32_0\\boost\\preprocessor\\iteration\\detail\\local.hpp"
if (P0::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P1::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
#line 117 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
if (ret) { std::advance(s.cur, d); } return ret;
}
};
#line 100 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
template<
class P0 , class P1 , class P2
>
struct longest<
P0 , P1 , P2
>
{
template< class State, class UserState >
static bool parse(State& s, UserState& us)
{
typedef typename state_iterator<State> ::type iter_t; typedef typename boost::iterator_difference<iter_t> ::type diff_t; bool ret = false; iter_t const cur0 = s.cur; diff_t d = 0;
#line 34 "D:\\Application\\boost_1_32_0\\boost\\preprocessor\\iteration\\detail\\local.hpp"
if (P0::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P1::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P2::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
#line 117 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
if (ret) { std::advance(s.cur, d); } return ret;
}
};
#line 100 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
template<
class P0 , class P1 , class P2 , class P3
>
struct longest<
P0 , P1 , P2 , P3
>
{
template< class State, class UserState >
static bool parse(State& s, UserState& us)
{
typedef typename state_iterator<State> ::type iter_t; typedef typename boost::iterator_difference<iter_t> ::type diff_t; bool ret = false; iter_t const cur0 = s.cur; diff_t d = 0;
#line 34 "D:\\Application\\boost_1_32_0\\boost\\preprocessor\\iteration\\detail\\local.hpp"
if (P0::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P1::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P2::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P3::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
#line 117 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
if (ret) { std::advance(s.cur, d); } return ret;
}
};
#line 100 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
template<
class P0 , class P1 , class P2 , class P3 , class P4
>
struct longest<
P0 , P1 , P2 , P3 , P4
>
{
template< class State, class UserState >
static bool parse(State& s, UserState& us)
{
typedef typename state_iterator<State> ::type iter_t; typedef typename boost::iterator_difference<iter_t> ::type diff_t; bool ret = false; iter_t const cur0 = s.cur; diff_t d = 0;
#line 34 "D:\\Application\\boost_1_32_0\\boost\\preprocessor\\iteration\\detail\\local.hpp"
if (P0::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P1::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P2::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P3::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P4::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
#line 117 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
if (ret) { std::advance(s.cur, d); } return ret;
}
};
#line 100 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
template<
class P0 , class P1 , class P2 , class P3 , class P4 , class P5
>
struct longest<
P0 , P1 , P2 , P3 , P4 , P5
>
{
template< class State, class UserState >
static bool parse(State& s, UserState& us)
{
typedef typename state_iterator<State> ::type iter_t; typedef typename boost::iterator_difference<iter_t> ::type diff_t; bool ret = false; iter_t const cur0 = s.cur; diff_t d = 0;
#line 34 "D:\\Application\\boost_1_32_0\\boost\\preprocessor\\iteration\\detail\\local.hpp"
if (P0::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P1::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P2::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P3::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P4::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P5::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
#line 117 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
if (ret) { std::advance(s.cur, d); } return ret;
}
};
#line 100 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
template<
class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6
>
struct longest<
P0 , P1 , P2 , P3 , P4 , P5 , P6
>
{
template< class State, class UserState >
static bool parse(State& s, UserState& us)
{
typedef typename state_iterator<State> ::type iter_t; typedef typename boost::iterator_difference<iter_t> ::type diff_t; bool ret = false; iter_t const cur0 = s.cur; diff_t d = 0;
#line 34 "D:\\Application\\boost_1_32_0\\boost\\preprocessor\\iteration\\detail\\local.hpp"
if (P0::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P1::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P2::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P3::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P4::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P5::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P6::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
#line 117 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
if (ret) { std::advance(s.cur, d); } return ret;
}
};
#line 100 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
template<
class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7
>
struct longest<
P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7
>
{
template< class State, class UserState >
static bool parse(State& s, UserState& us)
{
typedef typename state_iterator<State> ::type iter_t; typedef typename boost::iterator_difference<iter_t> ::type diff_t; bool ret = false; iter_t const cur0 = s.cur; diff_t d = 0;
#line 34 "D:\\Application\\boost_1_32_0\\boost\\preprocessor\\iteration\\detail\\local.hpp"
if (P0::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P1::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P2::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P3::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P4::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P5::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P6::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P7::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
#line 117 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
if (ret) { std::advance(s.cur, d); } return ret;
}
};
#line 100 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
template<
class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8
>
struct longest<
P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8
>
{
template< class State, class UserState >
static bool parse(State& s, UserState& us)
{
typedef typename state_iterator<State> ::type iter_t; typedef typename boost::iterator_difference<iter_t> ::type diff_t; bool ret = false; iter_t const cur0 = s.cur; diff_t d = 0;
#line 34 "D:\\Application\\boost_1_32_0\\boost\\preprocessor\\iteration\\detail\\local.hpp"
if (P0::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P1::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P2::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P3::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P4::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P5::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P6::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P7::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P8::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
#line 117 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
if (ret) { std::advance(s.cur, d); } return ret;
}
};
#line 100 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
template<
class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9
>
struct longest<
P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9
>
{
template< class State, class UserState >
static bool parse(State& s, UserState& us)
{
typedef typename state_iterator<State> ::type iter_t; typedef typename boost::iterator_difference<iter_t> ::type diff_t; bool ret = false; iter_t const cur0 = s.cur; diff_t d = 0;
#line 34 "D:\\Application\\boost_1_32_0\\boost\\preprocessor\\iteration\\detail\\local.hpp"
if (P0::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P1::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P2::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P3::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P4::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P5::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P6::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P7::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P8::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P9::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
#line 117 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
if (ret) { std::advance(s.cur, d); } return ret;
}
};
#line 100 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
template<
class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10
>
struct longest<
P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10
>
{
template< class State, class UserState >
static bool parse(State& s, UserState& us)
{
typedef typename state_iterator<State> ::type iter_t; typedef typename boost::iterator_difference<iter_t> ::type diff_t; bool ret = false; iter_t const cur0 = s.cur; diff_t d = 0;
#line 34 "D:\\Application\\boost_1_32_0\\boost\\preprocessor\\iteration\\detail\\local.hpp"
if (P0::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P1::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P2::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P3::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P4::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P5::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P6::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P7::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P8::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P9::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P10::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
#line 117 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
if (ret) { std::advance(s.cur, d); } return ret;
}
};
#line 100 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
template<
class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11
>
struct longest<
P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11
>
{
template< class State, class UserState >
static bool parse(State& s, UserState& us)
{
typedef typename state_iterator<State> ::type iter_t; typedef typename boost::iterator_difference<iter_t> ::type diff_t; bool ret = false; iter_t const cur0 = s.cur; diff_t d = 0;
#line 34 "D:\\Application\\boost_1_32_0\\boost\\preprocessor\\iteration\\detail\\local.hpp"
if (P0::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P1::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P2::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P3::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P4::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P5::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P6::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P7::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P8::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P9::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P10::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P11::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
#line 117 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
if (ret) { std::advance(s.cur, d); } return ret;
}
};
#line 100 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
template<
class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12
>
struct longest<
P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12
>
{
template< class State, class UserState >
static bool parse(State& s, UserState& us)
{
typedef typename state_iterator<State> ::type iter_t; typedef typename boost::iterator_difference<iter_t> ::type diff_t; bool ret = false; iter_t const cur0 = s.cur; diff_t d = 0;
#line 34 "D:\\Application\\boost_1_32_0\\boost\\preprocessor\\iteration\\detail\\local.hpp"
if (P0::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P1::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P2::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P3::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P4::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P5::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P6::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P7::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P8::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P9::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P10::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P11::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P12::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
#line 117 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
if (ret) { std::advance(s.cur, d); } return ret;
}
};
#line 100 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
template<
class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13
>
struct longest<
P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13
>
{
template< class State, class UserState >
static bool parse(State& s, UserState& us)
{
typedef typename state_iterator<State> ::type iter_t; typedef typename boost::iterator_difference<iter_t> ::type diff_t; bool ret = false; iter_t const cur0 = s.cur; diff_t d = 0;
#line 34 "D:\\Application\\boost_1_32_0\\boost\\preprocessor\\iteration\\detail\\local.hpp"
if (P0::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P1::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P2::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P3::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P4::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P5::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P6::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P7::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P8::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P9::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P10::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P11::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P12::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P13::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
#line 117 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
if (ret) { std::advance(s.cur, d); } return ret;
}
};
#line 100 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
template<
class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14
>
struct longest<
P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14
>
{
template< class State, class UserState >
static bool parse(State& s, UserState& us)
{
typedef typename state_iterator<State> ::type iter_t; typedef typename boost::iterator_difference<iter_t> ::type diff_t; bool ret = false; iter_t const cur0 = s.cur; diff_t d = 0;
#line 34 "D:\\Application\\boost_1_32_0\\boost\\preprocessor\\iteration\\detail\\local.hpp"
if (P0::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P1::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P2::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P3::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P4::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P5::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P6::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P7::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P8::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P9::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P10::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P11::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P12::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P13::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P14::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
#line 117 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
if (ret) { std::advance(s.cur, d); } return ret;
}
};
#line 100 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
template<
class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15
>
struct longest<
P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15
>
{
template< class State, class UserState >
static bool parse(State& s, UserState& us)
{
typedef typename state_iterator<State> ::type iter_t; typedef typename boost::iterator_difference<iter_t> ::type diff_t; bool ret = false; iter_t const cur0 = s.cur; diff_t d = 0;
#line 34 "D:\\Application\\boost_1_32_0\\boost\\preprocessor\\iteration\\detail\\local.hpp"
if (P0::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P1::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P2::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P3::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P4::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P5::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P6::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P7::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P8::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P9::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P10::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P11::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P12::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P13::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P14::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P15::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
#line 117 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
if (ret) { std::advance(s.cur, d); } return ret;
}
};
#line 100 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
template<
class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16
>
struct longest<
P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16
>
{
template< class State, class UserState >
static bool parse(State& s, UserState& us)
{
typedef typename state_iterator<State> ::type iter_t; typedef typename boost::iterator_difference<iter_t> ::type diff_t; bool ret = false; iter_t const cur0 = s.cur; diff_t d = 0;
#line 34 "D:\\Application\\boost_1_32_0\\boost\\preprocessor\\iteration\\detail\\local.hpp"
if (P0::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P1::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P2::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P3::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P4::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P5::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P6::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P7::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P8::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P9::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P10::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P11::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P12::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P13::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P14::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P15::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P16::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
#line 117 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
if (ret) { std::advance(s.cur, d); } return ret;
}
};
#line 100 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
template<
class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17
>
struct longest<
P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17
>
{
template< class State, class UserState >
static bool parse(State& s, UserState& us)
{
typedef typename state_iterator<State> ::type iter_t; typedef typename boost::iterator_difference<iter_t> ::type diff_t; bool ret = false; iter_t const cur0 = s.cur; diff_t d = 0;
#line 34 "D:\\Application\\boost_1_32_0\\boost\\preprocessor\\iteration\\detail\\local.hpp"
if (P0::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P1::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P2::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P3::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P4::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P5::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P6::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P7::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P8::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P9::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P10::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P11::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P12::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P13::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P14::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P15::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P16::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P17::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
#line 117 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
if (ret) { std::advance(s.cur, d); } return ret;
}
};
#line 100 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
template<
class P0 , class P1 , class P2 , class P3 , class P4 , class P5 , class P6 , class P7 , class P8 , class P9 , class P10 , class P11 , class P12 , class P13 , class P14 , class P15 , class P16 , class P17 , class P18
>
struct longest<
P0 , P1 , P2 , P3 , P4 , P5 , P6 , P7 , P8 , P9 , P10 , P11 , P12 , P13 , P14 , P15 , P16 , P17 , P18
>
{
template< class State, class UserState >
static bool parse(State& s, UserState& us)
{
typedef typename state_iterator<State> ::type iter_t; typedef typename boost::iterator_difference<iter_t> ::type diff_t; bool ret = false; iter_t const cur0 = s.cur; diff_t d = 0;
#line 34 "D:\\Application\\boost_1_32_0\\boost\\preprocessor\\iteration\\detail\\local.hpp"
if (P0::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P1::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P2::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P3::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P4::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P5::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P6::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P7::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P8::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P9::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P10::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P11::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P12::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P13::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P14::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P15::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P16::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P17::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
if (P18::parse(s, us)) { ret = true; d = std::max( std::distance(cur0, s.cur), d ); } s.cur = cur0;
#line 117 "D:\\Application\\biscuit_0_90_0\\biscuit\\parser\\detail\\preprocessing\\longest.hpp"
if (ret) { std::advance(s.cur, d); } return ret;
}
};
#line 89 "longest.hpp"
} 
