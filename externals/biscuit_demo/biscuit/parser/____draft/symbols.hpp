#pragma once

#include <boost/mpl/vector.hpp>
#include <boost/mpl/partition.hpp>
#include <boost/mpl/back_inserter.hpp>
#include <boost/mpl/place_holders.hpp>

namespace biscuit
{

	namespace detail
	{
		
		template< class Seqs >
		struct unwrap_or
		{
			
			typedef 
		};
		
		template< class Seqs >
		struct pop_fronts
		{
			typedef typename boost::mpl::filter_view<
				Seqs,
				boost::mpl::not_< boost::mpl::empty<boost::mpl::_> >
			>::type seqs_t;
			
			typedef typename boost::mpl::transform_view<
				seqs_t,
				boost::mpl::pop_front<boost::mpl::_>
			>::type type;
		};
		
		
		
	} // namespace detail

template< class Seqs, class Level >
struct symbols_base
{
	typedef Seqs seqs_t;
	
	typedef typename detail::get_pivot<seqs_t>::type pivot_t;
	
	typedef typename boost::mpl::partition<
		seqs_t,
		boost::is_same<pivot_t, boost::mpl::front<boost::mpl::_1>::type>,
		boost::mpl::back_inserter< boost::mpl::vector<> >,
		boost::mpl::back_inserter< boost::mpl::vector<> >
	>::type result_t;
	
	typedef typename detail::pop_fronts<result_t::first>::type A_t;

	typedef typename boost::mpl::minus< Level, boost::mpl::int_<1> >::type next_level_t;
	typedef typename symbols_base<result_t::second, next_level_t>::type B_t;
	
	typedef or_<
			seq< pivot_t, symbols_base<A_t> >,
			symbols_base<result_t::second>::type
	> type;
};

template< class P0 = na, class P1 = na, class P2 = na >
struct symbols :
	symbols_base< boost::mpl::vector3<P0, P1, P2>, boost::mpl::int_<5> >::type
{
};



} // namespace biscuit
