using NeoDatis.Tool.Wrappers;
using NeoDatis.Odb.Core.Query;
namespace NeoDatis.Odb.Core.Layers.Layer3.Engine
{
	/// <author>olivier</author>
	public abstract class AbstractStorageEngineReader : NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
	{
		private static readonly string LogId = "LocalStorageEngine";

		protected NeoDatis.Odb.Core.Layers.Layer3.IObjectReader objectReader;

		/// <summary>To check if database has already been closed</summary>
		protected bool isClosed;

		/// <summary>
		/// The file parameters - if we are accessing a file, it will be a
		/// IOFileParameters that contains the file name
		/// </summary>
		protected NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification baseIdentification;

		protected NeoDatis.Odb.Core.ICoreProvider provider;

		public virtual NeoDatis.Odb.Objects<T> GetObjects<T>(IQuery query, bool inMemory, int startIndex, int endIndex)
		{
			if (isClosed)
			{
				throw new ODBRuntimeException(NeoDatisError.OdbIsClosed.AddParameter(baseIdentification.GetIdentification()));
			}
            query.SetFullClassName(typeof(T));
			return objectReader.GetObjects<T>(query, inMemory, startIndex, endIndex);
		}

		public virtual void DefragmentTo(string newFileName)
		{
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			long totalNbObjects = 0;
			NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine newStorage = NeoDatis.Odb.OdbConfiguration
				.GetCoreProvider().GetClientStorageEngine(new NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter
				(newFileName, true, baseIdentification.GetUserName(), baseIdentification.GetPassword
				()));
			NeoDatis.Odb.Objects<object> defragObjects = null;
			int j = 0;
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = null;
			// User classes
			System.Collections.IEnumerator iterator = GetMetaModel().GetUserClasses().GetEnumerator
				();
			while (iterator.MoveNext())
			{
				ci = (NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo)iterator.Current;
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug("Reading " + ci.GetCommitedZoneInfo().GetNbObjects() 
						+ " objects of type " + ci.GetFullClassName());
				}
				defragObjects = GetObjects<object>(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(ci.GetFullClassName()), true, -1, -1);
				while (defragObjects.HasNext())
				{
					newStorage.Store(defragObjects.Next());
					totalNbObjects++;
					if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
					{
						if (j % 10000 == 0)
						{
							NeoDatis.Tool.DLogger.Info("\n" + totalNbObjects + " objects saved.");
						}
					}
					j++;
				}
			}
			// System classes
			iterator = GetMetaModel().GetSystemClasses().GetEnumerator();
			while (iterator.MoveNext())
			{
				ci = (NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo)iterator.Current;
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug("Reading " + ci.GetCommitedZoneInfo().GetNbObjects() 
						+ " objects of type " + ci.GetFullClassName());
				}
				defragObjects = GetObjects<object>(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
					(ci.GetFullClassName()), true, -1, -1);
				while (defragObjects.HasNext())
				{
					newStorage.Store(defragObjects.Next());
					totalNbObjects++;
					if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
					{
						if (j % 10000 == 0)
						{
							NeoDatis.Tool.DLogger.Info("\n" + totalNbObjects + " objects saved.");
						}
					}
					j++;
				}
			}
			newStorage.Commit();
			newStorage.Close();
			long time = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs() - start;
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Info("New storage " + newFileName + " created with " + totalNbObjects
					 + " objects in " + time + " ms.");
			}
		}

		protected virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel GetMetaModel()
		{
			return GetSession(true).GetMetaModel();
		}

		public abstract NeoDatis.Odb.Core.Transaction.ISession GetSession(bool throwExceptionIfDoesNotExist
			);

		public virtual void DeleteIndex(string className, string indexName, bool verbose)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo = GetMetaModel().GetClassInfo
				(className, true);
			if (!classInfo.HasIndex(indexName))
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.IndexDoesNotExist
					.AddParameter(indexName).AddParameter(className));
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex cii = classInfo.GetIndexWithName
				(indexName);
			if (verbose)
			{
				NeoDatis.Tool.DLogger.Info("Deleting index " + indexName + " on class " + className
					);
			}
			Delete(cii);
			classInfo.RemoveIndex(cii);
			if (verbose)
			{
				NeoDatis.Tool.DLogger.Info("Index " + indexName + " deleted");
			}
		}

		/// <summary>Used to rebuild an index</summary>
		public virtual void RebuildIndex(string className, string indexName, bool verbose
			)
		{
			if (verbose)
			{
				NeoDatis.Tool.DLogger.Info("Rebuilding index " + indexName + " on class " + className
					);
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo = GetMetaModel().GetClassInfo
				(className, true);
			if (!classInfo.HasIndex(indexName))
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.IndexDoesNotExist
					.AddParameter(indexName).AddParameter(className));
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex cii = classInfo.GetIndexWithName
				(indexName);
			DeleteIndex(className, indexName, verbose);
			AddIndexOn(className, indexName, classInfo.GetAttributeNames(cii.GetAttributeIds(
				)), verbose, !cii.IsUnique());
		}

		public virtual void AddIndexOn(string className, string indexName, string[] indexFields
			, bool verbose, bool acceptMultipleValuesForSameKey)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo = GetMetaModel().GetClassInfo
				(className, true);
			if (classInfo.HasIndex(indexName))
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.IndexAlreadyExist
					.AddParameter(indexName).AddParameter(className));
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex cii = classInfo.AddIndexOn(indexName
				, indexFields, acceptMultipleValuesForSameKey);
			NeoDatis.Btree.IBTree btree = null;
			if (acceptMultipleValuesForSameKey)
			{
				btree = new NeoDatis.Odb.Impl.Core.Btree.ODBBTreeMultiple(className, NeoDatis.Odb.OdbConfiguration
					.GetDefaultIndexBTreeDegree(), new NeoDatis.Odb.Impl.Core.Btree.LazyODBBTreePersister
					(this));
			}
			else
			{
				btree = new NeoDatis.Odb.Impl.Core.Btree.ODBBTreeSingle(className, NeoDatis.Odb.OdbConfiguration
					.GetDefaultIndexBTreeDegree(), new NeoDatis.Odb.Impl.Core.Btree.LazyODBBTreePersister
					(this));
			}
			cii.SetBTree(btree);
			Store(cii);
			// Now The index must be updated with all existing objects.
			if (classInfo.GetNumberOfObjects() == 0)
			{
				// There are no objects. Nothing to do
				return;
			}
			if (verbose)
			{
				NeoDatis.Tool.DLogger.Info("Creating index " + indexName + " on class " + className
					 + " - Class has already " + classInfo.GetNumberOfObjects() + " Objects. Updating index"
					);
			}
			if (verbose)
			{
				NeoDatis.Tool.DLogger.Info(indexName + " : loading " + classInfo.GetNumberOfObjects
					() + " objects from database");
			}
			// We must load all objects and insert them in the index!
			NeoDatis.Odb.Objects<object> objects = GetObjectInfos<object>(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(className), false, -1, -1, false);
			if (verbose)
			{
				NeoDatis.Tool.DLogger.Info(indexName + " : " + classInfo.GetNumberOfObjects() + " objects loaded"
					);
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = null;
			int i = 0;
			bool monitorMemory = NeoDatis.Odb.OdbConfiguration.IsMonitoringMemory();
			while (objects.HasNext())
			{
				nnoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo)objects.Next();
				btree.Insert(cii.ComputeKey(nnoi), nnoi.GetOid());
				if (verbose && i % 1000 == 0)
				{
					if (monitorMemory)
					{
						NeoDatis.Odb.Impl.Tool.MemoryMonitor.DisplayCurrentMemory("Index " + indexName + 
							" " + i + " objects inserted", true);
					}
				}
				i++;
			}
			if (verbose)
			{
				NeoDatis.Tool.DLogger.Info(indexName + " created!");
			}
		}

		public virtual NeoDatis.Odb.Objects<T> GetObjectInfos<T>(NeoDatis.Odb.Core.Query.IQuery
			 query, bool inMemory, int startIndex, int endIndex, bool returnObjects)
		{
			NeoDatis.Odb.Core.Query.Execution.IMatchingObjectAction queryResultAction = provider
				.GetCollectionQueryResultAction(this, query, inMemory, returnObjects);
			return objectReader.GetObjectInfos<T>(query, inMemory, startIndex, endIndex, returnObjects
				, queryResultAction);
		}

		public virtual NeoDatis.Odb.Objects<T> GetObjects<T>(System.Type clazz, bool inMemory
			, int startIndex, int endIndex)
		{
			if (isClosed)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbIsClosed
					.AddParameter(baseIdentification.GetIdentification()));
			}
			return objectReader.GetObjects<T>(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(OdbClassUtil.GetFullName(clazz)), inMemory, startIndex, endIndex);
		}

		public abstract NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList AddClasses(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList
			 arg1);

		public abstract void AddCommitListener(NeoDatis.Odb.Core.Layers.Layer3.ICommitListener
			 arg1);

		public abstract void AddDeleteTriggerFor(string arg1, NeoDatis.Odb.Core.Trigger.DeleteTrigger
			 arg2);

		public abstract void AddInsertTriggerFor(string arg1, NeoDatis.Odb.Core.Trigger.InsertTrigger
			 arg2);

		public abstract void AddSelectTriggerFor(string arg1, NeoDatis.Odb.Core.Trigger.SelectTrigger
			 arg2);

		public abstract void AddSession(NeoDatis.Odb.Core.Transaction.ISession arg1, bool
			 arg2);

		public abstract void AddUpdateTriggerFor(string arg1, NeoDatis.Odb.Core.Trigger.UpdateTrigger
			 arg2);

		public abstract NeoDatis.Odb.Core.Transaction.ISession BuildDefaultSession();

		public abstract NeoDatis.Odb.Core.Layers.Layer1.Introspector.IObjectIntrospector 
			BuildObjectIntrospector();

		public abstract NeoDatis.Odb.Core.Layers.Layer3.IObjectReader BuildObjectReader();

		public abstract NeoDatis.Odb.Core.Layers.Layer3.IObjectWriter BuildObjectWriter();

		public abstract NeoDatis.Odb.Core.Trigger.ITriggerManager BuildTriggerManager();

		public abstract NeoDatis.Odb.Core.Layers.Layer3.Engine.CheckMetaModelResult CheckMetaModelCompatibility
			(System.Collections.Generic.IDictionary<string, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			> arg1);

		public abstract void Close();

		public abstract void Commit();

		public abstract long Count(NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery arg1
			);

		public abstract NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery CriteriaQuery
			(System.Type arg1, NeoDatis.Odb.Core.Query.Criteria.ICriterion arg2);

		public abstract NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery CriteriaQuery
			(System.Type arg1);

		public abstract NeoDatis.Odb.OID Delete(object arg1);

		public abstract void DeleteObjectWithOid(NeoDatis.Odb.OID arg1);

		public abstract void Disconnect(object arg1);

		public abstract System.Collections.Generic.IList<NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.FullIDInfo
			> GetAllObjectIdInfos(string arg1, bool arg2);

		public abstract System.Collections.Generic.IList<long> GetAllObjectIds();

		public abstract NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification GetBaseIdentification
			();

		public abstract NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer3.ICommitListener
			> GetCommitListeners();

		public abstract NeoDatis.Odb.OID GetCurrentIdBlockMaxOid();

		public abstract int GetCurrentIdBlockNumber();

		public abstract long GetCurrentIdBlockPosition();

		public abstract NeoDatis.Odb.TransactionId GetCurrentTransactionId();

		public abstract NeoDatis.Odb.DatabaseId GetDatabaseId();

		public abstract NeoDatis.Odb.OID GetMaxOid();

		public abstract NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo GetMetaObjectFromOid
			(NeoDatis.Odb.OID arg1);

		public abstract object GetObjectFromOid(NeoDatis.Odb.OID arg1);

		public abstract NeoDatis.Odb.OID GetObjectId(object arg1, bool arg2);

		public abstract NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader GetObjectInfoHeaderFromOid
			(NeoDatis.Odb.OID arg1);

		public abstract NeoDatis.Odb.Core.Layers.Layer1.Introspector.IObjectIntrospector 
			GetObjectIntrospector();

		public abstract NeoDatis.Odb.Core.Layers.Layer3.IObjectReader GetObjectReader();

		public abstract NeoDatis.Odb.Core.Layers.Layer3.IObjectWriter GetObjectWriter();

		public abstract NeoDatis.Odb.Core.Layers.Layer3.IRefactorManager GetRefactorManager
			();

		public abstract NeoDatis.Odb.Core.Trigger.ITriggerManager GetTriggerManager();

		public abstract NeoDatis.Odb.Values GetValues(NeoDatis.Odb.Core.Query.IValuesQuery
			 arg1, int arg2, int arg3);

		public abstract int GetVersion();

		public abstract bool IsClosed();

		public abstract bool IsLocal();

		public abstract void Reconnect(object arg1);

		public abstract void ResetCommitListeners();

		public abstract void Rollback();

		public abstract void SetCurrentIdBlockInfos(long arg1, int arg2, NeoDatis.Odb.OID
			 arg3);

		public abstract void SetCurrentTransactionId(NeoDatis.Odb.TransactionId arg1);

		public abstract void SetDatabaseId(NeoDatis.Odb.DatabaseId arg1);

		public abstract void SetLastODBCloseStatus(bool arg1);

		public abstract void SetMetaModel(NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel 
			arg1);

		public abstract void SetNbClasses(long arg1);

		public abstract void SetVersion(int arg1);

		public abstract NeoDatis.Odb.OID Store(NeoDatis.Odb.OID arg1, object arg2);

		public abstract NeoDatis.Odb.OID Store(object arg1);

		public abstract NeoDatis.Odb.OID UpdateObject(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 arg1, bool arg2);

		public abstract NeoDatis.Odb.OID WriteObjectInfo(NeoDatis.Odb.OID arg1, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 arg2, long arg3, bool arg4);
	}
}
