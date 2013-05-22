namespace NeoDatis.Odb.Core.Server.Layers.Layer1
{
	public interface IClientObjectIntrospector : NeoDatis.Odb.Core.Layers.Layer1.Introspector.IObjectIntrospector
	{
		NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.OID> GetClientOids();

		void SynchronizeIds(NeoDatis.Odb.OID[] clientIds, NeoDatis.Odb.OID[] serverIds);
	}
}
