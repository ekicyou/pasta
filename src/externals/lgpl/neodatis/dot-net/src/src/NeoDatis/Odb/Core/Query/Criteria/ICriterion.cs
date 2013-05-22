namespace NeoDatis.Odb.Core.Query.Criteria
{
	/// <author>
	/// olivier
	/// An interface for all criteria
	/// </author>
	public interface ICriterion
	{
		/// <summary>To check if an object matches this criterion</summary>
		/// <param name="@object"></param>
		/// <returns>true if object matches the criteria</returns>
		bool Match(object @object);

		/// <summary>to be able to optimize query execution.</summary>
		/// <remarks>to be able to optimize query execution. Get only the field involved in the query instead of getting all the object
		/// 	</remarks>
		/// <returns>All involved fields in criteria, List of String</returns>
		NeoDatis.Tool.Wrappers.List.IOdbList<string> GetAllInvolvedFields();

		NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap GetValues();

		/// <summary>Gets thes whole query</summary>
		/// <returns>The owner query</returns>
		NeoDatis.Odb.Core.Query.IQuery GetQuery();

		void SetQuery(NeoDatis.Odb.Core.Query.IQuery query);

		bool CanUseIndex();

		/// <summary>a method to explicitly indicate that the criteria is ready.</summary>
		/// <remarks>a method to explicitly indicate that the criteria is ready.</remarks>
		void Ready();
	}
}
