namespace NeoDatis.Odb.Core.Query.Execution
{
	/// <summary>
	/// <p>
	/// A class to execute a query on more than one class and then merges the result.
	/// </summary>
	/// <remarks>
	/// <p>
	/// A class to execute a query on more than one class and then merges the result. It is used when polymophic is set to true because
	/// in this case, we must execute query on the main class and all its persistent subclasses
	/// </p>
	/// </P>
	/// </remarks>
	public class MultiClassGenericQueryExecutor : NeoDatis.Odb.Core.Query.Execution.IQueryExecutor
	{
		private static readonly string LogId = "MultiClassGenericQueryExecutor";

		private NeoDatis.Odb.Core.Query.Execution.IMultiClassQueryExecutor executor;

		public MultiClassGenericQueryExecutor(NeoDatis.Odb.Core.Query.Execution.IMultiClassQueryExecutor
			 executor)
		{
			this.executor = executor;
			// To avoid reseting the result for each query
			this.executor.SetExecuteStartAndEndOfQueryAction(false);
		}

		/// <summary>The main query execution method</summary>
		/// <param name="query"></param>
		/// <param name="inMemory"></param>
		/// <param name="startIndex"></param>
		/// <param name="endIndex"></param>
		/// <param name="returnObjects"></param>
		/// <returns></returns>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual NeoDatis.Odb.Objects<T> Execute<T>(bool inMemory, int startIndex, 
			int endIndex, bool returnObjects, NeoDatis.Odb.Core.Query.Execution.IMatchingObjectAction
			 queryResultAction)
		{
			if (executor.GetStorageEngine().IsClosed())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbIsClosed
					.AddParameter(executor.GetStorageEngine().GetBaseIdentification().GetIdentification
					()));
			}
			if (executor.GetStorageEngine().GetSession(true).IsRollbacked())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbHasBeenRollbacked
					);
			}
			// Get the main class
			string fullClassName = NeoDatis.Odb.Core.Query.QueryManager.GetFullClassName(executor
				.GetQuery());
			// this is done once.
			queryResultAction.Start();
			NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
				> allClassInfos = executor.GetStorageEngine().GetSession(true).GetMetaModel().GetPersistentSubclassesOf
				(fullClassName);
			int nbClasses = allClassInfos.Count;
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = null;
			for (int i = 0; i < nbClasses; i++)
			{
				ci = allClassInfos[i];
				// Sets the class info to the current
				executor.SetClassInfo(ci);
				// Then execute query
				executor.Execute<T>(inMemory, startIndex, endIndex, returnObjects, queryResultAction
					);
			}
			queryResultAction.End();
			return queryResultAction.GetObjects<T>();
		}

		public virtual bool ExecuteStartAndEndOfQueryAction()
		{
			return false;
		}
	}
}
