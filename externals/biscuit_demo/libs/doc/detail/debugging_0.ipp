struct start_tag { };
struct integer_tag { };
struct factor_tag { };
struct term_tag { };
struct expression_tag { };

template< class ForwardRange >
struct calculator_debug :
	grammar< calculator_debug, ForwardRange >, private boost::noncopyable
{
	calculator_debug(std::stack<long>& eval_) : eval(eval_) { }

	void push_int(iterator_type first, iterator_type last)
	{
		std::string s(first, last);
		long n = boost::lexical_cast<long>(s); // std::strtol(str, 0, 10);
		eval.push(n);
		std::cout << "push\t" << long(n) << std::endl;
	}

	// ...

	struct integer;
	struct factor;
	struct term;
	struct expression;

	// struct start : debugger<start,
	//  also ok, but long...
	struct start : debugger<start_tag, 
		expression
	>
	{ };

	struct integer : debugger<integer_tag,
		actor_< plus<digit>, &calculator_debug::push_int >
	>
	{ };
	
	// ...
