namespace NeoDatis.Odb.Test.Performance
{
	public class TestBatchInsert : NeoDatis.Odb.Test.ODBTest
	{
		public static int TestSize = 2000000;

		public static readonly string OdbFileName = "perf-batch.neodatis";

		public virtual void TestEmpty()
		{
		}

		// to avoid junit junit.framework.AssertionFailedError: No tests found
		// in ...
		/// <exception cref="System.Exception"></exception>
		public virtual void T1est1(bool force)
		{
			if (!force)
			{
				return;
			}
			//OdbConfiguration.setUseCache(false);
			DeleteBase(OdbFileName);
			//OdbConfiguration.set
			NeoDatis.Odb.ODB odb = Open(OdbFileName);
			for (int i = 0; i < TestSize; i++)
			{
				odb.Store(GetSimpleObjectInstance(i));
				if (i % 10000 == 0)
				{
					NeoDatis.Odb.Impl.Tool.MemoryMonitor.DisplayCurrentMemory(i + " objects", false);
					odb.Close();
					odb = Open(OdbFileName);
				}
			}
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestSelect()
		{
			//OdbConfiguration.setUseCache(false);
			//deleteBase(ODB_FILE_NAME);
			//OdbConfiguration.set
			NeoDatis.Odb.ODB odb = Open(OdbFileName);
			NeoDatis.Odb.Objects<NeoDatis.Odb.Test.VO.Login.Function> functions = odb.GetObjects
				(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(typeof(NeoDatis.Odb.Test.Performance.SimpleObject
				), NeoDatis.Odb.Core.Query.Criteria.Where.Equal("name", "Bonjour, comment allez vous?1000000"
				)));
			odb.Close();
			AssertEquals(1, functions.Count);
		}

		private NeoDatis.Odb.Test.Performance.SimpleObject GetSimpleObjectInstance(int i)
		{
			NeoDatis.Odb.Test.Performance.SimpleObject so = new NeoDatis.Odb.Test.Performance.SimpleObject
				();
			so.SetDate(new System.DateTime());
			so.SetDuration(i);
			so.SetName("Bonjour, comment allez vous?" + i);
			return so;
		}

		/// <exception cref="System.Exception"></exception>
		public static void Main2(string[] args)
		{
			// Thread.sleep(15000);
			// OdbConfiguration.setMessageStreamerClass(HessianMessageStreamer.class);
			NeoDatis.Odb.Test.Performance.TestBatchInsert pt = new NeoDatis.Odb.Test.Performance.TestBatchInsert
				();
			pt.T1est1(true);
		}
	}
}
