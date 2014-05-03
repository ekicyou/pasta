struct calculator : grammar< calculator, std::string >
{
	void do_int(iterator_type str, iterator_type end)
	{
		std::string s(str, end);
		std::cout << "PUSH(" << s << ')' << std::endl;
	}

	void do_add(iterator_type, iterator_type)		{ std::cout << "ADD\n"; }
	void do_subt(iterator_type, iterator_type)	{ std::cout << "SUBTRACT\n"; }
	void do_mult(iterator_type, iterator_type)	{ std::cout << "MULTIPLY\n"; }
	void do_div(iterator_type, iterator_type)		{ std::cout << "DIVIDE\n"; }
	void do_neg(iterator_type, iterator_type)		{ std::cout << "NEGATE\n"; }

	struct expression;
	struct term;
	struct factor;

	struct expression :
		seq<
			term,
			star<
				or_<
					actor_< seq< str<'+'>, term >, &calculator::do_add >,
					actor_< seq< str<'-'>, term >, &calculator::do_subt >
				>
			>
		>
	{ };
	
	struct term :
		seq<
			factor,
			star<
				or_<
					actor_< seq< str<'*'>, factor >, &calculator::do_mult >,
					actor_< seq< str<'/'>, factor >, &calculator::do_div >
				>
			>
		>
	{ };

	struct factor :
		or_<
			actor_< plus<digit>, &calculator::do_int >,
			seq< str<'('>, expression, str<')'> >,
			actor_< seq< str<'-'>, factor >, &calculator::do_neg >,
			seq< str<'+'>, factor >
		>
	{ };

	struct start : expression { };
};
