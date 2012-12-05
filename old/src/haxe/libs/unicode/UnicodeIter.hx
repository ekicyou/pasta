package unicode;

class UnicodeIter {
    var str : String;
    var pos : Int;
    public function new(s : String) {
        str = UnicodeTools.uValidate(s);
        pos = 0;
    }
    public function hasNext() {
        return (pos < str.length);
    }
#if (neko || php)
    public function next() {
        var c = str.charCodeAt(pos);
        if(c < 0x80) {
            pos++;
            return c;
        } else if(c < 0xE0) {
            c = ((c & 0x3F) << 6) | (str.charCodeAt(pos+1) & 0x7F);
            pos+=2;
            return c;
        } else if(c < 0xF0) {
            c = ((c & 0x1F) << 12) | ((str.charCodeAt(pos+1) & 0x7F) << 6) | (str.charCodeAt(pos+2) & 0x7F);
            pos += 3;
            return c;
        } else {
            c = ((c & 0x0F) << 18) | ((str.charCodeAt(pos+1) & 0x7F) << 12) | ((str.charCodeAt(pos+2) & 0x7F) << 6) | (str.charCodeAt(pos+3) & 0x7F);
            pos += 4;
            return c;
        }
    }
#else
    public function next() {
        var c = str.charCodeAt(pos);
        pos++;
        if( !Unicode.stringIsUtf32 ) {
            if( UnicodeTools.uIsHighSurrogate(c) ) {
                var d = str.charCodeAt(pos);
                pos++;
                c = Unicode.decodeSurrogate(c,d);
            }
        }
        return c;
    }
#end
}
