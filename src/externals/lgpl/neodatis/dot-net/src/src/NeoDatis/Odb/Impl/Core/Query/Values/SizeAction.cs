namespace NeoDatis.Odb.Impl.Core.Query.Values
{
	/// <summary>An action to retrieve a size of a list.</summary>
	/// <remarks>
	/// An action to retrieve a size of a list. It is used by the Object Values API.
	/// When calling odb.getValues(new ValuesCriteriaQuery(Handler.class, Where
	/// .equal("id", id)).size("parameters");
	/// The sublist action will return  Returns a view of the portion of this list between the specified fromIndex, inclusive, and toIndex, exclusive.
	/// if parameters list contains [param1,param2,param3,param4], sublist("parameters",1,2) will return a sublist
	/// containing [param2,param3]
	/// </remarks>
	/// <author>osmadja</author>
	[System.Serializable]
	public class SizeAction : NeoDatis.Odb.Core.Query.Values.AbstractQueryFieldAction
	{
		private long size;

		public SizeAction(string attributeName, string alias) : base(attributeName, alias
			, true)
		{
		}

		public override void Execute(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
			 values)
		{
			System.Collections.IList l = (System.Collections.IList)values[attributeName];
			this.size = l.Count;
		}

		public override object GetValue()
		{
			return size;
		}

		public override void End()
		{
		}

		// nothing to do
		public override void Start()
		{
		}

		// Nothing to do
		public virtual long GetSize()
		{
			return size;
		}

		public override NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction Copy()
		{
			return new NeoDatis.Odb.Impl.Core.Query.Values.SizeAction(attributeName, alias);
		}
	}
}
