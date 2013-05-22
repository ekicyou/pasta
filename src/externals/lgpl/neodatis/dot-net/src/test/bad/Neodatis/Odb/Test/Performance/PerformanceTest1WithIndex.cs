using NeoDatis.Odb.Impl.Core.Query.Criteria;
using NeoDatis.Odb.Core.Query.Criteria;
namespace NeoDatis.Odb.Test.Performance
{
	public class PerformanceTest1WithIndex : NeoDatis.Odb.Test.ODBTest
	{
		public static int TestSize = 110;

		public static string OdbFileName = "perf.neodatis";

		/// <exception cref="System.Exception"></exception>
		public virtual void Test1()
		{
			if (!testPerformance)
			{
				return;
			}
			if (isLocal)
			{
				T1estInsertSimpleObjectODB(20000);
			}
			else
			{
				T1estInsertSimpleObjectODB(2000);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Test2()
		{
			T1estInsertSimpleObjectODB(200);
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void T1estInsertSimpleObjectODB(int size)
		{
			OdbFileName = "perf-" + size + ".neodatis";
			TestSize = size;
			bool doUpdate = true;
			bool doDelete = true;
			int commitInterval = 100;
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
			NeoDatis.Odb.Objects<SimpleObject> l = null;
			SimpleObject so = null;
			// Insert TEST_SIZE objects
			Println("Inserting " + TestSize + " objects");
			t1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			odb = Open(OdbFileName);
			string[] fields = new string[] { "name" };
			odb.GetClassRepresentation(typeof(SimpleObject)).AddUniqueIndexOn
				("index1", fields, true);
			for (int i = 0; i < TestSize; i++)
			{
				object o = GetSimpleObjectInstance(i);
				odb.Store(o);
				if (i % 10000 == 0)
				{
					// println("i="+i);
					NeoDatis.Odb.Impl.Tool.MemoryMonitor.DisplayCurrentMemory(string.Empty + i, false
						);
				}
			}
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
			t4 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.Core.Query.IQuery q = null;
			for (int j = 0; j < TestSize; j++)
			{
				// println("Bonjour, comment allez vous?" + j);
				q = new CriteriaQuery(Where.Equal("name", "Bonjour, comment allez vous?" + j));
				NeoDatis.Odb.Objects<SimpleObject> objects = odb.GetObjects<SimpleObject>(q);
				AssertTrue(q.GetExecutionPlan().UseIndex());
				so = objects.GetFirst();
				if (!so.GetName().Equals("Bonjour, comment allez vous?" + j))
				{
					throw new System.Exception("error while getting object : expected = " + "Bonjour, comment allez vous?"
						 + j + " / actual = " + so.GetName());
				}
				if (j % 1000 == 0)
				{
					Println("got " + j + " objects");
				}
			}
			t5 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			odb.Close();
			odb = Open(OdbFileName);
			if (doUpdate)
			{
				Println("Updating " + TestSize + " objects");
				so = null;
				l = odb.GetObjects<SimpleObject>( inMemory);
				while (l.HasNext())
				{
					so = (SimpleObject)l.Next();
					so.SetName(so.GetName().ToUpper());
					odb.Store(so);
				}
			}
			t6 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			odb.Close();
			// if(true)return;
			t7 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			if (doDelete)
			{
				Println("Deleting " + TestSize + " objects");
				odb = Open(OdbFileName);
				Println("After open - before delete");
				l = odb.GetObjects<SimpleObject>(inMemory);
				t77 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				Println("After getting objects - before delete");
				int i = 0;
				while (l.HasNext())
				{
					so = (SimpleObject)l.Next();
					if (!so.GetName().StartsWith("BONJOUR"))
					{
						throw new System.Exception("Update  not ok for " + so.GetName());
					}
					odb.Delete(so);
					if (i % 10000 == 0)
					{
						Println("s=" + i);
					}
					// println("Cache="+Dummy.getEngine(odb).getSession().getCache().toString());
					i++;
				}
				odb.Close();
			}
			t8 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			// t4 2 times
			DisplayResult("ODB " + TestSize + " SimpleObject objects ", t1, t2, t4, t4, t5, t6
				, t7, t77, t8);
		}

		private SimpleObject GetSimpleObjectInstance(int i)
		{
			SimpleObject so = new SimpleObject
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
			if (isLocal)
			{
				AssertTrue("Performance", tinsert < 1050);
				AssertTrue("Performance", tselect < 535);
				AssertTrue("Performance", tupdate < 582);
				AssertTrue("Performance", tdelete < 740);
			}
			else
			{
				// System.out.println(tinsert);
				// System.out.println(tselect);
				// System.out.println(tupdate);
				// System.out.println(tdelete);
				AssertTrue(tinsert < 17000);
				AssertTrue(tselect < 25000);
				AssertTrue(tupdate < 32000);
				AssertTrue(tdelete < 15500);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public static void Main2(string[] args)
		{
			NeoDatis.Odb.Test.Performance.PerformanceTest1WithIndex pt = new NeoDatis.Odb.Test.Performance.PerformanceTest1WithIndex
				();
			// Thread.sleep(20000);
			// LogUtil.allOn(true);
			pt.T1estInsertSimpleObjectODB(10000);
		}
	}
}
