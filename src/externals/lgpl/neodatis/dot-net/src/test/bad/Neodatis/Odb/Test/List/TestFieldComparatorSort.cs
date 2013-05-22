namespace NeoDatis.Odb.Test.List
{
	public class TestFieldComparatorSort : NeoDatis.Odb.Test.ODBTest
	{
		public virtual void Test1()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = null;
			int k = 10;
			long t1 = Sharpen.Runtime.CurrentTimeMillis();
			odb = Open(baseName);
			for (int i = 0; i < k; i++)
			{
				odb.Store(new NeoDatis.Odb.Test.List.User("john" + (k - i), "john@ibm.com", "ny 875"
					, k - i + 1, new System.DateTime(t1 - i * 1000), i % 2 == 0));
				odb.Store(new NeoDatis.Odb.Test.List.User("john" + (k - i), "john@ibm.com", "ny 875"
					, k - i, new System.DateTime(t1 - i * 1000), (i + 1) % 2 == 0));
			}
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.List.User)).OrderByAsc("name,id");
			NeoDatis.Odb.Objects<NeoDatis.Odb.Test.List.User> users = odb.GetObjects(q);
			odb.Close();
			if (k < 11)
			{
				NeoDatis.Tool.DisplayUtility.Display("test1", users);
			}
			NeoDatis.Odb.Test.List.User user = users.GetFirst();
			AssertTrue(user.GetName().StartsWith("john1"));
			AssertEquals(1, user.GetId());
		}

		public virtual void Test1_2()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = null;
			int k = 10;
			long t1 = Sharpen.Runtime.CurrentTimeMillis();
			odb = Open(baseName);
			for (int i = 0; i < k; i++)
			{
				odb.Store(new NeoDatis.Odb.Test.List.User("john" + (k - i), "john@ibm.com", "ny 875"
					, k - i + 1, new System.DateTime(t1 - i * 1000), i % 2 == 0));
				odb.Store(new NeoDatis.Odb.Test.List.User("john" + (k - i), "john@ibm.com", "ny 875"
					, k - i, new System.DateTime(t1 - i * 1000), (i + 1) % 2 == 0));
			}
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.List.User)).OrderByDesc("name,id");
			NeoDatis.Odb.Objects<NeoDatis.Odb.Test.List.User> users = odb.GetObjects(q);
			odb.Close();
			if (k < 11)
			{
				NeoDatis.Tool.DisplayUtility.Display("test1", users);
			}
			NeoDatis.Odb.Test.List.User user = users.GetFirst();
			AssertTrue(user.GetName().StartsWith("john9"));
			AssertEquals(10, user.GetId());
		}

		public virtual void Test2()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = null;
			int k = 10;
			long t1 = Sharpen.Runtime.CurrentTimeMillis();
			string[] fields = new string[] { "ok", "id", "name" };
			odb = Open(baseName);
			for (int i = 0; i < k; i++)
			{
				odb.Store(new NeoDatis.Odb.Test.List.User("john" + (k - i), "john@ibm.com", "ny 875"
					, k - i + 1, new System.DateTime(t1 - i * 1000), i % 2 == 0));
				odb.Store(new NeoDatis.Odb.Test.List.User("john" + (k - i), "john@ibm.com", "ny 875"
					, k - i, new System.DateTime(t1 - i * 1000), (i + 1) % 2 == 0));
			}
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.List.User)).OrderByAsc("ok,id,name");
			NeoDatis.Odb.Objects<NeoDatis.Odb.Test.List.User> users = odb.GetObjects(q);
			odb.Close();
			if (k < 11)
			{
				NeoDatis.Tool.DisplayUtility.Display("test1", users);
			}
			NeoDatis.Odb.Test.List.User user = users.GetFirst();
			AssertTrue(user.GetName().StartsWith("john1"));
			AssertEquals(2, user.GetId());
		}

		public virtual void Test2_2()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = null;
			int k = 10;
			long t1 = Sharpen.Runtime.CurrentTimeMillis();
			string[] fields = new string[] { "ok", "id", "name" };
			odb = Open(baseName);
			for (int i = 0; i < k; i++)
			{
				odb.Store(new NeoDatis.Odb.Test.List.User("john" + (k - i), "john@ibm.com", "ny 875"
					, k - i + 1, new System.DateTime(t1 - i * 1000), i % 2 == 0));
				odb.Store(new NeoDatis.Odb.Test.List.User("john" + (k - i), "john@ibm.com", "ny 875"
					, k - i, new System.DateTime(t1 - i * 1000), (i + 1) % 2 == 0));
			}
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Core.Query.IQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.List.User)).OrderByDesc("ok,id,name");
			NeoDatis.Odb.Objects<NeoDatis.Odb.Test.List.User> users = odb.GetObjects(q);
			odb.Close();
			if (k < 11)
			{
				NeoDatis.Tool.DisplayUtility.Display("test1", users);
			}
			NeoDatis.Odb.Test.List.User user = users.GetFirst();
			AssertTrue(user.GetName().StartsWith("john10"));
			AssertEquals(11, user.GetId());
		}
	}
}
