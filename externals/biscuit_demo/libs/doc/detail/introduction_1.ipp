struct element; // forward declaration

struct content :
	seq<
		opt<CharData>,
		star<
			seq<
				or_<
					element, // the magical recursion!
					Reference,
					CDSect,
					PI,
					Comment
				>,
				opt<CharData>
			>
		>
	>
{ };

struct element :
	or_<
		EmptyElemTag, 
		seq<STag, content, ETag>
	>
{ };
