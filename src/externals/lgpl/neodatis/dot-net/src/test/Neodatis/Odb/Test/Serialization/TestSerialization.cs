using NUnit.Framework;
namespace NeoDatis.Odb.Test.Serialization
{
	[TestFixture]
    public class TestSerialization : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestAtomicNativeCollectionString()
		{
			string s1 = "ol√° chico";
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo anoi = null;
			anoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo(s1, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.StringId);
			string s = NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer.GetInstance
				().ToString(anoi);
			// println(s);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo anoi2 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo
				)NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer.GetInstance(
				).FromOneString(s);
			AssertEquals(anoi, anoi2);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestAtomicNativeCollectionDate()
		{
			System.DateTime date = new System.DateTime();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo anoi = null;
			anoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo(date, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.DateId);
			string s = NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer.GetInstance
				().ToString(anoi);
			// println(s);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo anoi2 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo
				)NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer.GetInstance(
				).FromOneString(s);
			AssertEquals(anoi, anoi2);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestAtomicNativeCollectionBigDecimal()
		{
			System.Decimal bd = new System.Decimal("123456789.987654321");
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo anoi = null;
			anoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo(bd, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.BigDecimalId);
			string s = NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer.GetInstance
				().ToString(anoi);
			// println(s);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo anoi2 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo
				)NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer.GetInstance(
				).FromOneString(s);
			AssertEquals(anoi, anoi2);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestAtomicNativeCollectionInt()
		{
			int i = 123456789;
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo anoi = null;
			anoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo(i, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.IntegerId);
			string s = NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer.GetInstance
				().ToString(anoi);
			// println(s);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo anoi2 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo
				)NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer.GetInstance(
				).FromOneString(s);
			AssertEquals(anoi, anoi2);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestAtomicNativeCollectionDouble()
		{
			double d = 123456789.789456123;
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo anoi = null;
			anoi = new NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo(d, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType
				.DoubleId);
			string s = NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer.GetInstance
				().ToString(anoi);
			// println(s);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo anoi2 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo
				)NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Serialization.Serializer.GetInstance(
				).FromOneString(s);
			AssertEquals(anoi, anoi2);
		}

		[Test]
        public virtual void TestRegExp()
		{
			// println("start");
			string token = "A;B;[C;D];E";
			// (*)&&^(*^)
			string pattern = "[\\[*\\]]";
			Java.Util.Regex.Pattern p = Java.Util.Regex.Pattern.Compile(pattern);
			string[] array = token.Split(pattern);
			Java.Util.Regex.Matcher m = p.Matcher(token);
			// println(token);
			// println(m.groupCount());
			for (int i = 0; i < array.Length; i++)
			{
			}
		}
		// println((i+1)+"="+array[i]);
	}
}
