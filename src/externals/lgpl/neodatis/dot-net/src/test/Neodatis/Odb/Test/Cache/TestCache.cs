using System.Threading;
using NeoDatis.Odb.Impl.Core.Query.Criteria;
using NeoDatis.Odb.Test.VO.Login;
using NeoDatis.Odb.Core.Query.Criteria;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.Cache
{
	[TestFixture]
    public class TestCache : NeoDatis.Odb.Test.ODBTest
	{
		public static int NbObjects = 300;

		/// <exception cref="System.Exception"></exception>
		public override void SetUp()
		{
			Thread.Sleep(100);
			// Configuration.setUseModifiedClass(true);
			DeleteBase("cache.neodatis");
			NeoDatis.Odb.ODB odb = Open("cache.neodatis");
			for (int i = 0; i < NbObjects; i++)
			{
				odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("function " + (i + i)));
				odb.Store(new NeoDatis.Odb.Test.VO.Login.User("olivier " + i, "olivier@neodatis.com "
					 + i, new NeoDatis.Odb.Test.VO.Login.Profile("profile " + i, new NeoDatis.Odb.Test.VO.Login.Function
					("inner function " + i))));
			}
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			NeoDatis.Odb.ODB odb = Open("cache.neodatis");
			NeoDatis.Odb.Objects<Function> l = odb.GetObjects<Function> (new CriteriaQuery(Where.Equal("name", "function 10")));
			AssertFalse(l.Count==0);
			// Cache must have only one object : The function
			AssertEquals(l.Count, NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy.GetEngine
				(odb).GetSession(true).GetCache().GetNumberOfObjects());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test2()
		{
			NeoDatis.Odb.ODB odb = Open("cache.neodatis");
			NeoDatis.Odb.Objects<User> l = odb.GetObjects<User>(new CriteriaQuery(Where.Equal("name", "olivier 10")));
			AssertFalse(l.Count==0);
			// Cache must have 3 times the number of Users in list l (check the
			// setup method to understand this)
			AssertEquals(l.Count * 3, NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy.GetEngine
				(odb).GetSession(true).GetCache().GetNumberOfObjects());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public override void TearDown()
		{
			DeleteBase("cache.neodatis");
		}
	}
}
