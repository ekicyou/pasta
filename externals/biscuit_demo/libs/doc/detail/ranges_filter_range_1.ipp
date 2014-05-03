BOOST_CHECK((
	match< str<'x','y','z'> >(
		make_filter_range< alpha >(
			make_filter_range< not_<space> >(
				make_filter_range< not_<digit> >("x & 4 y . 125 %  z")
			)
		)
	)
));
