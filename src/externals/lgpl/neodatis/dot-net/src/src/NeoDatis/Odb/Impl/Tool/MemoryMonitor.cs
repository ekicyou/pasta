using System.Diagnostics;

namespace NeoDatis.Odb.Impl.Tool
{
	public class MemoryMonitor
	{
	   
	   protected static PerformanceCounter memory = new PerformanceCounter("Memory", "Available MBytes"); 
	   public static void DisplayCurrentMemory(string label, bool all)
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append(label).Append(":Free=").Append(memory.NextValue()).Append("k / Total=").Append("?").Append("k");
			if (all)
			{
				buffer.Append(" - Cache Usage = ").Append(NeoDatis.Odb.Impl.Core.Transaction.Cache
					.Usage());
			}
			System.Console.Out.WriteLine(buffer.ToString());
		}
	}
}
