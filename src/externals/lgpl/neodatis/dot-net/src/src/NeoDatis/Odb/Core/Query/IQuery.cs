namespace NeoDatis.Odb.Core.Query
{
	public interface IQuery
	{
		/// <summary>To order by the result of a query in descendent order</summary>
		/// <param name="fields">A comma separated field list</param>
		/// <returns>this</returns>
		NeoDatis.Odb.Core.Query.IQuery OrderByDesc(string fields);

		/// <summary>To order by the result of a query in ascendent order</summary>
		/// <param name="fields">A comma separated field list</param>
		/// <returns>this</returns>
		NeoDatis.Odb.Core.Query.IQuery OrderByAsc(string fields);

		/// <summary>Returns true if the query has an order by clause</summary>
		/// <returns>true if has an order by flag</returns>
		bool HasOrderBy();

		/// <summary>Returns the field names of the order by</summary>
		/// <returns>The array of  fields of the order by</returns>
		string[] GetOrderByFieldNames();

		/// <returns>the type of the order by - ORDER_BY_NONE,ORDER_BY_DESC,ORDER_BY_ASC</returns>
		NeoDatis.Odb.Core.OrderByConstants GetOrderByType();

		NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine GetStorageEngine();

		void SetStorageEngine(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine
			);

		NeoDatis.Odb.Core.Query.Execution.IQueryExecutionPlan GetExecutionPlan();

		void SetExecutionPlan(NeoDatis.Odb.Core.Query.Execution.IQueryExecutionPlan plan);

		/// <summary>
		/// To specify that instances of subclass of the query class must not be load
		/// if true, when querying objects of class Class1, only direct instances of Class1 will be loaded.
		/// </summary>
		/// <remarks>
		/// To specify that instances of subclass of the query class must not be load
		/// if true, when querying objects of class Class1, only direct instances of Class1 will be loaded.
		/// If false, when querying objects of class Class1, direct instances of Class1 will be loaded and all instances of subclasses of Class1.
		/// </remarks>
		NeoDatis.Odb.Core.Query.IQuery SetPolymorphic(bool yes);

		bool IsPolymorphic();

		/// <summary>To indicate if a query must be executed on a single object with the specific OID.
		/// 	</summary>
		/// <remarks>To indicate if a query must be executed on a single object with the specific OID. Used for ValuesQeuries
		/// 	</remarks>
		/// <returns></returns>
		bool IsForSingleOid();

		/// <summary>used with isForSingleOid == true, to indicate we are working on a single object with a specific oid
		/// 	</summary>
		/// <returns></returns>
		NeoDatis.Odb.OID GetOidOfObjectToQuery();

        void SetFullClassName(System.Type type);
    }
}
