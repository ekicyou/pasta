using NUnit.Framework;
namespace NeoDatis.Odb.Test.Performance
{
	[TestFixture]
    public class TestWeakReference : NeoDatis.Odb.Test.ODBTest
	{
		[Test]
        public virtual void Test1()
		{
			int size = 200000;
			System.Collections.IDictionary map = new Java.Util.WeakHashMap();
			System.Collections.IList l = new System.Collections.ArrayList();
			for (int i = 0; i < size; i++)
			{
				object o = GetSimpleObjectInstance(i);
				l.Add(o);
				if (i % 50000 == 0)
				{
					Println("i=" + i);
				}
				map.Add(o, new Java.Lang.Ref.WeakReference(o));
			}
			Println("Test 1 ok");
			Println("Map size " + map.Count);
		}

		[Test]
        public virtual void Test1WithoutWeak()
		{
			int size = 40000;
			System.Collections.IDictionary map = new NeoDatis.Tool.Wrappers.Map.OdbHashMap();
			for (int i = 0; i < size; i++)
			{
				object o = GetSimpleObjectInstance(i);
				if (i % 50000 == 0)
				{
					NeoDatis.Odb.Impl.Tool.MemoryMonitor.DisplayCurrentMemory(string.Empty + i, true);
				}
				map.Add(o, o);
			}
			Println("Test 1 ok");
			Println("Map size " + map.Count);
		}

		[Test]
        public virtual void Test2()
		{
			int size = 20000;
			System.Collections.IDictionary map = new Java.Util.WeakHashMap();
			for (int i = 0; i < size; i++)
			{
				object o = GetSimpleObjectInstance(i);
				if (i % 50000 == 0)
				{
					Println("i=" + i);
				}
				map.Add(System.Convert.ToInt64(i), new Java.Lang.Ref.WeakReference(o));
			}
			Println("Test 2 ok");
			Println("Map size " + map.Count);
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

		public static void Main2(string[] args)
		{
			NeoDatis.Odb.Test.Performance.TestWeakReference t = new NeoDatis.Odb.Test.Performance.TestWeakReference
				();
			t.Test1();
		}
	}
}
