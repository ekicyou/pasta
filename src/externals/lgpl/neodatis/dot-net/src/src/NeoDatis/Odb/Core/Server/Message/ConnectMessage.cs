namespace NeoDatis.Odb.Core.Server.Message
{
	[System.Serializable]
	public class ConnectMessage : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		private string ip;

		private long dateTime;

		private string user;

		private string password;

		public ConnectMessage(string baseId, string ip, string user, string password) : base
			(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.Connect, baseId, null)
		{
			this.ip = ip;
			this.dateTime = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			this.user = user;
			this.password = password;
		}

		public virtual long GetDateTime()
		{
			return dateTime;
		}

		public virtual string GetIp()
		{
			return ip;
		}

		public override string ToString()
		{
			return "Connect";
		}

		public virtual string GetPassword()
		{
			return password;
		}

		public virtual string GetUser()
		{
			return user;
		}
	}
}
