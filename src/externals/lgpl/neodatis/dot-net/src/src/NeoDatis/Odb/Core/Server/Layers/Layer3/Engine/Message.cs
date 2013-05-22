namespace NeoDatis.Odb.Core.Server.Layers.Layer3.Engine
{
	[System.Serializable]
	public abstract class Message
	{
		private int commandId;

		private string baseId;

		private string connectionId;

		private string error;

		public Message(int commandId, string baseId, string connectionId)
		{
			this.commandId = commandId;
			this.baseId = baseId;
			this.connectionId = connectionId;
		}

		public virtual int GetCommandId()
		{
			return commandId;
		}

		public virtual string GetBaseIdentifier()
		{
			return baseId;
		}

		public virtual string GetConnectionId()
		{
			return connectionId;
		}

		public virtual string GetError()
		{
			return error;
		}

		public virtual void SetError(string error)
		{
			this.error = error;
		}

		public virtual bool HasError()
		{
			return error != null;
		}
	}
}
