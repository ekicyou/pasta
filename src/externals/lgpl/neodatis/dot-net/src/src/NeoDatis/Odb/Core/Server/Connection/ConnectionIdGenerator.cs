namespace NeoDatis.Odb.Core.Server.Connection
{
	public class ConnectionIdGenerator
	{
		public static string NewId(string ip, long dateTime, int sequence)
		{
			return ip + "-" + dateTime + "-" + sequence;
		}
	}
}
