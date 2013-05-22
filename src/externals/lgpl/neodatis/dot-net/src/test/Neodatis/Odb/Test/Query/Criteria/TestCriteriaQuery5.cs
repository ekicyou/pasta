using NeoDatis.Odb.Test.VO.Attribute;
using NeoDatis.Odb.Core.Query.Criteria;
using NUnit.Framework;
using NeoDatis.Odb.Impl.Core.Query.Criteria;
namespace NeoDatis.Odb.Test.Query.Criteria
{
	[TestFixture]
    public class TestCriteriaQuery5 : NeoDatis.Odb.Test.ODBTest
	{
		private System.DateTime correctDate;

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestCriteriaWithDate()
		{
            string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			for (int i = 0; i < 10; i++)
			{
				NeoDatis.Odb.Test.VO.Attribute.TestClass tc = new NeoDatis.Odb.Test.VO.Attribute.TestClass
					();
				tc.SetInt1(i);
				odb.Store(tc);
			}
			odb.Close();
			odb = Open(baseName);
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

		[Test]
        public virtual void TestIntLongCriteriaQuery()
		{
            string baseName = GetBaseName();
			
			NeoDatis.Odb.ODB odb = Open(baseName);
			ClassWithInt cwi = new ClassWithInt(1, "test");
			odb.Store(cwi);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Objects<ClassWithInt> os = odb.GetObjects<ClassWithInt>(new CriteriaQuery(Where.Equal("i", (long)1)));
			AssertEquals(1, os.Count);
			odb.Close();
		}

		[Test]
        public virtual void TestLongIntCriteriaQuery()
		{
            string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			ClassWithLong cwl = new ClassWithLong(1L, "test");
			odb.Store(cwl);
			odb.Close();
			odb = Open(baseName);
			Objects<ClassWithLong> os = odb.GetObjects<ClassWithLong>(new CriteriaQuery(Where.Equal("i", 1L)));
			AssertEquals(1, os.Count);
			odb.Close();
		}

		[Test]
        public virtual void TestLongIntCriteriaQueryGt()
		{
            string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			ClassWithLong cwl = new ClassWithLong(1L, "test");
			odb.Store(cwl);
			odb.Close();
			odb = Open(baseName);
			Objects<ClassWithLong> os = odb.GetObjects<ClassWithLong>(new CriteriaQuery(Where.Ge("i", 1L)));
			AssertEquals(1, os.Count);
			os = odb.GetObjects<ClassWithLong>(new CriteriaQuery(Where.Gt("i", 1L)));
			AssertEquals(0, os.Count);
			odb.Close();
		}
	}
}
