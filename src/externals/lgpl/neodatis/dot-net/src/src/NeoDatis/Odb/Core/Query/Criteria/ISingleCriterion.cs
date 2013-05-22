namespace NeoDatis.Odb.Core.Query.Criteria
{
	/// <author>
	/// olivier
	/// An interface for all criteria
	/// </author>
	public interface ISingleCriterion : NeoDatis.Odb.Core.Query.Criteria.ICriterion
	{
		/// <summary>Returns a list of attributes names that are involved in the query</summary>
		/// <returns>The attribute names</returns>
		System.Collections.IList GetAttributeNames();

		string GetAttributeName();

		bool Match(NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoi);

		bool Match(System.Collections.IDictionary map);
	}
}
