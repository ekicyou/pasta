namespace NeoDatis.Odb.Test.Performance
{
	public class PerformanceTest1 : NeoDatis.Odb.Test.ODBTest
	{
		public static int TestSize = 50000;

		public static readonly string OdbFileName = "perf.neodatis";

		public virtual void TestEmpty()
		{
		}

		// to avoid junit junit.framework.AssertionFailedError: No tests found
		// in ...
		/// <exception cref="System.Exception"></exception>
		public virtual void TestInsertSimpleObjectODB(bool force)
		{
			if (!force && !runAll)
			{
				return;
			}
			bool reconnectStatus = NeoDatis.Odb.OdbConfiguration.ReconnectObjectsToSession();
			//OdbConfiguration.setReconnectObjectsToSession(false);
			// Thread.sleep(20000);
			bool doUpdate = true;
			bool doDelete = true;
			// Configuration.setDatabaseCharacterEncoding(null);
			// LogUtil.logOn(FileSystemInterface.LOG_ID,true);
			// LogUtil.logOn(ObjectReader.LOG_ID,true);
			// Configuration.setUseLazyCache(true);
			bool inMemory = true;
			// Configuration.monitorMemory(true);
			// Configuration.setUseModifiedClass(true);
			// Deletes the database file
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
			NeoDatis.Odb.Objects l = null;
			NeoDatis.Odb.Test.Performance.SimpleObject so = null;
			// Insert TEST_SIZE objects
			Println("Inserting " + TestSize + " objects");
			t1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			odb = Open(OdbFileName);
			int i = 0;
			// odb.getClassRepresentation(SimpleObject.class).addFullInstantiationHelper(new
			// SimpleObjectFullInstantiationHelper());
			for (i = 0; i < TestSize; i++)
			{
				object o = GetSimpleObjectInstance(i);
				odb.Store(o);
				if (i % 10000 == 0)
				{
				}
			}
			// println("i="+i);
			// Monitor.displayCurrentMemory(""+i,true);
			// println("Cache="+Dummy.getEngine(odb).getSession().getCache().toString());
			t2 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			// Closes the database
			odb.Close();
			// if(true)return;
			t3 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			Println("Retrieving " + TestSize + " objects");
			// Reopen the database
			odb = Open(OdbFileName);
			// Gets the TEST_SIZE objects
			l = odb.GetObjects(typeof(NeoDatis.Odb.Test.Performance.SimpleObject), inMemory);
			t4 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			i = 0;
			while (l.HasNext())
			{
				object o = l.Next();
				if (i % 10000 == 0)
				{
				}
				// Monitor.displayCurrentMemory("select "+i,true);
				i++;
			}
			t5 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			if (doUpdate)
			{
				Println("Updating " + TestSize + " objects");
				i = 0;
				so = null;
				l.Reset();
				while (l.HasNext())
				{
					so = (NeoDatis.Odb.Test.Performance.SimpleObject)l.Next();
					so.SetName(so.GetName() + " updated");
					odb.Store(so);
					if (i % 10000 == 0)
					{
					}
					// Monitor.displayCurrentMemory(""+i);
					i++;
				}
			}
			t6 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			odb.Close();
			t7 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			if (doDelete)
			{
				Println("Deleting " + TestSize + " objects");
				odb = Open(OdbFileName);
				l = odb.GetObjects(typeof(NeoDatis.Odb.Test.Performance.SimpleObject), inMemory);
				t77 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				// println("After getting objects - before delete");
				i = 0;
				while (l.HasNext())
				{
					so = (NeoDatis.Odb.Test.Performance.SimpleObject)l.Next();
					if (!so.GetName().EndsWith("updated"))
					{
						throw new System.Exception("Update  not ok for " + so.GetName());
					}
					odb.Delete(so);
					if (i % 10000 == 0)
					{
					}
					// println("s="+i);
					i++;
				}
				odb.Close();
			}
			Java.Lang.Thread.Sleep(5000);
			Print(string.Format("NbFinalizers=%d   NbQueue=%d", NeoDatis.Odb.Test.Performance.SimpleObject
				.nbgc, NeoDatis.Odb.Impl.Core.Transaction.ReferenceQueueThread.nbObjects));
			t8 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.OdbConfiguration.SetReconnectObjectsToSession(reconnectStatus);
			DisplayResult("ODB " + TestSize + " SimpleObject objects ", t1, t2, t3, t4, t5, t6
				, t7, t77, t8);
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

		private void DisplayResult(string @string, long t1, long t2, long t3, long t4, long
			 t5, long t6, long t7, long t77, long t8)
		{
			string s1 = " total=" + (t8 - t1);
			string s2 = " total insert=" + (t3 - t1) + " -- " + "insert=" + (t2 - t1) + " commit="
				 + (t3 - t2) + " o/s=" + (float)TestSize / (float)((t3 - t1)) * 1000;
			string s3 = " total select=" + (t5 - t3) + " -- " + "select=" + (t4 - t3) + " get="
				 + (t5 - t4) + " o/s=" + (float)TestSize / (float)((t5 - t3)) * 1000;
			string s4 = " total update=" + (t7 - t5) + " -- " + "update=" + (t6 - t5) + " commit="
				 + (t7 - t6) + " o/s=" + (float)TestSize / (float)((t7 - t5)) * 1000;
			string s5 = " total delete=" + (t8 - t7) + " -- " + "select=" + (t77 - t7) + " - delete="
				 + (t8 - t77) + " o/s=" + (float)TestSize / (float)((t8 - t7)) * 1000;
			Println(@string + s1 + " | " + s2 + " | " + s3 + " | " + s4 + " | " + s5);
			long tinsert = t3 - t1;
			long tselect = t5 - t3;
			long tupdate = t7 - t5;
			long tdelete = t8 - t7;
			Println("Nb buffers ok = " + NeoDatis.Odb.Impl.Core.Layers.Layer3.Buffer.MultiBufferedIO
				.nbBufferOk + "   /   nb not ok = " + NeoDatis.Odb.Impl.Core.Layers.Layer3.Buffer.MultiBufferedIO
				.nbBufferNotOk);
			Println("Nb flushs= " + NeoDatis.Odb.Impl.Core.Layers.Layer3.Buffer.MultiBufferedIO
				.numberOfFlush + "   /   flush size = " + NeoDatis.Odb.Impl.Core.Layers.Layer3.Buffer.MultiBufferedIO
				.totalFlushSize + " / NbFlushForOverlap=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Buffer.MultiBufferedIO
				.nbFlushForOverlap);
			// println("Same position write = "+
			// MultiBufferedIO.nbSamePositionForWrite+
			// "   /   same pos for read = "+
			// MultiBufferedIO.nbSamePositionForRead);
			Println("Nb  =" + NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.nb);
			AssertTrue("Performance", tinsert < 1050);
			AssertTrue("Performance", tselect < 535);
			AssertTrue("Performance", tupdate < 582);
			AssertTrue("Performance", tdelete < 740);
		}

		/// <exception cref="System.Exception"></exception>
		public static void Main2(string[] args)
		{
			// Thread.sleep(15000);
			// OdbConfiguration.setMessageStreamerClass(HessianMessageStreamer.class);
			NeoDatis.Odb.Test.Performance.PerformanceTest1 pt = new NeoDatis.Odb.Test.Performance.PerformanceTest1
				();
			pt.TestInsertSimpleObjectODB(true);
		}
	}

	internal class SimpleObjectFullInstantiationHelper : NeoDatis.Odb.Core.Layers.Layer2.Instance.FullInstantiationHelper
	{
		public virtual object Instantiate(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi)
		{
			NeoDatis.Odb.Test.Performance.SimpleObject so = new NeoDatis.Odb.Test.Performance.SimpleObject
				();
			return so;
		}
	}
}
