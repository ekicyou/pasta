#include "stdafx.h"

#include <iostream>
#include <vector>
#include <algorithm>
#include <functional>

#include <boost/mpl/apply.hpp>
#include <boost/mpl/identity.hpp>
#include <boost/mpl/eval_if.hpp>
#include <boost/mpl/void.hpp>

#include <boost/function_types/function_type_result.hpp>
#include <boost/type_traits/remove_reference.hpp>

#pragma warning( push, 3 )
#include <boost/spirit/fusion/sequence/tuple_element.hpp>
#include <boost/spirit/fusion/sequence/tuple.hpp>
#include <boost/spirit/fusion/sequence/generate.hpp>
#include <boost/spirit/fusion/sequence/make_tuple.hpp>
#include <boost/spirit/fusion/sequence/joint_view.hpp>
#include <boost/spirit/fusion/sequence/get.hpp>
#pragma warning( pop )

namespace cranberry
{

template< int n >
struct place_holder_base
{
	struct result
	{
		template< class Args >
		struct apply
		{
			typedef typename boost::fusion::meta::generate<Args>::type tuple_t;
			typedef typename boost::fusion::tuple_element<n, tuple_t>::type type_;
			typedef typename boost::mpl::eval_if< boost::is_reference_wrapper<type_>,
				boost::unwrap_reference<type_>,
				boost::mpl::identity<type_>
			>::type type;
		};

		typedef result type;
	};

	template< class Args >
	static
	typename boost::mpl::apply<result, Args>::type
	operate(Args& args)
	{
		// Note: get<N> from fusion sequence is in TODO list.
		return boost::fusion::get<n>(boost::fusion::generate(args));
	}

	typedef place_holder_base type;
};

struct _1 : place_holder_base<0>::type { };
struct _2 : place_holder_base<1>::type { };
struct _3 : place_holder_base<2>::type { };
struct _4 : place_holder_base<3>::type { };
struct _5 : place_holder_base<4>::type { };

typedef boost::mpl::void_ na;

// primary decl.
template<
	class OpL, class OpR, class Result = na
>
struct plus;

// spec.
template<
	class OpL, class OpR
>
struct plus<OpL, OpR>
{
	struct result
	{
		template< class Args >
		struct apply
		{
			typedef typename boost::mpl::apply<typename OpL::result, Args>::type type_;
			// Note: Yes, not a reference!
			typedef typename boost::remove_reference<type_>::type type;
		};

		typedef result type;
	};

	template< class Args >
	static
	typename boost::mpl::apply<result, Args>::type
	operate(Args& args)
	{
		return OpL::operate(args) + OpR::operate(args);
	}
};

template<
	class OpL, class OpR, class Result
>
struct plus
{
	struct result
	{
		template< class Args >
		struct apply
		{
			typedef Result type;
		};

		typedef result type;
	};

	template< class Args >
	static
	typename boost::mpl::apply<result, Args>::type
	operate(Args& args)
	{
		return OpL::operate(args) + OpR::operate(args);
	}
};

// int
template< int x >
struct int_
{
	struct result
	{
		template< class Args >
		struct apply
		{
			typedef int type;
		};

		typedef result type;
	};

	template< class Args >
	static
	typename boost::mpl::apply<result, Args>::type
	operate(Args&)
	{
		return x;
	}
};

// std_cout
template< class Op >
struct std_cout
{
	struct result
	{
		template< class Args >
		struct apply
		{
			typedef void type;
		};

		typedef result type;
	};

	template< class Args >
	static
	typename boost::mpl::apply<result, Args>::type
	operate(Args& args)
	{
		std::cout << Op::operate(args) << std::endl;
	}
};

// bind
typedef boost::mpl::void_ na;

// primary decl.
template< class FtorOp, class Op0 = na, class Op1 = na >
struct bind;

template< class FtorOp >
struct bind_base
{
	struct result
	{
		template< class Args >
		struct apply
		{
			typedef typename boost::mpl::apply<typename FtorOp::result, Args>::type ftor_t_;
			typedef typename boost::mpl::eval_if< boost::is_reference_wrapper<ftor_t_>,
				boost::unwrap_reference<ftor_t_>,
				boost::mpl::identity<ftor_t_>
			>::type ftor_t;
			typedef typename boost::function_type_result<ftor_t>::type type;
		};

		typedef result type;
	};

