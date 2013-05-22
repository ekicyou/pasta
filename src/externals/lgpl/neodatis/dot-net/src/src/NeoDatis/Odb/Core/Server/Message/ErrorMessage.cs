namespace NeoDatis.Odb.Core.Server.Message
{
	[System.Serializable]
	public class ErrorMessage : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		public ErrorMessage(string baseId, string connectionId, string error) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command
			.Close, baseId, connectionId)
		{
			SetError(error);
		}

		public override string ToString()
		{
			return "Error";
		}
	}
}
