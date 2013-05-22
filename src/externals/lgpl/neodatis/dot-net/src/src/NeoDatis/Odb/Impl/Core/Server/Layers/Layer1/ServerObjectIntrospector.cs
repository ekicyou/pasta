namespace NeoDatis.Odb.Impl.Core.Server.Layers.Layer1
{
	public class ServerObjectIntrospector : NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector
	{
		public ServerObjectIntrospector(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine
			) : base(storageEngine)
		{
		}
	}
}
