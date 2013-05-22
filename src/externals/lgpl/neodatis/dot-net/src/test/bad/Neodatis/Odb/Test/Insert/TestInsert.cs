namespace NeoDatis.Odb.Test.Insert
{
	public class TestInsert : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void TestCompositeCollection2DifferentObjects()
		{
			DeleteBase("ti1.neodatis");
			NeoDatis.Odb.ODB odb = Open("ti1.neodatis");
			int nbUsers = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.User), true).Count;
			int nbProfiles = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Profile), true)
				.Count;
			int nbFunctions = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function), true
				).Count;
			NeoDatis.Odb.Test.VO.Login.Function login = new NeoDatis.Odb.Test.VO.Login.Function
				("login");
			NeoDatis.Odb.Test.VO.Login.Function logout = new NeoDatis.Odb.Test.VO.Login.Function
				("logout");
			NeoDatis.Odb.Test.VO.Login.Function disconnect = new NeoDatis.Odb.Test.VO.Login.Function
				("disconnect");
			System.Collections.IList list = new System.Collections.ArrayList();
			list.Add(login);
			list.Add(logout);
			System.Collections.IList list2 = new System.Collections.ArrayList();
			list.Add(login);
			list.Add(logout);
			NeoDatis.Odb.Test.VO.Login.Profile profile1 = new NeoDatis.Odb.Test.VO.Login.Profile
				("operator 1", list);
			NeoDatis.Odb.Test.VO.Login.Profile profile2 = new NeoDatis.Odb.Test.VO.Login.Profile
				("operator 2", list2);
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", profile1);
			NeoDatis.Odb.Test.VO.Login.User userB = new NeoDatis.Odb.Test.VO.Login.User("A√°sa Galv√£o Smadja"
				, "aisa@neodatis.com", profile2);
			odb.Store(user);
			odb.Store(userB);
			odb.Commit();
			NeoDatis.Odb.Objects functions = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				), true);
			NeoDatis.Odb.Objects profiles = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Profile
				), true);
			NeoDatis.Odb.Objects users = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.User
				), true);
			odb.Close();
			// assertEquals(nbUsers+2,users.size());
			NeoDatis.Odb.Test.VO.Login.User user2 = (NeoDatis.Odb.Test.VO.Login.User)users.GetFirst
				();
			AssertEquals(user.ToString(), user2.ToString());
			AssertEquals(nbProfiles + 2, profiles.Count);
			AssertEquals(nbFunctions + 2, functions.Count);
			DeleteBase("ti1.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestCompositeCollection1()
		{
			DeleteBase("t31.neodatis");
			NeoDatis.Odb.ODB odb = Open("t31.neodatis");
			NeoDatis.Odb.Test.VO.Login.Function login = new NeoDatis.Odb.Test.VO.Login.Function
				("login");
			System.Collections.IList list = new System.Collections.ArrayList();
			list.Add(login);
			NeoDatis.Odb.Test.VO.Login.Profile profile1 = new NeoDatis.Odb.Test.VO.Login.Profile
				("operator 1", list);
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", profile1);
			odb.Store(user);
			odb.Close();
			odb = Open("t31.neodatis");
			NeoDatis.Odb.Objects users = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.User
				), true);
			odb.Close();
			// assertEquals(nbUsers+2,users.size());
			NeoDatis.Odb.Test.VO.Login.User user2 = (NeoDatis.Odb.Test.VO.Login.User)users.GetFirst
				();
			AssertEquals(user.ToString(), user2.ToString());
			DeleteBase("t31.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			DeleteBase("t1.neodatis");
			// LogUtil.allOn(true);
			NeoDatis.Odb.ODB odb = Open("t1.neodatis");
			// LogUtil.objectWriterOn(true);
			NeoDatis.Odb.Test.VO.Login.Function login = new NeoDatis.Odb.Test.VO.Login.Function
				("login");
			System.Collections.IList list = new System.Collections.ArrayList();
			list.Add(login);
			NeoDatis.Odb.Test.VO.Login.Profile profile1 = new NeoDatis.Odb.Test.VO.Login.Profile
				("operator 1", list);
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", profile1);
			odb.Store(user);
			odb.Close();
			odb = Open("t1.neodatis");
			NeoDatis.Odb.Objects users = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.User
				), true);
			// assertEquals(nbUsers+2,users.size());
			NeoDatis.Odb.Test.VO.Login.User user2 = (NeoDatis.Odb.Test.VO.Login.User)users.GetFirst
				();
			odb.Close();
			AssertEquals(user.ToString(), user2.ToString());
			DeleteBase("t1.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestCompositeCollection2()
		{
			DeleteBase("t3.neodatis");
			// LogUtil.objectWriterOn(true);
			NeoDatis.Odb.ODB odb = Open("t3.neodatis");
			int nbUsers = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.User), true).Count;
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
			NeoDatis.Odb.Test.VO.Login.Profile profile1 = new NeoDatis.Odb.Test.VO.Login.Profile
				("operator 1", list);
			NeoDatis.Odb.Test.VO.Login.Profile profile2 = new NeoDatis.Odb.Test.VO.Login.Profile
				("operator 2", list);
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", profile1);
			NeoDatis.Odb.Test.VO.Login.User userB = new NeoDatis.Odb.Test.VO.Login.User("A√°sa Galv√£o Smadja"
				, "aisa@neodatis.com", profile2);
			odb.Store(user);
			odb.Store(userB);
			odb.Close();
			odb = Open("t3.neodatis");
			NeoDatis.Odb.Objects users = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.User
				), true);
			NeoDatis.Odb.Objects profiles = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Profile
				), true);
			NeoDatis.Odb.Objects functions = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				), true);
			// assertEquals(nbUsers+2,users.size());
			NeoDatis.Odb.Test.VO.Login.User user2 = (NeoDatis.Odb.Test.VO.Login.User)users.GetFirst
				();
			AssertEquals(user.ToString(), user2.ToString());
			AssertEquals(nbProfiles + 2, profiles.Count);
			AssertEquals(nbFunctions + 2, functions.Count);
			odb.Close();
			DeleteBase("t3.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestCompositeCollection3()
		{
			DeleteBase("t4.neodatis");
			NeoDatis.Odb.ODB odb = Open("t4.neodatis");
			// Configuration.addLogId("ObjectWriter");
			// Configuration.addLogId("ObjectReader");
			// Configuration.addLogId("FileSystemInterface");
			int nbUsers = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.User), true).Count;
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
			NeoDatis.Odb.Test.VO.Login.Profile profile1 = new NeoDatis.Odb.Test.VO.Login.Profile
				("operator 1", list);
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", profile1);
			NeoDatis.Odb.Test.VO.Login.User userB = new NeoDatis.Odb.Test.VO.Login.User("A√≠sa Galv√£o Smadja"
				, "aisa@neodatis.com", profile1);
			odb.Store(user);
			odb.Store(userB);
			odb.Close();
			odb = Open("t4.neodatis");
			NeoDatis.Odb.Objects users = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.User
				), true);
			NeoDatis.Odb.Objects profiles = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Profile
				), true);
			NeoDatis.Odb.Objects functions = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				), true);
			// assertEquals(nbUsers+2,users.size());
			NeoDatis.Odb.Test.VO.Login.User user2 = (NeoDatis.Odb.Test.VO.Login.User)users.GetFirst
				();
			AssertEquals(user.ToString(), user2.ToString());
			AssertEquals(nbProfiles + 1, profiles.Count);
			AssertEquals(nbFunctions + 2, functions.Count);
			odb.Close();
			DeleteBase("t4.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestCompositeCollection4()
		{
			DeleteBase("t5.neodatis");
			NeoDatis.Odb.ODB odb = Open("t5.neodatis");
			int nbUsers = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.User), true).Count;
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
			NeoDatis.Odb.Test.VO.Login.Profile profile1 = new NeoDatis.Odb.Test.VO.Login.Profile
				("operator 1", list);
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", profile1);
			NeoDatis.Odb.Test.VO.Login.User userB = new NeoDatis.Odb.Test.VO.Login.User("A√≠sa Galv√£o Smadja"
				, "aisa@neodatis.com", profile1);
			odb.Store(user);
			odb.Store(userB);
			odb.Commit();
			NeoDatis.Odb.Objects users = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.User
				), true);
			NeoDatis.Odb.Objects profiles = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Profile
				), true);
			NeoDatis.Odb.Objects functions = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				), true);
			odb.Close();
			// assertEquals(nbUsers+2,users.size());
			NeoDatis.Odb.Test.VO.Login.User user2 = (NeoDatis.Odb.Test.VO.Login.User)users.GetFirst
				();
			AssertEquals(user.ToString(), user2.ToString());
			AssertEquals(nbProfiles + 1, profiles.Count);
			AssertEquals(nbFunctions + 2, functions.Count);
		}

		// deleteBase("t5.neodatis");
		/// <exception cref="System.Exception"></exception>
		public virtual void TestSimple()
		{
			DeleteBase("t2.neodatis");
			NeoDatis.Odb.ODB odb = Open("t2.neodatis");
			int nbFunctions = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function), true
				).Count;
			NeoDatis.Odb.Test.VO.Login.Function login = new NeoDatis.Odb.Test.VO.Login.Function
				("login");
			NeoDatis.Odb.Test.VO.Login.Function logout = new NeoDatis.Odb.Test.VO.Login.Function
				("logout");
			odb.Store(login);
			odb.Store(logout);
			odb.Close();
			odb = Open("t2.neodatis");
			NeoDatis.Odb.Objects functions = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				), true);
			NeoDatis.Odb.Test.VO.Login.Function f1 = (NeoDatis.Odb.Test.VO.Login.Function)functions
				.GetFirst();
			f1.SetName("login1");
			odb.Store(f1);
			odb.Close();
			odb = Open("t2.neodatis");
			functions = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function), true);
			odb.Close();
			AssertEquals(2, functions.Count);
			AssertEquals("login1", ((NeoDatis.Odb.Test.VO.Login.Function)functions.GetFirst()
				).GetName());
			DeleteBase("t2.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestBufferSize()
		{
			int size = NeoDatis.Odb.OdbConfiguration.GetDefaultBufferSizeForData();
			NeoDatis.Odb.OdbConfiguration.SetDefaultBufferSizeForData(5);
			DeleteBase("ti1.neodatis");
			NeoDatis.Odb.ODB odb = Open("ti1.neodatis");
			System.Text.StringBuilder b = new System.Text.StringBuilder();
			for (int i = 0; i < 1000; i++)
			{
				b.Append("login - login ");
			}
			NeoDatis.Odb.Test.VO.Login.Function login = new NeoDatis.Odb.Test.VO.Login.Function
				(b.ToString());
			NeoDatis.Odb.Test.VO.Login.Profile profile1 = new NeoDatis.Odb.Test.VO.Login.Profile
				("operator 1", login);
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", profile1);
			odb.Store(user);
			odb.Commit();
			NeoDatis.Odb.Objects users = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.User
				), true);
			NeoDatis.Odb.Objects profiles = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Profile
				), true);
			NeoDatis.Odb.Objects functions = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				), true);
			odb.Close();
			// assertEquals(nbUsers+2,users.size());
			NeoDatis.Odb.Test.VO.Login.User user2 = (NeoDatis.Odb.Test.VO.Login.User)users.GetFirst
				();
			AssertEquals(user.ToString(), user2.ToString());
			AssertEquals(b.ToString(), user2.GetProfile().GetFunctions().GetEnumerator().Current
				.ToString());
			DeleteBase("ti1.neodatis");
			NeoDatis.Odb.OdbConfiguration.SetDefaultBufferSizeForData(size);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestDatePersistence()
		{
			NeoDatis.Odb.ODB odb = null;
			DeleteBase("date.neodatis");
			try
			{
				odb = Open("date.neodatis");
				NeoDatis.Odb.Test.VO.Attribute.TestClass tc1 = new NeoDatis.Odb.Test.VO.Attribute.TestClass
					();
				tc1.SetDate1(new System.DateTime());
				long t1 = tc1.GetDate1().Millisecond;
				odb.Store(tc1);
				odb.Close();
				odb = Open("date.neodatis");
				NeoDatis.Odb.Objects l = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Attribute.TestClass
					));
				AssertEquals(1, l.Count);
				NeoDatis.Odb.Test.VO.Attribute.TestClass tc2 = (NeoDatis.Odb.Test.VO.Attribute.TestClass
					)l.GetFirst();
				AssertEquals(t1, tc2.GetDate1().Millisecond);
				AssertEquals(tc1.GetDate1(), tc2.GetDate1());
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
			DeleteBase("date.neodatis");
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestStringPersistence()
		{
			NeoDatis.Odb.ODB odb = null;
			try
			{
				odb = Open("date.neodatis");
				NeoDatis.Odb.Test.VO.Attribute.TestClass tc1 = new NeoDatis.Odb.Test.VO.Attribute.TestClass
					();
				tc1.SetString1(string.Empty);
				odb.Store(tc1);
				odb.Close();
				odb = Open("date.neodatis");
				NeoDatis.Odb.Objects l = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Attribute.TestClass
					));
				AssertEquals(1, l.Count);
				NeoDatis.Odb.Test.VO.Attribute.TestClass tc2 = (NeoDatis.Odb.Test.VO.Attribute.TestClass
					)l.GetFirst();
				AssertEquals(string.Empty, tc2.GetString1());
				AssertEquals(null, tc2.GetBigDecimal1());
				AssertEquals(null, tc2.GetDouble1());
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
		public virtual void Test6()
		{
			DeleteBase("t1u.neodatis");
			NeoDatis.Odb.ODB odb = Open("t1u.neodatis");
			NeoDatis.Odb.Test.VO.Login.Function login = new NeoDatis.Odb.Test.VO.Login.Function
				("login");
			NeoDatis.Odb.Test.VO.Login.Function logout = new NeoDatis.Odb.Test.VO.Login.Function
				("logout");
			odb.Store(login);
			odb.Store(logout);
			odb.Close();
			odb = Open("t1u.neodatis");
			NeoDatis.Odb.Test.VO.Login.Function login2 = new NeoDatis.Odb.Test.VO.Login.Function
				("login2");
			NeoDatis.Odb.Test.VO.Login.Function logout2 = new NeoDatis.Odb.Test.VO.Login.Function
				("logout2");
			odb.Store(login2);
			odb.Store(logout2);
			// select without committing
			NeoDatis.Odb.Objects l = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				), true);
			AssertEquals(4, l.Count);
			// println(l);
			odb.Close();
			odb = Open("t1u.neodatis");
			l = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function), true);
			AssertEquals(4, l.Count);
			// println(l);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test7()
		{
			DeleteBase("t1u.neodatis");
			NeoDatis.Odb.ODB odb = Open("t1u.neodatis");
			NeoDatis.Odb.Test.VO.Login.Function login = new NeoDatis.Odb.Test.VO.Login.Function
				("login");
			NeoDatis.Odb.Test.VO.Login.Function logout = new NeoDatis.Odb.Test.VO.Login.Function
				("logout");
			odb.Store(login);
			odb.Store(logout);
			odb.Commit();
			NeoDatis.Odb.Test.VO.Login.Function input = new NeoDatis.Odb.Test.VO.Login.Function
				("input");
			odb.Store(input);
			odb.Close();
			odb = Open("t1u.neodatis");
			NeoDatis.Odb.Objects l = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.Function
				), true);
			AssertEquals(3, l.Count);
			// println(l);
			odb.Close();
		}

		/// <summary>Test with java util Date and java sql Date</summary>
		public virtual void Test8()
		{
			string baseName = GetBaseName();
			Println(baseName);
			NeoDatis.Odb.ODB odb = null;
			System.DateTime utilDate = new System.DateTime();
			System.DateTime sqlDate = new System.DateTime(utilDate.Millisecond + 10000);
			System.DateTime timestamp = new System.DateTime(utilDate.Millisecond + 20000);
			try
			{
				odb = Open(baseName);
				NeoDatis.Odb.Test.VO.Attribute.ObjectWithDates o = new NeoDatis.Odb.Test.VO.Attribute.ObjectWithDates
					("object1", utilDate, sqlDate, timestamp);
				odb.Store(o);
				odb.Close();
				odb = Open(baseName);
				NeoDatis.Odb.Objects<NeoDatis.Odb.Test.VO.Attribute.ObjectWithDates> dates = odb.
					GetObjects(typeof(NeoDatis.Odb.Test.VO.Attribute.ObjectWithDates));
				NeoDatis.Odb.Test.VO.Attribute.ObjectWithDates o2 = dates.GetFirst();
				Println(o2.GetName());
				Println(o2.GetJavaUtilDate());
				Println(o2.GetJavaSqlDte());
				Println(o2.GetTimestamp());
				AssertEquals("object1", o2.GetName());
				AssertEquals(utilDate, o2.GetJavaUtilDate());
				AssertEquals(sqlDate, o2.GetJavaSqlDte());
				AssertEquals(timestamp, o2.GetTimestamp());
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
		}
	}
}
