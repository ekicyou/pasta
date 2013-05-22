namespace NeoDatis.Odb.Core.Server.Message
{
	[System.Serializable]
	public class GetMessage : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		private NeoDatis.Odb.Core.Query.IQuery query;

		private int startIndex;

		private int endIndex;

		private bool inMemory;

		public GetMessage(string baseId, string connectionId, NeoDatis.Odb.Core.Query.IQuery
			 query, bool inMemory, int startIndex, int endIndex) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command
			.Get, baseId, connectionId)
		{
			this.query = query;
			this.startIndex = startIndex;
			this.endIndex = endIndex;
			this.inMemory = inMemory;
		}

		public GetMessage(string baseId, string connectionId, NeoDatis.Odb.Core.Query.IQuery
			 query) : this(baseId, connectionId, query, true, -1, -1)
		{
		}

		public virtual NeoDatis.Odb.Core.Query.IQuery GetQuery()
		{
			return query;
		}

		public virtual int GetEndIndex()
		{
			return endIndex;
		}

		public virtual bool IsInMemory()
		{
			return inMemory;
		}

		public virtual int GetStartIndex()
		{
			return startIndex;
		}

		public override string ToString()
		{
			return "GetObjects";
		}
	}
}
