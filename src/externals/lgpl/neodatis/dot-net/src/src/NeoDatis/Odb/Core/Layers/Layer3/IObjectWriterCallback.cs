namespace NeoDatis.Odb.Core.Layers.Layer3
{
	/// <summary>A callback interface - not used</summary>
	/// <author>osmadja</author>
	public interface IObjectWriterCallback
	{
		void MetaObjectHasBeenInserted(long oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi);

		void MetaObjectHasBeenUpdated(long oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi);
	}
}