	typedef bind_base type;
};

// spec.
template< class FtorOp >
struct bind<FtorOp> : 
	bind_base<FtorOp>::type
{
	typedef typename bind_base<FtorOp>::type super_t;
	typedef typename super_t::result result;

	template< class Args >
	static
	typename boost::mpl::apply<result, Args>::type
	operate(Args& args)
	{
		return FtorOp::operate(args)(
		);
	}
};

template< class FtorOp, class Op0 >
struct bind<FtorOp, Op0> :
	bind_base<FtorOp>::type
{
	typedef typename bind_base<FtorOp>::type super_t;
	typedef typename super_t::result result;

	template< class Args >
	static
	typename boost::mpl::apply<result, Args>::type
	operate(Args& args)
	{
		return
			FtorOp::operate(args)(
				Op0::operate(args)
			);
	}
};

template< class FtorOp, class Op0, class Op1 >
struct bind :
	bind_base<FtorOp>::type
{
	typedef typename bind_base<FtorOp>::type super_t;
	typedef typename super_t::result result;

	template< class Args >
	static
	typename boost::mpl::apply<result, Args>::type
	operate(Args& args)
	{
		return
			 FtorOp::operate(args)(
				Op0::operate(args),
				Op1::operate(args)
			);
	}
};

// apply
template< class Op, class States >
struct apply_type
{
	// 0
	typename boost::mpl::apply<
		typename Op::result,
		boost::fusion::joint_view<
			States,
			boost::fusion::tuple<>
		>
	>::type
	operator()()
	{
		typedef boost::fusion::tuple<> fun_args_t;
		fun_args_t fun_args = boost::fusion::make_tuple();
		boost::fusion::joint_view<States, fun_args_t> args(states, fun_args);
		return Op::operate(args);
	}

	// 1
	template< class FunArg0 >
	typename boost::mpl::apply<
		typename Op::result,
		boost::fusion::joint_view<
			States,
			boost::fusion::tuple<FunArg0>
		>
	>::type
	operator()(FunArg0 a0)
	{
		typedef boost::fusion::tuple<FunArg0> fun_args_t;
		fun_args_t fun_args = boost::fusion::make_tuple(a0);
		boost::fusion::joint_view<States, fun_args_t> args(states, fun_args);
		return Op::operate(args);
	}

	// 2
	template< class FunArg0, class FunArg1 >
	typename boost::mpl::apply<
		typename Op::result,
		boost::fusion::joint_view<
			States,
			boost::fusion::tuple<FunArg0, FunArg1>
		>
	>::type
	operator()(FunArg0 a0, FunArg1 a1)
	{
		typedef boost::fusion::tuple<FunArg0, FunArg1> fun_args_t;
		fun_args_t fun_args = boost::fusion::make_tuple(a0, a1);
		boost::fusion::joint_view<States, fun_args_t> args(states, fun_args);
		return Op::operate(args);
	}

	// 3
	template< class FunArg0, class FunArg1, class FunArg2 >
	typename boost::mpl::apply<
		typename Op::result,
		boost::fusion::joint_view<
			States,
			boost::fusion::tuple<FunArg0, FunArg1, FunArg2>
		>
	>::type
	operator()(FunArg0 a0, FunArg1 a1, FunArg2 a2)
	{
		typedef boost::fusion::tuple<FunArg0, FunArg1, FunArg2> fun_args_t;
		fun_args_t fun_args = boost::fusion::make_tuple(a0, a1, a2);
		boost::fusion::joint_view<States, fun_args_t> args(states, fun_args);
		return Op::operate(args);
	}

	// ctor
	apply_type(States const& s) : states(s) { };
	apply_type(apply_type const& other) : states(other.states) { }
	apply_type& operator=(apply_type const& other) { states = other.states; return *this; }
	States states;
};

// 0
template< class Op >
apply_type< Op, boost::fusion::tuple<> >
apply()
{
	boost::fusion::tuple<> states;
	return apply_type< Op, boost::fusion::tuple<> >(states);
}

// 1
template< class Op, class St0 >
apply_type< Op, boost::fusion::tuple<St0> >
apply(St0 s0)
{
	typedef boost::fusion::tuple<St0> states_t;
	states_t states(s0);
	return apply_type< Op, states_t >(states);
}

// 2
template< class Op, class St0, class St1 >
apply_type< Op, boost::fusion::tuple<St0, St1> >
apply(St0 s0, St1 s1)
{
	typedef boost::fusion::tuple<St0, St1> states_t;
	states_t states(s0, s1);
	return apply_type< Op, states_t >(states);
}

} // namespace cranberry

int free_plus(int x, int y)
{
	return x+y;
}

int main()
{
	using namespace cranberry;

	int x = 7;
	int a = apply< plus< _1, _2 > >(x)(8);
	int b = apply< plus< _1, _2 > >()(7, 8);
	int c = apply< plus< _1, _2 > >(7, 8)();
	int d = apply< plus< _1, int_<8> > >(7)();
	int e = apply< bind< _1, _2, _3 > >(std::plus<int>())(7, 8);
	int f = apply< bind< _1, _2, _3 > >(&free_plus)(7, 8);

	int ar[5] = {1,2,3,4,5};
	std::transform(ar, ar+5, ar, apply< plus< _1, _2 >, int const& >(x));
	std::transform(ar, ar+5, ar, apply< bind< _1, _2, _3 > >(std::plus<int>(), x));
	std::transform(ar, ar+5, ar, apply< bind< _1, _2, int_<7> > >(&free_plus));
	std::for_each(ar, ar+5, apply< std_cout< _1 > >());

	a; b; c; d; e; f;
	return 0;
}

