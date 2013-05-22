namespace NeoDatis.Odb.Test.Oid
{
	public class TestGetObjectByOid : NeoDatis.Odb.Test.ODBTest
	{
		/// <summary>Getting object by id after re opening database</summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test1()
		{
			DeleteBase("getid.neodatis");
			NeoDatis.Odb.Test.VO.Login.Function function1 = new NeoDatis.Odb.Test.VO.Login.Function
				("f1");
			NeoDatis.Odb.Test.VO.Login.Function function2 = new NeoDatis.Odb.Test.VO.Login.Function
				("f2");
			NeoDatis.Odb.ODB odb = Open("getid.neodatis");
			odb.Store(function1);
			odb.Store(function2);
			NeoDatis.Odb.OID id1 = odb.GetObjectId(function1);
			NeoDatis.Odb.OID id2 = odb.GetObjectId(function2);
			odb.Close();
			odb = Open("getid.neodatis");
			NeoDatis.Odb.Test.VO.Login.Function function1bis = (NeoDatis.Odb.Test.VO.Login.Function
				)odb.GetObjectFromId(id1);
			AssertEquals(function1.GetName(), function1bis.GetName());
			NeoDatis.Odb.Test.VO.Login.Function function2bis = (NeoDatis.Odb.Test.VO.Login.Function
				)odb.GetObjectFromId(id2);
			function2bis.SetName("function 2");
			odb.Store(function2bis);
			NeoDatis.Odb.OID id2bis = odb.GetObjectId(function2bis);
			odb.Close();
			odb = Open("getid.neodatis");
			NeoDatis.Odb.Test.VO.Login.Function function2ter = (NeoDatis.Odb.Test.VO.Login.Function
				)odb.GetObjectFromId(id2);
			AssertEquals("function 2", function2ter.GetName());
			odb.Close();
			DeleteBase("getid.neodatis");
		}

		/// <summary>Getting object by id during the same transaction</summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test2()
		{
			DeleteBase("getid.neodatis");
			NeoDatis.Odb.Test.VO.Login.Function function1 = new NeoDatis.Odb.Test.VO.Login.Function
				("f1");
			NeoDatis.Odb.Test.VO.Login.Function function2 = new NeoDatis.Odb.Test.VO.Login.Function
				("f2");
			NeoDatis.Odb.ODB odb = Open("getid.neodatis");
			odb.Store(function1);
			odb.Store(function2);
			NeoDatis.Odb.OID id1 = odb.GetObjectId(function1);
			NeoDatis.Odb.OID id2 = odb.GetObjectId(function2);
			NeoDatis.Odb.Test.VO.Login.Function function1bis = (NeoDatis.Odb.Test.VO.Login.Function
				)odb.GetObjectFromId(id1);
			odb.Close();
			AssertEquals(function1.GetName(), function1bis.GetName());
			DeleteBase("getid.neodatis");
		}

		/// <summary>
		/// Getting object by id after an in place update in the same transaction
		/// than the insert
		/// </summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test3()
		{
			DeleteBase("getid.neodatis");
			NeoDatis.Odb.Test.VO.Login.Function function1 = new NeoDatis.Odb.Test.VO.Login.Function
				("f1");
			NeoDatis.Odb.Test.VO.Login.Function function2 = new NeoDatis.Odb.Test.VO.Login.Function
				("f2");
			NeoDatis.Odb.ODB odb = Open("getid.neodatis");
			odb.Store(function1);
			odb.Store(function2);
			function1.SetName("f2");
			odb.Store(function1);
			NeoDatis.Odb.OID id1 = odb.GetObjectId(function1);
			NeoDatis.Odb.OID id2 = odb.GetObjectId(function2);
			NeoDatis.Odb.Test.VO.Login.Function function1bis = (NeoDatis.Odb.Test.VO.Login.Function
				)odb.GetObjectFromId(id1);
			odb.Close();
			AssertEquals(function1.GetName(), function1bis.GetName());
			DeleteBase("getid.neodatis");
		}

		/// <summary>
		/// Getting object by id after an update(not in place) in the same
		/// transaction than the insert
		/// </summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test4()
		{
			DeleteBase("getid.neodatis");
			NeoDatis.Odb.Test.VO.Login.Function function1 = new NeoDatis.Odb.Test.VO.Login.Function
				("f1");
			NeoDatis.Odb.Test.VO.Login.Function function2 = new NeoDatis.Odb.Test.VO.Login.Function
				("f2");
			NeoDatis.Odb.ODB odb = Open("getid.neodatis");
			odb.Store(function1);
			odb.Store(function2);
			function1.SetName("function login and logout");
			odb.Store(function1);
			NeoDatis.Odb.OID id1 = odb.GetObjectId(function1);
			NeoDatis.Odb.OID id2 = odb.GetObjectId(function2);
			NeoDatis.Odb.Test.VO.Login.Function function1bis = (NeoDatis.Odb.Test.VO.Login.Function
				)odb.GetObjectFromId(id1);
			odb.Close();
			AssertEquals(function1.GetName(), function1bis.GetName());
			DeleteBase("getid.neodatis");
		}

		/// <summary>Test performance of retrieving 2 objects by oid</summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test5()
		{
			DeleteBase("getid.neodatis");
			NeoDatis.Odb.Test.VO.Login.Function function1 = new NeoDatis.Odb.Test.VO.Login.Function
				("f1");
			NeoDatis.Odb.Test.VO.Login.Function function2 = new NeoDatis.Odb.Test.VO.Login.Function
				("f2");
			NeoDatis.Odb.ODB odb = Open("getid.neodatis");
			odb.Store(function1);
			odb.Store(function2);
			NeoDatis.Odb.OID id1 = odb.GetObjectId(function1);
			NeoDatis.Odb.OID id2 = odb.GetObjectId(function2);
			odb.Close();
			odb = Open("getid.neodatis");
			long t1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.Test.VO.Login.Function function1bis = (NeoDatis.Odb.Test.VO.Login.Function
				)odb.GetObjectFromId(id1);
			NeoDatis.Odb.Test.VO.Login.Function function2bis = (NeoDatis.Odb.Test.VO.Login.Function
				)odb.GetObjectFromId(id2);
			long t2 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			odb.Close();
			DeleteBase("getid.neodatis");
			AssertEquals(function1.GetName(), function1bis.GetName());
			AssertEquals(function2.GetName(), function2bis.GetName());
			long time = t2 - t1;
			Println(time);
			long acceptableTime = isLocal ? 1 : 17;
			if (time > acceptableTime)
			{
				// ms
				Fail("Getting two objects by oid lasted more than " + acceptableTime + "ms : " + 
					time);
			}
		}

		/// <summary>Test performance of retrieving many simple objects by oid</summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test6()
		{
			DeleteBase("getid.neodatis");
			int size = isLocal ? 20001 : 2001;
			NeoDatis.Odb.ODB odb = Open("getid.neodatis");
			NeoDatis.Odb.OID[] oids = new NeoDatis.Odb.OID[size];
			for (int i = 0; i < size; i++)
			{
				oids[i] = odb.Store(new NeoDatis.Odb.Test.VO.Login.Function("function " + i));
			}
			odb.Close();
			odb = Open("getid.neodatis");
			long t1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.VO.Login.Function f = (NeoDatis.Odb.Test.VO.Login.Function)odb.
					GetObjectFromId(oids[i]);
				AssertEquals("function " + i, f.GetName());
				if (i % 3000 == 0)
				{
					Println(i + "/" + size);
				}
			}
			long t2 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			odb.Close();
			DeleteBase("getid.neodatis");
			long time = t2 - t1;
			double timeForEachGet = (double)time / (double)size;
			double acceptableTime = isLocal ? 0.022 : 0.5;
			// 0.04294785260736963
			Println("time for each get = " + time + "/" + size + " = " + timeForEachGet);
			if (testPerformance && timeForEachGet > acceptableTime)
			{
				// ms
				Fail("Getting " + size + " simple objects by oid lasted more than " + acceptableTime
					 + "ms : " + timeForEachGet);
			}
		}

		/// <summary>Test performance of retrieving many complex objects by oid</summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test7()
		{
			DeleteBase("getid.neodatis");
			int size = isLocal ? 10001 : 1000;
			NeoDatis.Odb.ODB odb = Open("getid.neodatis");
			NeoDatis.Odb.OID[] oids = new NeoDatis.Odb.OID[size];
			for (int i = 0; i < size; i++)
			{
				oids[i] = odb.Store(GetInstance(i));
			}
			odb.Close();
			odb = Open("getid.neodatis");
			long t1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.VO.Login.User u = (NeoDatis.Odb.Test.VO.Login.User)odb.GetObjectFromId
					(oids[i]);
				AssertEquals("kiko" + i, u.GetName());
			}
			long t2 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			odb.Close();
			DeleteBase("getid.neodatis");
			long time = t2 - t1;
			double timeForEachGet = (double)time / (double)size;
			double acceptableTime = isLocal ? 0.086 : 1.6;
			// 0.1561843815618438
			Println("time for each get = " + timeForEachGet + " - Total time for " + size + " objects = "
				 + time);
			if (testPerformance && timeForEachGet > acceptableTime)
			{
				// ms
				Println("time for each get = " + timeForEachGet + " - Total time for " + size + " objects = "
					 + time);
				Fail("Getting " + size + " complex objects by oid lasted more than " + acceptableTime
					 + "ms : " + timeForEachGet);
			}
		}

		/// <summary>Trying to get an object with OID that does not exist</summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void TestGetOIDThatDoesNotExist()
		{
			DeleteBase("getid.neodatis");
			NeoDatis.Odb.Test.VO.Login.Function function2 = new NeoDatis.Odb.Test.VO.Login.Function
				("f2");
			NeoDatis.Odb.ODB odb = Open("getid.neodatis");
			NeoDatis.Odb.OID oid = NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(49);
			try
			{
				object o = odb.GetObjectFromId(oid);
			}
			catch (System.Exception e)
			{
				odb.Close();
				AssertFalse(e.Message.IndexOf(" does not exist in the database") == -1);
			}
		}

		private object GetInstance(int i)
		{
			NeoDatis.Odb.Test.VO.Login.Function login = new NeoDatis.Odb.Test.VO.Login.Function
				("login " + i);
			NeoDatis.Odb.Test.VO.Login.Function logout = new NeoDatis.Odb.Test.VO.Login.Function
				("logout" + i);
			System.Collections.IList list = new System.Collections.ArrayList();
			list.Add(login);
			list.Add(logout);
			NeoDatis.Odb.Test.VO.Login.Profile profile = new NeoDatis.Odb.Test.VO.Login.Profile
				("operator" + i, list);
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("kiko"
				 + i, "olivier@neodatis.com" + i, profile);
			return user;
		}
	}
}
