namespace NeoDatis.Odb.Core.Server.Message
{
	[System.Serializable]
	public class RollbackMessage : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		public RollbackMessage(string baseId, string connectionId) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command
			.Rollback, baseId, connectionId)
		{
		}

		public override string ToString()
		{
			return "Rollback";
		}
	}
}
