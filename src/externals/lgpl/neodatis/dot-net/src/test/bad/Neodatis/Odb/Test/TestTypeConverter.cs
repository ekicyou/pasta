namespace NeoDatis.Odb.Test
{
	/// <author>
	/// olivier s
	/// TODO To change the template for this generated type comment go to
	/// Window - Preferences - Java - Code Style - Code Templates
	/// </author>
	public class TestTypeConverter : NeoDatis.Odb.Test.ODBTest
	{
		public virtual void Test1()
		{
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeInt, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(typeof(int)));
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeBoolean, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(typeof(bool)));
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeByte, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(typeof(byte)));
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeChar, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(typeof(char)));
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeDouble, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(typeof(double)));
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeFloat, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(typeof(float)));
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeLong, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(typeof(long)));
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeShort, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(typeof(short)));
		}

		public virtual void Test2()
		{
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Integer, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(typeof(int)));
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Boolean, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(typeof(bool)));
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Byte, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(typeof(byte)));
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Character, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(typeof(char)));
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Double, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(typeof(double)));
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Float, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(typeof(float)));
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Long, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(typeof(long)));
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Short, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(typeof(short)));
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.String, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(typeof(string)));
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.BigDecimal, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(typeof(System.Decimal)));
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.BigInteger, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(typeof(System.Decimal)));
		}

		public virtual void Test3()
		{
			int[] array1 = new int[] { 1, 2 };
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Array, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(array1.GetType()));
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeInt, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(array1.GetType()).GetSubType());
			string[] array2 = new string[] { "1", "2" };
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.Array, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(array2.GetType()));
			AssertEquals(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.String, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.GetFromClass(array2.GetType()).GetSubType());
		}

		public virtual void Test4()
		{
			// println(int.class.getName());
			AssertEquals(typeof(int), typeof(int));
		}
	}
}
