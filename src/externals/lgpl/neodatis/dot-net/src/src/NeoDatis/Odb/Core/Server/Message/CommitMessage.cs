namespace NeoDatis.Odb.Core.Server.Message
{
	[System.Serializable]
	public class CommitMessage : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		public CommitMessage(string baseId, string connectionId) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command
			.Commit, baseId, connectionId)
		{
		}

		public override string ToString()
		{
			return "Commit";
		}
	}
}
