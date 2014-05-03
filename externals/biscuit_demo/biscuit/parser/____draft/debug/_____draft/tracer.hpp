#pragma once

#include <algorithm>
#include <iostream>
#include <iterator>
#include <typeinfo>
#include <boost/mpl/eval_if.hpp>
#include <boost/mpl/bool.hpp>
#include <boost/range/begin.hpp>
#include <boost/range/end.hpp>

#include "../state/state_iterator.hpp"
#include "../utility/is_debug.hpp"
#include "../utility/dostream.hpp"

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
			static int s_count = -1;
			return s_count;
		}
		
		struct indents
		{
			indents()
			{
				++class_trace_indent_count();
				int spaces = 2 * class_trace_indent_count();
				str = std::string(spaces, ' ');
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
					
					typedef std::ostream_iterator<val_t> oiter_t;
					// typedef basic_ostream_type< char, std::char_traits<char> >::type ostream_t;
					// ostream_t& out = GetOStream()(); // makes fatal error, and breaks VC7! clean up project!
					// oiter_t oit(out);

					oiter_t oit(std::cout);
					
					// Note: For the compiler, it seems better to define a temporary tid.
					std::type_info const& tid = typeid(ParserName);
					std::string tag = std::string(tid.name());

					std::string stag = inds.str + tag + '"';
					std::copy(boost::begin(stag), boost::end(stag), oit);
					std::copy(s.cur, s.last, oit);
					*oit = '"';
					*oit = '\n';

					bool ok = Parser::parse(s, us);

					char em = ok ? '/' : '#';
					std::string etag = inds.str + em + tag + '"';
					std::copy(boost::begin(etag), boost::end(etag), oit);
					std::copy(s.cur, s.last, oit);
					*oit = '"';
					*oit = '\n';
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

struct get_cout
{
	typedef basic_ostream_type<>::type ostream_t;

	ostream_t& operator()() const
	{
		return std::cout;
	}
};

struct get_dout
{
	typedef basic_ostream_type<>::type ostream_t;

	ostream_t& operator()() const
	{
		return dout;
	}
};

template<
	class ParserName, class Parser, class On = boost::mpl::true_ 
>
struct tracer :
	detail::tracer_base<ParserName, Parser, On>::type
{ };

} // namespace biscuit
