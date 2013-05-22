namespace NeoDatis.Odb.Core.Server.Message
{
	[System.Serializable]
	public class RollbackMessageResponse : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		private bool ok;

		public RollbackMessageResponse(string baseId, string connectionId, string error) : 
			base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.Rollback, baseId, connectionId
			)
		{
			SetError(error);
		}

		public RollbackMessageResponse(string baseId, string connectionId, bool ok) : base
			(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.Rollback, baseId, connectionId
			)
		{
			this.ok = ok;
		}

		public virtual bool IsOk()
		{
			return ok;
		}
	}
}
