namespace NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine
{
	/// <summary>An Adapter for IStorageEngine interface.</summary>
	/// <remarks>An Adapter for IStorageEngine interface.</remarks>
	/// <author>osmadja</author>
	public abstract class StorageEngineAdapter : NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
	{
		private NeoDatis.Odb.DatabaseId databaseId;

		/// <summary>To keep track of current transaction Id</summary>
		protected NeoDatis.Odb.TransactionId currentTransactionId;

		/// <summary>To manage triggers</summary>
		protected NeoDatis.Odb.Core.Trigger.ITriggerManager triggerManager;

		protected bool isClosed;

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo AddClass(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 newClassInfo, bool addDependentClasses)
		{
			return null;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList AddClasses(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList
			 classInfoList)
		{
			System.Collections.IEnumerator iterator = classInfoList.GetClassInfos().GetEnumerator
				();
			while (iterator.MoveNext())
			{
				AddClass((NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo)iterator.Current, false);
			}
			return classInfoList;
		}

		public abstract void AddDeleteTrigger(System.Type clazz, NeoDatis.Odb.Core.Trigger.DeleteTrigger
			 trigger);

		public abstract void AddInsertTrigger(System.Type clazz, NeoDatis.Odb.Core.Trigger.InsertTrigger
			 trigger);

		public abstract void AddSelectTrigger(System.Type clazz, NeoDatis.Odb.Core.Trigger.SelectTrigger
			 trigger);

		public abstract void AddUpdateTrigger(System.Type clazz, NeoDatis.Odb.Core.Trigger.UpdateTrigger
			 trigger);

		public virtual void Close()
		{
		}

		public virtual void Commit()
		{
		}

		public virtual long Count(string fullClassName)
		{
			return 0;
		}

		/// <exception cref="System.Exception"></exception>
		public virtual long Count(System.Type clazz)
		{
			return 0;
		}

		public virtual void DefragmentTo(string newFileName)
		{
		}

		public virtual NeoDatis.Odb.OID Delete(object @object)
		{
			return null;
		}

		public virtual NeoDatis.Odb.OID InternalDelete(NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
			 header)
		{
			return null;
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void DeleteObjectWithOid(long oid)
		{
		}

		public virtual System.Collections.Generic.IList<NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.FullIDInfo
			> GetAllObjectIdInfos(string objectType, bool displayObjects)
		{
			return null;
		}

		public virtual System.Collections.Generic.IList<long> GetAllObjectIds()
		{
			return null;
		}

		public virtual NeoDatis.Odb.OID GetCurrentIdBlockMaxOid()
		{
			return NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.NullObjectId;
		}

		public virtual int GetCurrentIdBlockNumber()
		{
			return 0;
		}

		public virtual long GetCurrentIdBlockPosition()
		{
			return 0;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface GetFsi
			()
		{
			return null;
		}

		public virtual NeoDatis.Odb.OID GetMaxOid()
		{
			return NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.NullObjectId;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel Get2MetaModel()
		{
			return null;
		}

		/// <exception cref="System.Exception"></exception>
		public virtual object GetObjectFromId(NeoDatis.Odb.OID id)
		{
			return null;
		}

		public virtual NeoDatis.Odb.OID GetObjectId(object @object, bool throwExceptionIfDoesNotExist
			)
		{
			return null;
		}

		/// <exception cref="System.Exception"></exception>
		public virtual NeoDatis.Odb.Objects<T> GetObjectInfos<T>(string fullClassName, NeoDatis.Odb.Core.Query.IQuery
			 query, bool inMemory, int startIndex, int endIndex, bool returnOjects)
		{
			return null;
		}

		public virtual NeoDatis.Odb.Objects<T> GetObjectInfos<T>(NeoDatis.Odb.Core.Query.IQuery
			 query, bool inMemory, int startIndex, int endIndex, bool returnOjects)
		{
			return null;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer3.IObjectReader GetObjectReader()
		{
			return null;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer3.IObjectWriter GetObjectWriter()
		{
			return null;
		}

		public virtual NeoDatis.Odb.Objects<T> GetObjects<T>(NeoDatis.Odb.Core.Query.IQuery
			 query, bool inMemory, int startIndex, int endIndex)
		{
			return null;
		}

		/// <exception cref="System.Exception"></exception>
		public virtual NeoDatis.Odb.Objects<T> GetObjects<T>(string fullClassName, NeoDatis.Odb.Core.Query.IQuery
			 query, bool inMemory, int startIndex, int endIndex)
		{
			return null;
		}

		public virtual NeoDatis.Odb.Core.Transaction.ISession GetSession(bool throwExceptionIfDoesNotExist
			)
		{
			return null;
		}

		public virtual int GetVersion()
		{
			return 0;
		}

		public virtual bool IsClosed()
		{
			return isClosed;
		}

		/// <exception cref="System.Exception"></exception>
		public virtual long MainStoreObject(object @object)
		{
			return 0;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo PersistClass(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 newClassInfo, int lastClassInfoIndex, bool addClass, bool addDependentClasses)
		{
			return null;
		}

		public virtual void Rollback()
		{
		}

		public virtual void SetCurrentIdBlockInfos(long currentBlockPosition, int currentBlockNumber
			, NeoDatis.Odb.OID maxId)
		{
		}

		public virtual void SetDatabaseId(long[] databaseId)
		{
		}

		public virtual void SetLastODBCloseStatus(bool lastCloseStatus)
		{
		}

		public virtual void SetMetaModel(NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel
			)
		{
		}

		public virtual void SetNbClasses(long nbClasses)
		{
		}

		public virtual void SetVersion(int version)
		{
		}

		public virtual NeoDatis.Odb.TransactionId GetCurrentTransactionId()
		{
			return currentTransactionId;
		}

		public virtual void SetCurrentTransactionId(NeoDatis.Odb.TransactionId transactionId
			)
		{
			currentTransactionId = transactionId;
		}

		public virtual void SetDatabaseId(NeoDatis.Odb.DatabaseId databaseId)
		{
			this.databaseId = databaseId;
		}

		public virtual NeoDatis.Odb.DatabaseId GetDatabaseId()
		{
			return databaseId;
		}

		public virtual void Disconnect(object @object)
		{
			if (NeoDatis.Odb.OdbConfiguration.ReconnectObjectsToSession())
			{
				NeoDatis.Odb.Impl.Core.Transaction.CacheFactory.GetCrossSessionCache(this.GetBaseIdentification
					().GetIdentification()).RemoveObject(@object);
			}
		}

		public virtual void Reconnect(object @object)
		{
		}

		// nothing to do
		public virtual NeoDatis.Odb.Core.Trigger.ITriggerManager GetTriggerManager()
		{
			return triggerManager;
		}

		public virtual void AddDeleteTriggerFor(string className, NeoDatis.Odb.Core.Trigger.DeleteTrigger
			 trigger)
		{
			triggerManager.AddDeleteTriggerFor(className, trigger);
		}

		public virtual void AddInsertTriggerFor(string className, NeoDatis.Odb.Core.Trigger.InsertTrigger
			 trigger)
		{
			triggerManager.AddInsertTriggerFor(className, trigger);
		}

		public virtual void AddSelectTriggerFor(string className, NeoDatis.Odb.Core.Trigger.SelectTrigger
			 trigger)
		{
			triggerManager.AddSelectTriggerFor(className, trigger);
		}

		public virtual void AddUpdateTriggerFor(string className, NeoDatis.Odb.Core.Trigger.UpdateTrigger
			 trigger)
		{
			triggerManager.AddUpdateTriggerFor(className, trigger);
		}

		public abstract NeoDatis.Odb.Objects<T> GetObjects<T>(System.Type clazz, bool inMemory
			, int startIndex, int endIndex);

		public virtual NeoDatis.Odb.Core.Layers.Layer3.Engine.CheckMetaModelResult CheckMetaModelCompatibility
			(System.Collections.Generic.IDictionary<string, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			> currentCIs)
		{
			return null;
		}

		public virtual NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery CriteriaQuery(
			System.Type clazz, NeoDatis.Odb.Core.Query.Criteria.ICriterion criterion)
		{
			NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(clazz, criterion);
			q.SetStorageEngine(this);
			if (criterion != null)
			{
				criterion.Ready();
			}
			return q;
		}

		public virtual NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery CriteriaQuery(
			System.Type clazz)
		{
			NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(clazz);
			q.SetStorageEngine(this);
			return q;
		}

		public abstract void AddCommitListener(NeoDatis.Odb.Core.Layers.Layer3.ICommitListener
			 arg1);

		public abstract void AddIndexOn(string arg1, string arg2, string[] arg3, bool arg4
			, bool arg5);

		public abstract void AddSession(NeoDatis.Odb.Core.Transaction.ISession arg1, bool
			 arg2);

		public abstract NeoDatis.Odb.Core.Transaction.ISession BuildDefaultSession();

		public abstract NeoDatis.Odb.Core.Layers.Layer1.Introspector.IObjectIntrospector 
			BuildObjectIntrospector();

		public abstract NeoDatis.Odb.Core.Layers.Layer3.IObjectReader BuildObjectReader();

		public abstract NeoDatis.Odb.Core.Layers.Layer3.IObjectWriter BuildObjectWriter();

		public abstract NeoDatis.Odb.Core.Trigger.ITriggerManager BuildTriggerManager();

		public abstract long Count(NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery arg1
			);

		public abstract void DeleteIndex(string arg1, string arg2, bool arg3);

		public abstract void DeleteObjectWithOid(NeoDatis.Odb.OID arg1);

		public abstract NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification GetBaseIdentification
			();

		public abstract NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer3.ICommitListener
			> GetCommitListeners();

		public abstract NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo GetMetaObjectFromOid
			(NeoDatis.Odb.OID arg1);

		public abstract object GetObjectFromOid(NeoDatis.Odb.OID arg1);

		public abstract NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader GetObjectInfoHeaderFromOid
			(NeoDatis.Odb.OID arg1);

		public abstract NeoDatis.Odb.Core.Layers.Layer1.Introspector.IObjectIntrospector 
			GetObjectIntrospector();

		public abstract NeoDatis.Odb.Core.Layers.Layer3.IRefactorManager GetRefactorManager
			();

		public abstract NeoDatis.Odb.Values GetValues(NeoDatis.Odb.Core.Query.IValuesQuery
			 arg1, int arg2, int arg3);

		public abstract bool IsLocal();

		public abstract void RebuildIndex(string arg1, string arg2, bool arg3);

		public abstract void ResetCommitListeners();

		public abstract NeoDatis.Odb.OID Store(NeoDatis.Odb.OID arg1, object arg2);

		public abstract NeoDatis.Odb.OID Store(object arg1);

		public abstract NeoDatis.Odb.OID UpdateObject(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 arg1, bool arg2);

		public abstract NeoDatis.Odb.OID WriteObjectInfo(NeoDatis.Odb.OID arg1, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 arg2, long arg3, bool arg4);
	}
}
