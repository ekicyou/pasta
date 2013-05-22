using NeoDatis.Tool.Wrappers.Map;
using NeoDatis.Odb.Core.Layers.Layer2.Meta;
using NeoDatis.Odb.Core;
using NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid;
using NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine;
using System;
namespace NeoDatis.Odb.Impl.Core.Transaction
{
	/// <summary>A cache of object.</summary>
	/// <remarks>
	/// A cache of object.
	/// <pre>
	/// Cache objects by object, by position, by oids,...
	/// </pre>
	/// </remarks>
	/// <author>olivier s</author>
	public class Cache : NeoDatis.Odb.Core.Transaction.ICache
	{
		protected static int nbObjects = 0;

		protected static int nbOids = 0;

		protected static int nbOih = 0;

		protected static int nbTransactionOids = 0;

		protected static int nbObjectPositionByIds = 0;

		protected static int nbCallsToGetObjectInfoHeaderFromOid = 0;

		protected static int nbCallsToGetObjectInfoHeaderFromObject = 0;

		protected static int nbCallsToGetObjectWithOid = 0;

		/// <summary>
		/// object cache - used to know if object exist in the cache TODO use
		/// hashcode instead?
		/// </summary>
		protected System.Collections.Generic.IDictionary<object, NeoDatis.Odb.OID> objects;

		/// <summary>Entry to get an object from its oid</summary>
		protected System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, object> oids;

		/// <summary>To resolve cyclic reference, keep track of objects being inserted</summary>
		protected System.Collections.Generic.IDictionary<object, NeoDatis.Odb.Impl.Core.Transaction.ObjectInsertingInfo
			> insertingObjects;

		/// <summary>To resolve cyclic reference, keep track of objects being read</summary>
		protected System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, object[]> readingObjectInfo;

		/// <summary>
		/// <pre>
		/// To resolve the update of an id object position:
		/// When an object is full updated(the current object is being deleted and a new one os being created),
		/// the id remain the same but its position change.
		/// </summary>
		/// <remarks>
		/// <pre>
		/// To resolve the update of an id object position:
		/// When an object is full updated(the current object is being deleted and a new one os being created),
		/// the id remain the same but its position change.
		/// But the update is done in transaction, so it is not flushed until the commit happens
		/// So after the update when i need the position to make the old object a pointer, i have no way to get
		/// the right position. To resolve this, i keep a cache of ids where i keep the non commited value
		/// </pre>
		/// </remarks>
		protected System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, IdInfo
			> objectPositionsByIds;

		/// <summary>
		/// To keep track of the oid that have been created or modified in the
		/// current transaction
		/// </summary>
		protected System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, NeoDatis.Odb.OID
			> unconnectedZoneOids;

		protected NeoDatis.Odb.Core.Transaction.ISession session;

		protected string name;

		/// <summary>
		/// Entry to get object info pointers (position,next object pos, previous
		/// object pos and class info pos) from the id
		/// </summary>
		protected System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, ObjectInfoHeader
			> objectInfoPointersCacheFromOid;

		public Cache(NeoDatis.Odb.Core.Transaction.ISession session, string name)
		{
			Init(session, name);
		}

		protected virtual void Init(NeoDatis.Odb.Core.Transaction.ISession session, string
			 name)
		{
			this.name = name;
			this.session = session;
			objects = new OdbHashMap<object, NeoDatis.Odb.OID>();
			oids = new OdbHashMap<NeoDatis.Odb.OID, object>();
			unconnectedZoneOids = new OdbHashMap<NeoDatis.Odb.OID, NeoDatis.Odb.OID>();
			objectInfoPointersCacheFromOid = new OdbHashMap<NeoDatis.Odb.OID, ObjectInfoHeader>();
			insertingObjects = new OdbHashMap<object, ObjectInsertingInfo>();
			readingObjectInfo = new OdbHashMap<NeoDatis.Odb.OID, object[]>();
			objectPositionsByIds = new OdbHashMap<NeoDatis.Odb.OID,IdInfo>();
		}

