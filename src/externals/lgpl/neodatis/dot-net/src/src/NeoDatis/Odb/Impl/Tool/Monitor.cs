namespace NeoDatis.Odb.Impl.Tool
{
	public class Monitor
	{
		public static void DisplayCurrentMemory(string label, bool all)
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append(label).Append(":Free=").Append(+Java.Lang.Runtime.GetRuntime().FreeMemory
				() / 1024).Append("k / Total=").Append(Java.Lang.Runtime.GetRuntime().TotalMemory
				() / 1024).Append("k");
			if (all)
			{
				buffer.Append(" - Cache Usage = ").Append(NeoDatis.Odb.Impl.Core.Transaction.Cache
					.Usage());
			}
			System.Console.Out.WriteLine(buffer.ToString());
		}
	}
}
