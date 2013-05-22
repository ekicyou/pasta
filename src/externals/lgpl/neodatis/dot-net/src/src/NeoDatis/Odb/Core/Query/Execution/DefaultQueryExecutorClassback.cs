namespace NeoDatis.Odb.Core.Query.Execution
{
	public class DefaultQueryExecutorClassback : NeoDatis.Odb.Core.Query.Execution.IQueryExecutorCallback
	{
		public virtual void ReadingObject(long index, long oid)
		{
			System.Console.Out.WriteLine((index + 1) + " : oid = " + oid);
		}
	}
}
