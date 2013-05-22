using NeoDatis.Odb.Core.Layers.Layer2.Meta;
using NeoDatis.Odb.Core;
using NeoDatis.Tool.Wrappers.Map;
namespace NeoDatis.Odb.Impl.Core.Transaction
{
	/// <summary>A temporary cache of objects.</summary>
	/// <remarks>A temporary cache of objects.</remarks>
	/// <author>olivier s</author>
	public class TmpCache : NeoDatis.Odb.Core.Transaction.ITmpCache
	{
		/// <summary>To resolve cyclic reference, keep track of objects being read</summary>
		protected System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, object[]> readingObjectInfo;

		protected NeoDatis.Odb.Core.Transaction.ISession session;

		protected string name;

		public TmpCache(NeoDatis.Odb.Core.Transaction.ISession session, string name)
		{
			Init(session, name);
		}

		protected virtual void Init(NeoDatis.Odb.Core.Transaction.ISession session, string
			 name)
		{
			this.name = name;
			this.session = session;
			readingObjectInfo = new OdbHashMap<NeoDatis.Odb.OID, object[]>();
		}

		public virtual bool IsReadingObjectInfoWithOid(NeoDatis.Odb.OID oid)
		{
			if (oid == null)
			{
				return false;
			}
			return readingObjectInfo.ContainsKey(oid);
		}

		public virtual NonNativeObjectInfo GetReadingObjectInfoFromOid(NeoDatis.Odb.OID oid)
		{
			if (oid == null)
			{
				throw new ODBRuntimeException(NeoDatisError.CacheNullOid);
			}
            object[] values = null;
            readingObjectInfo.TryGetValue(oid, out values);
			if (values == null)
			{
				return null;
			}
			return (NonNativeObjectInfo)values[1];
		}

		public virtual void StartReadingObjectInfoWithOid(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 objectInfo)
		{
			if (oid == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.CacheNullOid
					);
			}
            object[] objects = null;
            readingObjectInfo.TryGetValue(oid, out objects);
			// TODO : use a value object instead of an array!
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
		public virtual void ClearObjectInfos()
		{
			readingObjectInfo.Clear();
		}

		public virtual int Size()
		{
			// TODO Auto-generated method stub
			return readingObjectInfo.Count;
		}
	}
}
