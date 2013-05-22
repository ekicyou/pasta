namespace NeoDatis.Odb.Core.Server.Message
{
	[System.Serializable]
	public class AddIndexMessage : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		protected string className;

		protected string indexName;

		protected string[] indexFieldNames;

		protected bool acceptMultipleValuesForSameKey;

		protected bool verbose;

		public AddIndexMessage(string baseId, string connectionId, string className, string
			 indexName, string[] fieldNames, bool acceptMultipleValuesForSameKey, bool verbose
			) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.AddUniqueIndex, baseId
			, connectionId)
		{
			this.className = className;
			this.indexFieldNames = fieldNames;
			this.indexName = indexName;
			this.acceptMultipleValuesForSameKey = acceptMultipleValuesForSameKey;
			this.verbose = verbose;
		}

		public virtual string[] GetIndexFieldNames()
		{
			return indexFieldNames;
		}

		public virtual string GetIndexName()
		{
			return indexName;
		}

		public virtual string GetClassName()
		{
			return className;
		}

		public override string ToString()
		{
			return "AddIndex";
		}

		public virtual bool AcceptMultipleValuesForSameKey()
		{
			return acceptMultipleValuesForSameKey;
		}

		public virtual void SetAcceptMultipleValuesForSameKey(bool acceptMultipleValuesForSameKey
			)
		{
			this.acceptMultipleValuesForSameKey = acceptMultipleValuesForSameKey;
		}

		public virtual bool IsVerbose()
		{
			return verbose;
		}

		public virtual void SetVerbose(bool verbose)
		{
			this.verbose = verbose;
		}
	}
}
