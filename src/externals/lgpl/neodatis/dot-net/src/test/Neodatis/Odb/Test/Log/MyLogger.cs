using NUnit.Framework;
namespace NeoDatis.Odb.Test.Log
{
	/// <author>olivier</author>
	public class MyLogger : NeoDatis.Tool.ILogger
	{
		public virtual void Debug(object @object)
		{
		}

		public virtual void Error(object @object)
		{
		}

		public virtual void Error(object @object, System.Exception t)
		{
		}

		public virtual void Info(object @object)
		{
		}
	}
}
