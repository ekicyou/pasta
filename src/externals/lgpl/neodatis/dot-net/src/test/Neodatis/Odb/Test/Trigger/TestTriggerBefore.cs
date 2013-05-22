using NUnit.Framework;
namespace NeoDatis.Odb.Test.Trigger
{
	[TestFixture]
    public class TestTriggerBefore : NeoDatis.Odb.Test.ODBTest
	{
		// fails when the trigger is called after the object introspection (1.9
		// beta2)
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			if (!isLocal && !testNewFeature)
			{
				return;
			}
			NeoDatis.Odb.ODB odb = null;
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.Test.Trigger.MyTriggerBefore myTrigger = new NeoDatis.Odb.Test.Trigger.MyTriggerBefore
				();
			try
			{
				odb = Open(baseName);
				odb.AddInsertTrigger(typeof(NeoDatis.Odb.Test.Trigger.SimpleObject), myTrigger);
				NeoDatis.Odb.Test.Trigger.SimpleObject so = new NeoDatis.Odb.Test.Trigger.SimpleObject
					(5);
				NeoDatis.Odb.OID oid = odb.Store(so);
				AssertEquals(6, so.GetId());
				odb.Close();
				odb = Open(baseName);
				NeoDatis.Odb.Test.Trigger.SimpleObject so2 = (NeoDatis.Odb.Test.Trigger.SimpleObject
					)odb.GetObjectFromId(oid);
				AssertEquals(6, so2.GetId());
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
			DeleteBase(baseName);
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test2UpdateTriggers()
		{
			if (!testNewFeature)
			{
				return;
			}
			NeoDatis.Odb.ODB odb = null;
			string baseName = GetBaseName();
			DeleteBase(baseName);
			NeoDatis.Odb.Test.Trigger.MyUpdateTriggerBefore myTrigger = new NeoDatis.Odb.Test.Trigger.MyUpdateTriggerBefore
				();
			try
			{
				odb = Open(baseName);
				NeoDatis.Odb.Test.Trigger.SimpleObject so = new NeoDatis.Odb.Test.Trigger.SimpleObject
					(5);
				NeoDatis.Odb.OID oid = odb.Store(so);
				AssertEquals(5, so.GetId());
				odb.Close();
				odb = Open(baseName);
				odb.AddUpdateTrigger(typeof(NeoDatis.Odb.Test.Trigger.SimpleObject), myTrigger);
				NeoDatis.Odb.Test.Trigger.SimpleObject so2 = (NeoDatis.Odb.Test.Trigger.SimpleObject
					)odb.GetObjectFromId(oid);
				AssertEquals(5, so2.GetId());
				odb.Store(so2);
				odb.Close();
				AssertEquals(6, so2.GetId());
				odb = Open(baseName);
				so2 = (NeoDatis.Odb.Test.Trigger.SimpleObject)odb.GetObjectFromId(oid);
				AssertEquals(6, so2.GetId());
			}
			finally
			{
				if (odb != null && !odb.IsClosed())
				{
					odb.Close();
				}
			}
			DeleteBase(baseName);
		}
	}
}
