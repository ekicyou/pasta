struct expression ;
struct group      : seq< str<'('>, expression, str<')'> > { };
struct factor     : or_< integer, group > { };
struct term       : seq< factor, star< or_< seq< str<'*'>, factor >, seq< str<'/'>, factor > > > > { };
struct expression : seq< term, star< or_< seq< str<'+'>, term >, seq< str<'-'>, term > > > > { };
