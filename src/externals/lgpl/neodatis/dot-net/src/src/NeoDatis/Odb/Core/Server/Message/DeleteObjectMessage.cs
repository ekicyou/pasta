namespace NeoDatis.Odb.Core.Server.Message
{
	/// <summary>A StoreMessage is used by the Client/Server mode to store an object</summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public class DeleteObjectMessage : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		private NeoDatis.Odb.OID oid;

		public DeleteObjectMessage(string baseId, string connectionId, NeoDatis.Odb.OID oid
			) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.DeleteObject, baseId
			, connectionId)
		{
			this.oid = oid;
		}

		public virtual NeoDatis.Odb.OID GetOid()
		{
			return oid;
		}

		public override string ToString()
		{
			return "deleteObject";
		}
	}
}
