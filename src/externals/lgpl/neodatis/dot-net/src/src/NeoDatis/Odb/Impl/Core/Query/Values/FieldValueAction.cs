namespace NeoDatis.Odb.Impl.Core.Query.Values
{
	/// <summary>An action to retrieve an object field</summary>
	/// <author>osmadja</author>
	[System.Serializable]
	public class FieldValueAction : NeoDatis.Odb.Core.Query.Values.AbstractQueryFieldAction
	{
		/// <summary>The value of the attribute</summary>
		private object value;

		public FieldValueAction(string attributeName, string alias) : base(attributeName, 
			alias, true)
		{
			this.value = null;
		}

		public override void Execute(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
			 values)
		{
			this.value = values[attributeName];
			if (NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.IsCollection(this.value.GetType(
				)))
			{
				// For collection,we encapsulate it in an lazy load list that will create objects on demand
				System.Collections.Generic.ICollection<object> c = (System.Collections.Generic.ICollection
					<object>)this.value;
				NeoDatis.Odb.Impl.Core.Query.List.Objects.LazySimpleListOfAOI<object> l = new NeoDatis.Odb.Impl.Core.Query.List.Objects.LazySimpleListOfAOI
					<object>(c.Count, GetInstanceBuilder(), ReturnInstance());
				l.AddAll(c);
				this.value = l;
			}
		}

		public override object GetValue()
		{
			return value;
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append(attributeName).Append("=").Append(value);
			return buffer.ToString();
		}

		public override void End()
		{
		}

		// Nothing to do		
		public override void Start()
		{
		}

		// Nothing to do
		public override NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction Copy()
		{
			return new NeoDatis.Odb.Impl.Core.Query.Values.FieldValueAction(attributeName, alias
				);
		}
	}
}
