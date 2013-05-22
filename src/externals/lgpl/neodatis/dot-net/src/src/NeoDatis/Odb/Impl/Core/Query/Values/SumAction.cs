namespace NeoDatis.Odb.Impl.Core.Query.Values
{
	[System.Serializable]
	public class SumAction : NeoDatis.Odb.Core.Query.Values.AbstractQueryFieldAction
	{
		private System.Decimal sum;

		public SumAction(string attributeName, string alias) : base(attributeName, alias, 
			false)
		{
			sum = new System.Decimal(0);
		}

		public override void Execute(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
			 values)
		{
			System.Decimal n = (System.Decimal)values[attributeName];
			sum = NeoDatis.Tool.Wrappers.NeoDatisNumber.Add(sum, NeoDatis.Odb.Impl.Core.Query.Values.ValuesUtil
				.Convert(n));
		}

		public virtual System.Decimal GetSum()
		{
			return sum;
		}

		public override object GetValue()
		{
			return sum;
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
			return new NeoDatis.Odb.Impl.Core.Query.Values.SumAction(attributeName, alias);
		}
	}
}
