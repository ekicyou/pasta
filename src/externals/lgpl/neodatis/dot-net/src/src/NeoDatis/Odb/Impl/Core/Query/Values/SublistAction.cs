namespace NeoDatis.Odb.Impl.Core.Query.Values
{
	/// <summary>An action to retrieve a sublist of list.</summary>
	/// <remarks>
	/// An action to retrieve a sublist of list. It is used by the Object Values API.
	/// When calling odb.getValues(new ValuesCriteriaQuery(Handler.class, Where
	/// .equal("id", id)).sublist("parameters",fromIndex, size);
	/// The sublist action will return  Returns a view of the portion of this list between the specified fromIndex, inclusive, and toIndex, exclusive.
	/// if parameters list contains [param1,param2,param3,param4], sublist("parameters",1,2) will return a sublist
	/// containing [param2,param3]
	/// </remarks>
	/// <author>osmadja</author>
	[System.Serializable]
	public class SublistAction : NeoDatis.Odb.Core.Query.Values.AbstractQueryFieldAction
	{
		private NeoDatis.Tool.Wrappers.List.IOdbList<object> sublist;

		private int fromIndex;

		private int size;

		private bool throwExceptionIfOutOfBound;

		public SublistAction(string attributeName, string alias, int fromIndex, int size, 
			bool throwExceptionIfOutOfBound) : base(attributeName, alias, true)
		{
			this.fromIndex = fromIndex;
			this.size = size;
			this.throwExceptionIfOutOfBound = throwExceptionIfOutOfBound;
		}

		public SublistAction(string attributeName, string alias, int fromIndex, int toIndex
			) : base(attributeName, alias, true)
		{
			this.fromIndex = fromIndex;
			this.size = toIndex - fromIndex;
			this.throwExceptionIfOutOfBound = true;
		}

		public override void Execute(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
			 values)
		{
			System.Collections.Generic.IList<object> l = (System.Collections.Generic.IList<object
				>)values[attributeName];
			int localFromIndex = fromIndex;
			int localEndIndex = fromIndex + size;
			// If not throw exception, we must implement 
			// Index Out Of Bound protection
			if (!throwExceptionIfOutOfBound)
			{
				// Check from index
				if (localFromIndex > l.Count - 1)
				{
					localFromIndex = 0;
				}
				// Check end index
				if (localEndIndex > l.Count)
				{
					localEndIndex = l.Count;
				}
			}
			sublist = new NeoDatis.Odb.Impl.Core.Query.List.Objects.LazySimpleListOfAOI<object
				>(size, GetInstanceBuilder(), ReturnInstance());
			sublist.AddAll(NeoDatis.Tool.Wrappers.List.NeoDatisCollectionUtil.SublistGeneric(
				l, localFromIndex, localEndIndex));
		}

		public override object GetValue()
		{
			return sublist;
		}

		public override void End()
		{
		}

		// nothing to do
		public override void Start()
		{
		}

		// Nothing to do
		public virtual System.Collections.Generic.IList<object> GetSubList()
		{
			return sublist;
		}

		public override NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction Copy()
		{
			return new NeoDatis.Odb.Impl.Core.Query.Values.SublistAction(attributeName, alias
				, fromIndex, size, throwExceptionIfOutOfBound);
		}
	}
}
