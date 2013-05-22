namespace NeoDatis.Odb.Test.Update
{
	public class TestSimpleUpdateObject : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public override void TearDown()
		{
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			DeleteBase("t1u.neodatis");
			NeoDatis.Odb.ODB odb = Open("t1u.neodatis");
			NeoDatis.Odb.Test.VO.Login.Function login = new NeoDatis.Odb.Test.VO.Login.Function
				("login");
			NeoDatis.Odb.Test.VO.Login.Function logout = new NeoDatis.Odb.Test.VO.Login.Function
				("logout");
			odb.Store(login);
			Println("--------");
			odb.Store(login);
			odb.Store(logout);
			// odb.commit();
			odb.Close();
			odb = Open("t1u.neodatis");
			NeoDatis.Odb.Objects l = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				), true);
			NeoDatis.Odb.Test.VO.Login.Function f2 = (NeoDatis.Odb.Test.VO.Login.Function)l.GetFirst
				();
			f2.SetName("login function");
			odb.Store(f2);
			odb.Close();
			NeoDatis.Odb.ODB odb2 = Open("t1u.neodatis");
			NeoDatis.Odb.Test.VO.Login.Function f = (NeoDatis.Odb.Test.VO.Login.Function)odb2
				.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function)).GetFirst();
			AssertEquals("login function", f.GetName());
			odb2.Close();
			DeleteBase("t1u.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test2()
		{
			DeleteBase("t2.neodatis");
			NeoDatis.Odb.ODB odb = Open("t2.neodatis");
			int nbUsers = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.User)).Count;
			int nbProfiles = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Profile), true)
				.Count;
			int nbFunctions = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function), true
				).Count;
			NeoDatis.Odb.Test.VO.Login.Function login = new NeoDatis.Odb.Test.VO.Login.Function
				("login");
			NeoDatis.Odb.Test.VO.Login.Function logout = new NeoDatis.Odb.Test.VO.Login.Function
				("logout");
			System.Collections.IList list = new System.Collections.ArrayList();
			list.Add(login);
			list.Add(logout);
			NeoDatis.Odb.Test.VO.Login.Profile profile = new NeoDatis.Odb.Test.VO.Login.Profile
				("operator", list);
			NeoDatis.Odb.Test.VO.Login.User olivier = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", profile);
			NeoDatis.Odb.Test.VO.Login.User aisa = new NeoDatis.Odb.Test.VO.Login.User("A√≠sa Galv√£o Smadja"
				, "aisa@neodRMuatis.com", profile);
			odb.Store(olivier);
			odb.Store(aisa);
			odb.Commit();
			NeoDatis.Odb.Objects users = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.User
				), true);
			NeoDatis.Odb.Objects profiles = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Profile
				), true);
			NeoDatis.Odb.Objects functions = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				), true);
			odb.Close();
			// println("Users:"+users);
			Println("Profiles:" + profiles);
			Println("Functions:" + functions);
			odb = Open("t2.neodatis");
			NeoDatis.Odb.Objects l = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.User), 
				true);
			odb.Close();
			AssertEquals(nbUsers + 2, users.Count);
			NeoDatis.Odb.Test.VO.Login.User user2 = (NeoDatis.Odb.Test.VO.Login.User)users.GetFirst
				();
			AssertEquals(olivier.ToString(), user2.ToString());
			AssertEquals(nbProfiles + 1, profiles.Count);
			AssertEquals(nbFunctions + 2, functions.Count);
			NeoDatis.Odb.ODB odb2 = Open("t2.neodatis");
			l = odb2.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function), true);
			NeoDatis.Odb.Test.VO.Login.Function function = (NeoDatis.Odb.Test.VO.Login.Function
				)l.GetFirst();
			function.SetName("login function");
			odb2.Store(function);
			odb2.Close();
			NeoDatis.Odb.ODB odb3 = Open("t2.neodatis");
			NeoDatis.Odb.Objects l2 = odb3.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.User)
				, true);
			int i = 0;
			while (l2.HasNext() && i < System.Math.Min(2, l2.Count))
			{
				NeoDatis.Odb.Test.VO.Login.User user = (NeoDatis.Odb.Test.VO.Login.User)l2.Next();
				AssertEquals("login function", string.Empty + user.GetProfile().GetFunctions()[0]
					);
				i++;
			}
			odb3.Close();
			DeleteBase("t2.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test3()
		{
			DeleteBase("t1u2.neodatis");
			NeoDatis.Odb.ODB odb = Open("t1u2.neodatis");
			NeoDatis.Odb.Test.VO.Login.Function login = new NeoDatis.Odb.Test.VO.Login.Function
				(null);
			odb.Store(login);
			odb.Close();
			odb = Open("t1u2.neodatis");
			login = (NeoDatis.Odb.Test.VO.Login.Function)odb.GetObjects(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function), NeoDatis.Odb.Core.Query.Criteria.Where
				.IsNull("name"))).GetFirst();
			AssertTrue(login.GetName() == null);
			login.SetName("login");
			odb.Store(login);
			odb.Close();
			odb = Open("t1u2.neodatis");
			login = (NeoDatis.Odb.Test.VO.Login.Function)odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				)).GetFirst();
			AssertTrue(login.GetName().Equals("login"));
			odb.Close();
			DeleteBase("t1u2.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test5()
		{
			DeleteBase("t5.neodatis");
			NeoDatis.Odb.ODB odb = Open("t5.neodatis");
			long nbFunctions = odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Function)));
			long nbProfiles = odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(NeoDatis.Odb.Test.VO.Login.Profile)));
			long nbUsers = odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(
				typeof(NeoDatis.Odb.Test.VO.Login.User)));
			NeoDatis.Odb.Test.VO.Login.Function login = new NeoDatis.Odb.Test.VO.Login.Function
				("login");
			NeoDatis.Odb.Test.VO.Login.Function logout = new NeoDatis.Odb.Test.VO.Login.Function
				("logout");
			System.Collections.IList list = new System.Collections.ArrayList();
			list.Add(login);
			list.Add(logout);
			NeoDatis.Odb.Test.VO.Login.Profile profile = new NeoDatis.Odb.Test.VO.Login.Profile
				("operator", list);
			NeoDatis.Odb.Test.VO.Login.User olivier = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", profile);
			NeoDatis.Odb.Test.VO.Login.User aisa = new NeoDatis.Odb.Test.VO.Login.User("A√≠sa Galv√£o Smadja"
				, "aisa@neodatis.com", profile);
			odb.Store(olivier);
			odb.Store(profile);
			odb.Commit();
			odb.Close();
			odb = Open("t5.neodatis");
			NeoDatis.Odb.Objects users = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.User
				), true);
			NeoDatis.Odb.Objects profiles = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Profile
				), true);
			NeoDatis.Odb.Objects functions = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				), true);
			odb.Close();
			AssertEquals(nbUsers + 1, users.Count);
			AssertEquals(nbProfiles + 1, profiles.Count);
			AssertEquals(nbFunctions + 2, functions.Count);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test6()
		{
			// LogUtil.objectWriterOn(true);
			DeleteBase("t6.neodatis");
			NeoDatis.Odb.ODB odb = Open("t6.neodatis");
			NeoDatis.Odb.Test.VO.Login.Function login = new NeoDatis.Odb.Test.VO.Login.Function
				("login");
			NeoDatis.Odb.Test.VO.Login.Function logout = new NeoDatis.Odb.Test.VO.Login.Function
				("logout");
			System.Collections.IList list = new System.Collections.ArrayList();
			list.Add(login);
			list.Add(logout);
			NeoDatis.Odb.Test.VO.Login.Profile profile = new NeoDatis.Odb.Test.VO.Login.Profile
				("operator", list);
			NeoDatis.Odb.Test.VO.Login.User olivier = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", profile);
			odb.Store(olivier);
			odb.Close();
			Println("----------");
			odb = Open("t6.neodatis");
			NeoDatis.Odb.Objects users = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.User
				), true);
			NeoDatis.Odb.Test.VO.Login.User u1 = (NeoDatis.Odb.Test.VO.Login.User)users.GetFirst
				();
			u1.GetProfile().SetName("operator 234567891011121314");
			odb.Store(u1);
			odb.Close();
			odb = Open("t6.neodatis");
			NeoDatis.Odb.Objects profiles = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Profile
				), true);
			AssertEquals(1, profiles.Count);
			NeoDatis.Odb.Test.VO.Login.Profile p1 = (NeoDatis.Odb.Test.VO.Login.Profile)profiles
				.GetFirst();
			AssertEquals(u1.GetProfile().GetName(), p1.GetName());
		}
	}
}
