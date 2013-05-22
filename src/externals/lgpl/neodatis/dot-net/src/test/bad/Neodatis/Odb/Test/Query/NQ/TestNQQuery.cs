namespace NeoDatis.Odb.Test.Query.NQ
{
	public class TestNQQuery : NeoDatis.Odb.Test.ODBTest
	{
		public static int NbObjects = 10;

		/// <exception cref="System.Exception"></exception>
		public override void SetUp()
		{
			base.SetUp();
			DeleteBase("get.neodatis");
			// println("TestNQQuery.setUp");
			NeoDatis.Odb.ODB odb = Open("get.neodatis");
			for (int i = 0; i < NbObjects; i++)
			{
				odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("function " + i));
				odb.Store(new NeoDatis.Odb.Test.VO.Login.User("olivier " + i, "olivier@neodatis.org "
					 + "1", new NeoDatis.Odb.Test.VO.Login.Profile("profile " + i, new NeoDatis.Odb.Test.VO.Login.Function
					("inner function " + i))));
				odb.Store(new NeoDatis.Odb.Test.VO.Login.User("olivier " + i, "olivier@neodatis.org "
					 + "2", new NeoDatis.Odb.Test.VO.Login.Profile("profile " + i, new NeoDatis.Odb.Test.VO.Login.Function
					("inner function " + i))));
				odb.Store(new NeoDatis.Odb.Test.VO.Login.User("olivier " + i, "olivier@neodatis.org "
					 + "3", new NeoDatis.Odb.Test.VO.Login.Profile("profile " + i, new NeoDatis.Odb.Test.VO.Login.Function
					("inner function " + i))));
			}
			odb.Close();
		}

		// println("NbFunctions " + odb.count(Function.class));
		// println("NbUsers " + odb.count(User.class));
		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			if (!isLocal)
			{
				return;
			}
			NeoDatis.Odb.ODB odb = Open("get.neodatis");
			odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function), true);
			odb.Close();
			odb = Open("get.neodatis");
			// println("TestNQQuery.test1:"+odb.getObjects (Function.class,true));
			NeoDatis.Odb.Objects l = odb.GetObjects(new _SimpleNativeQuery_70());
			odb.Close();
			AssertFalse(l.IsEmpty());
			AssertEquals(NbObjects * 4, l.Count);
		}

		private sealed class _SimpleNativeQuery_70 : NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery
		{
			public _SimpleNativeQuery_70()
			{
			}

			public bool Match(NeoDatis.Odb.Test.VO.Login.Function function)
			{
				return true;
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test2()
		{
			if (!isLocal)
			{
				return;
			}
			NeoDatis.Odb.ODB odb = Open("get.neodatis");
			// println("++++TestNQQuery.test2:"+odb.getObjects
			// (Function.class,true));
			NeoDatis.Odb.Objects l = odb.GetObjects(new _SimpleNativeQuery_89());
			odb.Close();
			AssertFalse(l.IsEmpty());
			AssertEquals(1, l.Count);
		}

		private sealed class _SimpleNativeQuery_89 : NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery
		{
			public _SimpleNativeQuery_89()
			{
			}

			public bool Match(NeoDatis.Odb.Test.VO.Login.Function function)
			{
				return function.GetName().Equals("function 5");
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test3()
		{
			if (!isLocal)
			{
				return;
			}
			NeoDatis.Odb.ODB odb = Open("get.neodatis");
			NeoDatis.Odb.Objects l = odb.GetObjects(new _SimpleNativeQuery_105());
			odb.Close();
			AssertFalse(l.IsEmpty());
			AssertEquals(3, l.Count);
		}

		private sealed class _SimpleNativeQuery_105 : NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery
		{
			public _SimpleNativeQuery_105()
			{
			}

			public bool Match(NeoDatis.Odb.Test.VO.Login.User user)
			{
				return user.GetProfile().GetName().Equals("profile 5");
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test4()
		{
			if (!isLocal)
			{
				return;
			}
			NeoDatis.Odb.ODB odb = Open("get.neodatis");
			NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery query = new _SimpleNativeQuery_121();
			NeoDatis.Odb.Objects l = odb.GetObjects(query, true, 0, 5);
			odb.Close();
			AssertFalse(l.IsEmpty());
			AssertEquals(5, l.Count);
		}

		private sealed class _SimpleNativeQuery_121 : NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery
		{
			public _SimpleNativeQuery_121()
			{
			}

			public bool Match(NeoDatis.Odb.Test.VO.Login.User user)
			{
				return user.GetProfile().GetName().StartsWith("profile");
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test5()
		{
			if (!isLocal)
			{
				return;
			}
			NeoDatis.Odb.ODB odb = Open("get.neodatis");
			NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery query = new _SimpleNativeQuery_139();
			NeoDatis.Odb.Objects l = odb.GetObjects(query, true, 5, 6);
			odb.Close();
			AssertFalse(l.IsEmpty());
			AssertEquals(1, l.Count);
		}

		private sealed class _SimpleNativeQuery_139 : NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery
		{
			public _SimpleNativeQuery_139()
			{
			}

			public bool Match(NeoDatis.Odb.Test.VO.Login.User user)
			{
				return user.GetProfile().GetName().StartsWith("profile");
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test6Ordering()
		{
			if (!isLocal)
			{
				return;
			}
			NeoDatis.Odb.ODB odb = Open("get.neodatis");
			NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery query = new _SimpleNativeQuery_158();
			query.OrderByAsc("name");
			NeoDatis.Odb.Objects l = odb.GetObjects(query, true);
			int i = 0;
			while (l.HasNext())
			{
				NeoDatis.Odb.Test.VO.Login.User user = (NeoDatis.Odb.Test.VO.Login.User)l.Next();
				AssertEquals("olivier " + i, user.GetName());
				// println(user.getName());
				i++;
			}
			odb.Close();
		}

		private sealed class _SimpleNativeQuery_158 : NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery
		{
			public _SimpleNativeQuery_158()
			{
			}

			public bool Match(NeoDatis.Odb.Test.VO.Login.User user)
			{
				return user.GetProfile().GetName().StartsWith("profile") && user.GetEmail().StartsWith
					("olivier@neodatis.org 1");
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test7Ordering()
		{
			if (!isLocal)
			{
				return;
			}
			NeoDatis.Odb.ODB odb = Open("get.neodatis");
			NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery query = new _SimpleNativeQuery_183();
			query.OrderByDesc("name");
			NeoDatis.Odb.Objects l = odb.GetObjects(query, true);
			int i = l.Count - 1;
			while (l.HasNext())
			{
				NeoDatis.Odb.Test.VO.Login.User user = (NeoDatis.Odb.Test.VO.Login.User)l.Next();
				AssertEquals("olivier " + i, user.GetName());
				// println(user.getName());
				i--;
			}
			odb.Close();
		}

		private sealed class _SimpleNativeQuery_183 : NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery
		{
			public _SimpleNativeQuery_183()
			{
			}

			public bool Match(NeoDatis.Odb.Test.VO.Login.User user)
			{
				return user.GetProfile().GetName().StartsWith("profile") && user.GetEmail().StartsWith
					("olivier@neodatis.org 2");
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test8Ordering()
		{
			if (!isLocal)
			{
				return;
			}
			NeoDatis.Odb.ODB odb = Open("get.neodatis");
			NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery query = new _SimpleNativeQuery_208();
			query.OrderByAsc("name,email");
			NeoDatis.Odb.Objects l = odb.GetObjects(query, true);
			int i = 0;
			while (l.HasNext())
			{
				NeoDatis.Odb.Test.VO.Login.User user = (NeoDatis.Odb.Test.VO.Login.User)l.Next();
				// println(user.getName() + " / " + user.getEmail());
				AssertEquals("olivier " + i / 3, user.GetName());
				AssertEquals("olivier@neodatis.org " + ((i % 3) + 1), user.GetEmail());
				i++;
			}
			odb.Close();
		}

		private sealed class _SimpleNativeQuery_208 : NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery
		{
			public _SimpleNativeQuery_208()
			{
			}

			public bool Match(NeoDatis.Odb.Test.VO.Login.User user)
			{
				return user.GetProfile().GetName().StartsWith("profile");
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test9Ordering()
		{
			if (!isLocal)
			{
				return;
			}
			NeoDatis.Odb.ODB odb = Open("get.neodatis");
			NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery query = new _SimpleNativeQuery_235();
			query.OrderByDesc("name,email");
			NeoDatis.Odb.Objects l = odb.GetObjects(query, true);
			int i = l.Count - 1;
			while (l.HasNext())
			{
				NeoDatis.Odb.Test.VO.Login.User user = (NeoDatis.Odb.Test.VO.Login.User)l.Next();
				// println(user.getName() + " / " + user.getEmail());
				AssertEquals("olivier " + i / 3, user.GetName());
				AssertEquals("olivier@neodatis.org " + ((i % 3) + 1), user.GetEmail());
				i--;
			}
			odb.Close();
		}

		private sealed class _SimpleNativeQuery_235 : NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery
		{
			public _SimpleNativeQuery_235()
			{
			}

			public bool Match(NeoDatis.Odb.Test.VO.Login.User user)
			{
				return user.GetProfile().GetName().StartsWith("profile");
			}
		}

		/// <exception cref="System.Exception"></exception>
		public override void TearDown()
		{
			DeleteBase("get.neodatis");
		}
	}
}
