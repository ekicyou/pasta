namespace NeoDatis.Odb.Test.Nativemap
{
	public class NativeLongMap
	{
		internal int initialCapacity;

		internal int size;

		internal int secondSize;

		internal object[] array;

		public NativeLongMap(int initialCapacity)
		{
			array = new object[initialCapacity];
			size = initialCapacity;
			secondSize = size / 10;
		}

		public virtual object Get(long key)
		{
			int tkey = (int)key % size;
			NeoDatis.Odb.Test.Nativemap.Entry[] entries = (NeoDatis.Odb.Test.Nativemap.Entry[]
				)array[tkey];
			if (entries == null)
			{
				return null;
			}
			int i = 0;
			while (i < entries.Length)
			{
				if (entries[i] == null)
				{
					return null;
				}
				if (entries[i].key == key)
				{
					return entries[i].o;
				}
			}
			return null;
		}

		public virtual void Put(long key, object o)
		{
			int tkey = (int)key % size;
			NeoDatis.Odb.Test.Nativemap.Entry[] entries = null;
			if (array[tkey] == null)
			{
				entries = new NeoDatis.Odb.Test.Nativemap.Entry[secondSize];
				entries[0] = new NeoDatis.Odb.Test.Nativemap.Entry(key, o);
				array[tkey] = entries;
				return;
			}
			int i = 0;
			while (i < entries.Length)
			{
				if (entries[i] == null)
				{
					entries[i] = new NeoDatis.Odb.Test.Nativemap.Entry(key, o);
					return;
				}
				i++;
			}
			throw new System.Exception("Second array explosion");
		}

		public static void Main2(string[] args)
		{
			NeoDatis.Odb.Test.Nativemap.NativeLongMap nlm = new NeoDatis.Odb.Test.Nativemap.NativeLongMap
				(100);
			string s = new string("Ola");
			nlm.Put(1, s);
		}
	}

	internal class Entry
	{
		public long key;

		public object o;

		public Entry(long key, object o)
		{
			this.key = key;
			this.o = o;
		}
	}
}
