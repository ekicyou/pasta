using NeoDatis.Odb.Impl.Core.Query.Criteria;
using NeoDatis.Odb.Test.VO.Login;
using NeoDatis.Odb.Core.Query.Criteria;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.Update
{
	[TestFixture]
    public class TestUpdate : NeoDatis.Odb.Test.ODBTest
	{
		public static int NbObjects = 50;

		public static string FileName = "update.neodatis";

		private static bool first = true;

		/// <exception cref="System.Exception"></exception>
		public override void SetUp()
		{
			base.SetUp();
			DeleteBase(FileName);
			NeoDatis.Odb.ODB odb = Open(FileName);
			for (int i = 0; i < NbObjects; i++)
			{
				odb.Store(new Function("function " + (i + i)));
				odb.Store(new User("olivier " + i, "olivier@neodatis.com "
					 + i, new Profile("profile " + i, new Function
					("inner function " + i))));
			}
			odb.Close();
			odb = Open(FileName);
			NeoDatis.Odb.Objects<Function> l = odb.GetObjects<Function>();
			AssertEquals(2 * NbObjects, l.Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			NeoDatis.Odb.ODB odb = Open(FileName);
			NeoDatis.Odb.Core.Query.IQuery query = new CriteriaQuery(Where.Equal("name", "function 10"));
			NeoDatis.Odb.Objects<Function> l = odb.GetObjects<Function>(query);
			int size = l.Count;
			AssertFalse(l.Count==0);
			Function f = (Function)l.GetFirst
				();
			NeoDatis.Odb.OID id = odb.GetObjectId(f);
			AssertEquals("function 10", f.GetName());
			string newName = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs().ToString();
			f.SetName(newName);
			odb.Store(f);
			odb.Close();
			odb = Open(FileName);
			l = odb.GetObjects<Function>(query);
			query = new CriteriaQuery(Where.Equal("name", newName));
			AssertTrue(size == l.Count + 1);
			l = odb.GetObjects<Function>(query);
			AssertFalse(l.Count==0);
			AssertEquals(1, l.Count);
			AssertEquals(id, odb.GetObjectId(l.GetFirst()));
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test2()
		{
			NeoDatis.Odb.ODB odb = Open(FileName);
			int nbProfiles = odb.GetObjects<Profile>().Count;
			NeoDatis.Odb.Core.Query.IQuery query = new CriteriaQuery(Where.Equal("profile.name", "profile 10"));
			NeoDatis.Odb.Objects<User> l = odb.GetObjects<User>(query);
			int size = l.Count;
			AssertFalse(l.Count==0);
			User u = (User)l.GetFirst();
			AssertEquals("profile 10", u.GetProfile().GetName());
			Profile p2 = u.GetProfile();
			string newName = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs().ToString() +
				 "-";
			p2.SetName(newName);
			odb.Store(p2);
			odb.Close();
			odb = Open(FileName);
			l = odb.GetObjects<User>(query);
			AssertTrue(l.Count == size - 1);
			if (!isLocal)
			{
				query = new CriteriaQuery(Where.Equal("profile.name", newName));
			}
			else
			{
				query = new _SimpleNativeQuery_134(newName);
			}
			l = odb.GetObjects<User> (query);
			AssertFalse(l.Count==0);
			Objects<Profile> l2 = odb.GetObjects<Profile>(false);
			AssertEquals(nbProfiles, l2.Count);
			odb.Close();
		}

		private sealed class _SimpleNativeQuery_134 : NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery
		{
			public _SimpleNativeQuery_134(string newName)
			{
				this.newName = newName;
			}

			public bool Match(User user)
			{
				return user.GetProfile().GetName().Equals(newName);
			}

			private readonly string newName;
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test3()
		{
			NeoDatis.Odb.ODB odb = Open(FileName);
			NeoDatis.Odb.Core.Query.IQuery pquery = new CriteriaQuery(Where.Equal("name", "profile 10"));
			long nbProfiles = odb.Count(new CriteriaQuery());
			long nbProfiles10 = odb.GetObjects<Profile>(pquery).Count;
			NeoDatis.Odb.Core.Query.IQuery query = new CriteriaQuery(Where.Equal("profile.name", "profile 10"));
			NeoDatis.Odb.Objects<User> l = odb.GetObjects<User>(query);
			int size = l.Count;
			AssertFalse(l.Count==0);
			User u = (User)l.GetFirst();
			AssertEquals("profile 10", u.GetProfile().GetName());
			string newName = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs().ToString() +
				 "+";
			Profile p2 = u.GetProfile();
			p2.SetName(newName);
			odb.Store(u);
			odb.Close();
			odb = Open(FileName);
			l = odb.GetObjects<User>(query);
			AssertEquals(l.Count + 1, size);
			AssertEquals(nbProfiles10, odb.GetObjects<Profile>(pquery).Count + 1);
			if (!isLocal)
			{
				query = new CriteriaQuery(Where.Equal("profile.name", newName));
			}
			else
			{
				query = new _SimpleNativeQuery_179(newName);
			}
			l = odb.GetObjects<User>(query);
			AssertEquals(1, l.Count);
			Objects<Profile>  l2 = odb.GetObjects<Profile>(false);
			AssertEquals(nbProfiles, l2.Count);
			odb.Close();
		}

		private sealed class _SimpleNativeQuery_179 : NeoDatis.Odb.Core.Query.NQ.SimpleNativeQuery
		{
			public _SimpleNativeQuery_179(string newName)
			{
				this.newName = newName;
			}

			public bool Match(User user)
			{
				return user.GetProfile().GetName().Equals(newName);
			}

			private readonly string newName;
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test4()
		{
			DeleteBase(FileName);
			NeoDatis.Odb.ODB odb = Open(FileName);
			NeoDatis.Odb.OdbConfiguration.SetMaxNumberOfObjectInCache(10);
			try
			{
				System.Collections.IList list = new System.Collections.ArrayList();
				for (int i = 0; i < 15; i++)
				{
					Function function = new Function
						("function " + i);
					try
					{
						odb.Store(function);
					}
					catch (System.Exception e)
					{
						odb.Rollback();
						odb.Close();
						AssertTrue(e.Message.IndexOf("Cache is full!") != -1);
						return;
					}
					list.Add(function);
				}
				odb.Close();
				odb = Open(FileName);
				NeoDatis.Odb.Objects<Function> l = odb.GetObjects<Function>( true);
				l.Next();
				l.Next();
				odb.Store(l.Next());
				odb.Close();
				odb = Open(FileName);
				AssertEquals(15, odb.Count(new CriteriaQuery()));
				odb.Close();
			}
			finally
			{
				NeoDatis.Odb.OdbConfiguration.SetMaxNumberOfObjectInCache(300000);
			}
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test5()
		{
			try
			{
				DeleteBase(FileName);
				NeoDatis.Odb.ODB odb = Open(FileName);
				System.Collections.IList list = new System.Collections.ArrayList();
				for (int i = 0; i < 15; i++)
				{
					Function function = new Function
						("function " + i);
					odb.Store(function);
					list.Add(function);
				}
				odb.Close();
				NeoDatis.Odb.OdbConfiguration.SetMaxNumberOfObjectInCache(15);
				odb = Open(FileName);
				NeoDatis.Odb.Core.Query.IQuery query = new CriteriaQuery(Where
					.Or().Add(Where.Like("name", "%9")).Add(Where
					.Like("name", "%8")));
				NeoDatis.Odb.Objects<Function> l = odb.GetObjects<Function>(query, false);
				AssertEquals(2, l.Count);
				l.Next();
				odb.Store(l.Next());
				odb.Close();
				odb = Open(FileName);
				AssertEquals(15, odb.Count(new CriteriaQuery()));
				odb.Close();
			}
			finally
			{
				NeoDatis.Odb.OdbConfiguration.SetMaxNumberOfObjectInCache(300000);
			}
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test6()
		{
			MyObject mo = null;
			DeleteBase(FileName);
			NeoDatis.Odb.ODB odb = Open(FileName);
			mo = new MyObject(15, "oli");
			mo.SetDate(new System.DateTime());
			odb.Store(mo);
			odb.Close();
			odb = Open(FileName);
			MyObject mo2 = odb.GetObjects<MyObject>().GetFirst();
			mo2.SetDate(new System.DateTime(mo.GetDate().Millisecond + 10));
			mo2.SetSize(mo.GetSize() + 1);
			odb.Store(mo2);
			odb.Close();
			odb = Open(FileName);
			MyObject mo3 = (MyObject)odb.GetObjects<MyObject>().GetFirst();
			AssertEquals(mo3.GetDate().Millisecond, mo2.GetDate().Millisecond);
			AssertTrue(mo3.GetDate().Millisecond > mo.GetDate().Millisecond);
			AssertTrue(mo3.GetSize() == mo.GetSize() + 1);
			odb.Close();
		}

		// println("before:" + mo.getDate().getTime() + " - " + mo.getSize());
		// println("after:" + mo3.getDate().getTime() + " - " + mo3.getSize());
		/// <summary>
		/// When an object an a collection attribute, and this colllection is changed
		/// (adding one object),no update in place is possible for instance.
		/// </summary>
		/// <remarks>
		/// When an object an a collection attribute, and this colllection is changed
		/// (adding one object),no update in place is possible for instance.
		/// </remarks>
		/// <exception cref="System.Exception">System.Exception</exception>
		[Test]
        public virtual void Test7()
		{
			DeleteBase(FileName);
			NeoDatis.Odb.ODB odb = Open(FileName);
			Function function = new Function
				("login");
			Profile profile = new Profile
				("operator", function);
			User user = new User("olivier smadja"
				, "olivier@neodatis.com", profile);
			odb.Store(user);
			odb.Close();
			odb = Open(FileName);
			User user2 = odb.GetObjects<User>().GetFirst();
			user2.GetProfile().AddFunction(new Function("new Function"
				));
			odb.Store(user2);
			odb.Close();
			odb = Open(FileName);
			User user3 = (User)odb.GetObjects<User>().GetFirst();
			AssertEquals(2, user3.GetProfile().GetFunctions().Count);
			Function f1 = (Function)user3
				.GetProfile().GetFunctions()[0];
			Function f2 = (Function)user3
				.GetProfile().GetFunctions()[1];
			AssertEquals("login", f1.GetName());
			AssertEquals("new Function", f2.GetName());
			odb.Close();
		}

		/// <summary>setting one attribute to null</summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		[Test]
        public virtual void Test8()
		{
			DeleteBase(FileName);
			NeoDatis.Odb.ODB odb = Open(FileName);
			Function function = new Function
				("login");
			Profile profile = new Profile
				("operator", function);
			User user = new User("olivier smadja"
				, "olivier@neodatis.com", profile);
			odb.Store(user);
			odb.Close();
			odb = Open(FileName);
			User user2 = (User)odb.GetObjects<User>().GetFirst();
			user2.SetProfile(null);
			odb.Store(user2);
			odb.Close();
			odb = Open(FileName);
			User user3 = (User)odb.GetObjects<User>().GetFirst();
			AssertNull(user3.GetProfile());
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public override void TearDown()
		{
			NeoDatis.Odb.OdbConfiguration.SetMaxNumberOfObjectInCache(300000);
			DeleteBase(FileName);
		}

		/// <summary>Test updaing a non native attribute with a new non native object</summary>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestUpdateObjectReference()
		{
			DeleteBase(FileName);
			NeoDatis.Odb.ODB odb = Open(FileName);
			Function function = new Function
				("login");
			Profile profile = new Profile
				("operator", function);
			User user = new User("olivier smadja"
				, "olivier@neodatis.com", profile);
			odb.Store(user);
			odb.Close();
			Profile profile2 = new Profile
				("new operator", function);
			odb = Open(FileName);
			User user2 = (User)odb.GetObjects<User>().GetFirst();
			user2.SetProfile(profile2);
			odb.Store(user2);
			odb.Close();
			odb = Open(FileName);
			user2 = (User)odb.GetObjects<User>().GetFirst();
			AssertEquals("new operator", user2.GetProfile().GetName());
			AssertEquals(2, odb.GetObjects<Profile>().Count);
			odb.Close();
		}

		/// <summary>
		/// Test updaing a non native attribute with an already existing non native
		/// object - with commit
		/// </summary>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestUpdateObjectReference2()
		{
			DeleteBase(FileName);
			NeoDatis.Odb.ODB odb = Open(FileName);
			Function function = new Function
				("login");
			Profile profile = new Profile
				("operator", function);
			User user = new User("olivier smadja"
				, "olivier@neodatis.com", profile);
			odb.Store(user);
			odb.Close();
			Profile profile2 = new Profile
				("new operator", function);
			odb = Open(FileName);
			odb.Store(profile2);
			odb.Close();
			odb = Open(FileName);
			profile2 = odb.GetObjects<Profile>(new CriteriaQuery(Where.Equal("name", "new operator"))).GetFirst();
			User user2 = odb.GetObjects<User>().GetFirst();
			user2.SetProfile(profile2);
			odb.Store(user2);
			odb.Close();
			odb = Open(FileName);
			user2 = odb.GetObjects<User>().GetFirst();
			AssertEquals("new operator", user2.GetProfile().GetName());
			AssertEquals(2, odb.GetObjects<Profile>().Count);
			odb.Close();
		}

		/// <summary>
		/// Test updating a non native attribute with an already existing non native
		/// object without comit
		/// </summary>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestUpdateObjectReference3()
		{
			DeleteBase(FileName);
			NeoDatis.Odb.ODB odb = Open(FileName);
			Function function = new Function
				("login");
			Profile profile = new Profile
				("operator", function);
			User user = new User("olivier smadja"
				, "olivier@neodatis.com", profile);
			odb.Store(user);
			odb.Close();
			Profile profile2 = new Profile
				("new operator", function);
			odb = Open(FileName);
			odb.Store(profile2);
			User user2 = odb.GetObjects<User>().GetFirst();
			user2.SetProfile(profile2);
			odb.Store(user2);
			odb.Close();
			odb = Open(FileName);
			user2 = odb.GetObjects<User>().GetFirst();
			AssertEquals("new operator", user2.GetProfile().GetName());
			AssertEquals(2, odb.GetObjects<Profile>().Count);
			odb.Close();
		}

		/// <summary>
		/// Test updating a non native attribute than wall null with an already
		/// existing non native object without comit
		/// </summary>
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestUpdateObjectReference4()
		{
			DeleteBase(FileName);
			NeoDatis.Odb.ODB odb = Open(FileName);
			Function function = new Function
				("login");
			User user = new User("olivier smadja"
				, "olivier@neodatis.com", null);
			odb.Store(user);
			odb.Close();
			Profile profile2 = new Profile
				("new operator", function);
			odb = Open(FileName);
			odb.Store(profile2);
			User user2 = odb.GetObjects<User>().GetFirst();
			user2.SetProfile(profile2);
			odb.Store(user2);
			odb.Close();
			odb = Open(FileName);
			user2 = odb.GetObjects<User>().GetFirst();
			AssertEquals("new operator", user2.GetProfile().GetName());
			AssertEquals(1, odb.GetObjects<Profile>().Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestDirectSave()
		{
			if (!isLocal)
			{
				return;
			}
			NeoDatis.Odb.OdbConfiguration.SetSaveHistory(true);
			DeleteBase("btree.neodatis");
			NeoDatis.Odb.ODB odb = Open("btree.neodatis");
			Function function = new Function
				("f1");
			odb.Store(function);
			for (int i = 0; i < 2; i++)
			{
				function.SetName(function.GetName() + function.GetName() + function.GetName() + function
					.GetName());
				odb.Store(function);
			}
			NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy
				.GetEngine(odb);
			if (isLocal)
			{
			}
			
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = engine.GetSession(true).GetMetaModel
				().GetClassInfo(typeof(Function).FullName, true);
			Println(ci);
			AssertEquals(null, ci.GetCommitedZoneInfo().first);
			AssertEquals(null, ci.GetCommitedZoneInfo().last);
			AssertEquals(1, ci.GetUncommittedZoneInfo().GetNbObjects());
			odb.Close();
		}

		[Test]
        public virtual void TestUpdateRelation()
		{
			string baseName = GetBaseName();
			NeoDatis.Odb.ODB odb = Open(baseName);
			// first create a function
			Function f = new Function("f1"
				);
			odb.Store(f);
			odb.Close();
			odb = Open(baseName);
			// reloads the function
			NeoDatis.Odb.Objects<Function> functions = odb.GetObjects<Function>(new CriteriaQuery(Where.Equal("name", "f1")));
			Function f1 = functions.GetFirst();
			// Create a profile with the loaded function
			Profile profile = new Profile
				("test", f1);
			odb.Store(profile);
			odb.Close();
			odb = Open(baseName);
			NeoDatis.Odb.Objects<Profile> profiles = odb.GetObjects<Profile>();
			functions = odb.GetObjects<Function>();
			odb.Close();
			DeleteBase(baseName);
			AssertEquals(1, functions.Count);
			AssertEquals(1, profiles.Count);
		}
	}
}
