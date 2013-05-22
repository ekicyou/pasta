namespace NeoDatis.Tool
{
	public class ConsoleLogger : NeoDatis.Tool.ILogger
	{
		private int i;

		private NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine;

		public ConsoleLogger(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine)
		{
			this.engine = engine;
			i = 0;
		}

		public ConsoleLogger()
		{
			i = 0;
		}

		public virtual void Debug(object o)
		{
			System.Console.Out.WriteLine(o);
		}

		public virtual void Error(object o)
		{
			string header = "An internal error occured,please email the error stack trace displayed below to odb.support@neodatis.org";
			System.Console.Out.WriteLine(header);
			System.Console.Out.WriteLine(o);
		}

		public virtual void Error(object o, System.Exception throwable)
		{
			string header = "An internal error occured,please email the error stack trace displayed below to odb.support@neodatis.org";
			System.Console.Out.WriteLine(header);
			System.Console.Out.WriteLine(o);
			System.Console.Out.WriteLine(NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(throwable
				, false));
		}

		public virtual void Info(object o)
		{
			if (i % 20 == 0)
			{
				if (engine != null)
				{
					System.Console.Out.WriteLine(engine.GetSession(true).GetCache().ToString());
				}
			}
			System.Console.Out.WriteLine(o);
			i++;
		}
	}
}
