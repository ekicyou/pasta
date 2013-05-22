namespace NeoDatis.Tool.Mutex
{
	/// <summary>A mutex factory</summary>
	/// <author>osmadja</author>
	public class MutexFactory
	{
		private static System.Collections.Generic.IDictionary<string, NeoDatis.Tool.Mutex.Mutex
			> mutexs = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, NeoDatis.Tool.Mutex.Mutex
			>();

		private static bool debug = false;

		public static NeoDatis.Tool.Mutex.Mutex Get(string name)
		{
			lock (typeof(MutexFactory))
			{
				NeoDatis.Tool.Mutex.Mutex mutex = mutexs[name];
				if (mutex == null)
				{
					mutex = new NeoDatis.Tool.Mutex.Mutex(name);
					mutex.SetDebug(debug);
					mutexs.Add(name, mutex);
				}
				return mutex;
			}
		}

		public static void SetDebug(bool debugValue)
		{
			debug = debugValue;
		}
	}
}
