using NeoDatis.Odb.Test.VO.Attribute;
using NeoDatis.Odb.Core.Query.Criteria;
namespace NeoDatis.Odb.Test.Query.Criteria
{
	public class TestCriteriaQuery5 : NeoDatis.Odb.Test.ODBTest
	{
		private System.DateTime correctDate;

		public static readonly string BaseName = "criteria-native-object.neodatis";

		/// <exception cref="System.Exception"></exception>
		public virtual void TestCriteriaWithDate()
		{
			DeleteBase(BaseName);
			NeoDatis.Odb.ODB odb = Open(BaseName);
			for (int i = 0; i < 10; i++)
			{
				NeoDatis.Odb.Test.VO.Attribute.TestClass tc = new NeoDatis.Odb.Test.VO.Attribute.TestClass
					();
				tc.SetInt1(i);
				odb.Store(tc);
			}
			odb.Close();
			odb = Open(BaseName);
			NeoDatis.Odb.Objects<TestClass> os = odb.GetObjects<TestClass>(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(Where
				.Ge("int1", 0)));
			AssertEquals(10, os.Count);
			int j = 0;
			while (os.HasNext())
			{
				NeoDatis.Odb.Test.VO.Attribute.TestClass tc = (NeoDatis.Odb.Test.VO.Attribute.TestClass
					)os.Next();
				AssertEquals(j, tc.GetInt1());
				j++;
			}
			odb.Close();
		}

		public virtual void TestIntLongCriteriaQuery()
		{
			DeleteBase(BaseName);
			NeoDatis.Odb.ODB odb = Open(BaseName);
			NeoDatis.Odb.Test.Query.Criteria.ClassWithInt cwi = new NeoDatis.Odb.Test.Query.Criteria.ClassWithInt
				(1, "test");
			odb.Store(cwi);
			odb.Close();
			odb = Open(BaseName);
			NeoDatis.Odb.Objects<ClassWithInt> os = odb.GetObjects<ClassWithInt>(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("i", (long)1)));
			AssertEquals(1, os.Count);
			odb.Close();
		}

		public virtual void TestLongIntCriteriaQuery()
		{
			DeleteBase(BaseName);
			NeoDatis.Odb.ODB odb = Open(BaseName);
			NeoDatis.Odb.Test.Query.Criteria.ClassWithLong cwl = new NeoDatis.Odb.Test.Query.Criteria.ClassWithLong
				(1L, "test");
			odb.Store(cwl);
			odb.Close();
			odb = Open(BaseName);
			NeoDatis.Odb.Objects<ClassWithLong> os = odb.GetObjects<ClassWithLong>(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("i", (int)1)));
			AssertEquals(1, os.Count);
			odb.Close();
		}

		public virtual void TestLongIntCriteriaQueryGt()
		{
			DeleteBase(BaseName);
			NeoDatis.Odb.ODB odb = Open(BaseName);
			NeoDatis.Odb.Test.Query.Criteria.ClassWithLong cwl = new NeoDatis.Odb.Test.Query.Criteria.ClassWithLong
				(1L, "test");
			odb.Store(cwl);
			odb.Close();
			odb = Open(BaseName);
			NeoDatis.Odb.Objects<ClassWithLong> os = odb.GetObjects<ClassWithLong>(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(NeoDatis.Odb.Core.Query.Criteria.Where
				.Ge("i", (int)1)));
			AssertEquals(1, os.Count);
			os = odb.GetObjects<ClassWithLong>(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(NeoDatis.Odb.Core.Query.Criteria.Where
				.Gt("i", (int)1)));
			AssertEquals(0, os.Count);
			odb.Close();
		}
	}
}
