namespace NeoDatis.Odb.Impl.Core.Transaction
{
	/// <summary>An ODB Session.</summary>
	/// <remarks>
	/// An ODB Session. Keeps track of all the session operations. Caches objects and
	/// manage the transaction.
	/// The meta model of the database is stored in the session.
	/// </remarks>
	/// <author>osmadja</author>
	public abstract class Session : System.IComparable, NeoDatis.Odb.Core.Transaction.ISession
	{
		protected NeoDatis.Odb.Core.Transaction.ICache cache;

		/// <summary>A temporary cache used for object info read</summary>
		protected NeoDatis.Odb.Core.Transaction.ITmpCache tmpCache;

		protected bool rollbacked;

		public string id;

		protected string baseIdentification;

		protected NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel;

		public Session(string id, string baseIdentification)
		{
			cache = BuildCache();
			tmpCache = BuildTmpCache();
			this.id = id;
			this.baseIdentification = baseIdentification;
		}

		public abstract NeoDatis.Odb.Core.Transaction.ICache BuildCache();

		public abstract NeoDatis.Odb.Core.Transaction.ITmpCache BuildTmpCache();

		public virtual NeoDatis.Odb.Core.Transaction.ICache GetCache()
		{
			return cache;
		}

		public virtual NeoDatis.Odb.Core.Transaction.ITmpCache GetTmpCache()
		{
			return tmpCache;
		}

		public virtual void Rollback()
		{
			ClearCache();
			rollbacked = true;
		}

		public virtual void Close()
		{
			Clear();
		}

		public virtual void ClearCache()
		{
			cache.Clear(false);
		}

		public virtual bool IsRollbacked()
		{
			return rollbacked;
		}

		public virtual void Clear()
		{
			cache.Clear(true);
			if (metaModel != null)
			{
				metaModel.Clear();
			}
		}

		public virtual string GetId()
		{
			return id;
		}

		public virtual void SetId(string sessionId)
		{
			this.id = sessionId;
		}

		public abstract NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine GetStorageEngine();

		public abstract bool TransactionIsPending();

		public abstract void Commit();

		public abstract NeoDatis.Odb.Core.Transaction.ITransaction GetTransaction();

		public abstract void SetFileSystemInterfaceToApplyTransaction(NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface
			 fsi);

		public override string ToString()
		{
			NeoDatis.Odb.Core.Transaction.ITransaction transaction = null;
			transaction = GetTransaction();
			if (transaction == null)
			{
				return "name=" + baseIdentification + " sid=" + id + " - no transaction";
			}
			int n = transaction.GetNumberOfWriteActions();
			return "name=" + baseIdentification + " - sid=" + id + " - Nb Actions = " + n;
		}

		public override bool Equals(object obj)
		{
			if (obj == null || !(obj is NeoDatis.Odb.Impl.Core.Transaction.Session))
			{
				return false;
			}
			NeoDatis.Odb.Core.Transaction.ISession session = (NeoDatis.Odb.Core.Transaction.ISession
				)obj;
			return GetId().Equals(session.GetId());
		}

		public virtual int CompareTo(object o)
		{
			if (o == null || !(o is NeoDatis.Odb.Impl.Core.Transaction.Session))
			{
				return -100;
			}
			NeoDatis.Odb.Core.Transaction.ISession session = (NeoDatis.Odb.Core.Transaction.ISession
				)o;
			return GetId().CompareTo(session.GetId());
		}

		public virtual string GetBaseIdentification()
		{
			return baseIdentification;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel GetMetaModel()
		{
			if (metaModel == null)
			{
				// MetaModel can be null (this happens at the end of the
				// Transaction.commitMetaModel() method)when the user commited the
				// database
				// And continue using it. In this case, after the commit, the
				// metamodel is set to null
				// and lazy-reloaded when the user use the odb again.
				metaModel = new NeoDatis.Odb.Core.Layers.Layer2.Meta.SessionMetaModel();
				try
				{
					GetStorageEngine().GetObjectReader().ReadMetaModel(metaModel, true);
				}
				catch (System.Exception e)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InternalError
						.AddParameter("Session.getMetaModel"), e);
				}
			}
			return metaModel;
		}

		public virtual void SetMetaModel(NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel2
			)
		{
			this.metaModel = metaModel2;
		}

		public virtual void SetBaseIdentification(string baseIdentification)
		{
			this.baseIdentification = baseIdentification;
		}

		public virtual void RemoveObjectFromCache(object @object)
		{
			cache.RemoveObject(@object);
		}

		public virtual void AddObjectToCache(NeoDatis.Odb.OID oid, object @object, NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
			 oih)
		{
			if (@object == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.CacheNullObject
					.AddParameter(@object));
			}
			if (oid == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.CacheNullOid
					.AddParameter(oid));
			}
			if (oih == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.CacheNullObject
					.AddParameter(oih));
			}
			cache.AddObject(oid, @object, oih);
		}
	}
}
