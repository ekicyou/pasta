package unicode;

/**
    The UnicodeTools class contains some functionalities for Unicode [String]
    manipulation related to combining characters.
    This class needs the CombiningClassData class and you should notice
    that the executable file gets large.
**/
class CombiningTools {
    public static inline function ucIter(str : String) : Iterator<Array<Int>> {
        return new CombiningIter(str);
    }

    /**
        Returns the number of Combining Character Sequences in the String.
    **/
    public static function ucLength(str : String) : Int {
        var count = 0;
        for( c in UnicodeTools.uIter(str) ) {
            if( CombiningClassData.get(c) == null ) {
                count++;
            }
        }
        return count;
    }
    public static function ucOrdering(str : String) : String {
        // Insertion sort
        var ret = new StringBuf();
        for(seq in new CombiningIter(str)) {
            for(i in 1 ... seq.length) {
                var tmp = seq[i];
                var tmpccc = CombiningClassData.get(seq[i]);
                var j = i;
                while((j > 0) && CombiningClassData.get(seq[j-1]) > tmpccc) {
                    seq[j] = seq[j-1];
                    j--;
                }
                seq[j] = tmp;
            }
            ret.add(UnicodeTools.uToString(seq));
        }
        return ret.toString();
    }
    public static function ucSubstr(str : String, pos : Int, ?len : Int) {
        if( pos < 0 ) {
            pos = pos + ucLength(str);
        }
        var uciter = new CombiningIter(str);
        var i = 0;
        while( (i < pos) || !uciter.hasNext() ) {
            uciter.next();
            i++;
        }
        var ret = new StringBuf();
        for( s in uciter ) {
            ret.add(UnicodeTools.uToString(s));
        }
        return ret.toString();
    }
}
