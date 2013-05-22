namespace NeoDatis.Odb.Impl.Core.Query.Values
{
	/// <summary>An action to compute the max value of a field</summary>
	/// <author>osmadja</author>
	[System.Serializable]
	public class MaxValueAction : NeoDatis.Odb.Core.Query.Values.AbstractQueryFieldAction
	{
		private System.Decimal maxValue;

		private NeoDatis.Odb.OID oidOfMaxValues;

		public MaxValueAction(string attributeName, string alias) : base(attributeName, alias
			, false)
		{
			this.maxValue = new System.Decimal(long.MinValue);
			this.oidOfMaxValues = null;
		}

		public override void Execute(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
			 values)
		{
			System.Decimal n = (System.Decimal)values[attributeName];
			System.Decimal bd = NeoDatis.Odb.Impl.Core.Query.Values.ValuesUtil.Convert(n);
			if (bd.CompareTo(maxValue) > 0)
			{
				oidOfMaxValues = oid;
				maxValue = bd;
			}
		}

		public override object GetValue()
		{
			return maxValue;
		}

		public override void End()
		{
		}

		// nothing to do
		public override void Start()
		{
		}

		// Nothing to do
		public virtual NeoDatis.Odb.OID GetOidOfMaxValues()
		{
			return oidOfMaxValues;
		}

		public override NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction Copy()
		{
			return new NeoDatis.Odb.Impl.Core.Query.Values.MaxValueAction(attributeName, alias
				);
		}
	}
}
