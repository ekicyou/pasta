namespace NeoDatis.Odb.Core.Query.Values
{
	[System.Serializable]
	public abstract class AbstractQueryFieldAction : NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction
	{
		protected string attributeName;

		protected string alias;

		protected bool isMultiRow;

		protected NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder instanceBuilder;

		protected bool returnInstance;

		public AbstractQueryFieldAction(string attributeName, string alias, bool isMultiRow
			) : base()
		{
			this.attributeName = attributeName;
			this.alias = alias;
			this.isMultiRow = isMultiRow;
		}

		public virtual string GetAttributeName()
		{
			return attributeName;
		}

		public virtual string GetAlias()
		{
			return alias;
		}

		public abstract void Execute(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
			 values);

		public virtual bool IsMultiRow()
		{
			return isMultiRow;
		}

		public virtual void SetMultiRow(bool isMultiRow)
		{
			this.isMultiRow = isMultiRow;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder GetInstanceBuilder
			()
		{
			return instanceBuilder;
		}

		public virtual void SetInstanceBuilder(NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder
			 instanceBuilder)
		{
			this.instanceBuilder = instanceBuilder;
		}

		public virtual bool ReturnInstance()
		{
			return returnInstance;
		}

		public virtual void SetReturnInstance(bool returnInstance)
		{
			this.returnInstance = returnInstance;
		}

		public abstract NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction Copy();

		public abstract void End();

		public abstract object GetValue();

		public abstract void Start();
	}
}
