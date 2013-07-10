package pasta.parse;

/**
 * ...
 * @author dot.station
 */

class Parser 
{
	inline static var S1 = " ";
	inline static var S2 = "　";
	inline static var S3 = "\t";
	public inline static var Space = '(${S1}|${S2}|${S3})';
	
	inline static var M1 = "\\";
	inline static var M2 = "￥";
	inline static var M3 = "@";
	inline static var M4 = "＠";
	private inline static var Mark = "(${M1}|${M2}|${M3}|${M4})";

	public function new() 
	{
		
	}
	
}