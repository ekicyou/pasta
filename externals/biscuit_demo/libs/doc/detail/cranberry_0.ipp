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
