namespace NeoDatis.Odb.Core.Server.Message
{
	/// <summary>To get the number of objets of a class</summary>
	/// <author>osmadja</author>
	[System.Serializable]
	public class CountMessage : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		private NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery query;

		public CountMessage(string baseId, string connectionId, NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
			 query) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.Count, baseId
			, connectionId)
		{
			this.query = query;
		}

		public virtual NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery GetCriteriaQuery
			()
		{
			return query;
		}

		public override string ToString()
		{
			return "Count";
		}
	}
}