		public virtual void AddObject(OID oid, object o, ObjectInfoHeader objectInfoHeader)
		{
            if (oid == null)
			{
				throw new ODBRuntimeException(NeoDatisError.CacheNullOid);
			}
			if (CheckHeaderPosition() && objectInfoHeader.GetPosition() == -1)
			{
				throw new ODBRuntimeException(NeoDatisError.CacheNegativePosition
					.AddParameter("Adding OIH with position = -1"));
			}
			// TODO : Should remove first inserted object and not clear all cache
			if (objects.Count > OdbConfiguration.GetMaxNumberOfObjectInCache())
			{
				// clear();
				ManageFullCache();
			}
            
			oids[oid] = o;
			try
			{
				objects[o] = oid;
			}
			catch (System.ArgumentNullException)
			{
			}
			// FIXME URL in HashMap What should we do?
			// In some case, the object can throw exception when added to the
			// cache
			// because Map.put, end up calling the equals method that can throw
			// exception
			// This is the case of URL that has a transient attribute handler
			// that is used in the URL.equals method
			objectInfoPointersCacheFromOid[oid] = objectInfoHeader;
			// For monitoring purpose
			nbObjects = objects.Count;
			nbOids = oids.Count;
			nbOih = objectInfoPointersCacheFromOid.Count;
		}

		/// <summary>Only adds the Object info - used for non committed objects</summary>
		public virtual void AddObjectInfo(ObjectInfoHeader  objectInfoHeader)
		{
          
			if (objectInfoHeader.GetOid() == null)
			{
				throw new ODBRuntimeException(NeoDatisError.CacheNullOid);
			}
			if (objectInfoHeader.GetClassInfoId() == null)
			{
				throw new ODBRuntimeException(NeoDatisError.CacheObjectInfoHeaderWithoutClassId.AddParameter(objectInfoHeader.GetOid()));
			}
			// TODO : Should remove first inserted object and not clear all cache
			if (objectInfoPointersCacheFromOid.Count > OdbConfiguration.GetMaxNumberOfObjectInCache())
			{
				ManageFullCache();
			}
			objectInfoPointersCacheFromOid[objectInfoHeader.GetOid()] = objectInfoHeader;
			// For monitoring purpose
			nbObjects = objects.Count;
			nbOids = oids.Count;
			nbOih = objectInfoPointersCacheFromOid.Count;
		}

		protected virtual void ManageFullCache()
		{
			if (OdbConfiguration.AutomaticallyIncreaseCacheSize())
			{
				OdbConfiguration.SetMaxNumberOfObjectInCache((long)(OdbConfiguration
					.GetMaxNumberOfObjectInCache() * 1.2));
			}
		}

		//throw new ODBRuntimeException(Error.CACHE_IS_FULL.addParameter(objectInfoPointersCacheFromOid.size()).addParameter(OdbConfiguration.getMaxNumberOfObjectInCache()));
		public virtual void StartInsertingObjectWithOid(object o, NeoDatis.Odb.OID 
			oid, NonNativeObjectInfo nnoi)
		{
			// In this case oid can be -1,because object is beeing inserted and do
			// not have yet a defined oid.
			if (o == null)
			{
				return;
			}
			ObjectInsertingInfo oii = null;
            insertingObjects.TryGetValue(o,out oii);
			if (oii == null)
			{
				insertingObjects[o] = new ObjectInsertingInfo(oid, 1);
			}
			else
			{
				oii.level++;
			}
		}

		// No need to update the map, it is a reference.
		public virtual void UpdateIdOfInsertingObject(object o, NeoDatis.Odb.OID oid
			)
		{
			if (oid == null)
			{
				throw new ODBRuntimeException(NeoDatisError.CacheNullOid);
			}
			ObjectInsertingInfo oii = (ObjectInsertingInfo)insertingObjects[o];
			if (oii != null)
			{
				oii.oid = oid;
			}
		}

		public virtual void EndInsertingObject(object o)
		{
			ObjectInsertingInfo oii = (ObjectInsertingInfo)insertingObjects[o];
			if (oii.level == 1)
			{
				insertingObjects.Remove(o);
				oii = null;
			}
			else
			{
				oii.level--;
			}
		}

		public virtual void RemoveObjectWithOid(NeoDatis.Odb.OID oid)
		{
			if (oid == null)
			{
				throw new ODBRuntimeException(NeoDatisError.CacheNullOid);
			}
            object o = null;
            oids.TryGetValue(oid, out o);
			oids.Remove(oid);
            if (o != null)
            {
                objects.Remove(o);
            }
			// FIXME URL in HashMap What should we do?
			objectInfoPointersCacheFromOid.Remove(oid);
			unconnectedZoneOids.Remove(oid);
			// For monitoring purpose
			nbObjects = objects.Count;
			nbOids = oids.Count;
			nbOih = objectInfoPointersCacheFromOid.Count;
		}

