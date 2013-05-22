namespace NeoDatis.Odb.Core.Server.Message
{
	[System.Serializable]
	public class ConnectMessageResponse : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		private NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel;

		private NeoDatis.Odb.TransactionId transactionId;

		public ConnectMessageResponse(string baseId, string connectionId, string error) : 
			base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.Connect, baseId, connectionId
			)
		{
			SetError(error);
		}

		public ConnectMessageResponse(string baseId, string connectionId, NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel
			 metaModel, NeoDatis.Odb.TransactionId transactionId) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command
			.Connect, baseId, connectionId)
		{
			this.metaModel = metaModel;
			this.transactionId = transactionId;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel GetMetaModel()
		{
			return metaModel;
		}

		public virtual NeoDatis.Odb.TransactionId GetTransactionId()
		{
			return transactionId;
		}
	}
}
