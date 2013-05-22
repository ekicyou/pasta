namespace NeoDatis.Odb.Test.Query.Criteria
{
	public class TestCriteriaQuery : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			NeoDatis.Odb.ODB odb = Open("criteria.neodatis");
			NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery aq = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function), NeoDatis.Odb.Core.Query.Criteria.Where
				.Or().Add(NeoDatis.Odb.Core.Query.Criteria.Where.Equal("name", "function 2")).Add
				(NeoDatis.Odb.Core.Query.Criteria.Where.Equal("name", "function 3")));
			NeoDatis.Odb.Objects l = odb.GetObjects(aq, true, -1, -1);
			AssertEquals(2, l.Count);
			NeoDatis.Odb.Test.VO.Login.Function f = (NeoDatis.Odb.Test.VO.Login.Function)l.GetFirst
				();
			AssertEquals("function 2", f.GetName());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test2()
		{
			NeoDatis.Odb.ODB odb = Open("criteria.neodatis");
			NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery aq = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function), NeoDatis.Odb.Core.Query.Criteria.Where
				.Not(NeoDatis.Odb.Core.Query.Criteria.Where.Equal("name", "function 2")));
			NeoDatis.Odb.Objects l = odb.GetObjects(aq, true, -1, -1);
			AssertEquals(49, l.Count);
			NeoDatis.Odb.Test.VO.Login.Function f = (NeoDatis.Odb.Test.VO.Login.Function)l.GetFirst
				();
			AssertEquals("function 0", f.GetName());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test3()
		{
			NeoDatis.Odb.ODB odb = Open("criteria.neodatis");
			NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery aq = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function), NeoDatis.Odb.Core.Query.Criteria.Where
				.Not(NeoDatis.Odb.Core.Query.Criteria.Where.Or().Add(NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("name", "function 2")).Add(NeoDatis.Odb.Core.Query.Criteria.Where.Equal("name"
				, "function 3"))));
			NeoDatis.Odb.Objects l = odb.GetObjects(aq, true, -1, -1);
			AssertEquals(48, l.Count);
			NeoDatis.Odb.Test.VO.Login.Function f = (NeoDatis.Odb.Test.VO.Login.Function)l.GetFirst
				();
			AssertEquals("function 0", f.GetName());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test4Sort()
		{
			int d = NeoDatis.Odb.OdbConfiguration.GetDefaultIndexBTreeDegree();
			try
			{
				NeoDatis.Odb.OdbConfiguration.SetDefaultIndexBTreeDegree(40);
				NeoDatis.Odb.ODB odb = Open("criteria.neodatis");
				NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery aq = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.VO.Login.Function), NeoDatis.Odb.Core.Query.Criteria.Where
					.Not(NeoDatis.Odb.Core.Query.Criteria.Where.Or().Add(NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("name", "function 2")).Add(NeoDatis.Odb.Core.Query.Criteria.Where.Equal("name"
					, "function 3"))));
				aq.OrderByDesc("name");
				// aq.orderByAsc("name");
				NeoDatis.Odb.Objects l = odb.GetObjects(aq, true, -1, -1);
				AssertEquals(48, l.Count);
				NeoDatis.Odb.Test.VO.Login.Function f = (NeoDatis.Odb.Test.VO.Login.Function)l.GetFirst
					();
				AssertEquals("function 9", f.GetName());
				odb.Close();
			}
			finally
			{
				NeoDatis.Odb.OdbConfiguration.SetDefaultIndexBTreeDegree(d);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestDate1()
		{
			NeoDatis.Odb.ODB odb = Open("criteria.neodatis");
			NeoDatis.Odb.Test.Query.Criteria.MyDates myDates = new NeoDatis.Odb.Test.Query.Criteria.MyDates
				();
			System.DateTime d1 = new System.DateTime();
			Java.Lang.Thread.Sleep(100);
			System.DateTime d2 = new System.DateTime();
			Java.Lang.Thread.Sleep(100);
			System.DateTime d3 = new System.DateTime();
			myDates.SetDate1(d1);
			myDates.SetDate2(d3);
			myDates.SetI(5);
			odb.Store(myDates);
			odb.Close();
			odb = Open("criteria.neodatis");
			NeoDatis.Odb.Core.Query.IQuery query = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.Query.Criteria.MyDates), NeoDatis.Odb.Core.Query.Criteria.Where
				.And().Add(NeoDatis.Odb.Core.Query.Criteria.Where.Le("date1", d2)).Add(NeoDatis.Odb.Core.Query.Criteria.Where
				.Ge("date2", d2)).Add(NeoDatis.Odb.Core.Query.Criteria.Where.Equal("i", 5)));
			NeoDatis.Odb.Objects objects = odb.GetObjects(query);
			AssertEquals(1, objects.Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestIequal()
		{
			NeoDatis.Odb.ODB odb = Open("criteria.neodatis");
			NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery aq = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function), NeoDatis.Odb.Core.Query.Criteria.Where
				.Iequal("name", "FuNcTiOn 1"));
			aq.OrderByDesc("name");
			NeoDatis.Odb.Objects l = odb.GetObjects(aq, true, -1, -1);
			AssertEquals(1, l.Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestEqual2()
		{
			NeoDatis.Odb.ODB odb = Open("criteria.neodatis");
			NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery aq = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("name", "FuNcTiOn 1"));
			aq.OrderByDesc("name");
			NeoDatis.Odb.Objects l = odb.GetObjects(aq, true, -1, -1);
			AssertEquals(0, l.Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestILike()
		{
			NeoDatis.Odb.ODB odb = Open("criteria.neodatis");
			NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery aq = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function), NeoDatis.Odb.Core.Query.Criteria.Where
				.Ilike("name", "FUNc%"));
			aq.OrderByDesc("name");
			NeoDatis.Odb.Objects l = odb.GetObjects(aq, true, -1, -1);
			AssertEquals(50, l.Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestLike()
		{
			NeoDatis.Odb.ODB odb = Open("criteria.neodatis");
			NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery aq = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function), NeoDatis.Odb.Core.Query.Criteria.Where
				.Like("name", "func%"));
			aq.OrderByDesc("name");
			NeoDatis.Odb.Objects l = odb.GetObjects(aq, true, -1, -1);
			AssertEquals(50, l.Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestLike2()
		{
			NeoDatis.Odb.ODB odb = Open("criteria.neodatis");
			NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery aq = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function), NeoDatis.Odb.Core.Query.Criteria.Where
				.Like("name", "FuNc%"));
			aq.OrderByDesc("name");
			NeoDatis.Odb.Objects l = odb.GetObjects(aq, true, -1, -1);
			AssertEquals(0, l.Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public override void SetUp()
		{
			base.SetUp();
			DeleteBase("criteria.neodatis");
			NeoDatis.Odb.ODB odb = Open("criteria.neodatis");
			for (int i = 0; i < 50; i++)
			{
				odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("function " + i));
			}
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public override void TearDown()
		{
			DeleteBase("criteria.neodatis");
		}
	}
}
