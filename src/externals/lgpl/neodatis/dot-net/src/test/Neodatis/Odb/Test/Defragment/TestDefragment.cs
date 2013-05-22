using NUnit.Framework;
namespace NeoDatis.Odb.Test.Defragment
{
	[TestFixture]
    public class TestDefragment : NeoDatis.Odb.Test.ODBTest
	{
		/// <summary>The name of the database file</summary>
		public static readonly string OdbFileName1 = "defrag1.neodatis";

		public static readonly string OdbFileName2 = "defrag1-bis.neodatis";

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			if (!isLocal)
			{
				return;
			}
			DeleteBase(OdbFileName1);
			DeleteBase(OdbFileName2);
			NeoDatis.Odb.ODB odb = Open(OdbFileName1);
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier"
				, "olivier@neodatis.com", null);
			odb.Store(user);
			odb.Close();
			odb = Open(OdbFileName1);
			odb.DefragmentTo(Directory + OdbFileName2);
			NeoDatis.Odb.ODB newOdb = Open(OdbFileName2);
			// int n = odb.getObjects(User.class).size();
			// println("n="+n);
			System.Decimal nbUser = odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.User)));
			System.Decimal nbNewUser = newOdb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.User)));
			AssertEquals(nbUser, nbNewUser);
			AssertEquals(odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(
				NeoDatis.Odb.Test.VO.Login.Profile))), newOdb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Profile))));
			odb.Close();
			newOdb.Close();
			DeleteBase(OdbFileName1);
			DeleteBase(OdbFileName2);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test2()
		{
			if (!isLocal)
			{
				return;
			}
			DeleteBase(OdbFileName1);
			DeleteBase(OdbFileName2);
			NeoDatis.Odb.ODB odb = Open(OdbFileName1);
			NeoDatis.Odb.Test.VO.Login.Profile p = new NeoDatis.Odb.Test.VO.Login.Profile("profile"
				);
			for (int i = 0; i < 500; i++)
			{
				NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier "
					 + i, "olivier@neodatis.com " + i, p);
				odb.Store(user);
			}
			odb.Close();
			odb = Open(OdbFileName1);
			odb.DefragmentTo(Directory + OdbFileName2);
			NeoDatis.Odb.ODB newOdb = Open(OdbFileName2);
			AssertEquals(odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(
				NeoDatis.Odb.Test.VO.Login.User))), newOdb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.User))));
			AssertEquals(odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(
				NeoDatis.Odb.Test.VO.Login.Profile))), newOdb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Profile))));
			odb.Close();
			newOdb.Close();
			DeleteBase(OdbFileName1);
			DeleteBase(OdbFileName2);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test3()
		{
			if (!isLocal)
			{
				return;
			}
			DeleteBase(OdbFileName1);
			DeleteBase(OdbFileName2);
			NeoDatis.Odb.OdbConfiguration.SetAutomaticallyIncreaseCacheSize(true);
			NeoDatis.Odb.ODB odb = Open(OdbFileName1);
			for (int i = 0; i < 15000; i++)
			{
				NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier "
					 + i, "olivier@neodatis.com " + i, new NeoDatis.Odb.Test.VO.Login.Profile("profile"
					 + i));
				odb.Store(user);
			}
			odb.Close();
			odb = Open(OdbFileName1);
			odb.DefragmentTo(Directory + OdbFileName2);
			NeoDatis.Odb.ODB newOdb = Open(OdbFileName2);
			AssertEquals(odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(
				NeoDatis.Odb.Test.VO.Login.User))), newOdb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.User))));
			odb.Close();
			newOdb.Close();
			DeleteBase(OdbFileName1);
			DeleteBase(OdbFileName2);
		}
	}
}
