typedef seq<
	str<'/','*'>,
	star_until< any, str<'*','/'> >
> c_comment;
