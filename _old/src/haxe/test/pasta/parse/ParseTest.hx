package pasta.parse;

/**
 * ...
 * @author dot.station
 */

class ParseTest extends haxe.unit.TestCase
{

    public function test1() {
        assertEquals( "A", "A" );
        var text = pasta.parse.Parser.Space;
        assertEquals( text, "( |　|\t)" );
    }	
	
}