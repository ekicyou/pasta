namespace NeoDatis.Odb.Core.Server.Message
{
	[System.Serializable]
	public class DeleteIndexMessageResponse : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		public DeleteIndexMessageResponse(string baseId, string connectionId, string error
			) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.DeleteIndex, baseId
			, connectionId)
		{
			SetError(error);
		}

		public DeleteIndexMessageResponse(string baseId, string connectionId) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command
			.DeleteIndex, baseId, connectionId)
		{
		}
	}
}
