namespace NeoDatis.Odb.Core.Server.Message
{
	/// <summary>A StoreMessage is used by the Client/Server mode to store an object</summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public class StoreMessage : NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message
	{
		private NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi;

		private NeoDatis.Odb.OID[] clientIds;

		public StoreMessage(string baseId, string connectionId, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi, NeoDatis.Odb.OID[] localOids) : base(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command
			.Store, baseId, connectionId)
		{
			this.nnoi = nnoi;
			this.clientIds = localOids;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo GetNnoi()
		{
			return nnoi;
		}

		public virtual NeoDatis.Odb.OID[] GetClientIds()
		{
			return clientIds;
		}

		public override string ToString()
		{
			return "Store";
		}
	}
}
