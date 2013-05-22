namespace NeoDatis.Odb.Test.Conversion
{
	/// <author>olivier</author>
	public class TestNumberConversion : NeoDatis.Odb.Test.ODBTest
	{
		public virtual void Test1()
		{
			AssertEquals(0, NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.AttributeValueComparator
				.Compare(10, System.Convert.ToSingle(10)));
			AssertEquals(0, NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.AttributeValueComparator
				.Compare(10, System.Convert.ToInt64(10)));
			AssertEquals(0, NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.AttributeValueComparator
				.Compare(10, System.Convert.ToDouble(10)));
			AssertEquals(0, NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.AttributeValueComparator
				.Compare(10, (byte)10));
			AssertEquals(0, NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.AttributeValueComparator
				.Compare(10, 10));
			AssertEquals(0, NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.AttributeValueComparator
				.Compare(10, (short)10));
			AssertEquals(1, NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.AttributeValueComparator
				.Compare(10, (short)9));
			AssertEquals(10.CompareTo(9), NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.AttributeValueComparator
				.Compare(10, (short)9));
		}
	}
}
