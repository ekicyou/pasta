using NeoDatis.Odb.Core.Query.Criteria;
using NeoDatis.Odb.Test.VO.Login;
using NeoDatis.Odb.Impl.Core.Query.Criteria;
namespace NeoDatis.Odb.Test.Query.Criteria
{
	public class TestCriteriaQuery3 : NeoDatis.Odb.Test.ODBTest
	{
		public static readonly string BaseName = "complex-CriteriaQuery-query.neodatis";

		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			NeoDatis.Odb.ODB odb = Open(BaseName);
			CriteriaQuery query = new CriteriaQuery(Where.Equal("profile.name", "profile2"));
			NeoDatis.Odb.Objects<User> l = odb.GetObjects<User>(query);
			AssertEquals(1, l.Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestCriteriaQueryQueryWithObject()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			Profile p0 = new Profile("profile0"
				);
			p0.AddFunction(null);
			p0.AddFunction(new Function("f1"));
			p0.AddFunction(new Function("f2"));
			Profile p1 = new Profile("profile1"
				);
			p1.AddFunction(null);
			p1.AddFunction(new Function("f12"));
			p1.AddFunction(new Function("f22"));
			User user = new User("The user"
				, "themail", p0);
			User user2 = new User("The user2"
				, "themail2", p1);
			odb.Store(user);
			odb.Store(user2);
			odb.Close();
			odb = Open(baseName);
			Profile pp = (Profile)odb.GetObjects<Profile>
				(new CriteriaQuery(Where.Equal("name", "profile0"))).GetFirst();
			CriteriaQuery query = new CriteriaQuery(Where.Equal("profile", pp));
			NeoDatis.Odb.Objects<User> l = odb.GetObjects<User>(query);
			AssertEquals(1, l.Count);
			user = (User)l.GetFirst();
			AssertEquals("The user", user.GetName());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestCriteriaQueryQueryWithValueInList()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			Profile p0 = new Profile("profile0"
				);
			p0.AddFunction(null);
			p0.AddFunction(new Function("f1"));
			p0.AddFunction(new Function("f2"));
			Profile p1 = new Profile("profile1"
				);
			p1.AddFunction(null);
			p1.AddFunction(new Function("f12"));
			p1.AddFunction(new Function("f22"));
			User user = new User("The user"
				, "themail", p0);
			User user2 = new User("The user2"
				, "themail2", p1);
			odb.Store(user);
			odb.Store(user2);
			odb.Close();
			odb = Open(baseName);
			Function f2bis = (Function)
				odb.GetObjects<Function> (new CriteriaQuery(Where.Equal("name", "f2"))).GetFirst();
			CriteriaQuery query = odb.CriteriaQuery(typeof(
				User), Where.Contain
				("profile.functions", f2bis));
			NeoDatis.Odb.Objects<User> l = odb.GetObjects<User>(query);
			AssertEquals(1, l.Count);
			user = (User)l.GetFirst();
			AssertEquals("The user", user.GetName());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestCriteriaQueryQueryWithValueInList3()
		{
			NeoDatis.Odb.ODB odb = Open(BaseName);
			Profile p0 = new Profile("profile0"
				);
			p0.AddFunction(null);
			p0.AddFunction(null);
			p0.AddFunction(null);
			Profile p1 = new Profile("profile1"
				);
			p1.AddFunction(null);
			p1.AddFunction(null);
			p1.AddFunction(new Function("f22"));
			User user = new User("The user"
				, "themail", p0);
			User user2 = new User("The user2"
				, "themail2", p1);
			odb.Store(user);
			odb.Store(user2);
			odb.Close();
			odb = Open(BaseName);
			Function f2bis = (Function)
				odb.GetObjects<Function>(new CriteriaQuery(Where.Equal("name", "f22"))).GetFirst();
			CriteriaQuery query = odb.CriteriaQuery(typeof(User), Where.Contain("profile.functions", f2bis));
			NeoDatis.Odb.Objects<User> l = odb.GetObjects<User>(query);
			AssertEquals(1, l.Count);
			user = (User)l.GetFirst();
			AssertEquals("The user2", user.GetName());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestCriteriaQueryQueryWithValueInList2()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			Profile p0 = new Profile("profile0"
				);
			p0.AddFunction(new Function("f1"));
			p0.AddFunction(new Function("f2"));
			Profile p1 = new Profile("profile1"
				);
			p0.AddFunction(new Function("f12"));
			p0.AddFunction(new Function("f22"));
			User user = new User("The user"
				, "themail", p0);
			User user2 = new User("The user2"
				, "themail2", p1);
			odb.Store(user);
			odb.Store(user2);
			odb.Close();
			odb = Open(baseName);
			Function f2bis = (Function)
				odb.GetObjects<Function>(new CriteriaQuery(Where.Equal("name", "f2"))).GetFirst();
			CriteriaQuery query = odb.CriteriaQuery(typeof(Profile), Where.Contain("functions", f2bis));
			NeoDatis.Odb.Objects<Profile> l = odb.GetObjects<Profile>(query);
			AssertEquals(1, l.Count);
			p1 = l.GetFirst();
			AssertEquals("profile0", p1.GetName());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestCriteriaQueryQueryWithValueInList2_with_null_object()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			Profile p0 = new Profile("profile0"
				);
			p0.AddFunction(new Function("f1"));
			p0.AddFunction(new Function("f2"));
			Profile p1 = new Profile("profile1"
				);
			p0.AddFunction(new Function("f12"));
			p0.AddFunction(new Function("f22"));
			User user = new User("The user"
				, "themail", p0);
			User user2 = new User("The user2"
				, "themail2", p1);
			odb.Store(user);
			odb.Store(user2);
			odb.Close();
			odb = Open(baseName);
			Function f2bis = new Function
				("f2");
			CriteriaQuery query = new CriteriaQuery
				(typeof(Profile), Where
				.Contain("functions", null));
			NeoDatis.Odb.Objects<Profile> l = odb.GetObjects<Profile>(query
				);
			AssertEquals(1, l.Count);
			p1 = l.GetFirst();
			AssertEquals("profile1", p1.GetName());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestListSize0()
		{
			NeoDatis.Odb.ODB odb = null;
			try
			{
				odb = Open(BaseName);
				CriteriaQuery query = new CriteriaQuery(Where.SizeEq("profile.functions", 0));
				NeoDatis.Odb.Objects<User> l = odb.GetObjects<User>(query);
				AssertEquals(1, l.Count);
				User u = (User)l.GetFirst();
				AssertEquals("profile no function", u.GetProfile().GetName());
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestListSize4()
		{
			NeoDatis.Odb.ODB odb = null;
			try
			{
				odb = Open(BaseName);
				CriteriaQuery query = new CriteriaQuery(Where.SizeEq("profile.functions", 4));
				NeoDatis.Odb.Objects<User> l = odb.GetObjects<User>(query);
				AssertEquals(1, l.Count);
				User u = (User)l.GetFirst();
				AssertEquals("big profile", u.GetProfile().GetName());
				AssertEquals(4, u.GetProfile().GetFunctions().Count);
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestListSize1()
		{
			NeoDatis.Odb.ODB odb = null;
			try
			{
				odb = Open(BaseName);
				CriteriaQuery query = new CriteriaQuery
					(typeof(User), Where
					.SizeEq("profile.functions", 1));
				NeoDatis.Odb.Objects<User> l = odb.GetObjects<User>(query);
				AssertEquals(10, l.Count);
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestListSizeGt2()
		{
			NeoDatis.Odb.ODB odb = null;
			try
			{
				odb = Open(BaseName);
				CriteriaQuery query = new CriteriaQuery(Where.SizeGt("profile.functions", 2));
				NeoDatis.Odb.Objects<User> l = odb.GetObjects<User>(query);
				AssertEquals(1, l.Count);
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestListSizeNotEqulTo1()
		{
			NeoDatis.Odb.ODB odb = null;
			try
			{
				odb = Open(BaseName);
				CriteriaQuery query = new CriteriaQuery(Where.SizeNe("profile.functions", 1));
				NeoDatis.Odb.Objects<User> l = odb.GetObjects<User>(query);
				AssertEquals(2, l.Count);
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
		}

		/// <exception cref="System.Exception"></exception>
		public override void SetUp()
		{
			base.SetUp();
			DeleteBase(BaseName);
			NeoDatis.Odb.ODB odb = Open(BaseName);
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			int size = 10;
			for (int i = 0; i < size; i++)
			{
				User u = new User("user" + 
					i, "email" + i, new Profile("profile" + i, new Function
					("function " + i)));
				odb.Store(u);
			}
			User user = new User("big user"
				, "big email", new Profile("big profile", new Function
				("big function 1")));
			user.GetProfile().AddFunction(new Function("big function 2"
				));
			user.GetProfile().AddFunction(new Function("big function 3"
				));
			user.GetProfile().AddFunction(new Function("big function 4"
				));
			odb.Store(user);
			user = new User("user no function", "email no function"
				, new Profile("profile no function"));
			odb.Store(user);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public override void TearDown()
		{
			DeleteBase(BaseName);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestCriteriaQueryQueryWithValueInList4()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			System.Collections.Generic.IList<string> strings = new System.Collections.Generic.List
				<string>();
			ClassWithListOfString c = new ClassWithListOfString
				("name", strings);
			c.GetStrings().Add("s1");
			c.GetStrings().Add("s2");
			c.GetStrings().Add("s3");
			System.Collections.Generic.IList<string> strings2 = new System.Collections.Generic.List
				<string>();
			ClassWithListOfString c2 = new ClassWithListOfString
				("name", strings2);
			c2.GetStrings().Add("s1");
			c2.GetStrings().Add("s2");
			c2.GetStrings().Add("s3");
			odb.Store(c);
			odb.Store(c2);
			odb.Close();
			odb = Open(baseName);
			CriteriaQuery query = new CriteriaQuery	(Where.Contain("strings", "s2222"));
            NeoDatis.Odb.Objects<ClassWithListOfString> l = odb.GetObjects < ClassWithListOfString>(query);
			AssertEquals(0, l.Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestCriteriaQueryQueryWithValueInList5()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			System.Collections.Generic.IList<string> strings = new System.Collections.Generic.List
				<string>();
			ClassWithListOfString c = new ClassWithListOfString
				("name", strings);
			c.GetStrings().Add("s1");
			c.GetStrings().Add(null);
			c.GetStrings().Add("s3");
			System.Collections.Generic.IList<string> strings2 = new System.Collections.Generic.List
				<string>();
			ClassWithListOfString c2 = new ClassWithListOfString
				("name", null);
			odb.Store(c);
			odb.Store(c2);
			odb.Close();
			odb = Open(baseName);
			CriteriaQuery query = new CriteriaQuery
				(typeof(ClassWithListOfString), Where
				.Contain("strings", null));
			NeoDatis.Odb.Objects<ClassWithListOfString> l =
                odb.GetObjects <ClassWithListOfString>(query);
			odb.Close();
			AssertEquals(1, l.Count);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestCriteriaQueryQueryWithValueInList6()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			System.Collections.Generic.IList<string> strings = new System.Collections.Generic.List
				<string>();
			ClassWithListOfString c = new ClassWithListOfString
				("name", strings);
			c.GetStrings().Add("s1");
			c.GetStrings().Add(null);
			c.GetStrings().Add("s3");
			System.Collections.Generic.IList<string> strings2 = new System.Collections.Generic.List
				<string>();
			ClassWithListOfString c2 = new ClassWithListOfString
				("name", null);
			odb.Store(c);
			odb.Store(c2);
			odb.Close();
			odb = Open(baseName);
			CriteriaQuery query = new CriteriaQuery
				(typeof(ClassWithListOfString), Where
				.Contain("strings", "s4"));
			NeoDatis.Odb.Objects<ClassWithListOfString> l =
                odb.GetObjects <ClassWithListOfString>(query);
			odb.Close();
			AssertEquals(0, l.Count);
		}
	}
}
