package unicode;

class Unicode {
    public static inline var version            :String = "5.2.0";
    public static inline var maxUnicodeChar     :Int    = 0x10FFFF;
    public static inline var replacementChar    :Int    = 0xFFFD;
    public static inline var minHighSurrogates  :Int    = 0xD800;
    public static inline var maxHighSurrogates  :Int    = 0xDBFF;
    public static inline var minLowSurrogates   :Int    = 0xDC00;
    public static inline var maxLowSurrogates   :Int    = 0xDFFF;

    public static inline function decodeSurrogate(c:Int, d:Int) : Int {
        return (c - 0xD7C0 << 10) | (d & 0x3FF);
    }
    public static inline function encodeHighSurrogate(c:Int) {
        return (c >> 10) + 0xD7C0;
    }
    public static inline function encodeLowSurrogate(c:Int) {
        return (c & 0x3FF) | 0xDC00;
    }
#if cpp
    public static var stringIsUtf32(_strIsUtf32, null) :Bool;
    static var _str_is_utf32 :Null<Bool>;
    static function _strIsUtf32() {
        if(_str_is_utf32 != null) return _str_is_utf32;
        if("𠀋".length == 1) {
            /* U+2000B is encoded to 0x0002000B */
            _str_is_utf32 = true;
        } else {
            /* U+2000B is encoded to a surrogate pair 0xD840 0xDC0B */
            _str_is_utf32 = false;
        }
        return _str_is_utf32;
    }
#else
    public static inline var stringIsUtf32 :Bool = false;
#end
}
