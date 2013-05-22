namespace NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Engine
{
	public class ServerObjectWriterCallback : NeoDatis.Odb.Core.Layers.Layer3.IObjectWriterCallback
	{
		public virtual void MetaObjectHasBeenInserted(long oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi)
		{
			NeoDatis.Tool.DLogger.Info("Object " + nnoi + " has been inserted with id " + oid
				);
		}

		public virtual void MetaObjectHasBeenUpdated(long oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi)
		{
			NeoDatis.Tool.DLogger.Info("Object " + nnoi + " has been updated with id " + oid);
		}
	}
}
