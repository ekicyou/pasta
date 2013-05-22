namespace NeoDatis.Odb.Core.Server.Message
{
	[System.Serializable]
	public class CloseMessage : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		public CloseMessage(string baseId, string connectionId) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command
			.Close, baseId, connectionId)
		{
		}

		public override string ToString()
		{
			return "Close";
		}
	}
}
