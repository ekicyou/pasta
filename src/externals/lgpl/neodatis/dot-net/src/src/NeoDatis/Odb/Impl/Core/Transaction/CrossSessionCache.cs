using System.Collections.Generic;
using NeoDatis.Odb.Core.Transaction;
using System;
using NeoDatis.Odb;
using NeoDatis.Tool.Wrappers.Map;
namespace NeoDatis.Odb.Impl.Core.Transaction
{
	/// <summary>A cache that survives the sessions.</summary>
	/// <remarks>A cache that survives the sessions. It is uses to automatically reconnect object to sessions
	/// 	</remarks>
	/// <author>mayworm,olivier</author>
	public class CrossSessionCache : ICrossSessionCache
	{
		/// <summary>The cache for NeoDatis OID.</summary>
		/// <remarks>
		/// The cache for NeoDatis OID. This cache supports a weak reference and it is
		/// sync
		/// </remarks>
		private IDictionary<object, OID> objects;

		/// <summary>
		/// When objects are deleted by oid, the cost is too high to search the object by the oid, so we just keep the deleted oid,
		/// and when looking for an object, check if the oid if is the deleted oids, if yes, return null and delete the object
		/// </summary>
		private IDictionary<OID, OID> deletedOids;

		/// <summary>To keep track of all caches</summary>
		private static IDictionary<string, ICrossSessionCache> instances = new OdbHashMap<string, ICrossSessionCache>();

		/// <summary>Protected constructor for factory-based construction</summary>
		protected CrossSessionCache()
		{
			objects = new Dictionary<object,OID>();
			deletedOids = new Dictionary<OID,OID>();
		}

		/// <summary>Gets the unique instance for the cache for the identification</summary>
		public static NeoDatis.Odb.Core.Transaction.ICrossSessionCache GetInstance(string
			 baseIdentification)
		{
            ICrossSessionCache cache = null;
            instances.TryGetValue(baseIdentification, out cache);
			if (cache == null)
			{
				lock (instances)
				{
					cache = new CrossSessionCache();
					instances[baseIdentification] = cache;
				}
			}
			return cache;
		}

		public virtual void AddObject(object o, OID oid)
		{
			if (o == null)
			{
				return;
			}
			// throw new
			// ODBRuntimeException(NeoDatisError.CACHE_NULL_OBJECT.addParameter(object));
			try
			{
				objects.Add(o, oid);
			}
			catch (System.ArgumentNullException)
			{
			}
		}

		// FIXME URL in HashMap What should we do?
		// In some case, the object can throw exception when added to the
		// cache
		// because Map.put, end up calling the equals method that can throw
		// exception
		// This is the case of URL that has a transient attribute handler
		// that is used in the URL.equals method
		public virtual void Clear()
		{
			objects.Clear();
			deletedOids.Clear();
		}

		public virtual bool ExistObject(object o)
		{
			if (o == null)
			{
				return false;
			}
			OID oid = objects[o];
			// Then check if oid is in the deleted oid list
			if (deletedOids.ContainsKey(oid))
			{
				// The object has been marked as deleted
				// removes it from the cache
				objects.Remove(o);
				deletedOids.Remove(oid);
				return false;
			}
			return true;
		}

		public virtual OID GetOid(object o)
		{
			if (o == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.CacheNullObject
					.AddParameter(o));
			}
			OID oid = objects[o];
			if (oid != null)
			{
				if (deletedOids.ContainsKey(oid))
				{
					// The object has been marked as deleted
					// removes it from the cache
					objects.Remove(o);
					deletedOids.Remove(oid);
					return null;
				}
				return oid;
			}
			return null;
		}

		public virtual bool IsEmpty()
		{
			return objects.Count==0;
		}

		public virtual void RemoveObject(object o)
		{
			if (o == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.CacheNullObject
					.AddParameter(" while removing object from the cache"));
			}
			OID oid = objects[o];
			objects.Remove(o);
			if (oid!=null)
			{
				// Add the oid to deleted oid
				// see junit org.neodatis.odb.test.fromusers.gyowanny_queiroz.TestBigDecimal.test13
				deletedOids.Add(oid, oid);
			}
		}

      public virtual void RemoveOid(OID oid)
		{
			deletedOids.Add(oid, oid);
		}

		public virtual int Size()
		{
			return objects.Count;
		}

		public override string ToString()
		{
			return string.Format("Cross session cache with %d objects", objects.Count);
		}

		public static void ClearAll()
		{
			System.Collections.Generic.IEnumerator<string> names = instances.Keys.GetEnumerator
				();
			while (names.MoveNext())
			{
				string name = names.Current;
				NeoDatis.Odb.Core.Transaction.ICrossSessionCache cache = instances[name];
				cache.Clear();
			}
			instances.Clear();
		}
	}
}
