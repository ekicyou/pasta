namespace NeoDatis.Odb.Test.Performance
{
	public class TestLazy : NeoDatis.Odb.Test.ODBTest
	{
		public const int Size = 4000;

		public static readonly string Filename = "lazy.neodatis";

		/// <summary>Test the timeof lazy get</summary>
		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			DeleteBase(Filename);
			// println("Start inserting " + SIZE + " objects");
			long startinsert = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.ODB odb = Open(Filename);
			for (int i = 0; i < Size; i++)
			{
				odb.Store(GetInstance());
			}
			odb.Close();
			long endinsert = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			// println("End inserting " + SIZE + " objects  - " +
			// (endinsert-startinsert) + " ms");
			// println("totalObjects = "+ odb.count(User.class));
			odb = Open(Filename);
			long start1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.Objects lazyList = odb.GetObjects(typeof(NeoDatis.Odb.Test.VO.Login.User
				), false);
			long end1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			long startget1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			while (lazyList.HasNext())
			{
				// t1 = OdbTime.getCurrentTimeInMs();
				lazyList.Next();
			}
			// t2 = OdbTime.getCurrentTimeInMs();
			// println(t2-t1);
			long endget1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			AssertEquals(odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(
				NeoDatis.Odb.Test.VO.Login.User))), lazyList.Count);
			odb.Close();
			long t01 = end1 - start1;
			long tget1 = endget1 - startget1;
			// long t2 = end2-start2;
			// long tget2 = endget2-startget2;
			// println("t1(lazy)="+t1 + " - " +tget1+ "      t2(memory)="+t2 +" - "
			// + tget2);
			// println("t1(lazy)="+t1 + " - " +tget1);
			// assertTrue(t1<t2);
			// println(endinsert-startinsert);
			bool c = (isLocal ? 501 : 15000) > tget1;
			Println("Time for " + Size + " lazy gets : " + (tget1));
			if (!c)
			{
				Println("Time for " + Size + " lazy gets : " + (tget1));
			}
			DeleteBase(Filename);
			if (testPerformance && !c)
			{
				Fail("Time for " + Size + " lazy gets : " + (tget1));
			}
		}

		private object GetInstance()
		{
			NeoDatis.Odb.Test.VO.Login.Function login = new NeoDatis.Odb.Test.VO.Login.Function
				("login");
			NeoDatis.Odb.Test.VO.Login.Function logout = new NeoDatis.Odb.Test.VO.Login.Function
				("logout");
			System.Collections.IList list = new System.Collections.ArrayList();
			list.Add(login);
			list.Add(logout);
			NeoDatis.Odb.Test.VO.Login.Profile profile = new NeoDatis.Odb.Test.VO.Login.Profile
				("operator", list);
			NeoDatis.Odb.Test.VO.Login.User user = new NeoDatis.Odb.Test.VO.Login.User("olivier smadja"
				, "olivier@neodatis.com", profile);
			return user;
		}
	}
}
