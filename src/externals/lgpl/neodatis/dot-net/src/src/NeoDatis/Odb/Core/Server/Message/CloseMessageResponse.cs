namespace NeoDatis.Odb.Core.Server.Message
{
	[System.Serializable]
	public class CloseMessageResponse : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		public CloseMessageResponse(string baseId, string connectionId, string error) : base
			(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.Close, baseId, connectionId
			)
		{
			SetError(error);
		}

		public CloseMessageResponse(string baseId, string connectionId) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command
			.Close, baseId, connectionId)
		{
		}
	}
}
