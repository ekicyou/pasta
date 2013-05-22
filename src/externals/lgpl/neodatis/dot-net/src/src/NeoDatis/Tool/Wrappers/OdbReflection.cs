namespace NeoDatis.Tool.Wrappers
{

   using System;
   using System.Reflection;

	public class OdbReflection 
	{
	
		public static int GetArrayLength(object array)
		{
			Array realArray = (Array) array; 
			return realArray.GetLength(0);
		}
		public static Object GetArrayElement(object array, int index)
		{
			Array realArray = (Array) array;
			return realArray.GetValue(index);
		}

        public static MethodInfo[] GetMethods(Type clazz)
        {
		   return clazz.GetMethods(BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic);
	     }		
	     
	    public static Type[] GetAttributeTypes(MethodInfo method){
	       ParameterInfo[] pinfo =  method.GetParameters();
	       Type[] types = new Type[pinfo.Length];
	       for(int i=0;i<pinfo.Length;i++)
			{
				types[i] = pinfo[i].ParameterType;
			}
			
		   return types;
	   }
	}
}