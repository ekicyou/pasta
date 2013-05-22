namespace NeoDatis.Tool
{
	public class DisplayUtility
	{
		public static string ByteArrayToString(byte[] bytes)
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			for (int i = 0; i < bytes.Length; i++)
			{
				buffer.Append((int)bytes[i]).Append(" ");
			}
			return buffer.ToString();
		}

		public static string LongArrayToString(long[] longs)
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			for (int i = 0; i < longs.Length; i++)
			{
				buffer.Append(longs[i]).Append(" ");
			}
			return buffer.ToString();
		}

		public static string ObjectArrayToString(object[] objects)
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			for (int i = 0; i < objects.Length; i++)
			{
				buffer.Append(objects[i]).Append(" ");
			}
			return buffer.ToString();
		}

		public static void Display(string title, System.Collections.ICollection list)
		{
			System.Console.Out.WriteLine("***" + title);
			System.Collections.IEnumerator iterator = list.GetEnumerator();
			int i = 1;
			while (iterator.MoveNext())
			{
				System.Console.Out.WriteLine(i + "=" + iterator.Current);
				i++;
			}
		}

		public static string ListToString(System.Collections.IList list)
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			for (int i = 0; i < list.Count; i++)
			{
				buffer.Append((i + 1) + "=" + list[i]).Append("\n");
			}
			return buffer.ToString();
		}
	}
}
