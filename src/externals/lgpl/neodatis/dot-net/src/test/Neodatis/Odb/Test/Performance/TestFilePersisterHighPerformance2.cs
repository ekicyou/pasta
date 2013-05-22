using NeoDatis.Odb.Test.VO.Login;
using System.Collections;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.Performance
{
	[TestFixture]
    public class TestFilePersisterHighPerformance2 : NeoDatis.Odb.Test.ODBTest
	{
		public static int TestSize = isLocal ? 1000 : 200;

		public static readonly string OdbFileName = "perf.neodatis";

		public static readonly string Db4oFileName = "perf.yap";

		// public static final String ODB_FILE_NAME = "k:/tmp/perf.neodatis";
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestInsertUserODB()
		{
			DeleteBase(OdbFileName);
			long t1;
			long t2;
			long t3;
			long t4;
			long t5;
			long t6;
			long t7;
			long t77;
			long t8;
			t1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.ODB odb = Open(OdbFileName);
			for (int i = 0; i < TestSize; i++)
			{
				object o = GetUserInstance();
				odb.Store(o);
				if (i % 1000 == 0)
				{
					System.Console.Out.Write(".");
				}
			}
			t2 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			// assertEquals(TEST_SIZE,
			// odb.getSession().getCache().getNumberOfObjects ());
			NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy
				.GetEngine(odb);
			if (isLocal)
			{
				Println("NB WAs=" + engine.GetSession(true).GetTransaction().GetNumberOfWriteActions
					());
			}
			odb.Commit();
			t3 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			Println("end of insert");
			NeoDatis.Odb.Objects<User> l = odb.GetObjects<User>(false);
			t4 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			int nbObjects = l.Count;
			Println(nbObjects + " objects ");
			User user = null;
			while (l.HasNext())
			{
				// println(i);
				user = (User)l.Next();
			}
			// assertEquals(TEST_SIZE,
			// odb.getSession().getCache().getNumberOfObjects ());
			Println("end of real get objects");
			t5 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			user = null;
			int j = 0;
			l.Reset();
			while (l.HasNext())
			{
				// println(i);
				user = (User)l.Next();
				user.SetName(user.GetName() + " updated" + j);
				odb.Store(user);
				j++;
			}
			t6 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			Println("end of update");
			odb.Close();
			t7 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			odb = Open(OdbFileName);
			l = odb.GetObjects<User>();
			t77 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			j = 0;
			while (l.HasNext())
			{
				user = (User)l.Next();
				Println(j + " " + user.GetName());
				AssertTrue(user.GetName().EndsWith("updated" + j));
				odb.Delete(user);
				j++;
			}
			odb.Close();
			t8 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			odb = Open(OdbFileName);
			AssertEquals(0, odb.GetObjects<User>().Count);
			odb.Close();
			DisplayResult("ODB " + TestSize + " User objects ", t1, t2, t3, t4, t5, t6, t7, t77
				, t8);
			DeleteBase(OdbFileName);
		}

		// println("calls="+ObjectReader.calls + " / time="
		// +ObjectReader.timeToGetObjectFromId );
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestInsertSimpleObjectODB()
		{
			DeleteBase(OdbFileName);
			long t1 = 0;
			long t2 = 0;
			long t3 = 0;
			long t4 = 0;
			long t5 = 0;
			long t6 = 0;
			long t7 = 0;
			long t77 = 0;
			long t8 = 0;
			NeoDatis.Odb.ODB odb = null;
			NeoDatis.Odb.Objects<SimpleObject> l = null;
			SimpleObject so = null;
			t1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			odb = Open(OdbFileName);
			for (int i = 0; i < TestSize; i++)
			{
				object o = GetSimpleObjectInstance(i);
				odb.Store(o);
				if (i % 20000 == 0)
				{
					System.Console.Out.Write(".");
					Println("After insert=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy.GetEngine
						(odb).GetSession(true).GetCache().ToString());
				}
			}
			//
			NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy
				.GetEngine(odb);
			if (isLocal)
			{
				// println("NB WA="+WriteAction.count);
				Println("NB WAs=" + engine.GetSession(true).GetTransaction().GetNumberOfWriteActions
					());
			}
			t2 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			odb.Commit();
			if (isLocal)
			{
				Println("after commit : NB WAs=" + engine.GetSession(true).GetTransaction().GetNumberOfWriteActions
					());
			}
			// if(true)return;
			// println("After commit="+Dummy.getEngine(odb).getSession().getCache().toString());
			// println("NB WA="+WriteAction.count);
			t3 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			// println("end of insert");
			l = odb.GetObjects<SimpleObject>(false);
			// println("end of getObjects ");
			t4 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			// println("After getObjects ="+Dummy.getEngine(odb).getSession().getCache().toString());
			// println("NB WA="+WriteAction.count);
			if (isLocal)
			{
				Println("after select : NB WAs=" + engine.GetSession(true).GetTransaction().GetNumberOfWriteActions
					());
			}
			int nbObjects = l.Count;
			Println(nbObjects + " objects ");
			int k = 0;
			while (l.HasNext())
			{
				object o = l.Next();
				if (k % 9999 == 0)
				{
					Println(((SimpleObject)o).GetName());
				}
				k++;
			}
			// println("end of real get ");
			t5 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			Println("select " + (t5 - t3) + " - " + (t5 - t4));
			so = null;
			k = 0;
			l.Reset();
			while (l.HasNext())
			{
				so = (SimpleObject)l.Next();
				so.SetName(so.GetName() + " updated");
				odb.Store(so);
				if (k % 10000 == 0)
				{
					Println("update " + k);
					if (isLocal)
					{
						Println("after update : NB WAs=" + engine.GetSession(true).GetTransaction().GetNumberOfWriteActions
							());
						Println("After update=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy.GetEngine
							(odb).GetSession(true).GetCache().ToString());
					}
				}
				k++;
			}
			if (isLocal)
			{
				Println("after update : NB WAs=" + engine.GetSession(true).GetTransaction().GetNumberOfWriteActions
					());
			}
			t6 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			odb.Close();
			t7 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			odb = Open(OdbFileName);
			l = odb.GetObjects<SimpleObject>(false);
			t77 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			int j = 0;
			while (l.HasNext())
			{
				so = (SimpleObject)l.Next();
				AssertTrue(so.GetName().EndsWith("updated"));
				odb.Delete(so);
				if (j % 10000 == 0)
				{
					Println("delete " + j);
				}
				j++;
			}
			odb.Close();
			t8 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			odb = Open(OdbFileName);
			AssertEquals(0, odb.GetObjects<SimpleObject>().Count);
			odb.Close();
			DisplayResult("ODB " + TestSize + " SimpleObject objects ", t1, t2, t3, t4, t5, t6
				, t7, t77, t8);
		}

		private void DisplayResult(string @string, long t1, long t2, long t3, long t4, long
			 t5, long t6, long t7, long t77, long t8)
		{
			string s1 = " total=" + (t8 - t1);
			string s2 = " total insert=" + (t3 - t1) + " -- " + "insert=" + (t2 - t1) + " commit="
				 + (t3 - t2);
			string s3 = " total select=" + (t5 - t3) + " -- " + "select=" + (t4 - t3) + " get="
				 + (t5 - t4);
			string s4 = " total update=" + (t7 - t5) + " -- " + "update=" + (t6 - t5) + " commit="
				 + (t7 - t6);
			string s5 = " total delete=" + (t8 - t7) + " -- " + "select=" + (t77 - t7) + " - delete="
				 + (t8 - t77);
			Println(@string + s1 + " | " + s2 + " | " + s3 + " | " + s4 + " | " + s5);
		}

		private object GetUserInstance()
		{
			Function login = new Function
				("login");
			Function logout = new Function
				("logout");
			System.Collections.Generic.IList<Function> list = new System.Collections.Generic.List<Function>();
			list.Add(login);
			list.Add(logout);
			Profile profile = new Profile("operator", list);
			User user = new User("olivier smadja"
				, "olivier@neodatis.com", profile);
			return user;
		}

		private NeoDatis.Odb.Test.VO.Attribute.TestClass GetTestClassInstance()
		{
			NeoDatis.Odb.Test.VO.Attribute.TestClass tc = new NeoDatis.Odb.Test.VO.Attribute.TestClass
				();
			tc.SetBigDecimal1(new System.Decimal(1.123456789));
			tc.SetBoolean1(true);
			tc.SetChar1('d');
			tc.SetDouble1(154.78998989);
			tc.SetInt1(78964);
			tc.SetString1("Ola chico como vc est√° ???");
			tc.SetDate1(new System.DateTime());
			return tc;
		}

		private SimpleObject GetSimpleObjectInstance(int i)
		{
			SimpleObject so = new SimpleObject
				();
			so.SetDate(new System.DateTime());
			so.SetDuration(i);
			so.SetName("ola chico, como vc esta?" + i);
			return so;
		}

		/// <exception cref="System.Exception"></exception>
		public static void Main2(string[] args)
		{
			new NeoDatis.Odb.Test.Performance.TestFilePersisterHighPerformance2().TestInsertSimpleObjectODB
				();
		}
		// new TestFilePersisterHighPerformance2().testInsertUserODB();
		// new TestFilePersisterHighPerformance2().testDeleteSimpleObjectODB();
	}
}
