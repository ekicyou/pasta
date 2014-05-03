#pragma once

#include <boost/iterator/iterator_traits.hpp>
#include <boost/range/result_iterator.hpp>

#include "parser/directive/requires.hpp"
#include "parser/directive/transform.hpp"
#include "parser/actor.hpp"
#include "parser/value.hpp"
#include "state/eos.hpp"

namespace biscuit
{

template< class Derived, class ForwardRange >
struct grammar
{
	typedef typename boost::range_result_iterator<ForwardRange>::type iterator_type;
	typedef typename boost::iterator_value<iterator_type>::type value_type;

	// actor_
	template<
		void (Derived::*func)(iterator_type, iterator_type)
	>
	struct action_
	{
		template< class Iter, class UserState >
		void operator()(Iter f, Iter l, UserState& us)
		{
			(us.*func)(f, l);
		}
	};
	
	template<
		class Parser,
		void (Derived::*func)(iterator_type, iterator_type)
	>
	struct actor_ : biscuit::actor< Parser, action_<func> >
	{ };
	
	// value_
	template<
		value_type (Derived::*get_value)()
	>
	struct value_ftor_
	{
		template< class UserState >
		value_type operator()(UserState& us)
		{
			return (us.*get_value)();
		}
	};
	
	template<
		value_type (Derived::*get_value)()
	>
	struct value_ : value< value_ftor_<get_value> >
	{ };

	// requires_
	template<
		bool (Derived::*func)(iterator_type, iterator_type)
	>
	struct requires_pred_
	{
		template< class Iter, class UserState >
		bool operator()(Iter f, Iter l, UserState& us)
		{
			return (us.*func)(f, l);
		}
	};
	
	template<
		class Parser,
		bool (Derived::*func)(iterator_type, iterator_type)
	>
	struct requires_ : biscuit::requires< Parser, requires_pred_<func> >
	{ };
	
	// transform_
	template<
		value_type (Derived::*func)(iterator_type, iterator_type)
	>
	struct transform_ftor_
	{
		template< class Iter, class UserState >
		value_type operator()(Iter f, Iter l, UserState& us)
		{
			return (us.*func)(f, l);
		}
	};
	
	template<
		class Parser,
		value_type (Derived::*func)(iterator_type, iterator_type)
	>
	struct transform_ : biscuit::transform< Parser, transform_ftor_<func> >
	{ };
	
}; // struct grammar

} // namespace biscuit
