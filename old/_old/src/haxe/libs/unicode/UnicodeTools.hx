package unicode;

/**
    The UnicodeTools class contains some functionalities for Unicode [String]
    manipulation.
**/
class UnicodeTools {
    public static function uIsValidChar(code : Int) : Bool {
        return (0 <= code && code <= Unicode.maxUnicodeChar)
            && !uIsHighSurrogate(code)
            && !uIsLowSurrogate(code)
            && !(0xFDD0 <= code && code <= 0xFDEF)
            && !(code & 0xFFFE == 0xFFFE);
    }
    public static inline function uIsHighSurrogate(code : Int) : Bool {
        return (Unicode.minHighSurrogates <= code && code <= Unicode.maxHighSurrogates);
    }
    public static inline function uIsLowSurrogate(code : Int) : Bool {
        return (Unicode.minLowSurrogates <= code && code <= Unicode.maxLowSurrogates);
    }

    /**
        Returns the number of Unicode characters in the String.
    **/
    public static inline function uLength(s : String) : Int {
#if neko
        return neko.Utf8.length(s);
#elseif php
        return php.Utf8.length(s);
#else
        if(Unicode.stringIsUtf32) {
            return s.length;
        } else {
            return _utf16length(s);
        }
#end
    }

    /**
        Returns the Unicode Code Point at the given position.
        Returns [null] if outside of String bounds.
    **/
    public static function uCodeAt(s : String, index : Int) : Null<Int> {
        if( (index < 0) || (uLength(s) <= index) ) {
            return null;
        }
#if neko
        return neko.Utf8.charCodeAt(s, index);
#elseif php
        return php.Utf8.charCodeAt(s, index);
#else
        if(Unicode.stringIsUtf32) {
            return s.charCodeAt(index);
        } else {
            return _utf16charCodeAt(s, index);
        }
#end
    }

    /**
        Returns the Unicode character at the given position.
        Returns the empty String if outside of String bounds.
    **/
    public static function uCharAt(s : String, index : Int) : String {
        var code = uCodeAt(s, index);
        if( code == null ) {
            return "";
        }
        return uCodeToString(code);
    }
    public static inline function uValidate(s : String) : String {
        if(!uIsValid(s)) {
            throw UnicodeException.InvalidUtfStream;
        }
        return s;
    }
    public static function uIsValid(s : String) : Bool {
#if neko
        return neko.Utf8.validate(s);
#elseif php
        return php.Utf8.validate(s);
#else
        if(Unicode.stringIsUtf32) {
            return _utf32validate(s);
        } else {
            return _utf16validate(s);
        }
#end
    }
    public static function uSubstr(s : String, pos : Int, ?len : Null<Int>) {
#if neko
        if(pos < 0) pos = uLength(s) + pos;
        if(len == null) len = uLength(s) - pos;
        return neko.Utf8.sub(s, pos, len);
#elseif php
        if(pos < 0) pos = uLength(s) + pos;
        if(len == null) len = uLength(s) - pos;
        return php.Utf8.sub(s, pos, len);
#else
        if(Unicode.stringIsUtf32) {
            return s.substr(pos, len);
        } else {
            return _utf16substr(s, pos, len);
        }
#end
    }
    public static function uIndexOf(s : String, value : String, ?startIndex : Int = 0) : Int {
        return _posToUpos(s, s.indexOf(value, _uposToPos(s, startIndex)));
    }
    public static function uLastIndexOf(s : String, value : String, ?startIndex : Null<Int>) : Int {
        var _startIndex = if(startIndex == null) s.length else _uposToPos(s, startIndex);
        return _posToUpos(s, s.lastIndexOf(value, _startIndex));
    }
    public static inline function uIter(s : String) : Iterator<Int> {
        return new UnicodeIter(s);
    }

