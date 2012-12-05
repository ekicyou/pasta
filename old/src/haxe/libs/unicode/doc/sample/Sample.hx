using StringTools;
using unicode.UnicodeTools;
using unicode.CombiningTools;

class Sample {
    public static function main() {
        var str = ":)ESWN";
        str = str.replace(":)", (0x263A).uCodeToString());
        str = str.replace("ESWN", [0x1F000, 0x1F001, 0x1F002, 0x1F003].uToString());
        trace("str.length: " + str.length + " str.uLength(): " + str.uLength());
        for( c in str.uIter() ) {
            trace("U+" + c.hex(4) + ": " + c.uCodeToString());
        }
        
        var str2 = "Ti"+[0x65,0x302,0x301].uToString()+"ng Vi"+[0x65,0x323,0x302].uToString()+"t"
            +" fran"+[0x63,0x327].uToString()+"ais";
        trace(str2);
        for( cs in str2.ucIter() ) {
            trace(cs.uToString());
        }
        trace(str2.ucSubstr(-8));
    }
}
