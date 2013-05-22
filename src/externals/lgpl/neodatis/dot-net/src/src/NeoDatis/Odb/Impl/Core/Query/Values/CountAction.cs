namespace NeoDatis.Odb.Impl.Core.Query.Values
{
	/// <summary>An action to count objects of a  query</summary>
	/// <author>osmadja</author>
	[System.Serializable]
	public class CountAction : NeoDatis.Odb.Core.Query.Values.AbstractQueryFieldAction
	{
		private static System.Decimal One = NeoDatis.Tool.Wrappers.NeoDatisNumber.NewBigInteger
			(1);

		private System.Decimal count;

		public CountAction(string alias) : base(alias, alias, false)
		{
			count = NeoDatis.Tool.Wrappers.NeoDatisNumber.NewBigInteger(0);
		}

		public override void Execute(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
			 values)
		{
			count = NeoDatis.Tool.Wrappers.NeoDatisNumber.Add(count, One);
		}

		public virtual System.Decimal GetCount()
		{
			return count;
		}

		public override object GetValue()
		{
			return count;
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
			return new NeoDatis.Odb.Impl.Core.Query.Values.CountAction(alias);
		}
	}
}