    public static function uCodeToString(code : Int) : String {
#if neko
        var b = new neko.Utf8();
        b.addChar(code);
        return b.toString();
#elseif php
        return php.Utf8.uchr(code);
#else
        if( !uIsValidChar(code) ) {
            return null;
        }
        if(Unicode.stringIsUtf32 || code <= 0xFFFF) {
            return String.fromCharCode(code);
        } else {
            return String.fromCharCode(Unicode.encodeHighSurrogate(code))
                + String.fromCharCode(Unicode.encodeLowSurrogate(code));
        }
#end
    }
    public static function uToString(code : Iterable<Int>) : String {
        var b = new StringBuf();
        for(c in code) {
            var s = uCodeToString(c);
            if( s == null ) s = uCodeToString(Unicode.replacementChar);
            b.add(s);
        }
        return b.toString();
    }
    
    static inline function _posToUpos(s : String, pos : Int) : Int {
#if neko
        return neko.Utf8.length(s.substr(0, pos));
#elseif php
        return php.Utf8.length(s.substr(0, pos));
#else
        if(Unicode.stringIsUtf32) {
            return pos;
        } else {
            return _utf16length(s.substr(0, pos));
        }
#end
    }
    static inline function _uposToPos(s : String, upos : Int) : Int {
#if neko
        return neko.Utf8.sub(s, 0, upos).length;
#elseif php
        return php.Utf8.sub(s, 0, upos).length;
#else
        if(Unicode.stringIsUtf32) {
            return upos;
        } else {
            return _utf16substr(s, 0, upos).length;
        }
#end
    }

#if (!neko && !php)
    static function _utf16length(s : String) : Int {
        var len = s.length;
        var count = 0;
        var i = 0;
        while( i < len ) {
            count++;
            var c = s.charCodeAt(i);
            if( uIsHighSurrogate(c) ) {
                i++;
            }
            i++;
        }
        return count;
    }
    static function _utf16charCodeAt(s : String, index : Int) : Null<Int> {
        var surrogate_count = 0;
        var c : Null<Int> = null;
        for(i in 0 ... index + 1) {
            c = s.charCodeAt(i + surrogate_count);
            if( uIsHighSurrogate(c) ) {
                var d = s.charCodeAt(i + surrogate_count + 1);
                if(d == null) {
                    c = null;
                } else {
                    c = Unicode.decodeSurrogate(c,d);
                }
                surrogate_count++;
            }
        }
        return c;
    }
    static function _utf32validate(s) : Bool {
        for(i in 0 ... s.length) {
            var c = s.charCodeAt(i);
            if( !uIsValidChar(c) ) {
                return false;
            }
        }
        return true;
    }
    static function _utf16validate(s) : Bool {
        var i = 0;
        var len = s.length;
        while(i < len) {
            var c = s.charCodeAt(i++);
            if( uIsValidChar(c) ) {
                continue;
            }
            if( uIsHighSurrogate(c) ) {
                var d = s.charCodeAt(i++);
                if( (d != null)
                    && uIsLowSurrogate(d)
                    && ( Unicode.decodeSurrogate(c,d) & 0xFFFE != 0xFFFE) ) {
                    continue;
                }
            }
            return false;
        }
        return true;
    }
    static function _utf16substr(s, pos, len : Null<Int>) {
        if(pos < 0) pos = uLength(s) + pos;
        var surrogate_count = 0;
        var i = 0;
        while(i < pos) {
            var c = s.charCodeAt(i + surrogate_count);
            if(c == null) throw "_utf16substr:pos";
            if( uIsHighSurrogate(c) ) {
                surrogate_count++;
            }
            i++;
        }
        var _pos = i + surrogate_count;
        if(len == null) {
            return s.substr(_pos);
        }
        while(i < pos + len) {
            var c = s.charCodeAt(i + surrogate_count);
            if(c == null) throw "_utf16substr:len";
            if( uIsHighSurrogate(c) ) {
                surrogate_count++;
            }
            i++;
        }
        var _len = i + surrogate_count - _pos;
        return s.substr(_pos, _len);
    }
#end
}
