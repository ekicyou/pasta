namespace NeoDatis.Odb.Core.Server.Message
{
	/// <summary>A StoreMessageResponse is used by the Client/Server mode to answer a StoreMessage
	/// 	</summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public class DeleteObjectMessageResponse : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		private NeoDatis.Odb.OID oid;

		public DeleteObjectMessageResponse(string baseId, string connectionId, string error
			) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.Store, baseId, connectionId
			)
		{
			SetError(error);
		}

		public DeleteObjectMessageResponse(string baseId, string connectionId, NeoDatis.Odb.OID
			 oid) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.Store, baseId
			, connectionId)
		{
			this.oid = oid;
		}

		public virtual NeoDatis.Odb.OID GetOid()
		{
			return oid;
		}
	}
}
