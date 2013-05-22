namespace NeoDatis.Odb.Impl.Core.Query.Criteria
{
	public class CriteriaQueryManager
	{
		public static bool Match(NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery query
			, System.Collections.IDictionary map)
		{
			return query.Match(map);
		}

		public static bool Match(NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery query
			, object @object)
		{
			return query.Match((NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo)@object
				);
		}

		public static string GetFullClassName(NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
			 query)
		{
			return query.GetFullClassName();
		}
	}
}
