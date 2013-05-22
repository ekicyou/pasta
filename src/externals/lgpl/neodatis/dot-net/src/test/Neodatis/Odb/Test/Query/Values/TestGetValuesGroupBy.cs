using NUnit.Framework;
namespace NeoDatis.Odb.Test.Query.Values
{
	[TestFixture]
    public class TestGetValuesGroupBy : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			DeleteBase("values2");
			NeoDatis.Odb.ODB odb = Open("values2");
			NeoDatis.Odb.Test.VO.Attribute.TestClass tc1 = new NeoDatis.Odb.Test.VO.Attribute.TestClass
				();
			tc1.SetInt1(45);
			odb.Store(tc1);
			NeoDatis.Odb.Test.VO.Attribute.TestClass tc2 = new NeoDatis.Odb.Test.VO.Attribute.TestClass
				();
			tc2.SetInt1(45);
			odb.Store(tc2);
			NeoDatis.Odb.Test.VO.Attribute.TestClass tc3 = new NeoDatis.Odb.Test.VO.Attribute.TestClass
				();
			tc3.SetInt1(46);
			odb.Store(tc3);
			odb.Close();
			odb = Open("values2");
			NeoDatis.Odb.Core.Query.IValuesQuery vq = new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Attribute.TestClass)).Sum("int1", "sum of int1").GroupBy
				("int1");
			vq.OrderByAsc("int1");
			NeoDatis.Odb.Values values = odb.GetValues(vq);
			AssertEquals(2, values.Count);
			Println(values);
			NeoDatis.Odb.ObjectValues ov = values.NextValues();
			AssertEquals(System.Decimal.ValueOf(90), ov.GetByAlias("sum of int1"));
			ov = values.NextValues();
			AssertEquals(System.Decimal.ValueOf(46), ov.GetByAlias("sum of int1"));
			odb.Close();
			AssertEquals(2, values.Count);
		}

		/// <exception cref="System.IO.IOException"></exception>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test2()
		{
			DeleteBase("values2");
			NeoDatis.Odb.ODB odb = Open("values2");
			NeoDatis.Odb.Test.VO.Attribute.TestClass tc1 = new NeoDatis.Odb.Test.VO.Attribute.TestClass
				();
			tc1.SetInt1(45);
			odb.Store(tc1);
			NeoDatis.Odb.Test.VO.Attribute.TestClass tc2 = new NeoDatis.Odb.Test.VO.Attribute.TestClass
				();
			tc2.SetInt1(45);
			odb.Store(tc2);
			NeoDatis.Odb.Test.VO.Attribute.TestClass tc3 = new NeoDatis.Odb.Test.VO.Attribute.TestClass
				();
			tc3.SetInt1(46);
			odb.Store(tc3);
			odb.Close();
			odb = Open("values2");
			NeoDatis.Odb.Core.Query.IValuesQuery vq = new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Attribute.TestClass)).Sum("int1", "sum of int1").Count
				("count").GroupBy("int1");
			vq.OrderByAsc("int1");
			NeoDatis.Odb.Values values = odb.GetValues(vq);
			Println(values);
			NeoDatis.Odb.ObjectValues ov = values.NextValues();
			AssertEquals(System.Decimal.ValueOf(90), ov.GetByAlias("sum of int1"));
			AssertEquals(System.Decimal.Parse(2), ov.GetByAlias("count"));
			ov = values.NextValues();
			AssertEquals(System.Decimal.ValueOf(46), ov.GetByAlias("sum of int1"));
			AssertEquals(System.Decimal.Parse(1), ov.GetByAlias("count"));
			odb.Close();
			AssertEquals(2, values.Count);
		}

		/// <summary>
		/// Retrieving the name of the profile, the number of user for that profile
		/// and their average login number grouped by the name of the profile
		/// </summary>
		/// <exception cref="System.IO.IOException">System.IO.IOException</exception>
		/// <exception cref="System.Exception">System.Exception</exception>
		[Test]
        public virtual void Test3()
		{
			DeleteBase("values2");
			NeoDatis.Odb.ODB odb = Open("values2");
			NeoDatis.Odb.Test.VO.Login.Profile p1 = new NeoDatis.Odb.Test.VO.Login.Profile("profile1"
				, new NeoDatis.Odb.Test.VO.Login.Function("f1"));
			NeoDatis.Odb.Test.VO.Login.Profile p2 = new NeoDatis.Odb.Test.VO.Login.Profile("profile2"
				, new NeoDatis.Odb.Test.VO.Login.Function("f2"));
			NeoDatis.Odb.Test.VO.Login.User u1 = new NeoDatis.Odb.Test.VO.Login.User2("user1"
				, "user@neodatis.org", p1, 1);
			NeoDatis.Odb.Test.VO.Login.User u2 = new NeoDatis.Odb.Test.VO.Login.User2("user2"
				, "user@neodatis.org", p1, 2);
			NeoDatis.Odb.Test.VO.Login.User u3 = new NeoDatis.Odb.Test.VO.Login.User2("user3"
				, "user@neodatis.org", p1, 3);
			NeoDatis.Odb.Test.VO.Login.User u4 = new NeoDatis.Odb.Test.VO.Login.User2("user4"
				, "user@neodatis.org", p2, 4);
			NeoDatis.Odb.Test.VO.Login.User u5 = new NeoDatis.Odb.Test.VO.Login.User2("user5"
				, "user@neodatis.org", p2, 5);
			odb.Store(u1);
			odb.Store(u2);
			odb.Store(u3);
			odb.Store(u4);
			odb.Store(u5);
			odb.Close();
			odb = Open("values2");
			NeoDatis.Odb.Core.Query.IValuesQuery q = new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.User2)).Field("profile.name").Count("count").
				Avg("nbLogins", "avg").GroupBy("profile.name");
			q.OrderByAsc("name");
			NeoDatis.Odb.Values values = odb.GetValues(q);
			Println(values);
			NeoDatis.Odb.ObjectValues ov = values.NextValues();
			AssertEquals(2, values.Count);
			AssertEquals("profile1", ov.GetByAlias("profile.name"));
			AssertEquals(new System.Decimal("3"), ov.GetByAlias("count"));
			AssertEquals(new System.Decimal("2.00"), ov.GetByAlias("avg"));
			odb.Close();
			AssertEquals(2, values.Count);
		}
	}
}
