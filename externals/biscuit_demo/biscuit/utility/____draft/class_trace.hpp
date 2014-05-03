#pragma once

///////////////////////////////////////////////////////////////////////////////
// Note:
//  get_class_trace depends on optimization..., but this is the modern programming.
//
#include <boost/mpl/and.hpp>
#include <boost/mpl/apply.hpp>
#include <boost/mpl/bool.hpp>
#include <boost/mpl/if.hpp>
#include <boost/mpl/placeholders.hpp>
#include <typeinfo>
#include "is_debug.hpp"

namespace biscuit {

	namespace detail {

	// Note: if inline missed, the compiler doesn't remove s_count.
	inline int& class_trace_indent_count()
	{
		static int s_count = 0;
		return s_count;
	}

	template< class T, int SpaceCount >
	struct class_trace_impl_on
	{
		explicit class_trace_impl_on(LPCTSTR str)
		{
			int spaces = SpaceCount*class_trace_indent_count();
			for (int i = 0; i < spaces; ++i) {
				ATLTRACE(_T(" "));
			}
			// Note: For the compiler, it seems better to define a temporary tid.
			const std::type_info& tid = typeid(T);
			ATLTRACE(tid.name());
			ATLTRACE(_T("::"));
			ATLTRACE(str);
			ATLTRACE(_T("\n"));
			++class_trace_indent_count();
		}

		~class_trace_impl_on()
		{
			--class_trace_indent_count();
		}

		typedef class_trace_impl_on type;
	};

	template< class, int >
	struct class_trace_impl_off
	{
		explicit class_trace_impl_off(LPCTSTR) { }
		
		typedef class_trace_impl_off type;
	};

	} // namespace detail


template< class T, bool bOn, int SpaceCount = 2 >
struct get_class_trace
{
	// Note: eval_if can make C4510 and C4610 warnings
	typedef typename boost::mpl::if_<
		boost::mpl::and_< utility::is_debug, boost::mpl::bool_<bOn> >,
		detail::class_trace_impl_on<T, SpaceCount>,
		detail::class_trace_impl_off<T, SpaceCount>
	>::type type;
};

} // namespace biscuit