		public virtual void RemoveObject(object o)
		{
			if (o == null)
			{
				throw new ODBRuntimeException(NeoDatisError.CacheNullObject.AddParameter(" while removing object from the cache"));
			}
            NeoDatis.Odb.OID oid = null;
            objects.TryGetValue(o, out oid);
			oids.Remove(oid);
			try
			{
				objects.Remove(o);
			}
			catch (System.ArgumentNullException)
			{
			}
			// FIXME URL in HashMap What should we do?
			objectInfoPointersCacheFromOid.Remove(oid);
			unconnectedZoneOids.Remove(oid);
			// For monitoring purpose
			nbObjects = objects.Count;
			nbOids = oids.Count;
			nbOih = objectInfoPointersCacheFromOid.Count;
		}

		public virtual bool ExistObject(object @object)
		{
			return objects.ContainsKey(@object);
		}

		public virtual object GetObjectWithOid(NeoDatis.Odb.OID oid)
		{
			if (oid == null)
			{
				throw new ODBRuntimeException(NeoDatisError.CacheNullOid
					.AddParameter(oid));
			}
            object o = null;
            oids.TryGetValue(oid, out o);
			nbCallsToGetObjectWithOid++;
			return o;
		}

		public virtual ObjectInfoHeader GetObjectInfoHeaderFromObject(object o, bool throwExceptionIfNotFound)
		{
            NeoDatis.Odb.OID oid = null;
            objects.TryGetValue(o, out oid);

            ObjectInfoHeader oih = null;
            objectInfoPointersCacheFromOid.TryGetValue(oid, out oih);
			if (oih == null && throwExceptionIfNotFound)
			{
				throw new ODBRuntimeException(NeoDatisError.ObjectDoesNotExistInCache.AddParameter(o.ToString()));
			}
			nbCallsToGetObjectInfoHeaderFromObject++;
			return oih;
		}

		public virtual ObjectInfoHeader GetObjectInfoHeaderFromOid(NeoDatis.Odb.OID oid, bool throwExceptionIfNotFound)
		{
			if (oid == null)
			{
				throw new ODBRuntimeException(NeoDatisError.CacheNullOid);
			}
            ObjectInfoHeader oih = null;
            objectInfoPointersCacheFromOid.TryGetValue(oid, out oih);
			if (oih == null && throwExceptionIfNotFound)
			{
				throw new ODBRuntimeException(NeoDatisError.ObjectWithOidDoesNotExistInCache.AddParameter(oid));
			}
			nbCallsToGetObjectInfoHeaderFromOid++;
			return oih;
		}

		public virtual NeoDatis.Odb.OID GetOid(object o, bool throwExceptionIfNotFound
			)
		{
            OID oid = null;
            objects.TryGetValue(o, out oid);
			if (oid != null)
			{
				return oid;
			}
			if (throwExceptionIfNotFound)
			{
				throw new ODBRuntimeException(NeoDatisError.ObjectDoesNotExistInCache);
			}
			return StorageEngineConstant.NullObjectId;
		}

		public virtual void SavePositionOfObjectWithOid(NeoDatis.Odb.OID oid, long objectPosition
			)
		{
			if (oid == null)
			{
				throw new ODBRuntimeException(NeoDatisError.CacheNullOid);
			}
			IdInfo idInfo = new IdInfo(oid, objectPosition,IDStatus.Active);
			objectPositionsByIds[oid] = idInfo;
			// For monitoring purpose
			nbObjects = objects.Count;
			nbOids = oids.Count;
			nbOih = objectInfoPointersCacheFromOid.Count;
		}

		public virtual void MarkIdAsDeleted(NeoDatis.Odb.OID oid)
		{
			if (oid == null)
			{
				throw new ODBRuntimeException(NeoDatisError.CacheNullOid);
			}
            IdInfo idInfo = null;
            
            objectPositionsByIds.TryGetValue(oid,out idInfo);
			if (idInfo != null)
			{
				idInfo.status = IDStatus.Deleted;
			}
			else
			{
				idInfo = new IdInfo(oid, -1, IDStatus.Deleted);
				objectPositionsByIds[oid] = idInfo;
			}
		}

