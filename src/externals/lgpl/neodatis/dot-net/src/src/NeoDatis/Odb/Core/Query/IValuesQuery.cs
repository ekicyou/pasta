namespace NeoDatis.Odb.Core.Query
{
	public interface IValuesQuery : NeoDatis.Odb.Core.Query.IQuery
	{
		NeoDatis.Odb.Core.Query.IValuesQuery Count(string alias);

		NeoDatis.Odb.Core.Query.IValuesQuery Sum(string fieldName);

		NeoDatis.Odb.Core.Query.IValuesQuery Sum(string fieldName, string alias);

		NeoDatis.Odb.Core.Query.IValuesQuery Avg(string fieldName, string alias);

		NeoDatis.Odb.Core.Query.IValuesQuery Avg(string fieldName);

		NeoDatis.Odb.Core.Query.IValuesQuery Max(string fieldName, string alias);

		NeoDatis.Odb.Core.Query.IValuesQuery Max(string fieldName);

		NeoDatis.Odb.Core.Query.IValuesQuery Field(string fieldName);

		NeoDatis.Odb.Core.Query.IValuesQuery Field(string fieldName, string alias);

		NeoDatis.Odb.Core.Query.IValuesQuery Sublist(string attributeName, string alias, 
			int fromIndex, int size, bool throwException);

		NeoDatis.Odb.Core.Query.IValuesQuery Sublist(string attributeName, int fromIndex, 
			int size, bool throwException);

		NeoDatis.Odb.Core.Query.IValuesQuery Sublist(string attributeName, string alias, 
			int fromIndex, int toIndex);

		NeoDatis.Odb.Core.Query.IValuesQuery Sublist(string attributeName, int fromIndex, 
			int toIndex);

		NeoDatis.Odb.Core.Query.IValuesQuery Size(string attributeName);

		NeoDatis.Odb.Core.Query.IValuesQuery Size(string attributeName, string alias);

		NeoDatis.Odb.Core.Query.IValuesQuery GroupBy(string fieldList);

		string[] GetGroupByFieldList();

		bool HasGroupBy();

		NeoDatis.Tool.Wrappers.List.IOdbList<string> GetAllInvolvedFields();

		/// <summary>A collection of IQueryFieldAction</summary>
		NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction
			> GetObjectActions();

		/// <summary>To indicate if a query will return one row (for example, sum, average, max and min, or will return more than one row
		/// 	</summary>
		bool IsMultiRow();

		/// <returns></returns>
		bool ReturnInstance();

		/// <summary>To indicate if query execution must build instances or return object representation, Default value is true(return instance)
		/// 	</summary>
		void SetReturnInstance(bool returnInstance);
	}
}
