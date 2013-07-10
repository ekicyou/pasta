package ;

/**
 * ...
 * @author dot.station
 */

class PastaTest 
{
	
	static function main() 
	{
        var r = new haxe.unit.TestRunner();
		
        // your can add others TestCase here
        r.add(new pasta.parse.ParseTest());

        // finally, run the tests
        r.run();
	}
	
}