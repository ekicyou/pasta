using System;

namespace NeoDatis.Tool.Wrappers{

	// @TODO
	public class ClassLoader {
		public Type LoadClass(String className){
			return Type.GetType(className);
		}
		public static object GetCurrent()
		{
		   return null;
		}
	}
}