		public virtual bool IsDeleted(NeoDatis.Odb.OID oid)
		{
			if (oid == null)
			{
				throw new ODBRuntimeException(NeoDatisError.CacheNullOid);
			}
            IdInfo idInfo = null;
            objectPositionsByIds.TryGetValue(oid, out idInfo);
			if (idInfo != null)
			{
				return idInfo.status == IDStatus.Deleted;
			}
			return false;
		}

		/// <summary>
		/// Returns the position or -1 if it is not is the cache or
		/// StorageEngineConstant.NULL_OBJECT_ID_ID if it has been marked as deleted
		/// </summary>
		public virtual long GetObjectPositionByOid(NeoDatis.Odb.OID oid)
		{
			if (oid == null)
			{
				return StorageEngineConstant.NullObjectIdId;
			}
            IdInfo idInfo = null;
            objectPositionsByIds.TryGetValue(oid, out idInfo);
			if (idInfo != null)
			{
				if (!IDStatus.IsActive(idInfo.status))
				{
					return StorageEngineConstant.DeletedObjectPosition;
				}
				return idInfo.position;
			}
			// object is not in the cache
			return StorageEngineConstant.ObjectIsNotInCache;
		}

		public virtual void ClearOnCommit()
		{
			objectPositionsByIds.Clear();
			unconnectedZoneOids.Clear();
		}

		public virtual void Clear(bool setToNull)
		{
			if (objects != null)
			{
				objects.Clear();
				oids.Clear();
				objectInfoPointersCacheFromOid.Clear();
				insertingObjects.Clear();
				objectPositionsByIds.Clear();
				readingObjectInfo.Clear();
				unconnectedZoneOids.Clear();
			}
			if (setToNull)
			{
				objects = null;
				oids = null;
				objectInfoPointersCacheFromOid = null;
				insertingObjects = null;
				objectPositionsByIds = null;
				readingObjectInfo = null;
				unconnectedZoneOids = null;
			}
		}

		public virtual void ClearInsertingObjects()
		{
			insertingObjects.Clear();
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append("C=");
			buffer.Append(objects.Count).Append(" objects ");
			buffer.Append(oids.Count).Append(" oids ");
			buffer.Append(objectInfoPointersCacheFromOid.Count).Append(" pointers");
			buffer.Append(objectPositionsByIds.Count).Append(" pos by oid");
			// buffer.append(insertingObjects.size()).append(" inserting
			// objects\n");
			// buffer.append(readingObjectInfo.size()).append(" reading objects");
			return buffer.ToString();
		}

		public virtual string ToCompleteString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append(objects.Count).Append(" Objects=").Append(objects).Append("\n");
			buffer.Append(oids.Count).Append(" Objects from pos").Append(oids).Append("\n");
			buffer.Append(objectInfoPointersCacheFromOid.Count).Append(" Pointers=").Append(objectInfoPointersCacheFromOid
				);
			return buffer.ToString();
		}

		public virtual int GetNumberOfObjects()
		{
			return objects.Count;
		}

		public virtual int GetNumberOfObjectHeader()
		{
			return objectInfoPointersCacheFromOid.Count;
		}

		public virtual NeoDatis.Odb.OID IdOfInsertingObject(object o)
		{
			if (o == null)
			{
				return StorageEngineConstant.NullObjectId;
			}
            ObjectInsertingInfo oii = null;
            
            insertingObjects.TryGetValue(o,out oii);

			if (oii != null)
			{
				return oii.oid;
			}
			return StorageEngineConstant.NullObjectId;
		}

		public virtual int InsertingLevelOf(object o)
		{
            ObjectInsertingInfo oii = null;
            
            insertingObjects.TryGetValue(o, out oii);
			if (oii == null)
			{
				return 0;
			}
			return oii.level;
		}

		public virtual bool IsReadingObjectInfoWithOid(NeoDatis.Odb.OID oid)
		{
			if (oid == null)
			{
				return false;
			}
			// throw new
			// ODBRuntimeException(Error.CACHE_NULL_OID);
			return readingObjectInfo[oid] != null;
		}

