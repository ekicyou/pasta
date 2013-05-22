namespace NeoDatis.Odb.Core.Server.Message
{
	/// <summary>A StoreMessage is used by the Client/Server mode to store an object</summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public class DeleteBaseMessage : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		public DeleteBaseMessage(string baseId) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command
			.DeleteBase, baseId, null)
		{
		}

		public override string ToString()
		{
			return "DeleteBase";
		}
	}
}
