using NeoDatis.Odb.Test.VO.Login;
using NeoDatis.Odb.Impl.Core.Query.Criteria;
namespace NeoDatis.Odb.Test.Resistance
{
	public class TestResistance1 : NeoDatis.Odb.Test.ODBTest
	{
		private static readonly string FileName = "resistance1";

		/// <summary>
		/// 1) insert 10000 objects 2) update 5000 * 10 times 3) delete other 5000
		/// 4) check count : must be 5000 5) re-update 5000 * 10 times 6) delete the
		/// other 5000 7) check count - must be zero
		/// </summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test1WithCommit()
		{
			if (!runAll)
			{
				return;
			}
			NeoDatis.Tool.StopWatch stopWatch = new NeoDatis.Tool.StopWatch();
			stopWatch.Start();
			int size = 10000;
			int size2 = 5000;
			int nbFunctions = 2;
			DeleteBase(FileName);
			NeoDatis.Odb.ODB odb = Open(FileName);
			Function f1 = new Function(
				"function 1");
			// Create Objects
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.VO.Login.Profile p = new NeoDatis.Odb.Test.VO.Login.Profile("profile number "
					 + i, f1);
				for (int j = 0; j < nbFunctions; j++)
				{
					p.AddFunction(new Function(" inner function of profile : number "
						 + i + " - " + j));
				}
				User user = new User("user name "
					 + i, "user email " + i, p);
				odb.Store(user);
				if (i % 100 == 0)
				{
				}
			}
			// println("insert " + i);
			odb.Close();
			Println("created");
			// Updates 10 times the objects
			odb = Open(FileName);
			NeoDatis.Odb.Objects<User> objects = odb.GetObjects<User>();
			Println("got the object " + objects.Count);
			for (int k = 0; k < 10; k++)
			{
				objects.Reset();
				long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				for (int i = 0; i < size2; i++)
				{
					User user = (User)objects.Next
						();
					user.GetProfile().SetName(user.GetProfile().GetName() + "-updated");
					odb.Store(user);
					if (i % 100 == 0)
					{
					}
				}
				// println("update " + i + " - " + k);
				Println("Update " + k + " - " + (NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs
					() - start) + " ms");
			}
			Println("updated");
			// Delete the rest of the objects
			for (int i = size2; i < size; i++)
			{
				odb.Delete(objects.Next());
				if (i % 100 == 0)
				{
					Println("delete " + i);
				}
			}
			Println("deleted");
			odb.Close();
			// Check object count
			odb = Open(FileName);
			objects = odb.GetObjects<User>();
			AssertEquals(size2, objects.Count);
			// Check data of the objects
			int a = 0;
			while (objects.HasNext())
			{
				User user = (User)objects.Next
					();
				AssertEquals("user name " + a, user.GetName());
				AssertEquals("user email " + a, user.GetEmail());
				AssertEquals("profile number " + a + "-updated-updated-updated-updated-updated-updated-updated-updated-updated-updated"
					, user.GetProfile().GetName());
				a++;
			}
			Println("checked");
			for (int k = 0; k < 10; k++)
			{
				objects.Reset();
				for (int i = 0; i < size2; i++)
				{
					User user = (User)objects.Next
						();
					user.GetProfile().SetName(user.GetProfile().GetName() + "-updated" + "-");
					odb.Store(user);
				}
			}
			Println("re-updated");
			odb.Close();
			// delete objects
			odb = Open(FileName);
			objects = odb.GetObjects<User>();
			a = 0;
			while (objects.HasNext())
			{
				odb.Delete(objects.Next());
				a++;
			}
			AssertEquals(size2, a);
			odb.Close();
			odb = Open(FileName);
			AssertEquals(0, odb.GetObjects<User>().Count);
			AssertEquals(0, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(User))));
			Println("deleted");
			odb.Close();
			stopWatch.End();
			Println("Total time 1 = " + stopWatch.GetDurationInMiliseconds());
			if (stopWatch.GetDurationInMiliseconds() > 90700)
			{
				Fail("time is > than " + 90700 + " = " + stopWatch.GetDurationInMiliseconds());
			}
		}

		/// <summary>
		/// 1) insert 10000 objects 2) update 5000 * 10 times 3) delete other 5000
		/// 4) check count : must be 5000 5) re-update 5000 * 10 times 6) delete the
		/// other 5000 7) check count - must be zero
		/// </summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test1WithoutCommit()
		{
			if (!runAll)
			{
				return;
			}
			NeoDatis.Tool.StopWatch stopWatch = new NeoDatis.Tool.StopWatch();
			stopWatch.Start();
			int size = 10000;
			int size2 = 5000;
			int nbFunctions = 10;
			DeleteBase(FileName);
			NeoDatis.Odb.ODB odb = Open(FileName);
			Function f1 = new Function(
				"function 1");
			// Create Objects
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.VO.Login.Profile p = new NeoDatis.Odb.Test.VO.Login.Profile("profile number "
					 + i, f1);
				for (int j = 0; j < nbFunctions; j++)
				{
					p.AddFunction(new Function(" inner function of profile : number "
						 + i + " - " + j));
				}
				User user = new User("user name "
					 + i, "user email " + i, p);
				odb.Store(user);
			}
			Println("created");
			// Updates 10 times the objects
			NeoDatis.Odb.Objects<User> objects = odb.GetObjects<User>();
			for (int k = 0; k < 10; k++)
			{
				objects.Reset();
				for (int i = 0; i < size2; i++)
				{
					User user = (User)objects.Next
						();
					user.GetProfile().SetName(user.GetProfile().GetName() + "-updated");
					odb.Store(user);
				}
			}
			Println("updated");
			// Delete the rest of the objects
			for (int i = size2; i < size; i++)
			{
				odb.Delete(objects.Next());
			}
			Println("deleted");
			// Check object count
			objects = odb.GetObjects<User>();
			AssertEquals(size2, objects.Count);
			// Check data of the objects
			int a = 0;
			while (objects.HasNext())
			{
				User user = (User)objects.Next
					();
				AssertEquals("user name " + a, user.GetName());
				AssertEquals("user email " + a, user.GetEmail());
				AssertEquals("profile number " + a + "-updated-updated-updated-updated-updated-updated-updated-updated-updated-updated"
					, user.GetProfile().GetName());
				a++;
			}
			Println("checked");
			for (int k = 0; k < 10; k++)
			{
				objects.Reset();
				for (int i = 0; i < size2; i++)
				{
					User user = (User)objects.Next
						();
					user.GetProfile().SetName(user.GetProfile().GetName() + "-updated" + "-");
					odb.Store(user);
				}
			}
			Println("re-updated");
			objects = odb.GetObjects<User>();
			NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.Dummy
				.GetEngine(odb);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.CIZoneInfo uncommited = engine.GetSession(true
				).GetMetaModel().GetClassInfo(typeof(User).FullName, 
				true).GetUncommittedZoneInfo();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.CIZoneInfo commited = engine.GetSession(true
				).GetMetaModel().GetClassInfo(typeof(User).FullName, 
				true).GetCommitedZoneInfo();
			Println("Before commit : uncommited=" + uncommited);
			Println("Before commit : commited=" + commited);
			a = 0;
			while (objects.HasNext())
			{
				// println("a="+a);
				odb.Delete(objects.Next());
				a++;
			}
			AssertEquals(size2, a);
			AssertEquals(0, odb.GetObjects<User>().Count);
			AssertEquals(0, odb.Count(new CriteriaQuery(typeof(User))));
			Println("deleted");
			odb.Close();
			stopWatch.End();
			Println("Total time 2 = " + stopWatch.GetDurationInMiliseconds());
			if (stopWatch.GetDurationInMiliseconds() > 108438)
			{
				Fail("time is > than " + 108438 + " = " + stopWatch.GetDurationInMiliseconds());
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test1WithCommit2()
		{
			if (!runAll)
			{
				return;
			}
			NeoDatis.Tool.StopWatch stopWatch = new NeoDatis.Tool.StopWatch();
			stopWatch.Start();
			int size = 2;
			int size2 = 1;
			int nbFunctions = 1;
			DeleteBase(FileName);
			NeoDatis.Odb.ODB odb = Open(FileName);
			Function f1 = new Function(
				"function 1");
			Println(odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(
				User))));
			// Create Objects
			for (int i = 0; i < size; i++)
			{
				NeoDatis.Odb.Test.VO.Login.Profile p = new NeoDatis.Odb.Test.VO.Login.Profile("profile number "
					 + i, f1);
				for (int j = 0; j < nbFunctions; j++)
				{
					p.AddFunction(new Function(" inner function of profile : number "
						 + i + " - " + j));
				}
				User user = new User("user name "
					 + i, "user email " + i, p);
				odb.Store(user);
				if (i % 100 == 0)
				{
					Println("insert " + i);
				}
			}
			odb.Close();
			Println("created");
			// Updates 10 times the objects
			odb = Open(FileName);
			NeoDatis.Odb.Objects<User> objects = odb.GetObjects<User>();
			Println("got the object " + objects.Count);
			for (int k = 0; k < 3; k++)
			{
				objects.Reset();
				long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				for (int i = 0; i < size; i++)
				{
					User user = (User)objects.Next
						();
					user.GetProfile().SetName(user.GetProfile().GetName() + "-updated");
					odb.Store(user);
					if (i % 100 == 0)
					{
						Println("update " + i + " - " + k);
					}
				}
				Println("Update " + k + " - " + (NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs
					() - start) + " ms");
			}
			Println("updated");
			Println("deleted");
			odb.Close();
			// Check object count
			odb = Open(FileName);
			objects = odb.GetObjects<User>();
			AssertEquals(objects.Count, size);
			// Check data of the objects
			int a = 0;
			while (objects.HasNext())
			{
				User user = (User)objects.Next
					();
				AssertEquals("user name " + a, user.GetName());
				AssertEquals("user email " + a, user.GetEmail());
				AssertEquals("profile number " + a + "-updated-updated-updated", user.GetProfile(
					).GetName());
				a++;
			}
			Println("checked");
			for (int k = 0; k < 10; k++)
			{
				objects.Reset();
				for (int i = 0; i < size2; i++)
				{
					User user = (User)objects.Next
						();
					user.GetProfile().SetName(user.GetProfile().GetName() + "-updated" + "-");
					odb.Store(user);
				}
			}
			Println("re-updated");
			odb.Close();
			// delete objects
			odb = Open(FileName);
			objects = odb.GetObjects<User>();
			a = 0;
			while (objects.HasNext())
			{
				odb.Delete(objects.Next());
				a++;
			}
			AssertEquals(size, a);
			odb.Close();
			odb = Open(FileName);
			AssertEquals(0, odb.GetObjects<User>().Count);
			AssertEquals(0, odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(typeof(User))));
			Println("deleted");
			odb.Close();
			stopWatch.End();
			Println("Total time 1 = " + stopWatch.GetDurationInMiliseconds());
			if (stopWatch.GetDurationInMiliseconds() > 90700)
			{
				Fail("time is > than " + 90700 + " = " + stopWatch.GetDurationInMiliseconds());
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test1WithCommit3()
		{
			if (!runAll)
			{
				return;
			}
			NeoDatis.Tool.StopWatch stopWatch = new NeoDatis.Tool.StopWatch();
			stopWatch.Start();
			DeleteBase(FileName);
			NeoDatis.Odb.ODB odb = Open(FileName);
			Function f1 = new Function(
				"function 1");
			Println(odb.Count(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(
				User))));
			NeoDatis.Odb.Test.VO.Login.Profile p = new NeoDatis.Odb.Test.VO.Login.Profile("profile number 0"
				, f1);
			p.AddFunction(new Function("f1"));
			User user = new User("user name 0"
				, "user email 0", p);
			odb.Store(user);
			NeoDatis.Odb.Test.VO.Login.Profile p2 = new NeoDatis.Odb.Test.VO.Login.Profile("profile number 0"
				, f1);
			p2.AddFunction(new Function("f2"));
			User user2 = new User("user name 0"
				, "user email 0", p2);
			odb.Store(user2);
			odb.Close();
			odb = Open(FileName);
			NeoDatis.Odb.Objects<User> objects = null;
			for (int k = 0; k < 2; k++)
			{
				System.Console.Out.WriteLine(":" + k);
				objects = odb.GetObjects<User>();
				while (objects.HasNext())
				{
					user = (User)objects.Next();
					user.GetProfile().SetName(user.GetProfile().GetName() + "-updated");
					Println(user.GetProfile().GetName());
					odb.Store(user);
				}
			}
			odb.Close();
			odb = Open(FileName);
			objects = odb.GetObjects<User>();
			AssertEquals(2, objects.Count);
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test1WithCommit4()
		{
			if (!runAll)
			{
				return;
			}
			NeoDatis.Tool.StopWatch stopWatch = new NeoDatis.Tool.StopWatch();
			stopWatch.Start();
			DeleteBase(FileName);
			NeoDatis.Odb.ODB odb = Open(FileName);
			Function f1 = new Function(
				"function1");
			odb.Store(f1);
			Function f2 = new Function(
				"function2");
			odb.Store(f2);
			odb.Close();
			odb = Open(FileName);
            NeoDatis.Odb.Objects<Function> objects = odb.GetObjects<Function>();
			Function f = null;
			Println("got the object " + objects.Count);
			for (int k = 0; k < 2; k++)
			{
				objects.Reset();
				while (objects.HasNext())
				{
					f = (Function)objects.Next();
					f.SetName(f.GetName() + "updated-");
					odb.Store(f);
				}
			}
			odb.Close();
			odb = Open(FileName);
			objects = odb.GetObjects<Function>();
			odb.Close();
		}
	}
}
