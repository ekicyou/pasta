using System;

namespace NeoDatis.Tool.Wrappers{

/**
 * @author olivier
 *
 */
	public class OdbClassUtil {
		public static bool IsEnum(System.Type type){
			return type.IsEnum;
		}
		
		public static string GetClassName(string fullClassName)
		{
			int index = fullClassName.LastIndexOf('.');
			if (index == -1)
			{
				// no dot -> must be a primitive type
				return fullClassName;
			}
			// get class name
			string className = OdbString.Substring(fullClassName, index + 1, fullClassName.Length);
			return className;
		}

		public static string GetPackageName(string fullClassName)
		{
			int index = fullClassName.LastIndexOf('.');
			if (index == -1)
			{
				// no dot -> must be a primitive type
				return string.Empty;
			}
			// get package class name
			return OdbString.Substring(fullClassName, 0, index);
		}
		
		public static System.String GetFullName(string assemblyQualifiedName)
        {
            
           string [] ss=assemblyQualifiedName.Split(',');
           if (ss.Length != 2)
           {
               throw new Exception(assemblyQualifiedName + " should have a , with assembly name");
           }
           return ss[0] + ","+ss[1];
        }
        public static System.String GetFullName(Type type)
        {
            if (type == null)
            {
                Console.WriteLine(" type is null in GettFullName :-(");
            }
            //Console.WriteLine(" getting full name of " + type.Name);
            int index = type.Assembly.FullName.IndexOf(',');
            return type.FullName + ","+type.Assembly.FullName.Substring(0,index);
        }
	}
}