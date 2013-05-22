namespace NeoDatis.Odb.Core.Server.Message
{
	[System.Serializable]
	public class AddIndexMessageResponse : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		public AddIndexMessageResponse(string baseId, string connectionId, string error) : 
			base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.AddUniqueIndex, baseId
			, connectionId)
		{
			SetError(error);
		}

		public AddIndexMessageResponse(string baseId, string connectionId) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command
			.AddUniqueIndex, baseId, connectionId)
		{
		}
	}
}