		public virtual NonNativeObjectInfo GetReadingObjectInfoFromOid
			(NeoDatis.Odb.OID oid)
		{
			if (oid == null)
			{
				throw new ODBRuntimeException(NeoDatisError.CacheNullOid);
			}
			object[] values = (object[])readingObjectInfo[oid];
			if (values == null)
			{
				return null;
			}
			return (NonNativeObjectInfo)values[1];
		}

		public virtual void StartReadingObjectInfoWithOid(NeoDatis.Odb.OID oid, NonNativeObjectInfo
			 objectInfo)
		{
			if (oid == null)
			{
				throw new ODBRuntimeException(NeoDatisError.CacheNullOid);
			}
			object[] objects = (object[])readingObjectInfo[oid];
			if (objects == null)
			{
				// The key is the oid, the value is an array of 2 objects :
				// 1-the read count, 2-The object info
				// Here we are saying that the object with oid 'oid' is
				// being read for the first time
				object[] values = new object[] { (short)1, objectInfo };
				readingObjectInfo[oid] = values;
			}
			else
			{
				// Here the object is already being read. It is necessary to
				// increase the read count
				short currentReadCount = ((short)objects[0]);
				objects[0] = (short)(currentReadCount + 1);
			}
		}

		// Object is in memory, do not need to re-put in map. The key has
		// not changed
		// readingObjectInfo.put(oid, objects);
		public virtual void EndReadingObjectInfo(NeoDatis.Odb.OID oid)
		{
			if (oid == null)
			{
				throw new ODBRuntimeException(NeoDatisError.CacheNullOid.AddParameter(oid));
			}
			object[] values = (object[])readingObjectInfo[oid];
			if (values == null || values[0] == null)
			{
				throw new ODBRuntimeException(NeoDatisError.ObjectInfoNotInTempCache
					.AddParameter(oid).AddParameter("?"));
			}
			short readCount = ((short)values[0]);
			if (readCount == 1)
			{
				readingObjectInfo.Remove(oid);
			}
			else
			{
				values[0] = (short)(readCount - 1);
			}
		}

		// Object is in memory, do not need to re-put in map. The key has
		// not changed
		// readingObjectInfo.put(oid, values);
		public virtual System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, object> GetOids()
		{
			return oids;
		}

		public virtual void SetOids(System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, object> oids)
		{
			this.oids = oids;
		}

		public virtual System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, ObjectInfoHeader> GetObjectInfoPointersCacheFromOid()
		{
			return objectInfoPointersCacheFromOid;
		}

		public virtual void SetObjectInfoPointersCacheFromOid(System.Collections.Generic.IDictionary
			<NeoDatis.Odb.OID, ObjectInfoHeader> objectInfoPointersCacheFromOid
			)
		{
			this.objectInfoPointersCacheFromOid = objectInfoPointersCacheFromOid;
		}

		public virtual System.Collections.Generic.IDictionary<object, NeoDatis.Odb.OID> GetObjects
			()
		{
			return objects;
		}

		public virtual void SetObjects(System.Collections.Generic.IDictionary<object, NeoDatis.Odb.OID
			> objects)
		{
			this.objects = objects;
		}

		public virtual bool ObjectWithIdIsInCommitedZone(NeoDatis.Odb.OID oid)
		{
			return !unconnectedZoneOids.ContainsKey(oid);
		}

		public virtual void AddOIDToUnconnectedZone(NeoDatis.Odb.OID oid)
		{
			unconnectedZoneOids.Add(oid, oid);
		}

		public static string Usage()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append("NbObj=").Append(nbObjects);
			buffer.Append(" - NbOIDs=").Append(nbOids);
			buffer.Append(" - NbObjPos=").Append(nbObjectPositionByIds);
			buffer.Append(" - NbOIHs=").Append(nbOih);
			buffer.Append(" - NbTransOIDs=").Append(nbTransactionOids);
			buffer.Append(" - Calls2getObjectWitOid=").Append(nbCallsToGetObjectWithOid);
			buffer.Append(" - Calls2getObjectInfoHeaderFromOid=").Append(nbCallsToGetObjectInfoHeaderFromOid
				);
			buffer.Append(" - Calls2getObjectInfoHeaderFromObject=").Append(nbCallsToGetObjectInfoHeaderFromObject
				);
			return buffer.ToString();
		}

		protected virtual bool CheckHeaderPosition()
		{
			return false;
		}
	}
}
