namespace NeoDatis.Odb.Core.Query.Execution
{
	/// <summary>Used to implement generic action on matching object.</summary>
	/// <remarks>
	/// Used to implement generic action on matching object. The Generic query executor is responsible for checking if an object meets the criteria conditions.
	/// Then an(some) object actions are called to execute what must be done with matching objects. A ValuesQuery can contain more than
	/// one QueryFieldAction.
	/// </remarks>
	/// <author>osmadja</author>
	public interface IQueryFieldAction
	{
		void Start();

		void End();

		void Execute(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
			 values);

		object GetValue();

		string GetAttributeName();

		string GetAlias();

		/// <summary>To indicate if a query will return one row (for example, sum, average, max and min, or will return more than one row
		/// 	</summary>
		bool IsMultiRow();

		void SetMultiRow(bool isMultiRow);

		/// <summary>used to create a copy!</summary>
		NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction Copy();

		void SetInstanceBuilder(NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder
			 builder);

		NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder GetInstanceBuilder();

		/// <param name="returnInstance"></param>
		void SetReturnInstance(bool returnInstance);

		bool ReturnInstance();
	}
}
