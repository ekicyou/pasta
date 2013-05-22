using NeoDatis.Odb.Core.Query.NQ;
using NeoDatis.Tool.Wrappers;
using NeoDatis.Odb.Core;
using System.Reflection;
using NeoDatis.Odb.Core.Query;
using System.Collections.Generic;
using NeoDatis.Tool.Wrappers.Map;
namespace NeoDatis.Odb.Impl.Core.Query.NQ
{
	public class NativeQueryManager
	{
		private static string MatchMethodName = "Match";

		private static IDictionary<IQuery, MethodInfo> methodsCache = new OdbHashMap<IQuery, MethodInfo>();

		public static string GetClass(NativeQuery query)
		{
			return OdbClassUtil.GetFullName(query.GetObjectType());
		}

		public static string GetFullClassName(SimpleNativeQuery	 query)
		{
			System.Type clazz = query.GetType();
			System.Reflection.MethodInfo[] methods = OdbReflection.GetMethods(clazz);
			for (int i = 0; i < methods.Length; i++)
			{
				System.Reflection.MethodInfo method = methods[i];
				System.Type[] attributes = OdbReflection.GetAttributeTypes(method);
				if (method.Name.Equals(MatchMethodName) && attributes.Length == 1)
				{
					clazz = attributes[0];
					methodsCache.Add(query, method);
					return OdbClassUtil.GetFullName(clazz);
				}
			}
			throw new ODBRuntimeException(NeoDatisError.QueryNqMatchMethodNotImplemented.AddParameter(query.GetType().FullName));
		}

		public static bool Match(NativeQuery query, object o)
		{
			return query.Match(o);
		}

		public static bool Match(SimpleNativeQuery query, object o)
		{
			MethodInfo method = methodsCache[query];
			object[] @params = new object[] { o };
			object result;
			try
			{
				result = method.Invoke(query, @params);
			}
			catch (System.Exception e)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.QueryNqExceptionRaisedByNativeQueryExecution
					.AddParameter(query.GetType().FullName), e);
			}
			return ((bool)result);
		}
	}
}
