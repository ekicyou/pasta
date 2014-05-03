#line 11 "easy_seq.ipp"
namespace biscuit
{
#line 15 "easy_seq.ipp"
template< class P0 = eps , class P1 = eps , class P2 = eps , class P3 = eps , class P4 = eps , class P5 = eps , class P6 = eps , class P7 = eps , class P8 = eps , class P9 = eps , class P10 = eps , class P11 = eps , class P12 = eps , class P13 = eps , class P14 = eps , class P15 = eps , class P16 = eps , class P17 = eps , class P18 = eps , class P19 = eps , class P20 = eps , class P21 = eps , class P22 = eps , class P23 = eps , class P24 = eps , class P25 = eps , class P26 = eps , class P27 = eps , class P28 = eps , class P29 = eps , class P30 = eps , class P31 = eps , class P32 = eps , class P33 = eps , class P34 = eps , class P35 = eps , class P36 = eps , class P37 = eps , class P38 = eps , class P39 = eps , class P40 = eps , class P41 = eps , class P42 = eps , class P43 = eps , class P44 = eps , class P45 = eps , class P46 = eps , class P47 = eps , class P48 = eps , class P49 = eps >
struct seq
{
template< class State, class UserState >
static bool parse(State& s, UserState& us)
{
typename state_iterator<State> ::type cur0 = s.cur;

if (
P0::parse(s, us) && P1::parse(s, us) && P2::parse(s, us) && P3::parse(s, us) && P4::parse(s, us) && P5::parse(s, us) && P6::parse(s, us) && P7::parse(s, us) && P8::parse(s, us) && P9::parse(s, us) && P10::parse(s, us) && P11::parse(s, us) && P12::parse(s, us) && P13::parse(s, us) && P14::parse(s, us) && P15::parse(s, us) && P16::parse(s, us) && P17::parse(s, us) && P18::parse(s, us) && P19::parse(s, us) && P20::parse(s, us) && P21::parse(s, us) && P22::parse(s, us) && P23::parse(s, us) && P24::parse(s, us) && P25::parse(s, us) && P26::parse(s, us) && P27::parse(s, us) && P28::parse(s, us) && P29::parse(s, us) && P30::parse(s, us) && P31::parse(s, us) && P32::parse(s, us) && P33::parse(s, us) && P34::parse(s, us) && P35::parse(s, us) && P36::parse(s, us) && P37::parse(s, us) && P38::parse(s, us) && P39::parse(s, us) && P40::parse(s, us) && P41::parse(s, us) && P42::parse(s, us) && P43::parse(s, us) && P44::parse(s, us) && P45::parse(s, us) && P46::parse(s, us) && P47::parse(s, us) && P48::parse(s, us) && P49::parse(s, us) &&
true
)
{
return true;
}
else
{
s.cur = cur0;
return false;
}
}
};
