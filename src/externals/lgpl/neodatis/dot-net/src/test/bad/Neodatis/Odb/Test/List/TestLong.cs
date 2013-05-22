namespace NeoDatis.Odb.Test.List
{
	public class TestLong : NeoDatis.Odb.Test.NeoDatisAssert
	{
		/// <summary>Just check ordering of LinkedHashMap</summary>
		public virtual void TestOrderedMap()
		{
			System.Collections.IDictionary m = new Java.Util.LinkedHashMap();
			for (int i = 0; i < 10; i++)
			{
				m.Add("key" + i, "value" + i);
			}
			System.Collections.IEnumerator iterator = m.Keys.GetEnumerator();
			int j = 0;
			while (iterator.MoveNext())
			{
				AssertEquals("key" + j, iterator.Current);
				j++;
			}
		}

		public static void Main2(string[] args)
		{
			NeoDatis.Odb.Impl.Tool.MemoryMonitor.DisplayCurrentMemory("start", true);
			int size = 3400000;
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			System.Collections.IList l = new System.Collections.ArrayList();
			for (int i = 0; i < size; i++)
			{
				l.Add(new NeoDatis.Odb.Test.List.MyInt(i));
			}
			long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			NeoDatis.Odb.Impl.Tool.MemoryMonitor.DisplayCurrentMemory("end " + (end - start) 
				+ "ms", true);
		}
	}
}
