using NeoDatis.Odb.Test.VO.Login;
using NeoDatis.Odb.Impl.Core.Query.Criteria;
using NeoDatis.Odb.Core.Query.Criteria;
using System.Threading;
using NUnit.Framework;
using System;
namespace NeoDatis.Odb.Test.Query.Criteria
{
	[TestFixture]
    public class TestCriteriaQuery : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
            string baseName = GetBaseName();
            SetUp(baseName);
			NeoDatis.Odb.ODB odb = Open(baseName);
			CriteriaQuery aq = new CriteriaQuery(Where
				.Or().Add(Where.Equal("name", "function 2")).Add
				(Where.Equal("name", "function 3")));
			NeoDatis.Odb.Objects<Function> l = odb.GetObjects<Function>(aq, true, -1, -1);
			AssertEquals(2, l.Count);
			Function f = l.GetFirst
				();
			AssertEquals("function 2", f.GetName());
            Println(l);
			odb.Close();
            Console.ReadLine();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test2()
		{
            string baseName = GetBaseName();
            SetUp(baseName);
			NeoDatis.Odb.ODB odb = Open(baseName);
			CriteriaQuery aq = new CriteriaQuery
				(typeof(Function), Where
				.Not(Where.Equal("name", "function 2")));
            NeoDatis.Odb.Objects<Function> l = odb.GetObjects<Function>(aq, true, -1, -1);
			AssertEquals(49, l.Count);
			Function f = (Function)l.GetFirst
				();
			AssertEquals("function 0", f.GetName());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test3()
		{
            string baseName = GetBaseName();
            SetUp(baseName);
			NeoDatis.Odb.ODB odb = Open(baseName);
			CriteriaQuery aq = new CriteriaQuery
				(typeof(Function), Where
				.Not(Where.Or().Add(Where
				.Equal("name", "function 2")).Add(Where.Equal("name"
				, "function 3"))));
			NeoDatis.Odb.Objects<Function> l = odb.GetObjects<Function>(aq, true, -1, -1);
			AssertEquals(48, l.Count);
			Function f = (Function)l.GetFirst
				();
			AssertEquals("function 0", f.GetName());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test4Sort()
		{
            string baseName = GetBaseName();
            SetUp(baseName);
			int d = NeoDatis.Odb.OdbConfiguration.GetDefaultIndexBTreeDegree();
			try
			{
				NeoDatis.Odb.OdbConfiguration.SetDefaultIndexBTreeDegree(40);
				NeoDatis.Odb.ODB odb = Open(baseName);
				CriteriaQuery aq = new CriteriaQuery
					(typeof(Function), Where
					.Not(Where.Or().Add(Where
					.Equal("name", "function 2")).Add(Where.Equal("name"
					, "function 3"))));
				aq.OrderByDesc("name");
				// aq.orderByAsc("name");
				NeoDatis.Odb.Objects<Function> l = odb.GetObjects<Function>(aq, true, -1, -1);
                odb.Close();

				AssertEquals(48, l.Count);
				Function f = (Function)l.GetFirst
					();
				AssertEquals("function 9", f.GetName());
			}
			finally
			{
				NeoDatis.Odb.OdbConfiguration.SetDefaultIndexBTreeDegree(d);
			}
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestDate1()
		{
            string baseName = GetBaseName();
            SetUp(baseName);
			NeoDatis.Odb.ODB odb = Open(baseName);
			NeoDatis.Odb.Test.Query.Criteria.MyDates myDates = new NeoDatis.Odb.Test.Query.Criteria.MyDates
				();
			System.DateTime d1 = new System.DateTime();
            Thread.Sleep(100);
			
			System.DateTime d2 = new System.DateTime();
            Thread.Sleep(100);
			System.DateTime d3 = new System.DateTime();
			myDates.SetDate1(d1);
			myDates.SetDate2(d3);
			myDates.SetI(5);
			odb.Store(myDates);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery query = new CriteriaQuery
				(typeof(MyDates), Where
				.And().Add(Where.Le("date1", d2)).Add(Where
				.Ge("date2", d2)).Add(Where.Equal("i", 5)));
			NeoDatis.Odb.Objects<MyDates> objects = odb.GetObjects<MyDates>(query);
			AssertEquals(1, objects.Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestIequal()
		{

            string baseName = GetBaseName();
            SetUp(baseName);
			NeoDatis.Odb.ODB odb = Open(baseName);
			CriteriaQuery aq = new CriteriaQuery
				(typeof(Function), Where
				.Iequal("name", "FuNcTiOn 1"));
			aq.OrderByDesc("name");
            NeoDatis.Odb.Objects<Function> l = odb.GetObjects<Function>(aq, true, -1, -1);
			AssertEquals(1, l.Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestEqual2()
		{
            string baseName = GetBaseName();
            SetUp(baseName);
			NeoDatis.Odb.ODB odb = Open(baseName);
			CriteriaQuery aq = new CriteriaQuery
				(typeof(Function), Where
				.Equal("name", "FuNcTiOn 1"));
			aq.OrderByDesc("name");
            NeoDatis.Odb.Objects<Function> l = odb.GetObjects<Function>(aq, true, -1, -1);
			AssertEquals(0, l.Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestILike()
		{
            string baseName = GetBaseName();
            SetUp(baseName);
			NeoDatis.Odb.ODB odb = Open(baseName);
			CriteriaQuery aq = new CriteriaQuery
				(typeof(Function), Where
				.Ilike("name", "FUNc%"));
			aq.OrderByDesc("name");
            NeoDatis.Odb.Objects<Function> l = odb.GetObjects<Function>(aq, true, -1, -1);
			AssertEquals(50, l.Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestLike()
		{
            string baseName = GetBaseName();
            SetUp(baseName);
			NeoDatis.Odb.ODB odb = Open(baseName);
			CriteriaQuery aq = new CriteriaQuery
				(typeof(Function), Where
				.Like("name", "func%"));
			aq.OrderByDesc("name");
            NeoDatis.Odb.Objects<Function> l = odb.GetObjects<Function>(aq, true, -1, -1);
			AssertEquals(50, l.Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestLike2()
		{
            string baseName = GetBaseName();
            SetUp(baseName);
			NeoDatis.Odb.ODB odb = Open(baseName);
			CriteriaQuery aq = new CriteriaQuery
				(typeof(Function), Where
				.Like("name", "FuNc%"));
			aq.OrderByDesc("name");
            NeoDatis.Odb.Objects<Function> l = odb.GetObjects<Function>(aq, true, -1, -1);
			AssertEquals(0, l.Count);
			odb.Close();
		}

		public void SetUp(string baseName)
		{
			base.SetUp();
			DeleteBase(baseName);
			NeoDatis.Odb.ODB odb = Open(baseName);
			for (int i = 0; i < 50; i++)
			{
				odb.Store(new Function("function " + i));
			}
			odb.Close();
		}

		
		public void TearDown(string baseName)
		{
			DeleteBase(baseName);
		}
	}
}
