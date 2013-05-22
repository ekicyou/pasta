namespace NeoDatis.Odb.Core.Server.Message
{
	[System.Serializable]
	public class GetObjectFromIdMessage : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		private NeoDatis.Odb.OID oid;

		public GetObjectFromIdMessage(string baseId, string connectionId, NeoDatis.Odb.OID
			 oid) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.GetObjectFromId
			, baseId, connectionId)
		{
			this.oid = oid;
		}

		public virtual NeoDatis.Odb.OID GetOid()
		{
			return oid;
		}

		public override string ToString()
		{
			return "GetObjectFromId";
		}
	}
}
