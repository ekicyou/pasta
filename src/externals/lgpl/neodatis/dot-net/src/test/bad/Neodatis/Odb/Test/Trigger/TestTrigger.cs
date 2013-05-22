
using NeoDatis.Odb.Test.VO.Login;
namespace NeoDatis.Odb.Test.Trigger
{
	public class TestTrigger : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			if (!isLocal)
			{
				return;
			}
			NeoDatis.Odb.ODB odb = null;
			DeleteBase("trigger.neodatis");
			NeoDatis.Odb.Test.Trigger.MyTrigger myTrigger = new NeoDatis.Odb.Test.Trigger.MyTrigger
				();
			try
			{
				odb = Open("trigger.neodatis");
				odb.AddInsertTrigger(typeof(User), myTrigger);
				Function f1 = new Function(
					"function1");
				Function f2 = new Function(
					"function2");
				Profile profile = new Profile
					("profile1", f1);
				User user = new User("oli", 
					"oli@neodatis.com", profile);
				odb.Store(user);
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
			odb = Open("trigger.neodatis");
			odb.Close();
			DeleteBase("trigger.neodatis");
			AssertEquals(1, myTrigger.nbInsertsBefore);
			AssertEquals(1, myTrigger.nbInsertsAfter);
		}

		// To test if triggers are called on recursive objects
		/// <exception cref="System.Exception"></exception>
		public virtual void Test2()
		{
			if (!isLocal)
			{
				return;
			}
			NeoDatis.Odb.ODB odb = null;
			DeleteBase("trigger.neodatis");
			NeoDatis.Odb.Test.Trigger.MyTrigger myTrigger = new NeoDatis.Odb.Test.Trigger.MyTrigger
				();
			try
			{
				odb = Open("trigger.neodatis");
				odb.AddInsertTrigger(typeof(Function), myTrigger);
				Function f1 = new Function(
					"function1");
				Function f2 = new Function(
					"function2");
				Profile profile = new Profile
					("profile1", f1);
				User user = new User("oli", 
					"oli@neodatis.com", profile);
				odb.Store(user);
				odb.Store(f2);
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
			odb = Open("trigger.neodatis");
			odb.Close();
			DeleteBase("trigger.neodatis");
			AssertEquals(2, myTrigger.nbInsertsBefore);
			AssertEquals(2, myTrigger.nbInsertsAfter);
		}

		// To test select triggers
		/// <exception cref="System.Exception"></exception>
		public virtual void TestSelectTrigger()
		{
			if (!isLocal)
			{
				return;
			}
			NeoDatis.Odb.ODB odb = null;
			DeleteBase("trigger.neodatis");
			NeoDatis.Odb.Test.Trigger.MySelectTrigger myTrigger = new NeoDatis.Odb.Test.Trigger.MySelectTrigger
				();
			try
			{
				odb = Open("trigger.neodatis");
				Function f1 = new Function(
					"function1");
				Function f2 = new Function(
					"function2");
				Profile profile = new Profile
					("profile1", f1);
				User user = new User("oli", 
					"oli@neodatis.com", profile);
				odb.Store(user);
				odb.Store(f2);
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
			odb = Open("trigger.neodatis");
			odb.AddSelectTrigger(typeof(Function), myTrigger);
			NeoDatis.Odb.Objects<Function> functions = odb.GetObjects<Function>();
			odb.Close();
			DeleteBase("trigger.neodatis");
			AssertEquals(2, functions.Count);
			AssertEquals(2, myTrigger.nbCalls);
		}
	}
}
