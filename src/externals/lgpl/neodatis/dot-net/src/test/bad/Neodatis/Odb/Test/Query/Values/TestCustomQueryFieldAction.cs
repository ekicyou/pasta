namespace NeoDatis.Odb.Test.Query.Values
{
	[System.Serializable]
	public class TestCustomQueryFieldAction : NeoDatis.Odb.Impl.Core.Query.Values.CustomQueryFieldAction
	{
		private System.Decimal myValue;

		public TestCustomQueryFieldAction()
		{
			this.myValue = new System.Decimal(0);
		}

		public override void Execute(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
			 values)
		{
			System.Decimal n = NeoDatis.Odb.Impl.Core.Query.Values.ValuesUtil.Convert((System.Decimal
				)values[attributeName]);
			myValue = myValue.Add(new System.Decimal(n.ToString()).Multiply(new System.Decimal
				(2)));
		}

		public override object GetValue()
		{
			return myValue;
		}

		public override bool IsMultiRow()
		{
			return false;
		}

		public override void Start()
		{
		}

		// Nothing to do
		public override void End()
		{
		}
		// Nothing to do
	}
}
