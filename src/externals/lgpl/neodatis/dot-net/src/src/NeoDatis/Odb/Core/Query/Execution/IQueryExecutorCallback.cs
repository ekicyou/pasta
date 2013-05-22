namespace NeoDatis.Odb.Core.Query.Execution
{
	public interface IQueryExecutorCallback
	{
		void ReadingObject(long index, long oid);
	}
}
