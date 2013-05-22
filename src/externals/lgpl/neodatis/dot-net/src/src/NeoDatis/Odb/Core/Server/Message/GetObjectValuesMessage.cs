namespace NeoDatis.Odb.Core.Server.Message
{
	/// <summary>A message to get object values</summary>
	/// <author>osmadja</author>
	[System.Serializable]
	public class GetObjectValuesMessage : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		private NeoDatis.Odb.Core.Query.IValuesQuery query;

		private int startIndex;

		private int endIndex;

		public GetObjectValuesMessage(string baseId, string connectionId, NeoDatis.Odb.Core.Query.IValuesQuery
			 query, int startIndex, int endIndex) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command
			.GetObjectValues, baseId, connectionId)
		{
			this.query = query;
			this.startIndex = startIndex;
			this.endIndex = endIndex;
		}

		public GetObjectValuesMessage(string baseId, string connectionId, NeoDatis.Odb.Core.Query.IValuesQuery
			 query) : this(baseId, connectionId, query, -1, -1)
		{
		}

		public virtual NeoDatis.Odb.Core.Query.IValuesQuery GetQuery()
		{
			return query;
		}

		public virtual int GetEndIndex()
		{
			return endIndex;
		}

		public virtual int GetStartIndex()
		{
			return startIndex;
		}

		public override string ToString()
		{
			return "GetObjectValues";
		}
	}
}
