package unicode;

class CombiningIter {
    var uiter : UnicodeIter;
    var buffer : Int;
    public function new(s : String) {
        uiter = new UnicodeIter(s);
        if( uiter.hasNext() ) {
            buffer = uiter.next();
        } else {
            buffer = null;
        }
    }
    public function hasNext() {
        return (buffer != null);
    }
    public function next() {
        var ret = [buffer];
        buffer = null;
        while( uiter.hasNext() ) {
            buffer = uiter.next();
            var c = CombiningClassData.get(buffer);
            if((c == null) || (c == 0)) {
                break;
            }
            ret.push(buffer);
            buffer = null;
        }
        return ret;
    }
}
