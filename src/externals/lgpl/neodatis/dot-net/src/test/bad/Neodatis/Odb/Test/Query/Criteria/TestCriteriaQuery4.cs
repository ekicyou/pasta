using NeoDatis.Odb.Test.VO.Attribute;
using NeoDatis.Odb.Impl.Core.Query.Criteria;
using NeoDatis.Odb.Core.Query.Criteria;
namespace NeoDatis.Odb.Test.Query.Criteria
{
	public class TestCriteriaQuery4 : NeoDatis.Odb.Test.ODBTest
	{
		private System.DateTime correctDate;

		public static readonly string BaseName = "soda-native-object.neodatis";

		/// <exception cref="System.Exception"></exception>
		public virtual void TestSodaWithDate()
		{
			NeoDatis.Odb.ODB odb = Open(BaseName);
			CriteriaQuery query = new CriteriaQuery(Where.And().Add(Where.Equal("string1", "test class with values"
				)).Add(Where.Equal("date1", new System.DateTime(correctDate.Millisecond))));
			NeoDatis.Odb.Objects<TestClass> l = odb.GetObjects<TestClass>(query);
			// assertEquals(1,l.size());
			query = new CriteriaQuery(Where.And().Add(Where.Equal("string1", "test class with values")).Add(Where.Ge("date1", new System.DateTime(correctDate.Millisecond))));
			l = odb.GetObjects<TestClass>(query);
			if (l.Count != 1)
			{
				query = new CriteriaQuery(Where.Equal("string1", "test class with null BigDecimal"
					));
				NeoDatis.Odb.Objects<TestClass> l2 = odb.GetObjects<TestClass>(query);
				Println(l2);
				Println(correctDate.Millisecond);
				l = odb.GetObjects<TestClass>(query);
			}
			AssertEquals(1, l.Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestSodaWithBoolean()
		{
			NeoDatis.Odb.ODB odb = Open(BaseName);
			CriteriaQuery query = new CriteriaQuery(Where.Equal("boolean1", true));
			NeoDatis.Odb.Objects<TestClass> l = odb.GetObjects<TestClass>(query);
			AssertTrue(l.Count > 1);
			query = new CriteriaQuery(Where.Equal("boolean1", true));
			l = odb.GetObjects<TestClass>(query);
			AssertTrue(l.Count > 1);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestSodaWithInt()
		{
			NeoDatis.Odb.ODB odb = Open(BaseName);
			CriteriaQuery query = new CriteriaQuery(Where.Equal("int1", 190));
			NeoDatis.Odb.Objects<TestClass> l = odb.GetObjects<TestClass>(query);
			AssertEquals(1, l.Count);
			query = new CriteriaQuery(Where.Gt("int1", 189));
			l = odb.GetObjects<TestClass>(query);
			AssertTrue(l.Count >= 1);
			query = new CriteriaQuery(Where.Lt("int1", 191));
			l = odb.GetObjects<TestClass>(query);
			AssertTrue(l.Count >= 1);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestSodaWithDouble()
		{
			NeoDatis.Odb.ODB odb = Open(BaseName);
			CriteriaQuery query = new CriteriaQuery(Where.Equal("double1", 190.99));
			NeoDatis.Odb.Objects<TestClass> l = odb.GetObjects<TestClass>(query);
			AssertEquals(1, l.Count);
			query = new CriteriaQuery(Where.Gt("double1", (double)189));
			l = odb.GetObjects<TestClass>(query);
			AssertTrue(l.Count >= 1);
			query = new CriteriaQuery(Where.Lt("double1", (double)191));
            l = odb.GetObjects<TestClass>(query);
			AssertTrue(l.Count >= 1);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestIsNull()
		{
			NeoDatis.Odb.ODB odb = null;
			try
			{
				odb = Open(BaseName);
				CriteriaQuery query = new CriteriaQuery(Where.IsNull("bigDecimal1"));
                NeoDatis.Odb.Objects<TestClass> l = odb.GetObjects<TestClass>(query);
				AssertEquals(2, l.Count);
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestIsNotNull()
		{
			NeoDatis.Odb.ODB odb = null;
			try
			{
				odb = Open(BaseName);
				CriteriaQuery query = new CriteriaQuery(Where.IsNotNull("bigDecimal1"));
                NeoDatis.Odb.Objects<TestClass> l = odb.GetObjects<TestClass>(query);
				AssertEquals(51, l.Count);
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
		}

		/// <exception cref="System.Exception"></exception>
		public override void SetUp()
		{
			base.SetUp();
			DeleteBase(BaseName);
			NeoDatis.Odb.ODB odb = Open(BaseName);
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			int size = 50;
			for (int i = 0; i < size; i++)
			{
				TestClass tc = new TestClass
					();
				tc.SetBigDecimal1(new System.Decimal(i));
				tc.SetBoolean1(i % 3 == 0);
				tc.SetChar1((char)(i % 5));
				tc.SetDate1(new System.DateTime(1000 + start + i));
				tc.SetDouble1(((double)(i % 10)) / size);
				tc.SetInt1(size - i);
				tc.SetString1("test class " + i);
				odb.Store(tc);
			}
			TestClass testClass = new TestClass
				();
			testClass.SetBigDecimal1(new System.Decimal(190.95));
			testClass.SetBoolean1(true);
			testClass.SetChar1('s');
			correctDate = new System.DateTime();
			testClass.SetDate1(correctDate);
			testClass.SetDouble1(190.99);
			testClass.SetInt1(190);
			testClass.SetString1("test class with values");
			odb.Store(testClass);
			TestClass testClass2 = new TestClass
				();
			testClass2.SetBigDecimal1(0);
			testClass2.SetBoolean1(true);
			testClass2.SetChar1('s');
			correctDate = new System.DateTime();
			testClass2.SetDate1(correctDate);
			testClass2.SetDouble1(191.99);
			testClass2.SetInt1(1901);
			testClass2.SetString1("test class with null BigDecimal");
			odb.Store(testClass2);
			TestClass testClass3 = new TestClass
				();
			odb.Store(testClass3);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public override void TearDown()
		{
			DeleteBase(BaseName);
		}
	}
}
