namespace NeoDatis.Odb.Core.Server.Message
{
	/// <summary>Message used to retrieve the object info header from an object oid</summary>
	/// <author>olivier</author>
	[System.Serializable]
	public class GetObjectHeaderFromIdMessage : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		private NeoDatis.Odb.OID oid;

		public GetObjectHeaderFromIdMessage(string baseId, string connectionId, NeoDatis.Odb.OID
			 oid) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.GetObjectHeaderFromId
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
			return "GetObjectHeaderFromId";
		}
	}
}
