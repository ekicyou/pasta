namespace NeoDatis.Odb.Test.Trigger
{
	public class TestAutoIncrementTrigger : NeoDatis.Odb.Test.ODBTest
	{
		public static readonly string Base = "trigger-auto-increment.neodatis";

		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			if (!isLocal)
			{
				return;
			}
			NeoDatis.Odb.ODB odb = null;
			DeleteBase(Base);
			try
			{
				odb = Open(Base);
				odb.AddInsertTrigger(typeof(NeoDatis.Odb.Test.Trigger.ObjectWithAutoIncrementId), 
					new NeoDatis.Odb.Test.Trigger.LocalAutoIncrementTrigger());
				NeoDatis.Odb.Test.Trigger.ObjectWithAutoIncrementId o = new NeoDatis.Odb.Test.Trigger.ObjectWithAutoIncrementId
					("Object 1");
				odb.Store(o);
				AssertEquals(1, o.GetId());
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
		public virtual void Test2Objects()
		{
			if (!isLocal)
			{
				return;
			}
			NeoDatis.Odb.ODB odb = null;
			DeleteBase(Base);
			try
			{
				odb = Open(Base);
				odb.AddInsertTrigger(typeof(NeoDatis.Odb.Test.Trigger.ObjectWithAutoIncrementId), 
					new NeoDatis.Odb.Test.Trigger.LocalAutoIncrementTrigger());
				NeoDatis.Odb.Test.Trigger.ObjectWithAutoIncrementId o = new NeoDatis.Odb.Test.Trigger.ObjectWithAutoIncrementId
					("Object 1");
				odb.Store(o);
				AssertEquals(1, o.GetId());
				odb.Close();
				odb = Open(Base);
				odb.AddInsertTrigger(typeof(NeoDatis.Odb.Test.Trigger.ObjectWithAutoIncrementId), 
					new NeoDatis.Odb.Test.Trigger.LocalAutoIncrementTrigger());
				o = new NeoDatis.Odb.Test.Trigger.ObjectWithAutoIncrementId("Object 2");
				odb.Store(o);
				AssertEquals(2, o.GetId());
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
		public virtual void Test1000Objects()
		{
			if (!isLocal)
			{
				return;
			}
			NeoDatis.Odb.ODB odb = null;
			DeleteBase(Base);
			try
			{
				odb = Open(Base);
				odb.AddInsertTrigger(typeof(NeoDatis.Odb.Test.Trigger.ObjectWithAutoIncrementId), 
					new NeoDatis.Odb.Test.Trigger.LocalAutoIncrementTrigger());
				for (int i = 0; i < 1000; i++)
				{
					NeoDatis.Odb.Test.Trigger.ObjectWithAutoIncrementId o = new NeoDatis.Odb.Test.Trigger.ObjectWithAutoIncrementId
						("Object " + (i + 1));
					odb.Store(o);
					AssertEquals(i + 1, o.GetId());
				}
				odb.Close();
				odb = Open(Base);
				odb.AddInsertTrigger(typeof(NeoDatis.Odb.Test.Trigger.ObjectWithAutoIncrementId), 
					new NeoDatis.Odb.Test.Trigger.LocalAutoIncrementTrigger());
				for (int i = 0; i < 1000; i++)
				{
					NeoDatis.Odb.Test.Trigger.ObjectWithAutoIncrementId o = new NeoDatis.Odb.Test.Trigger.ObjectWithAutoIncrementId
						("Object - bis - " + (i + 1));
					odb.Store(o);
					AssertEquals(1000 + i + 1, o.GetId());
				}
				odb.Close();
			}
			finally
			{
			}
		}
	}
}
