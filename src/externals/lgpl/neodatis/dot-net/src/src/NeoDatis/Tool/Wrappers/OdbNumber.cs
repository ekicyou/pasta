namespace NeoDatis.Tool.Wrappers{

using System;

public class OdbNumber {
	public static Decimal NewBigInteger(long l){
		return new Decimal(l);
	}
}
}