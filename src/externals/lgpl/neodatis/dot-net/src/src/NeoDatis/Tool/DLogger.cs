namespace NeoDatis.Tool
{
	/// <summary>
	/// Simple logging class
	/// <p>
	/// </p>
	/// </summary>
	public class DLogger
	{
		private static System.Collections.Generic.IList<NeoDatis.Tool.ILogger> iloggers = 
			new System.Collections.Generic.List<NeoDatis.Tool.ILogger>();

		public static void Register(NeoDatis.Tool.ILogger logger)
		{
			iloggers.Add(logger);
		}

		public static void Debug(object @object)
		{
			System.Console.Out.WriteLine(@object == null ? "null" : @object.ToString());
			for (int i = 0; i < iloggers.Count; i++)
			{
				iloggers[i].Debug(@object);
			}
		}

		public static void Info(object @object)
		{
			System.Console.Out.WriteLine(@object == null ? "null" : @object.ToString());
			for (int i = 0; i < iloggers.Count; i++)
			{
				iloggers[i].Info(@object);
			}
		}

		/// <param name="@object">The object to be logged</param>
		public static void Error(object @object)
		{
			System.Console.Out.WriteLine(@object == null ? "null" : @object.ToString());
			for (int i = 0; i < iloggers.Count; i++)
			{
				iloggers[i].Error(@object);
			}
		}

		public static void Error(object @object, System.Exception t)
		{
			System.Console.Out.WriteLine(@object == null ? "null" : @object.ToString());
			System.Console.Out.WriteLine(NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(t
				, false));
			for (int i = 0; i < iloggers.Count; i++)
			{
				iloggers[i].Error(@object, t);
			}
		}
	}
}
