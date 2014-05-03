#pragma once

#include <algorithm>
#include <iostream>
#include <iterator>
#include <typeinfo>
#include <boost/mpl/eval_if.hpp>
#include <boost/mpl/bool.hpp>

#include "../state/state_iterator.hpp"
#include "../utility/is_debug.hpp"

namespace biscuit
{

	namespace detail
	{
		// Note: if inline missed, the compiler doesn't remove s_count.
		inline int& class_trace_indent_count()
		{
			static int s_count = -1;
			return s_count;
		}
		
		struct indents
		{
			indents()
			{	
				++class_trace_indent_count();
			}
			
			~indents()
			{
				--class_trace_indent_count();
			}

			template< class OStream >
			OStream& output(OStream& out)
			{
				int spaces = 2 * class_trace_indent_count();
				for (int i = 0; i < spaces; ++i)
					out << ' ';

				return out;
			}
		};

		template< class ParserName, class Parser, class On >
		struct tracer_base
		{
			struct on_release
			{
				template< class State, class UserState >
				static bool parse(State& s, UserState& us)
				{
					return Parser::parse(s, us);
				}
				
				typedef on_release type;
			};
			
			struct on_debug
			{
				template< class State, class UserState >
				static bool parse(State& s, UserState& us)
				{
					typedef typename state_iterator<State>::type iter_t;
					typedef typename boost::iterator_value<iter_t>::type val_t;

					indents inds;
		
					// Note: For the compiler, it seems better to define a temporary tid.
					std::type_info const& tid = typeid(ParserName);
					std::string tag = std::string(tid.name()) + ":\t";
					inds.output(std::cout) << tag << '"';
					std::copy(s.cur, s.last, std::ostream_iterator<val_t>(std::cout));
					std::cout << '"' << std::endl;

					bool ok = Parser::parse(s, us);

					char em = ok ? '/' : '#';
					inds.output(std::cout) << em << tag << '"';
					std::copy(s.cur, s.last, std::ostream_iterator<val_t>(std::cout));
					std::cout << '"' << std::endl;
					return ok;
				}
				
				typedef on_debug type;
			};
			
			typedef typename boost::mpl::eval_if<
				boost::mpl::and_<is_debug, On>,
				on_debug,
				on_release
			>::type type;
		};
	} // namespace detail

template<
	class ParserName, class Parser, bool on = true
>
struct tracer :
	detail::tracer_base< ParserName, Parser, boost::mpl::bool_<on> >::type
{ };


#if 0

	namespace detail
	{
		template< class ParserName, class Parser >
		struct tracer_on_release
		{
			template< class State, class UserState >
			static bool parse(State& s, UserState& us)
			{
				return Parser::parse(s, us);
			}
			
			typedef tracer_on_release type;
		};

		template< class ParserName, class Parser >		
		struct tracer_on_debug
		{
			template< class State, class UserState >
			static bool parse2(State& s, UserState& us)
			{
				typedef typename state_iterator<State>::type iter_t;
				typedef typename boost::iterator_value<iter_t>::type val_t;
				
				std::type_info const& tid = typeid(ParserName);
				std::string tag = std::string(tid.name()) + ":\t\t";
				std::cout << tag;
				std::copy(s.cur, s.last, std::ostream_iterator<val_t>(std::cout));
				std::cout << std::endl;
				
				bool ok = Parser::parse(s, us);
				char em = ok ? '/' : '#';
				std::cout << em << tag;
				std::copy(s.cur, s.last, std::ostream_iterator<val_t>(std::cout));
				std::cout << std::endl;

				return ok;
			}
			
			typedef tracer_on_debug type;
		};
	} // namespace detail

template<
	class ParserName, class Parser
>
struct tracer : detail::tracer_on_debug<ParserName, Parser>
{
	typedef typename detail::tracer_on_debug<ParserName, Parser> super_t;

	template< class State, class UserState >
	static bool parse(State& s, UserState& us)
	{
		return true;//super_t::parse2(s, us);
	}

};
/*
boost::mpl::apply<
	typename boost::mpl::eval_if<
		is_debug,
		detail::tracer_on_debug<boost::mpl::_,boost::mpl::_>,
		detail::tracer_on_release<boost::mpl::_, boost::mpl::_>
	>::type,
	ParserName,
	Parser
>::type
{ };*/

#endif


} // namespace biscuit
