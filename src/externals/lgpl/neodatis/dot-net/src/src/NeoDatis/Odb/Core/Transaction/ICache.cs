namespace NeoDatis.Odb.Core.Transaction
{
	public interface ICache
	{
		void AddObject(NeoDatis.Odb.OID oid, object @object, NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
			 objectInfoHeader);

		void StartInsertingObjectWithOid(object @object, NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi);

		void UpdateIdOfInsertingObject(object @object, NeoDatis.Odb.OID oid);

		void EndInsertingObject(object @object);

		void AddObjectInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader objectInfoHeader
			);

		void RemoveObjectWithOid(NeoDatis.Odb.OID oid);

		void RemoveObject(object @object);

		bool ExistObject(object @object);

		object GetObjectWithOid(NeoDatis.Odb.OID oid);

		NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader GetObjectInfoHeaderFromObject
			(object @object, bool throwExceptionIfNotFound);

		NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader GetObjectInfoHeaderFromOid(
			NeoDatis.Odb.OID oid, bool throwExceptionIfNotFound);

		NeoDatis.Odb.OID GetOid(object @object, bool throwExceptionIfNotFound);

		/// <summary>To resolve uncommitted updates where the oid change and is not committed yet
		/// 	</summary>
		void SavePositionOfObjectWithOid(NeoDatis.Odb.OID oid, long objectPosition);

		void MarkIdAsDeleted(NeoDatis.Odb.OID oid);

		bool IsDeleted(NeoDatis.Odb.OID oid);

		long GetObjectPositionByOid(NeoDatis.Odb.OID oid);

		void ClearOnCommit();

		void Clear(bool setToNull);

		void ClearInsertingObjects();

		string ToString();

		string ToCompleteString();

		int GetNumberOfObjects();

		int GetNumberOfObjectHeader();

		NeoDatis.Odb.OID IdOfInsertingObject(object @object);

		int InsertingLevelOf(object @object);

		bool IsReadingObjectInfoWithOid(NeoDatis.Odb.OID oid);

		NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo GetReadingObjectInfoFromOid
			(NeoDatis.Odb.OID oid);

		/// <summary>
		/// To resolve cyclic reference, keep track of objects being read The read
		/// count is used to store how many times the object has been recursively
		/// read
		/// </summary>
		/// <param name="oid">The Object OID</param>
		/// <param name="objectInfo">The object info (not fully set) that is being read</param>
		void StartReadingObjectInfoWithOid(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 objectInfo);

		void EndReadingObjectInfo(NeoDatis.Odb.OID oid);

		System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, object> GetOids();

		System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
			> GetObjectInfoPointersCacheFromOid();

		System.Collections.Generic.IDictionary<object, NeoDatis.Odb.OID> GetObjects();

		bool ObjectWithIdIsInCommitedZone(NeoDatis.Odb.OID oid);

		void AddOIDToUnconnectedZone(NeoDatis.Odb.OID oid);
	}
}
