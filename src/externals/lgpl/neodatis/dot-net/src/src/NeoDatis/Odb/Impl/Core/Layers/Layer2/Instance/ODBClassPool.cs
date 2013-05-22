using System;
using System.Reflection;
using NeoDatis.Odb.Core;
namespace NeoDatis.Odb.Impl.Core.Layers.Layer2.Instance
{
	/// <summary>A simple class pool, to optimize instance creation</summary>
	/// <author>osmadja</author>
	public class ODBClassPool : NeoDatis.Odb.Core.Layers.Layer2.Instance.IClassPool
	{
		private static System.Collections.Generic.IDictionary<string, System.Type> classMap
			 = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, System.Type>();

		private static System.Collections.Generic.IDictionary<string, System.Reflection.ConstructorInfo
			> construtorsMap = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, System.Reflection.ConstructorInfo
			>();

		public virtual void Reset()
		{
			classMap.Clear();
			construtorsMap.Clear();
		}

		public virtual System.Type GetClass(string className)
		{
			lock (this)
			{
                System.Type clazz = null;
                classMap.TryGetValue(className, out clazz);
				if (clazz == null)
				{
					try
					{
						
                        clazz = System.Type.GetType(className);
					}
					catch (System.Exception e)
					{
						throw new ODBRuntimeException(NeoDatisError.ClassPoolCreateClass.AddParameter(className), e);
					}
					classMap[className] = clazz;
				}
				return clazz;
			}
		}

		public virtual ConstructorInfo GetConstrutor(string className)
		{
            ConstructorInfo ci = null;
            construtorsMap.TryGetValue(className, out ci);
            return ci;
		}

		public virtual void AddConstrutor(string className, System.Reflection.ConstructorInfo
			 constructor)
		{
			construtorsMap[className] = constructor;
		}
	}
}
