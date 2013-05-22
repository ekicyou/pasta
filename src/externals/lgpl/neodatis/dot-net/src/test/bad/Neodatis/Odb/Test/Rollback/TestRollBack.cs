using NeoDatis.Odb.Test.VO.Login;
using NeoDatis.Tool.Wrappers;
namespace NeoDatis.Odb.Test.Rollback
{
	public class TestRollBack : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			DeleteBase("rollback.neodatis");
			NeoDatis.Odb.ODB odb = Open("rollback.neodatis", "u1", "p1");
			odb.Store(new Function("f1"));
			odb.Store(new Function("f2"));
			odb.Store(new Function("f3"));
			odb.Close();
			odb = Open("rollback.neodatis", "u1", "p1");
			odb.Store(new Function("f3"));
			odb.Rollback();
			odb.Close();
			odb = Open("rollback.neodatis", "u1", "p1");
			AssertEquals(3, odb.GetObjects<Function>().Count
				);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test2()
		{
			DeleteBase("rollback.neodatis");
			NeoDatis.Odb.ODB odb = Open("rollback.neodatis", "u1", "p1");
			odb.Store(new Function("f1"));
			odb.Store(new Function("f2"));
			odb.Store(new Function("f3"));
			odb.Close();
			odb = Open("rollback.neodatis", "u1", "p1");
			odb.Store(new Function("f3"));
			odb.Rollback();
			// odb.close();
			try
			{
				AssertEquals(3, odb.GetObjects<Function>().Count
					);
			}
			catch (NeoDatis.Odb.ODBRuntimeException e)
			{
				string s = NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(e, false);
				AssertFalse(s.IndexOf("ODB session has been rollbacked") == -1);
			}
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test3RollbackOneStore()
		{
			DeleteBase("rollback.neodatis");
			NeoDatis.Odb.ODB odb = Open("rollback.neodatis", "u1", "p1");
			odb.Store(new Function("f1"));
			odb.Store(new Function("f2"));
			odb.Store(new Function("f3"));
			odb.Close();
			odb = Open("rollback.neodatis", "u1", "p1");
			odb.Store(new Function("f3"));
			odb.Rollback();
			odb.Close();
			odb = Open("rollback.neodatis", "u1", "p1");
			AssertEquals(3, odb.GetObjects<Function>().Count
				);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test4RollbackXXXStores()
		{
			DeleteBase("rollback.neodatis");
			NeoDatis.Odb.ODB odb = Open("rollback.neodatis", "u1", "p1");
			odb.Store(new Function("f1"));
			odb.Store(new Function("f2"));
			odb.Store(new Function("f3"));
			odb.Close();
			odb = Open("rollback.neodatis", "u1", "p1");
			for (int i = 0; i < 500; i++)
			{
				odb.Store(new Function("f3 - " + i));
			}
			odb.Rollback();
			odb.Close();
			odb = Open("rollback.neodatis", "u1", "p1");
			AssertEquals(3, odb.GetObjects<Function>().Count
				);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test5RollbackDelete()
		{
			DeleteBase("rollback.neodatis");
			NeoDatis.Odb.ODB odb = Open("rollback.neodatis", "u1", "p1");
			odb.Store(new Function("f1"));
			odb.Store(new Function("f2"));
			odb.Store(new Function("f3"));
			odb.Close();
			odb = Open("rollback.neodatis", "u1", "p1");
			NeoDatis.Odb.Objects<Function> objects = odb.GetObjects<Function>();
			while (objects.HasNext())
			{
				odb.Delete(objects.Next());
			}
			odb.Rollback();
			odb.Close();
			odb = Open("rollback.neodatis", "u1", "p1");
			AssertEquals(3, odb.GetObjects<Function>().Count
				);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test6RollbackDeleteAndStore()
		{
			DeleteBase("rollback.neodatis");
			NeoDatis.Odb.ODB odb = Open("rollback.neodatis", "u1", "p1");
			odb.Store(new Function("f1"));
			odb.Store(new Function("f2"));
			odb.Store(new Function("f3"));
			odb.Close();
			odb = Open("rollback.neodatis", "u1", "p1");
			NeoDatis.Odb.Objects<Function> objects = odb.GetObjects<Function>();
			while (objects.HasNext())
			{
				odb.Delete(objects.Next());
			}
			for (int i = 0; i < 500; i++)
			{
				odb.Store(new Function("f3 - " + i));
			}
			odb.Rollback();
			odb.Close();
			odb = Open("rollback.neodatis", "u1", "p1");
			AssertEquals(3, odb.GetObjects<Function>().Count
				);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test7Update()
		{
			DeleteBase("rollback.neodatis");
			NeoDatis.Odb.ODB odb = Open("rollback.neodatis", "u1", "p1");
			odb.Store(new Function("1function"));
			odb.Store(new Function("2function"));
			odb.Store(new Function("3function"));
			odb.Close();
			odb = Open("rollback.neodatis", "u1", "p1");
			NeoDatis.Odb.Objects<Function> objects = odb.GetObjects<Function>();
			while (objects.HasNext())
			{
				Function f = (Function)objects
					.Next();
				f.SetName(OdbString.Substring(f.GetName(), 1));
				odb.Store(f);
			}
			odb.Rollback();
			odb.Close();
			odb = Open("rollback.neodatis", "u1", "p1");
			AssertEquals(3, odb.GetObjects<Function>().Count
				);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test8RollbackDeleteAndStore()
		{
			DeleteBase("rollback.neodatis");
			NeoDatis.Odb.ODB odb = Open("rollback.neodatis", "u1", "p1");
			odb.Store(new Function("f1"));
			odb.Store(new Function("f2"));
			odb.Store(new Function("f3"));
			odb.Close();
			odb = Open("rollback.neodatis", "u1", "p1");
			NeoDatis.Odb.Objects<Function> objects = odb.GetObjects<Function>();
			while (objects.HasNext())
			{
				Function f = (Function)objects
					.Next();
				f.SetName(OdbString.Substring(f.GetName(), 1));
				odb.Store(f);
			}
			objects.Reset();
			while (objects.HasNext())
			{
				odb.Delete(objects.Next());
			}
			for (int i = 0; i < 500; i++)
			{
				odb.Store(new Function("f3 - " + i));
			}
			odb.Rollback();
			odb.Close();
			odb = Open("rollback.neodatis", "u1", "p1");
			AssertEquals(3, odb.GetObjects<Function>().Count
				);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public override void TearDown()
		{
			DeleteBase("rollback.neodatis");
		}
	}
}
