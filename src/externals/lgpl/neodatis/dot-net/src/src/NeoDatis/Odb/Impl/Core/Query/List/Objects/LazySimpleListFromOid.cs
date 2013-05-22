using System.Collections.ObjectModel;
using NeoDatis.Odb.Core.Layers.Layer3;
using NeoDatis.Tool.Wrappers.List;
using NeoDatis.Tool.Wrappers;
namespace NeoDatis.Odb.Impl.Core.Query.List.Objects
{
	/// <summary>A simple list to hold query result.</summary>
	/// <remarks>
	/// A simple list to hold query result. It is used when no index and no order by is
	/// used and inMemory = false
	/// This collection does not store the objects, it only holds the OIDs of the objects. When user ask an object
	/// the object is lazy loaded by the getObjectFromId method
	/// </remarks>
	/// <author>osmadja</author>
	[System.Serializable]
	public class LazySimpleListFromOid<T> : OdbArrayList<T>, NeoDatis.Odb.Objects<T>
	{
		/// <summary>a cursor when getting objects</summary>
		private int currentPosition;

		/// <summary>The odb engine to lazily get objects</summary>
		private NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine;

		/// <summary>indicate if objects must be returned as instance (true) or as non native objects (false)
		/// 	</summary>
		private bool returnInstance;

        private OdbArrayList<OID> oids;

		public LazySimpleListFromOid(int size, IStorageEngine engine, bool returnObjects) : base(size)
		{
			this.engine = engine;
			this.returnInstance = returnObjects;
            this.oids = new OdbArrayList<OID>();
		}

		public virtual bool AddWithKey(OdbComparable key, T @object)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotImplemented	);
		}

		public virtual bool AddWithKey(int key, T @object)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotImplemented
				);
		}

		public virtual T GetFirst()
		{
			try
			{
				return Get(0);
			}
			catch (System.Exception e)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ErrorWhileGettingObjectFromListAtIndex
					.AddParameter(0), e);
			}
		}

		public override T Get(int index)
		{
			NeoDatis.Odb.OID oid = oids[index];
			try
			{
				if (returnInstance)
				{
					return (T)engine.GetObjectFromOid(oid);
				}
				return (T)engine.GetObjectReader().GetObjectFromOid(oid, false, false);
			}
			catch (System.Exception)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ErrorWhileGettingObjectFromListAtIndex
					.AddParameter(index));
			}
		}

		public virtual bool HasNext()
		{
			return currentPosition < oids.Count;
		}

		public virtual System.Collections.Generic.IEnumerator<T> Iterator(NeoDatis.Odb.Core.OrderByConstants
			 orderByType)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotImplemented
				);
		}

        public int Count
        {
            get
            {
                return oids.Count;
            }
        }
		public virtual T Next()
		{
			try
			{
				return Get(currentPosition++);
			}
			catch (System.Exception e)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ErrorWhileGettingObjectFromListAtIndex
					.AddParameter(0), e);
			}
		}

		public virtual void Reset()
		{
			currentPosition = 0;
		}
        public void AddOid(OID oid)
        {
            oids.Add(oid);
        }
	}
}
