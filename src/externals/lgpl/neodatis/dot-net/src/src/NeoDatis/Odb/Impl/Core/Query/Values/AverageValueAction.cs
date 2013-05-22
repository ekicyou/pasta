namespace NeoDatis.Odb.Impl.Core.Query.Values
{
	/// <summary>An action to compute the average value of a field</summary>
	/// <author>osmadja</author>
	[System.Serializable]
	public class AverageValueAction : NeoDatis.Odb.Core.Query.Values.AbstractQueryFieldAction
	{
		private static System.Decimal One = new System.Decimal(1);

		private System.Decimal totalValue;

		private System.Decimal nbValues;

		private System.Decimal average;

		private int scale;

		private int roundType;

		public AverageValueAction(string attributeName, string alias) : base(attributeName
			, alias, false)
		{
			this.totalValue = new System.Decimal(0);
			this.nbValues = new System.Decimal(0);
			this.attributeName = attributeName;
			this.scale = NeoDatis.Odb.OdbConfiguration.GetScaleForAverageDivision();
			this.roundType = NeoDatis.Odb.OdbConfiguration.GetRoundTypeForAverageDivision();
		}

		public override void Execute(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
			 values)
		{
			System.Decimal n = (System.Decimal)values[attributeName];
			totalValue = NeoDatis.Tool.Wrappers.NeoDatisNumber.Add(totalValue, NeoDatis.Odb.Impl.Core.Query.Values.ValuesUtil
				.Convert(n));
			nbValues = NeoDatis.Tool.Wrappers.NeoDatisNumber.Add(nbValues, One);
		}

		public override object GetValue()
		{
			return average;
		}

		public override void End()
		{
			average = NeoDatis.Tool.Wrappers.NeoDatisNumber.Divide(totalValue, nbValues, roundType
				, scale);
		}

		public override void Start()
		{
		}

		public override NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction Copy()
		{
			return new NeoDatis.Odb.Impl.Core.Query.Values.AverageValueAction(attributeName, 
				alias);
		}
	}
}
