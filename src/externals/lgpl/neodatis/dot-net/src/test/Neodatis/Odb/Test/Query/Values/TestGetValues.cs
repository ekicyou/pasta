using NUnit.Framework;
namespace NeoDatis.Odb.Test.Query.Values
{
	[TestFixture]
    public class TestGetValues : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			DeleteBase("valuesA");
			NeoDatis.Odb.ODB odb = Open("valuesA");
			odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("f1"));
			odb.Close();
			odb = Open("valuesA");
			NeoDatis.Odb.Values values = odb.GetValues(new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function)).Field("name"));
			Println(values);
			NeoDatis.Odb.ObjectValues ov = values.NextValues();
			odb.Close();
			AssertEquals("f1", ov.GetByAlias("name"));
			AssertEquals("f1", ov.GetByIndex(0));
		}

		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test2()
		{
			DeleteBase("valuesA");
			NeoDatis.Odb.ODB odb = Open("valuesA");
			odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("f1"));
			odb.Close();
			odb = Open("valuesA");
			NeoDatis.Odb.Values values = odb.GetValues(new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function)).Field("name", "Alias of the field"
				));
			Println(values);
			NeoDatis.Odb.ObjectValues ov = values.NextValues();
			odb.Close();
			AssertEquals("f1", ov.GetByAlias("Alias of the field"));
			AssertEquals("f1", ov.GetByIndex(0));
		}

		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test3()
		{
			DeleteBase("valuesA2");
			NeoDatis.Odb.ODB odb = Open("valuesA2");
			odb.Store(new NeoDatis.Odb.Test.VO.Login.User("user1", "email1", new NeoDatis.Odb.Test.VO.Login.Profile
				("profile name", new NeoDatis.Odb.Test.VO.Login.Function("f111"))));
			odb.Close();
			odb = Open("valuesA2");
			NeoDatis.Odb.Values values = odb.GetValues(new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.User)).Field("name").Field("profile.name"));
			Println(values);
			NeoDatis.Odb.ObjectValues ov = values.NextValues();
			odb.Close();
			AssertEquals("user1", ov.GetByAlias("name"));
			AssertEquals("user1", ov.GetByIndex(0));
			AssertEquals("profile name", ov.GetByAlias("profile.name"));
			AssertEquals("profile name", ov.GetByIndex(1));
		}

		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test4()
		{
			DeleteBase("valuesA2");
			NeoDatis.Odb.ODB odb = Open("valuesA2");
			NeoDatis.Odb.Test.VO.Attribute.TestClass tc1 = new NeoDatis.Odb.Test.VO.Attribute.TestClass
				();
			tc1.SetInt1(45);
			odb.Store(tc1);
			NeoDatis.Odb.Test.VO.Attribute.TestClass tc2 = new NeoDatis.Odb.Test.VO.Attribute.TestClass
				();
			tc2.SetInt1(5);
			odb.Store(tc2);
			odb.Close();
			odb = Open("valuesA2");
			NeoDatis.Odb.Values values = odb.GetValues(new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Attribute.TestClass)).Sum("int1", "sum of int1").Count
				("nb objects"));
			Println(values);
			NeoDatis.Odb.ObjectValues ov = values.NextValues();
			odb.Close();
			AssertEquals(50, ov.GetByAlias("sum of int1"));
			AssertEquals(2, ov.GetByAlias("nb objects"));
			AssertEquals(1, values.Count);
		}

		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test5()
		{
			DeleteBase("valuesA2");
			NeoDatis.Odb.ODB odb = Open("valuesA2");
			int size = isLocal ? 100000 : 1000;
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.VO.Attribute.TestClass tc1 = new NeoDatis.Odb.Test.VO.Attribute.TestClass
					();
				tc1.SetInt1(45);
				odb.Store(tc1);
			}
			odb.Close();
			odb = Open("valuesA2");
			NeoDatis.Odb.Values values = odb.GetValues(new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Attribute.TestClass)).Count("nb objects"));
			Println(values);
			NeoDatis.Odb.ObjectValues ov = values.NextValues();
			odb.Close();
			AssertEquals(size, ov.GetByAlias("nb objects"));
			AssertEquals(1, values.Count);
		}

		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test6()
		{
			DeleteBase("valuesA2");
			NeoDatis.Odb.ODB odb = Open("valuesA2");
			int size = isLocal ? 100000 : 1000;
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.VO.Attribute.TestClass tc1 = new NeoDatis.Odb.Test.VO.Attribute.TestClass
					();
				tc1.SetInt1(i);
				odb.Store(tc1);
			}
			odb.Close();
			odb = Open("valuesA2");
			NeoDatis.Odb.Values values = odb.GetValues(new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Attribute.TestClass), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("int1", 2)).Count("nb objects"));
			Println(values);
			NeoDatis.Odb.ObjectValues ov = values.NextValues();
			odb.Close();
			AssertEquals(1, ov.GetByAlias("nb objects"));
			AssertEquals(1, values.Count);
		}

		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test7()
		{
			DeleteBase("valuesA2");
			NeoDatis.Odb.ODB odb = Open("valuesA2");
			int size = isLocal ? 100000 : 1000;
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.VO.Attribute.TestClass tc1 = new NeoDatis.Odb.Test.VO.Attribute.TestClass
					();
				tc1.SetInt1(i);
				odb.Store(tc1);
			}
			odb.Close();
			odb = Open("valuesA2");
			decimal nb = odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(
				NeoDatis.Odb.Test.VO.Attribute.TestClass), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("int1", 2)));
			Println(nb);
			odb.Close();
			AssertEquals(1, nb);
		}

		/// <summary>Max and average</summary>
		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test8()
		{
			DeleteBase("valuesA2");
			NeoDatis.Odb.ODB odb = Open("valuesA2");
			int size = isLocal ? 100000 : 1000;
			long sum = 0;
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.VO.Attribute.TestClass tc1 = new NeoDatis.Odb.Test.VO.Attribute.TestClass
					();
				tc1.SetInt1(i);
				odb.Store(tc1);
				sum += i;
			}
			odb.Close();
			odb = Open("valuesA2");
			NeoDatis.Odb.Values values = odb.GetValues(new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Attribute.TestClass)).Max("int1", "max of int1").Avg
				("int1", "avg of int1").Sum("int1", "sum of int1"));
			NeoDatis.Odb.ObjectValues ov = values.NextValues();
			System.Decimal max = (System.Decimal)ov.GetByAlias("max of int1");
			System.Decimal avg = (System.Decimal)ov.GetByAlias("avg of int1");
			System.Decimal bsum = (System.Decimal)ov.GetByAlias("sum of int1");
			Println(max);
			Println(avg);
			Println(bsum);
			odb.Close();
			AssertEquals(new System.Decimal(sum), bsum);
			AssertEquals(new System.Decimal(size - 1), max);
			AssertEquals(bsum/size, avg);
		}

		/// <summary>Min</summary>
		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test9()
		{
			DeleteBase("valuesA2");
			NeoDatis.Odb.ODB odb = Open("valuesA2");
			int size = isLocal ? 100000 : 1000;
			long sum = 0;
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.VO.Attribute.TestClass tc1 = new NeoDatis.Odb.Test.VO.Attribute.TestClass
					();
				tc1.SetInt1(i);
				odb.Store(tc1);
				sum += i;
			}
			odb.Close();
			odb = Open("valuesA2");
			NeoDatis.Odb.Values values = odb.GetValues(new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Attribute.TestClass)).Min("int1", "min of int1").Avg
				("int1", "avg of int1").Sum("int1", "sum of int1"));
			NeoDatis.Odb.ObjectValues ov = values.NextValues();
			System.Decimal min = (System.Decimal)ov.GetByAlias("min of int1");
			System.Decimal avg = (System.Decimal)ov.GetByAlias("avg of int1");
			System.Decimal bsum = (System.Decimal)ov.GetByAlias("sum of int1");
			Println(min);
			Println(avg);
			Println(bsum);
			odb.Close();
			AssertEquals(new System.Decimal(sum), bsum);
			AssertEquals(new System.Decimal(0), min);
			AssertEquals(bsum/2, avg);
		}

		/// <summary>Custom</summary>
		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test10()
		{
			DeleteBase("valuesA2");
			NeoDatis.Odb.ODB odb = Open("valuesA2");
			NeoDatis.Tool.StopWatch t = new NeoDatis.Tool.StopWatch();
			t.Start();
			int size = 1000;
			long sum = 0;
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.VO.Attribute.TestClass tc1 = new NeoDatis.Odb.Test.VO.Attribute.TestClass
					();
				tc1.SetInt1(i);
				odb.Store(tc1);
				sum += i;
			}
			odb.Close();
			t.End();
			Println(" time for insert = " + t.GetDurationInMiliseconds());
			odb = Open("valuesA2");
			t.Start();
			NeoDatis.Odb.Core.Query.Values.ICustomQueryFieldAction custom = new NeoDatis.Odb.Test.Query.Values.TestCustomQueryFieldAction
				();
			NeoDatis.Odb.Values values = odb.GetValues(new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Attribute.TestClass)).Custom("int1", "custom of int1"
				, custom));
			t.End();
			NeoDatis.Odb.ObjectValues ov = values.NextValues();
			System.Decimal c = (System.Decimal)ov.GetByAlias("custom of int1");
			Println(c);
			Println(" time for count = " + t.GetDurationInMiliseconds());
			odb.Close();
		}

		// assertEquals(bsum.divide(new
		// BigDecimal(size),2,BigDecimal.ROUND_HALF_DOWN), avg);
		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test16()
		{
			DeleteBase("valuesA2");
			NeoDatis.Odb.ODB odb = null;
			NeoDatis.Tool.StopWatch t = new NeoDatis.Tool.StopWatch();
			int size = isLocal ? 10000 : 100;
			for (int j = 0; j < 10; j++)
			{
				t.Start();
				odb = Open("valuesA2");
				for (int i = 0; i < size; i++)
				{
					NeoDatis.Odb.Test.VO.Attribute.TestClass tc1 = new NeoDatis.Odb.Test.VO.Attribute.TestClass
						();
					tc1.SetInt1(45);
					odb.Store(tc1);
				}
				odb.Close();
				t.End();
				Println(" time for insert = " + t.GetDurationInMiliseconds());
			}
			odb = Open("valuesA2");
			t.Start();
			NeoDatis.Odb.Values values = odb.GetValues(new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Attribute.TestClass)).Count("nb objects"));
			t.End();
			Println(values);
			Println(" time for count = " + t.GetDurationInMiliseconds());
			NeoDatis.Odb.ObjectValues ov = values.NextValues();
			odb.Close();
			AssertEquals(size * 10, ov.GetByAlias("nb objects"));
			AssertEquals(1, values.Count);
		}

		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test17()
		{
			DeleteBase("valuesA2");
			NeoDatis.Odb.ODB odb = Open("valuesA2");
			odb.Store(new NeoDatis.Odb.Test.VO.Login.User("user1", "email1", new NeoDatis.Odb.Test.VO.Login.Profile
				("profile name", new NeoDatis.Odb.Test.VO.Login.Function("f111"))));
			odb.Close();
			odb = Open("valuesA2");
			NeoDatis.Odb.Values values = odb.GetValues(new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.User)).Field("name").Field("profile"));
			Println(values);
			NeoDatis.Odb.ObjectValues ov = values.NextValues();
			odb.Close();
			AssertEquals("user1", ov.GetByAlias("name"));
			AssertEquals("user1", ov.GetByIndex(0));
			NeoDatis.Odb.Test.VO.Login.Profile p2 = (NeoDatis.Odb.Test.VO.Login.Profile)ov.GetByAlias
				("profile");
			AssertEquals("profile name", p2.GetName());
		}
	}
}
