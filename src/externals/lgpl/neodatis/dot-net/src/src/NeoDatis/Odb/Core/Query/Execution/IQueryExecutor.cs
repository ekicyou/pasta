namespace NeoDatis.Odb.Core.Query.Execution
{
	public interface IQueryExecutor
	{
		/// <summary>The main query execution method</summary>
		/// <param name="inMemory"></param>
		/// <param name="startIndex"></param>
		/// <param name="endIndex"></param>
		/// <param name="returnObjects"></param>
		/// <param name="queryResultAction"></param>
		/// <returns></returns>
		/// <exception cref="System.Exception">System.Exception</exception>
		NeoDatis.Odb.Objects<T> Execute<T>(bool inMemory, int startIndex, int endIndex, bool
			 returnObjects, NeoDatis.Odb.Core.Query.Execution.IMatchingObjectAction queryResultAction
			);
	}
}
