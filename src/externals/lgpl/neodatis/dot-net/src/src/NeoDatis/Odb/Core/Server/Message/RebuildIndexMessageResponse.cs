namespace NeoDatis.Odb.Core.Server.Message
{
	[System.Serializable]
	public class RebuildIndexMessageResponse : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		public RebuildIndexMessageResponse(string baseId, string connectionId, string error
			) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.RebuildIndex, baseId
			, connectionId)
		{
			SetError(error);
		}

		public RebuildIndexMessageResponse(string baseId, string connectionId) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command
			.RebuildIndex, baseId, connectionId)
		{
		}
	}
}
