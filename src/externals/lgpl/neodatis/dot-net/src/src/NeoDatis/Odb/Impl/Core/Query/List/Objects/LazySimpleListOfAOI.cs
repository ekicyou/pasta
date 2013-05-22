using System;
namespace NeoDatis.Odb.Impl.Core.Query.List.Objects
{
	/// <summary>A simple list to hold query result.</summary>
	/// <remarks>
	/// A simple list to hold query result. It is used when no index and no order by
	/// This collection does not store the objects, it only holds the Abstract Object Info (AOI) of the objects. When user ask an object
	/// the object is lazy loaded by the buildInstance method
	/// </remarks>
	/// <author>osmadja</author>
	[System.Serializable]
	public class LazySimpleListOfAOI<T> : NeoDatis.Tool.Wrappers.List.OdbArrayList<T>
		, NeoDatis.Odb.Objects<T>
	{
		/// <summary>a cursor when getting objects</summary>
		private int currentPosition;

		/// <summary>The odb engine to lazily get objects</summary>
		[System.NonSerialized]
		private NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder instanceBuilder;

		/// <summary>this session id is used to store the odb session id.</summary>
		/// <remarks>
		/// this session id is used to store the odb session id. When in true client server mode, when the lazy list is sent
		/// back to the client, the instance builder (declared as transient) will be null on the client side.
		/// Then the client will use the Lookup class with the base id to obtain the client instance builder
		/// </remarks>
		private string sessionId;

		/// <summary>indicate if objects must be returned as instance (true) or as non native objects (false)
		/// 	</summary>
		private bool returnInstance;

		public LazySimpleListOfAOI() : base(10)
		{
		}

		/// <param name="size"></param>
		/// <param name="builder"></param>
		/// <param name="returnInstance"></param>
		public LazySimpleListOfAOI(int size, NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder
			 builder, bool returnInstance) : base(10)
		{
			// If in client server mode, the instance builder will be set on the client.
			if (builder.IsLocal())
			{
				this.instanceBuilder = builder;
			}
			this.sessionId = builder.GetSessionId();
			this.returnInstance = returnInstance;
		}

		public virtual bool AddWithKey(NeoDatis.Tool.Wrappers.OdbComparable key, T @object
			)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotImplemented
				);
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
				return this[0];
			}
			catch (System.Exception e)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ErrorWhileGettingObjectFromListAtIndex
					.AddParameter(0), e);
			}
		}

		public override T Get(int index)
		{
			object o = base[index];
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
				)o;
			try
			{
				if (aoi.IsNull())
				{
					return default(T);
				}
				if (returnInstance)
				{
					if (aoi.IsNative())
					{
						return (T)aoi.GetObject();
					}
					if (instanceBuilder == null)
					{
						// Lookup the instance builder
						instanceBuilder = (NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder)NeoDatis.Odb.Core.Lookup.LookupFactory
							.Get(sessionId).Get(NeoDatis.Odb.Impl.Core.Lookup.Lookups.InstanceBuilder);
						if (instanceBuilder == null)
						{
							throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.LookupKeyNotFound
								.AddParameter(NeoDatis.Odb.Impl.Core.Lookup.Lookups.InstanceBuilder));
						}
					}
					return (T)instanceBuilder.BuildOneInstance((NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
						)aoi);
				}
				// No need to return Instance return the layer 2 representation
				o = aoi;
				return (T)o;
			}
			catch (System.Exception e)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ErrorWhileGettingObjectFromListAtIndex
					.AddParameter(index), e);
			}
		}

		public virtual bool HasNext()
		{
			return currentPosition < Count;
		}

		public virtual System.Collections.Generic.IEnumerator<T> Iterator(NeoDatis.Odb.Core.OrderByConstants
			 orderByType)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotImplemented
				);
		}

		public virtual T Next()
		{
			try
			{
				return this[currentPosition++];
			}
			catch (System.Exception e)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ErrorWhileGettingObjectFromListAtIndex
					.AddParameter(0), e);
			}
		}
        public void AddOid(OID oid)
        {
            throw new Exception("Add Oid not implemented ");
        }

		public virtual void Reset()
		{
			currentPosition = 0;
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append("list with ").Append(Count).Append(" elements");
			return buffer.ToString();
		}
	}
}
