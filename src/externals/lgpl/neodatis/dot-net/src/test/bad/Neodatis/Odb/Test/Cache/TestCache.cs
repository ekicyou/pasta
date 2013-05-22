namespace NeoDatis.Odb.Test.Cache
{
	public class TestCache : NeoDatis.Odb.Test.ODBTest
	{
		public static int NbObjects = 300;

		/// <exception cref="System.Exception"></exception>
		public override void SetUp()
		{
			Java.Lang.Thread.Sleep(100);
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
		public virtual void Test1()
		{
			NeoDatis.Odb.ODB odb = Open("cache.neodatis");
			NeoDatis.Odb.Objects l = odb.GetObjects(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("name", "function 10")));
			AssertFalse(l.IsEmpty());
			// Cache must have only one object : The function
			AssertEquals(l.Count, NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy.GetEngine
				(odb).GetSession(true).GetCache().GetNumberOfObjects());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test2()
		{
			NeoDatis.Odb.ODB odb = Open("cache.neodatis");
			NeoDatis.Odb.Objects l = odb.GetObjects(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.User), NeoDatis.Odb.Core.Query.Criteria.Where
				.Equal("name", "olivier 10")));
			AssertFalse(l.IsEmpty());
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
