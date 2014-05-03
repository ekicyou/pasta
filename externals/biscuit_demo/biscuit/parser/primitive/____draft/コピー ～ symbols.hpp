#pragma once

#include <boost/mpl/filter_view.hpp>
#include <boost/mpl/transform_view.hpp>
#include <boost/mpl/vector.hpp>
#include <boost/mpl/stable_partition.hpp>
#include <boost/mpl/back_inserter.hpp>
#include <boost/mpl/placeholders.hpp>
#include <boost/mpl/minus.hpp>
#include <boost/mpl/int.hpp>
#include <boost/mpl/front.hpp>
#include <boost/mpl/pop_front.hpp>
#include <boost/mpl/not.hpp>
#include <boost/mpl/empty.hpp>
#include <boost/mpl/size.hpp>
#include <boost/mpl/eval_if.hpp>
#include <boost/mpl/equal_to.hpp>
#include <boost/mpl/assert.hpp>
#include <boost/type_traits/is_same.hpp>
#include <boost/mpl/bool.hpp>
#include <boost/mpl/plus.hpp>
#include <boost/mpl/print.hpp>
#include <boost/mpl/begin.hpp>
#include <boost/mpl/erase.hpp>
#include <boost/mpl/find_if.hpp>

#include "../seq.hpp"
#include "../or.hpp"
#include "../eps.hpp"
#include "../../debug.hpp"
#include "../nothing.hpp"

struct symbols_inner_tag { };

namespace biscuit
{

template< class Seqs, class Level >
struct symbols_base;

	namespace detail
	{
		template< class Seqs >
		struct get_pivot
		{
			typedef typename boost::mpl::front<
				typename boost::mpl::front<Seqs>::type
			>::type type;

			BOOST_MPL_ASSERT_NOT(( boost::is_same< boost::mpl::void_, type > ));
		};
		
		template< class Seqs >
		struct symbols_base;

		template< class Pivot >
		struct first_group_road_stop
		{
			typedef Pivot type;
		};

		template< class Pivot, class SymbolsBase >
		struct first_group_road_go
		{
			typedef seq< Pivot, typename SymbolsBase::type > type;
		};

		// Note: If you forget to include <boost/mpl/pop_front.hpp>,
		//       pop_fronts fail...silently...
		template< class Seqs >
		struct pop_fronts
		{
			typedef typename boost::mpl::transform_view<
				Seqs,
				boost::mpl::pop_front<boost::mpl::_1>
			>::type type;

			/*
			BOOST_MPL_ASSERT((
				boost::mpl::equal_to<
					boost::mpl::plus< boost::mpl::size< typename boost::mpl::front<type>::type >, boost::mpl::int_<1> >,
					boost::mpl::size< typename boost::mpl::front<Seqs>::type >
				>
			));
			*/
		};

		template< class Pivot, class SymbolsBase >
		struct first_group_road
		{
			typedef seq< Pivot, typename SymbolsBase::type > type;
		};

		template< class Seqs >
		struct has_null_seq
		{
			typedef typename boost::mpl::find_if<
				Seqs,
				boost::mpl::empty<boost::mpl::_1>
			>::type iter_t;

			typedef boost::is_same<
				iter_t,
				typename boost::mpl::end<Seqs>::type
			> type;
		};

		template< class Seqs >
		struct symbols_base
		{
			typedef typename get_pivot<Seqs>::type pivot_t;

			typedef typename boost::mpl::filter_view<
				Seqs,
				boost::mpl::not_< boost::mpl::empty<boost::mpl::_1> >
			>::type seqs_t;

			typedef typename boost::mpl::stable_partition<
				seqs_t,
				boost::mpl::equal_to< pivot_t, boost::mpl::front<boost::mpl::_1> >,
				boost::mpl::back_inserter< boost::mpl::vector<> >,
				boost::mpl::back_inserter< boost::mpl::vector<> >
			>::type result_t;

			typedef typename result_t::first first_group_t;
			typedef typename result_t::second second_group_t;
      typedef typename pop_fronts<first_group_t>::type first_group_rest_t;

			typedef typename boost::mpl::eval_if<
				typename has_null_seq<Seqs>::type,
				boost::mpl::identity<eps>,
				boost::mpl::identity<nothing>
			>::type point_t;

			typedef typename boost::mpl::eval_if<
				boost::mpl::empty<first_group_rest_t>,
				boost::mpl::identity<nothing>,
				symbols_base<first_group_rest_t>
			>::type first_group_road_t;

/*
					first_group_road_go< pivot_t, symbols_base<first_group_rest_t> >
				//boost::mpl::empty< typename boost::mpl::front<first_group_rest_t>::type >, // last one
				//first_group_road_stop< pivot_t >,
				//first_group_road_go< pivot_t, symbols_base<first_group_rest_t> >
			>::type first_group_road_t;
*/
			// second_group_road_t
			typedef typename boost::mpl::eval_if<
				boost::mpl::empty<second_group_t>,
				boost::mpl::identity<nothing>,
				symbols_base<second_group_t>
			>::type second_group_road_t;

			typedef or_<
				point_t,
				first_group_road_t,
				second_group_road_t
			> type_;

			typedef debugger< symbols_inner_tag, type_, boost::mpl::false_ > type;
		};
	} // namespace detail

struct symbols_tag { };

template< class P0, class P1, class P2  >
struct symbols : debugger<symbols_tag,
	typename detail::symbols_base< boost::mpl::vector<P0, P1, P2> >::type,
	boost::mpl::false_
>
{
};

} // namespace biscuit
