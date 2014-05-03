#pragma once

#include "../state/eos.hpp"
#include "../state/state_iterator.hpp"
#include "actor.hpp"
#include "any.hpp"
#include "before.hpp"
#include "begin.hpp"
#include "end.hpp"
#include "plus.hpp"

namespace biscuit
{

template<
	class Parser // assume... width == 1
>
struct not_
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		typename state_iterator<State>::type const cur0 = s.cur;
		
		if (Parser::parse(s, us))
		{
			s.cur = cur0;
			return false;
		}
		
		if (eos(s))
			return false;

		++s.cur;
		return true;
	}
};

template< class Parser >
struct not_< not_<Parser> > // : Parser
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		return Parser::parse(s, us);
	}
};

template< class Action, class Parser >
struct not_< actor<Action,Parser> >;
//	: actor< Action, not_<Parser> > { }; // that's it?

template<>
struct not_<any>;


template<
	class Parser
>
struct not_< before<Parser> >
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		return !before<Parser>::parse(s, us);
	}
};

template<>
struct not_<begin>
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		return !bos(s)
	}
};

template<>
struct not_<end>
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		return !eos(s)
	}
};

template< class Parser >
struct not_< plus<Parser> >
{
	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		return !Parser::parser(s);
	}
};

} // namespace biscuit
