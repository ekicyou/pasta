namespace NeoDatis.Odb.Core.Server.Message
{
	/// <summary>A StoreMessageResponse is used by the Client/Server mode to answer a StoreMessage
	/// 	</summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public class StoreMessageResponse : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		private NeoDatis.Odb.OID oid;

		private NeoDatis.Odb.OID[] clientIds;

		private NeoDatis.Odb.OID[] serverIds;

		private bool newObject;

		public StoreMessageResponse(string baseId, string connectionId, string error) : base
			(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.Store, baseId, connectionId
			)
		{
			SetError(error);
		}

		public StoreMessageResponse(string baseId, string connectionId, NeoDatis.Odb.OID 
			oid, bool newObject, NeoDatis.Odb.OID[] clientIds, NeoDatis.Odb.OID[] serverIds)
			 : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.Store, baseId, connectionId
			)
		{
			this.oid = oid;
			this.newObject = newObject;
			this.clientIds = clientIds;
			this.serverIds = serverIds;
		}

		public virtual bool IsNewObject()
		{
			return newObject;
		}

		public virtual NeoDatis.Odb.OID GetOid()
		{
			return oid;
		}

		public virtual NeoDatis.Odb.OID[] GetClientIds()
		{
			return clientIds;
		}

		public virtual NeoDatis.Odb.OID[] GetServerIds()
		{
			return serverIds;
		}
	}
}
