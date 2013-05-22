namespace NeoDatis.Odb.Core.Server.Message
{
	[System.Serializable]
	public class GetSessionsMessage : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		public GetSessionsMessage() : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command
			.GetSessions, null, null)
		{
		}

		public override string ToString()
		{
			return "GetSessions";
		}
	}
}
