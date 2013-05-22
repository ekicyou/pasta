using NeoDatis.Tool.Wrappers;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.Performance
{
	public class PerformanceTest1OnlySelect
	{
		public static int TestSize = 50000;

		public static readonly string OdbFileName = "perf-select.neodatis";

		/// <exception cref="System.Exception"></exception>
		public virtual void BuildBase()
		{
			bool inMemory = true;
			// Deletes the database file
			NeoDatis.Tool.IOUtil.DeleteFile(OdbFileName);
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
			NeoDatis.Odb.Test.Performance.SimpleObject so = null;
			// Insert TEST_SIZE objects
			System.Console.Out.WriteLine("Inserting " + TestSize + " objects");
			t1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			odb = NeoDatis.Odb.ODBFactory.Open(OdbFileName);
			for (int i = 0; i < TestSize; i++)
			{
				object o = GetSimpleObjectInstance(i);
				odb.Store(o);
				if (i % 10000 == 0)
				{
					// System.out.println("i="+i);
					NeoDatis.Odb.Impl.Tool.MemoryMonitor.DisplayCurrentMemory(string.Empty + i, true);
				}
			}
			// System.out.println("Cache="+Dummy.getEngine(odb).getSession().getCache().toString());
			t2 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			// Closes the database
			odb.Close();
		}

		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void TestSelectSimpleObjectODB()
		{
			long t3 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			bool inMemory = true;
			System.Console.Out.WriteLine("Retrieving " + TestSize + " objects");
			// Reopen the database
			NeoDatis.Odb.ODB odb = NeoDatis.Odb.ODBFactory.Open(OdbFileName);
			// Gets the TEST_SIZE objects
			NeoDatis.Odb.Objects<SimpleObject> l = odb.GetObjects<SimpleObject>(inMemory);
			System.Console.Out.WriteLine(l.GetType().FullName);
			long t4 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			System.Console.Out.WriteLine("l.size=" + l.Count);
			int i = 0;
			while (l.HasNext())
			{
				object o = l.Next();
				if (i % 10000 == 0)
				{
					NeoDatis.Odb.Impl.Tool.MemoryMonitor.DisplayCurrentMemory("select " + i, true);
				}
				// System.out.println("Cache="+Dummy.getEngine(odb).getSession().getCache().toString());
				i++;
			}
			long t5 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			odb.Close();
			DisplayResult("ODB " + TestSize + " SimpleObject objects ", t3, t4, t5);
			System.Console.Out.WriteLine("buffer Ok=" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Buffer.MultiBufferedIO
				.nbBufferOk + " / buffer not ok =" + NeoDatis.Odb.Impl.Core.Layers.Layer3.Buffer.MultiBufferedIO
				.nbBufferNotOk);
			System.Console.Out.WriteLine("nb1=" + NeoDatis.Odb.Core.Layers.Layer3.Engine.FileSystemInterface
				.nbCall1 + " / nb2 =" + NeoDatis.Odb.Core.Layers.Layer3.Engine.FileSystemInterface
				.nbCall2);
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

		private void DisplayResult(string @string, long t1, long t2, long t3)
		{
			string s1 = " total=" + (t3 - t1);
			string s3 = " total select=" + (t3 - t1) + " -- " + "select=" + (t2 - t1) + " get="
				 + (t3 - t2);
			string s4 = " time/object=" + (float)(t3 - t1) / +TestSize;
			System.Console.Out.WriteLine(@string + s1 + " | " + s3 + " | " + s4);
		}

		/// <exception cref="System.Exception"></exception>
		public static void Main2(string[] args)
		{
			NeoDatis.Odb.Test.Performance.PerformanceTest1OnlySelect pt = new NeoDatis.Odb.Test.Performance.PerformanceTest1OnlySelect
				();
			OdbThread.Sleep(20000);
			pt.TestSelectSimpleObjectODB();
		}
	}
}
