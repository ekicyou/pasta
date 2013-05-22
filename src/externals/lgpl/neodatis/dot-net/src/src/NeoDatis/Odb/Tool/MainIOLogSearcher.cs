namespace NeoDatis.Odb.Tool
{
	public class MainIOLogSearcher
	{
		/// <param name="args"></param>
		/// <exception cref="System.IO.IOException"></exception>
		public static void Main(string[] args)
		{
			NeoDatis.Odb.Tool.IOLogSearcher logSearcher = new NeoDatis.Odb.Tool.IOLogSearcher
				(18885, 19025);
			logSearcher.Search("c:/tmp/Document2.txt");
		}
	}
}
