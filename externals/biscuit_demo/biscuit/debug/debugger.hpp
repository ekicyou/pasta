#pragma once

#include <iostream>
#include <typeinfo>
#include <boost/mpl/eval_if.hpp>
#include <boost/mpl/bool.hpp>
#include <boost/range/result_iterator.hpp>
#include <boost/range/begin.hpp>
#include <boost/range/end.hpp>

#include "../state/state_iterator.hpp"
#include "../utility/is_debug.hpp"

#if !defined(BISCUIT_DEBUG_OUT)
#define BISCUIT_DEBUG_OUT std::cout
#endif

namespace biscuit
{

template< class Char = char, class Traits = std::char_traits<Char> >
struct basic_ostream_type
{
	typedef std::basic_ostream<Char, Traits> type;
};

	namespace detail
	{
		// Note: if inline missed, the compiler doesn't remove s_count.
		inline int& class_trace_indent_count()
		{
			static int s_count = 0;
			return s_count;
		}
		
		struct indents
		{
			indents()
			{
				int spaces = 2 * class_trace_indent_count();
				str = std::string(spaces, ' ');
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

			std::string str;
		};

		template< class ParserName, class Parser, class On > //class GetOStream >
		struct debugger_base
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
					typedef boost::range_result_iterator<std::string>::type siter_t;

					indents inds;
					std::type_info const& tid = typeid(ParserName);
					std::string tag = std::string(tid.name());

					{
						std::string stag = inds.str + tag + ": \"";
						for (siter_t it = boost::begin(stag); it != boost::end(stag); ++it)
						{
							BISCUIT_DEBUG_OUT << *it;
						}
						for (iter_t it = s.cur; it != s.last; ++it)
						{
							BISCUIT_DEBUG_OUT << *it;
						}
						BISCUIT_DEBUG_OUT << '"';
						BISCUIT_DEBUG_OUT << '\n';
					}

					bool ok = Parser::parse(s, us);
					
					{
						char em = ok ? '/' : '#';
						std::string etag = inds.str + em + tag + ": \"";
						for (siter_t it = boost::begin(etag); it != boost::end(etag); ++it)
						{
							BISCUIT_DEBUG_OUT << *it;
						}
						for (iter_t it = s.cur; it != s.last; ++it)
						{
							BISCUIT_DEBUG_OUT << *it;
						}
						BISCUIT_DEBUG_OUT << '"';
						BISCUIT_DEBUG_OUT << '\n';
					}

					return ok;
				}
				
				typedef on_debug type;
			};
			
			typedef typename boost::mpl::eval_if<
				boost::mpl::and_< is_debug, On >,
				on_debug,
				on_release
			>::type type;
		};
	} // namespace detail

template<
	class ParserName, class Parser, class On = boost::mpl::true_ 
>
struct debugger :
	detail::debugger_base<ParserName, Parser, On>::type
{ };

} // namespace biscuit
