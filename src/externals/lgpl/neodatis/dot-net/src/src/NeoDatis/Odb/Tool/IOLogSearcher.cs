namespace NeoDatis.Odb.Tool
{
	public class IOLogSearcher
	{
		private long start;

		private long end;

		private System.Collections.IList result;

		public IOLogSearcher(long start, long end)
		{
			this.start = start;
			this.end = end;
			result = new System.Collections.ArrayList(10000);
		}

		/// <exception cref="System.IO.IOException"></exception>
		public virtual void Search(string fileName)
		{
			Java.IO.BufferedReader @in = new Java.IO.BufferedReader(new Java.IO.FileReader(fileName
				));
			string line = null;
			do
			{
				line = @in.ReadLine();
				if (line != null)
				{
					ManageOneLine(line);
				}
			}
			while (line != null);
		}

		public virtual void ManageOneLine(string line)
		{
			if (!line.StartsWith("writing"))
			{
				return;
			}
			string[] array = line.Split(" ");
			string type = array[1];
			string data = array[2];
			string position = array[4];
			long nposition = long.Parse(position);
			if (nposition >= start && nposition <= end)
			{
				//result.add(line);
				System.Console.Out.WriteLine(line);
			}
		}
	}
}
