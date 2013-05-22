namespace NeoDatis.Odb.Test.Index
{
	public class TestCreateObjectAfterInsert : NeoDatis.Odb.Test.ODBTest
	{
		public virtual void TestLong()
		{
			Println(string.Empty + long.MaxValue);
			long l = long.MaxValue - 1;
			l = l + 1;
			Println(string.Empty + l);
			Println(string.Empty + l + 1);
		}

		/// <summary>Test the creation of an index after having created objects.</summary>
		/// <remarks>
		/// Test the creation of an index after having created objects. In this case
		/// ODB should creates the index and update it with already existing objects
		/// </remarks>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test1Object()
		{
			string OdbFileName = "index2";
			NeoDatis.Odb.ODB odb = null;
			try
			{
				DeleteBase(OdbFileName);
				odb = Open(OdbFileName);
				NeoDatis.Odb.Test.Index.IndexedObject io = new NeoDatis.Odb.Test.Index.IndexedObject
					("name", 5, new System.DateTime());
				odb.Store(io);
				odb.Close();
				odb = Open(OdbFileName);
				string[] names = new string[] { "name" };
				odb.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject)).AddUniqueIndexOn
					("index1", names, true);
				NeoDatis.Odb.Objects objects = odb.GetObjects(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("name", "name")), true);
				AssertEquals(1, objects.Count);
			}
			catch (System.Exception e)
			{
				Sharpen.Runtime.PrintStackTrace(e);
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
			}
		}

		/// <summary>Test the creation of an index after having created objects.</summary>
		/// <remarks>
		/// Test the creation of an index after having created objects. In this case
		/// ODB should creates the index and update it with already existing objects
		/// </remarks>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test20000Objects()
		{
			string OdbFileName = "index2";
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.ODB odb = null;
			int size = isLocal ? 20000 : 2000;
			try
			{
				DeleteBase(OdbFileName);
				odb = Open(OdbFileName);
				for (int i = 0; i < size; i++)
				{
					NeoDatis.Odb.Test.Index.IndexedObject io = new NeoDatis.Odb.Test.Index.IndexedObject
						("name" + i, i, new System.DateTime());
					odb.Store(io);
				}
				odb.Close();
				odb = Open(OdbFileName);
				string[] names = new string[] { "name" };
				odb.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject)).AddUniqueIndexOn
					("index1", names, true);
				NeoDatis.Odb.Objects objects = odb.GetObjects(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("name", "name0")), true);
				AssertEquals(1, objects.Count);
				objects = odb.GetObjects(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(
					typeof(NeoDatis.Odb.Test.Index.IndexedObject)), true);
				NeoDatis.Odb.Impl.Tool.MemoryMonitor.DisplayCurrentMemory("BTREE", true);
				AssertEquals(size, objects.Count);
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
				long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				Println((end - start) + "ms");
			}
		}

		/// <summary>Test the creation of an index after having created objects.</summary>
		/// <remarks>
		/// Test the creation of an index after having created objects. In this case
		/// ODB should creates the index and update it with already existing objects
		/// </remarks>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test100000Objects()
		{
			string OdbFileName = "index2";
			NeoDatis.Odb.ODB odb = null;
			int size = isLocal ? 100000 : 10001;
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.OdbConfiguration.MonitorMemory(true);
			NeoDatis.Odb.OdbConfiguration.SetReconnectObjectsToSession(false);
			try
			{
				Println("MaxNbObjects/cache = " + NeoDatis.Odb.OdbConfiguration.GetMaxNumberOfObjectInCache
					());
				DeleteBase(OdbFileName);
				odb = Open(OdbFileName);
				for (int i = 0; i < size; i++)
				{
					NeoDatis.Odb.Test.Index.IndexedObject io = new NeoDatis.Odb.Test.Index.IndexedObject
						("name" + i, i, new System.DateTime());
					odb.Store(io);
					if (i % 10000 == 0)
					{
						NeoDatis.Odb.Impl.Tool.MemoryMonitor.DisplayCurrentMemory(i + " objects created", 
							true);
					}
				}
				odb.Close();
				Println("\n\n END OF INSERT \n\n");
				odb = Open(OdbFileName);
				string[] names = new string[] { "name" };
				odb.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject)).AddUniqueIndexOn
					("index1", names, true);
				Println("\n\n after create index\n\n");
				NeoDatis.Odb.Objects objects = odb.GetObjects(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("name", "name0")), true);
				Println("\n\nafter get Objects\n\n");
				AssertEquals(1, objects.Count);
				objects = odb.GetObjects(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(
					typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("duration", 9)), true);
				AssertEquals(1, objects.Count);
				objects = odb.GetObjects(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(
					typeof(NeoDatis.Odb.Test.Index.IndexedObject)), true);
				AssertEquals(size, objects.Count);
			}
			catch (System.Exception e)
			{
				throw;
			}
			finally
			{
				long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				Println((end - start) + "ms");
				NeoDatis.Odb.OdbConfiguration.MonitorMemory(false);
				odb.Close();
			}
		}

		/// <summary>Test the creation of an index after having created objects.</summary>
		/// <remarks>
		/// Test the creation of an index after having created objects. In this case
		/// ODB should creates the index and update it with already existing objects
		/// </remarks>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void Test100000ObjectsIntiNdex()
		{
			string OdbFileName = "index2";
			NeoDatis.Odb.ODB odb = null;
			int size = isLocal ? 90000 : 10100;
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.OdbConfiguration.MonitorMemory(true);
			try
			{
				DeleteBase(OdbFileName);
				odb = Open(OdbFileName);
				for (int i = 0; i < size; i++)
				{
					NeoDatis.Odb.Test.Index.IndexedObject io = new NeoDatis.Odb.Test.Index.IndexedObject
						("name" + i, i, new System.DateTime());
					odb.Store(io);
					if (i % 10000 == 0)
					{
						NeoDatis.Odb.Impl.Tool.MemoryMonitor.DisplayCurrentMemory(i + " objects created", 
							true);
					}
				}
				odb.Close();
				Println("\n\n END OF INSERT \n\n");
				odb = Open(OdbFileName);
				string[] names = new string[] { "duration" };
				odb.GetClassRepresentation(typeof(NeoDatis.Odb.Test.Index.IndexedObject)).AddUniqueIndexOn
					("index1", names, true);
				Println("\n\n after create index\n\n");
				NeoDatis.Odb.Objects objects = odb.GetObjects(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("name", "name0")), true);
				Println("\n\nafter get Objects\n\n");
				AssertEquals(1, objects.Count);
				objects = odb.GetObjects(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(
					typeof(NeoDatis.Odb.Test.Index.IndexedObject), NeoDatis.Odb.Core.Query.Criteria.Where
					.Equal("duration", 10000)), true);
				AssertEquals(1, objects.Count);
				objects = odb.GetObjects(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(
					typeof(NeoDatis.Odb.Test.Index.IndexedObject)), true);
				AssertEquals(size, objects.Count);
			}
			catch (System.Exception e)
			{
				throw;
			}
			finally
			{
				long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				Println((end - start) + "ms");
				NeoDatis.Odb.OdbConfiguration.MonitorMemory(false);
			}
		}
	}
}
