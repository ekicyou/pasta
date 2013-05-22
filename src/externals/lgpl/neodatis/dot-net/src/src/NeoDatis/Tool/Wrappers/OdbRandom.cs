namespace NeoDatis.Tool.Wrappers{

using System;

public class OdbRandom {
	protected static Random random = new Random();
	public static int GetRandomInteger(){
		return random.Next();
	}
	public static double GetRandomDouble(){
		return random.NextDouble();
	}
	

}
}