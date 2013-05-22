namespace NeoDatis.Odb.Core.Transaction
{
	/// <summary>An interface for temporary cache</summary>
	public interface ITmpCache
	{
		NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo GetReadingObjectInfoFromOid
			(NeoDatis.Odb.OID oid);

		bool IsReadingObjectInfoWithOid(NeoDatis.Odb.OID oid);

		void StartReadingObjectInfoWithOid(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 objectInfo);

		void ClearObjectInfos();

		int Size();
	}
}
