namespace NeoDatis.Odb.Core.Server.Message
{
	/// <summary>A StoreMessageResponse is used by the Client/Server mode to answer a StoreMessage
	/// 	</summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public class DeleteBaseMessageResponse : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		public DeleteBaseMessageResponse(string baseId, string error) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command
			.DeleteBase, baseId, null)
		{
			SetError(error);
		}

		public DeleteBaseMessageResponse(string baseId) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command
			.DeleteBase, baseId, null)
		{
		}
	}
}
