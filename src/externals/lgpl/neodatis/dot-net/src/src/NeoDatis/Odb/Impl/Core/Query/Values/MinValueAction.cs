namespace NeoDatis.Odb.Impl.Core.Query.Values
{
	/// <summary>An action to compute the max value of a field</summary>
	/// <author>osmadja</author>
	[System.Serializable]
	public class MinValueAction : NeoDatis.Odb.Core.Query.Values.AbstractQueryFieldAction
	{
		private System.Decimal minValue;

		private NeoDatis.Odb.OID oidOfMinValues;

		public MinValueAction(string attributeName, string alias) : base(attributeName, alias
			, false)
		{
			this.minValue = new System.Decimal(long.MaxValue);
			this.oidOfMinValues = null;
		}

		public override void Execute(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
			 values)
		{
			System.Decimal n = (System.Decimal)values[attributeName];
			System.Decimal bd = NeoDatis.Odb.Impl.Core.Query.Values.ValuesUtil.Convert(n);
			if (minValue.CompareTo(bd) > 0)
			{
				oidOfMinValues = oid;
				minValue = bd;
			}
		}

		public override object GetValue()
		{
			return minValue;
		}

		public override void End()
		{
		}

		// nothing to do
		public override void Start()
		{
		}

		// Nothing to do
		public virtual NeoDatis.Odb.OID GetOidOfMinValues()
		{
			return oidOfMinValues;
		}

		public override NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction Copy()
		{
			return new NeoDatis.Odb.Impl.Core.Query.Values.MinValueAction(attributeName, alias
				);
		}
	}
}
