using NeoDatis.Odb.Core.Transaction;
using NeoDatis.Odb.Core.Layers.Layer2.Meta;
using NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine;
using NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector;
using System.Collections.Generic;
using System;
using NeoDatis.Tool.Wrappers;
using NeoDatis.Odb.Core.Layers.Layer1.Introspector;
using NeoDatis.Tool.Wrappers.List;
namespace NeoDatis.Odb.Core.Layers.Layer3.Engine
{
	/// <summary>
	/// The storage Engine
	/// <pre>
	/// The Local Storage Engine class in the most important class in ODB.
	/// </summary>
	/// <remarks>
	/// The storage Engine
	/// <pre>
	/// The Local Storage Engine class in the most important class in ODB. It manages reading, writing and querying objects.
	/// All write operations are delegated to the ObjectWriter class.
	/// All read operations are delegated to the ObjectReader class.
	/// All Id operations are delegated to the IdManager class.
	/// All Introspecting operations are delegated to the ObjectIntrospector class.
	/// All Trigger operations are delegated to the TriggerManager class.
	/// All session related operations are executed by The Session class. Session Class using the Transaction
	/// class are responsible for ACID behavior.
	/// </pre>
	/// </remarks>
	public abstract class AbstractStorageEngine : NeoDatis.Odb.Core.Layers.Layer3.Engine.AbstractStorageEngineReader
		, NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
	{
		private static readonly string LogId = "LocalStorageEngine";

		private int version;

		private NeoDatis.Odb.DatabaseId databaseId;

		private NeoDatis.Odb.Core.Layers.Layer3.IObjectWriter objectWriter;

		protected NeoDatis.Odb.Core.Layers.Layer1.Introspector.IObjectIntrospector objectIntrospector;

		protected NeoDatis.Odb.Core.Layers.Layer1.Introspector.IClassIntrospector classIntrospector;

		/// <summary>The meta-model number of classes - used only for meta model loading</summary>
		private long nbClasses;

		/// <summary>the last odb close status - to check if a recover is necessary</summary>
		private bool lastOdbCloseStatus;

		/// <summary>The position of the current block where IDs are stored</summary>
		private long currentIdBlockPosition;

		/// <summary>The current id block number</summary>
		private int currentIdBlockNumber;

		/// <summary>The max id already allocated in the current id block</summary>
		private NeoDatis.Odb.OID currentIdBlockMaxOid;

		protected NeoDatis.Odb.Core.Trigger.ITriggerManager triggerManager;

		protected NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer3.ICommitListener
			> commitListeners;

		/// <summary>
		/// Used to know if the storage engine is executed in local mode (embedded
		/// mode) or client server mode
		/// </summary>
		protected bool isLocal;

		/// <summary>To keep track of current transaction Id</summary>
		protected NeoDatis.Odb.TransactionId currentTransactionId;

		/// <summary>This is a visitor used to execute some specific action(like calling 'Before Insert Trigger')  when introspecting an object
		/// 	</summary>
		protected NeoDatis.Odb.Core.Layers.Layer1.Introspector.IIntrospectionCallback introspectionCallbackForInsert;

		/// <summary>This is a visitor used to execute some specific action when introspecting an object
		/// 	</summary>
		protected NeoDatis.Odb.Core.Layers.Layer1.Introspector.IIntrospectionCallback introspectionCallbackForUpdate;

		/// <summary>The database file name</summary>
		/// <></>
		public AbstractStorageEngine(NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification 
			parameters)
		{
			this.provider = NeoDatis.Odb.OdbConfiguration.GetCoreProvider();
			this.baseIdentification = parameters;
			Init();
		}

		protected virtual void Init()
		{
			CheckRuntimeCompatibility();
			isClosed = false;
			isLocal = baseIdentification.IsLocal();
			// The check if it is a new Database must be executed before object
			// writer initialization. Because Object Writer Init
			// Creates the file so the check (which is based on the file existence
			// would always return false*/
			bool isNewDatabase = IsNewDatabase();
			commitListeners = new OdbArrayList<ICommitListener>();
			classIntrospector = provider.GetClassIntrospector();
			ISession session = BuildDefaultSession();
			// Object Writer must be created before object Reader
			objectWriter = BuildObjectWriter();
			// Object writer is a two Phase init object
			objectWriter.Init2();
			objectReader = BuildObjectReader();
			AddSession(session, false);
			// If the file does not exist, then a default header must be created
			if (isNewDatabase)
			{
				objectWriter.CreateEmptyDatabaseHeader(OdbTime.GetCurrentTimeInMs(), baseIdentification.GetUserName(), baseIdentification.GetPassword());
			}
			else
			{
				try
				{
					GetObjectReader().ReadDatabaseHeader(baseIdentification.GetUserName(), baseIdentification.GetPassword());
				}
				catch (ODBAuthenticationRuntimeException e)
				{
					Close();
					throw;
				}
			}
			objectWriter.AfterInit();
			objectIntrospector = BuildObjectIntrospector();
			this.triggerManager = BuildTriggerManager();
			// This forces the initialization of the meta model
			MetaModel metaModel = GetMetaModel();
			if (OdbConfiguration.CheckModelCompatibility())
			{
				CheckMetaModelCompatibility(classIntrospector.Instrospect(metaModel.GetAllClasses()));
			}
			// logically locks access to the file (only for this Virtual machine)
			FileMutex.GetInstance().OpenFile(GetStorageDeviceName());
			// Updates the Transaction Id in the file
			objectWriter.WriteLastTransactionId(GetCurrentTransactionId());
			this.objectWriter.SetTriggerManager(this.triggerManager);
			this.introspectionCallbackForInsert = new DefaultInstrumentationCallbackForStore(this, triggerManager, false);
			this.introspectionCallbackForUpdate = new DefaultInstrumentationCallbackForStore(this, triggerManager, true);
		}

		public override void AddSession(ISession session, bool
			 readMetamodel)
		{
			// Associate current session to the fsi -> all transaction writes
			// will be applied to this FileSystemInterface
			session.SetFileSystemInterfaceToApplyTransaction(objectWriter.GetFsi());
			if (readMetamodel)
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel = null;
				try
				{
					objectReader.ReadDatabaseHeader(baseIdentification.GetUserName(), baseIdentification
						.GetPassword());
				}
				catch (NeoDatis.Odb.ODBAuthenticationRuntimeException e)
				{
					Close();
					throw;
				}
				metaModel = new NeoDatis.Odb.Core.Layers.Layer2.Meta.SessionMetaModel();
				session.SetMetaModel(metaModel);
				metaModel = objectReader.ReadMetaModel(metaModel, true);
				// Updates the Transaction Id in the file
				objectWriter.WriteLastTransactionId(GetCurrentTransactionId());
			}
		}

		/// <summary>
		/// Receive the current class info (loaded from current java classes present on classpath
		/// and check against the persisted meta model
		/// </summary>
		/// <param name="currentCIs"></param>
		public override CheckMetaModelResult CheckMetaModelCompatibility(System.Collections.Generic.IDictionary<string, ClassInfo> currentCIs)
		{
			ClassInfo persistedCI = null;
			ClassInfo currentCI = null;
			ClassInfoCompareResult result = null;
			CheckMetaModelResult checkMetaModelResult = new CheckMetaModelResult();
			// User classes
			IEnumerator<ClassInfo> iterator = GetMetaModel().GetUserClasses().GetEnumerator();
			while (iterator.MoveNext())
			{
				persistedCI = iterator.Current;
				currentCI = currentCIs[persistedCI.GetFullClassName()];
				result = persistedCI.ExtractDifferences(currentCI, true);
				if (!result.IsCompatible())
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.IncompatibleMetamodel
						.AddParameter(result.ToString()));
				}
				if (result.HasCompatibleChanges())
				{
					checkMetaModelResult.Add(result);
				}
			}
			// System classes
			iterator = GetMetaModel().GetSystemClasses().GetEnumerator();
			while (iterator.MoveNext())
			{
				persistedCI = iterator.Current;
				currentCI = currentCIs[persistedCI.GetFullClassName()];
				result = persistedCI.ExtractDifferences(currentCI, true);
				if (!result.IsCompatible())
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.IncompatibleMetamodel
						.AddParameter(result.ToString()));
				}
				if (result.HasCompatibleChanges())
				{
					checkMetaModelResult.Add(result);
				}
			}
			for (int i = 0; i < checkMetaModelResult.Size(); i++)
			{
				result = checkMetaModelResult.GetResults()[i];
				NeoDatis.Tool.DLogger.Info("Class " + result.GetFullClassName() + " has changed :"
					);
				NeoDatis.Tool.DLogger.Info(result.ToString());
			}
			if (!checkMetaModelResult.GetResults().IsEmpty())
			{
				UpdateMetaModel();
				checkMetaModelResult.SetModelHasBeenUpdated(true);
			}
			return checkMetaModelResult;
		}

		/// <summary>This is a runtime compatibility check.</summary>
		/// <remarks>This is a runtime compatibility check. Java version must be greater than 1.5
		/// 	</remarks>
		public virtual void CheckRuntimeCompatibility()
		{
			if (!NeoDatis.Odb.OdbConfiguration.CheckRuntimeVersion())
			{
				return;
			}
			string runtimeVersion = null;
			try
			{
				runtimeVersion = NeoDatis.Tool.Wrappers.OdbSystem.GetProperty("java.version");
			}
			catch (System.Exception)
			{
			}
			// TODO Auto-generated catch block
			//e.printStackTrace();
			if (runtimeVersion != null)
			{
				// android : returns "0" as java runtime version=> ignore runtime versin check for android
				string os = NeoDatis.Tool.Wrappers.OdbSystem.GetProperty("os.name");
				string osArc = NeoDatis.Tool.Wrappers.OdbSystem.GetProperty("os.arch");
				string javaVendor = NeoDatis.Tool.Wrappers.OdbSystem.GetProperty("java.vendor");
				// This is just a protection
				if (javaVendor == null)
				{
					NeoDatis.Tool.DLogger.Info("Current JVM does not have 'java.vendor' property defined => unable to check JVM runtime compatibility"
						);
					return;
				}
				// android : we assume that java version is ok
				// Because the java version is equal to "0" on android :-(
				if (javaVendor.Equals("The Android Project"))
				{
					return;
				}
				// else we need to check
				// First : protection against bad formed version
				if (runtimeVersion == null || runtimeVersion.Length < 3)
				{
					NeoDatis.Tool.DLogger.Info("Current JVM does not have correct vava version => unable to check JVM runtime compatibility"
						);
					return;
				}
				double version = float.Parse(NeoDatis.Tool.Wrappers.OdbString.Substring(runtimeVersion
					, 0, 3));
				if (version < 1.5)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.IncompatibleJavaVm
						.AddParameter(runtimeVersion));
				}
			}
		}

		public virtual void UpdateMetaModel()
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel = GetMetaModel();
			NeoDatis.Tool.DLogger.Info("Automatic refactoring : updating meta model");
			// User classes
			System.Collections.IEnumerator iterator = metaModel.GetUserClasses().GetEnumerator
				();
			while (iterator.MoveNext())
			{
				objectWriter.UpdateClassInfo((NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo)iterator
					.Current, true);
			}
			// System classes
			iterator = metaModel.GetSystemClasses().GetEnumerator();
			while (iterator.MoveNext())
			{
				objectWriter.UpdateClassInfo((NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo)iterator
					.Current, true);
			}
		}

		private string GetStorageDeviceName()
		{
			return baseIdentification.GetIdentification();
		}

		private bool IsNewDatabase()
		{
			return baseIdentification.IsNew();
		}

		public override NeoDatis.Odb.OID Store(object @object)
		{
			return Store(null, @object);
		}

		public override NeoDatis.Odb.OID Store(NeoDatis.Odb.OID oid, object @object)
		{
			if (isClosed)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbIsClosed
					.AddParameter(baseIdentification.GetIdentification()));
			}
			// triggers before
			// triggerManager.manageInsertTriggerBefore(object.getClass().getName(),
			// object);
			NeoDatis.Odb.OID newOid = InternalStore(oid, @object);
			// triggers after - fixme
			// triggerManager.manageInsertTriggerAfter(object.getClass().getName(),
			// object, newOid);
			GetSession(true).GetCache().ClearInsertingObjects();
			return newOid;
		}

		/// <summary>Store an object in ODBFactory database.</summary>
		/// <remarks>
		/// Store an object in ODBFactory database.
		/// <pre>
		/// Transforms the object into meta representation and calls the internalStoreObject
		/// </pre>
		/// </remarks>
		/// <param name="@object"></param>
		/// <returns>The object insertion position</returns>
		/// <exception cref="System.IO.IOException">System.IO.IOException</exception>
		protected virtual NeoDatis.Odb.OID InternalStore(object @object)
		{
			return InternalStore(null, @object);
		}

		/// <summary>Store an object with the specific id</summary>
		/// <param name="oid"></param>
		/// <param name="@object"></param>
		/// <returns></returns>
		/// <></>
		protected virtual NeoDatis.Odb.OID InternalStore(OID oid, object o)
		{
			if (GetSession(true).IsRollbacked())
			{
				throw new ODBRuntimeException(NeoDatisError.OdbHasBeenRollbacked.AddParameter(GetBaseIdentification().ToString()));
			}
			if (o == null)
			{
				throw new ODBRuntimeException(NeoDatisError.OdbCanNotStoreNullObject);
			}
			Type clazz = o.GetType();
			if (ODBType.IsNative(clazz))
			{
				throw new ODBRuntimeException(NeoDatisError.OdbCanNotStoreNativeObjectDirectly
					.AddParameter(clazz.FullName).AddParameter(ODBType.GetFromClass(clazz).GetName()).AddParameter(clazz.FullName));
			}
			// The object must be transformed into meta representation
			ClassInfo ci = null;
			string className = OdbClassUtil.GetFullName(clazz);
			// first checks if the class of this object already exist in the
			// metamodel
			if (GetMetaModel().ExistClass(className))
			{
				ci = GetMetaModel().GetClassInfo(className, true);
			}
			else
			{
				ClassInfoList ciList = classIntrospector.Introspect(o.GetType(), true);
				// All new classes found
				objectWriter.AddClasses(ciList);
				ci = ciList.GetMainClassInfo();
			}
			// first detects if we must perform an insert or an update
			// If object is in the cache, we must perform an update, else an insert
			bool mustUpdate = false;
			ICache cache = GetSession(true).GetCache();
			if (o != null)
			{
				OID cacheOid = cache.IdOfInsertingObject(o);
				if (cacheOid != null)
				{
					return cacheOid;
				}
				// throw new ODBRuntimeException("Inserting meta representation of
				// an object without the object itself is not yet supported");
				mustUpdate = cache.ExistObject(o);
			}
			// The introspection callback is used to execute some specific task (like calling trigger, for example) while introspecting the object
			IIntrospectionCallback callback = introspectionCallbackForInsert;
			if (mustUpdate)
			{
				callback = introspectionCallbackForUpdate;
			}
			// Transform the object into an ObjectInfo
			NonNativeObjectInfo nnoi = (NonNativeObjectInfo)objectIntrospector.GetMetaRepresentation(o, ci, true, null, callback);
			// During the introspection process, if object is to be updated, then the oid has been set
			mustUpdate = nnoi.GetOid() != null;
			if (mustUpdate)
			{
				return objectWriter.UpdateNonNativeObjectInfo(nnoi, false);
			}
			return objectWriter.InsertNonNativeObject(oid, nnoi, true);
		}

		/// <summary>Warning,</summary>
		public override void DeleteObjectWithOid(NeoDatis.Odb.OID oid)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = null;
			NeoDatis.Odb.Core.Transaction.ISession lsession = GetSession(true);
			NeoDatis.Odb.Core.Transaction.ICache cache = lsession.GetCache();
			// Check if oih is in the cache
			oih = cache.GetObjectInfoHeaderFromOid(oid, false);
			if (oih == null)
			{
				oih = objectReader.ReadObjectInfoHeaderFromOid(oid, true);
			}
			object @object = null;
			if (NeoDatis.Odb.OdbConfiguration.ReconnectObjectsToSession())
			{
				NeoDatis.Odb.Impl.Core.Transaction.CacheFactory.GetCrossSessionCache(GetBaseIdentification
					().GetIdentification()).RemoveOid(oid);
			}
			objectWriter.Delete(oih);
			// removes the object from the cache
			cache.RemoveObjectWithOid(oih.GetOid());
		}

		/// <summary>Actually deletes an object database</summary>
		public override NeoDatis.Odb.OID Delete(object @object)
		{
			NeoDatis.Odb.Core.Transaction.ISession lsession = GetSession(true);
			if (lsession.IsRollbacked())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbHasBeenRollbacked
					.AddParameter(baseIdentification.ToString()));
			}
			if (@object == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbCanNotDeleteNullObject
					);
			}
			NeoDatis.Odb.Core.Transaction.ICache cache = lsession.GetCache();
			bool throwExceptionIfNotInCache = false;
			// Get header of the object (position, previous object position, next
			// object position and class info position)
			// Header must come from cache because it may have been updated before.
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader header = cache.GetObjectInfoHeaderFromObject
				(@object, throwExceptionIfNotInCache);
			if (header == null)
			{
				NeoDatis.Odb.OID cachedOid = cache.GetOid(@object, false);
				//reconnect object is turn on tries to get object from cross session
				if (cachedOid == null && NeoDatis.Odb.OdbConfiguration.ReconnectObjectsToSession(
					))
				{
					NeoDatis.Odb.Core.Transaction.ICrossSessionCache crossSessionCache = NeoDatis.Odb.Impl.Core.Transaction.CacheFactory
						.GetCrossSessionCache(GetBaseIdentification().GetIdentification());
					cachedOid = crossSessionCache.GetOid(@object);
				}
				if (cachedOid == null)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ObjectDoesNotExistInCacheForDelete
						.AddParameter(@object.GetType().FullName).AddParameter(@object.ToString()));
				}
				header = objectReader.ReadObjectInfoHeaderFromOid(cachedOid, false);
			}
			triggerManager.ManageDeleteTriggerBefore(@object.GetType().FullName, @object, header
				.GetOid());
			NeoDatis.Odb.OID oid = objectWriter.Delete(header);
			triggerManager.ManageDeleteTriggerAfter(@object.GetType().FullName, @object, oid);
			// removes the object from the cache
			cache.RemoveObjectWithOid(header.GetOid());
			if (NeoDatis.Odb.OdbConfiguration.ReconnectObjectsToSession())
			{
				NeoDatis.Odb.Impl.Core.Transaction.CacheFactory.GetCrossSessionCache(GetBaseIdentification
					().GetIdentification()).RemoveObject(@object);
			}
			return oid;
		}

		/// <summary>Returns a string of the meta-model</summary>
		/// <returns>The engine description</returns>
		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append(GetMetaModel().ToString());
			return buffer.ToString();
		}

		public override void Close()
		{
			if (isClosed)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbIsClosed
					.AddParameter(baseIdentification.GetIdentification()));
			}
			// When not local (client server) session can be null
			NeoDatis.Odb.Core.Transaction.ISession lsession = GetSession(isLocal);
			if (baseIdentification.CanWrite())
			{
				objectWriter.WriteLastODBCloseStatus(true, false);
			}
			objectWriter.Flush();
			if (isLocal && lsession.TransactionIsPending())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.TransactionIsPending
					.AddParameter(lsession.GetId()));
			}
			isClosed = true;
			objectReader.Close();
			objectWriter.Close();
			// Logically release this file (only for this virtual machine)
			NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.FileMutex.GetInstance().ReleaseFile(GetStorageDeviceName
				());
			if (lsession != null)
			{
				lsession.Close();
			}
			if (objectIntrospector != null)
			{
				objectIntrospector.Clear();
				objectIntrospector = null;
			}
			// remove trigger manager
			provider.RemoveLocalTriggerManager(this);
		}

		public override long Count(NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery query
			)
		{
			if (isClosed)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbIsClosed
					.AddParameter(baseIdentification.GetIdentification()));
			}
			NeoDatis.Odb.Core.Query.IValuesQuery q = new NeoDatis.Odb.Impl.Core.Query.Values.ValuesCriteriaQuery
				(query).Count("count");
			NeoDatis.Odb.Values values = GetValues(q, -1, -1);
			long count = (long)values.NextValues().GetByIndex(0);
			return count;
		}

		public override NeoDatis.Odb.Core.Layers.Layer3.IObjectReader GetObjectReader()
		{
			return objectReader;
		}

		public override NeoDatis.Odb.Core.Layers.Layer3.IObjectWriter GetObjectWriter()
		{
			return objectWriter;
		}

		public override void Commit()
		{
			if (IsClosed())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbIsClosed
					.AddParameter(baseIdentification.GetIdentification()));
			}
			GetSession(true).Commit();
			objectWriter.Flush();
		}

		public override void Rollback()
		{
			GetSession(true).Rollback();
		}

		public override NeoDatis.Odb.OID GetObjectId(object @object, bool throwExceptionIfDoesNotExist
			)
		{
			NeoDatis.Odb.OID oid = null;
			if (@object != null)
			{
				oid = GetSession(true).GetCache().GetOid(@object, false);
				// If cross cache session is on, just check if current object has the OID on the cache
				if (oid == null && NeoDatis.Odb.OdbConfiguration.ReconnectObjectsToSession())
				{
					NeoDatis.Odb.Core.Transaction.ICrossSessionCache cache = NeoDatis.Odb.Impl.Core.Transaction.CacheFactory
						.GetCrossSessionCache(GetBaseIdentification().GetIdentification());
					oid = cache.GetOid(@object);
					if (oid != null)
					{
						return oid;
					}
				}
				oid = GetSession(true).GetCache().GetOid(@object, false);
				if (oid == null && throwExceptionIfDoesNotExist)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.UnknownObjectToGetOid
						.AddParameter(@object.ToString()));
				}
				return oid;
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbCanNotReturnOidOfNullObject
				);
		}

		public override object GetObjectFromOid(NeoDatis.Odb.OID oid)
		{
			if (oid == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.CanNotGetObjectFromNullOid
					);
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = GetObjectReader()
				.ReadNonNativeObjectInfoFromOid(null, oid, true, true);
			if (nnoi.IsDeletedObject())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ObjectIsMarkedAsDeletedForOid
					.AddParameter(oid));
			}
			object o = nnoi.GetObject();
			if (o == null)
			{
				o = GetObjectReader().GetInstanceBuilder().BuildOneInstance(nnoi);
			}
			NeoDatis.Odb.Core.Transaction.ISession lsession = GetSession(true);
			// Here oid can be different from nnoi.getOid(). This is the case when
			// the oid is an external oid. That`s why we use
			// nnoi.getOid() to put in the cache
			lsession.GetCache().AddObject(nnoi.GetOid(), o, nnoi.GetHeader());
			lsession.GetTmpCache().ClearObjectInfos();
			return o;
		}

		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo GetMetaObjectFromOid
			(NeoDatis.Odb.OID oid)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = GetObjectReader()
				.ReadNonNativeObjectInfoFromOid(null, oid, true, false);
			GetSession(true).GetTmpCache().ClearObjectInfos();
			return nnoi;
		}

		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader GetObjectInfoHeaderFromOid
			(NeoDatis.Odb.OID oid)
		{
			return GetObjectReader().ReadObjectInfoHeaderFromOid(oid, true);
		}

		public override System.Collections.Generic.IList<long> GetAllObjectIds()
		{
			return objectReader.GetAllIds(NeoDatis.Odb.Core.Layers.Layer3.IDTypes.Object);
		}

		public override System.Collections.Generic.IList<NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.FullIDInfo
			> GetAllObjectIdInfos(string objectType, bool displayObjects)
		{
			return objectReader.GetAllIdInfos(objectType, NeoDatis.Odb.Core.Layers.Layer3.IDTypes
				.Object, displayObjects);
		}

		public override void SetVersion(int version)
		{
			this.version = version;
		}

		public override void SetDatabaseId(NeoDatis.Odb.DatabaseId databaseId)
		{
			this.databaseId = databaseId;
		}

		public override void SetNbClasses(long nbClasses)
		{
			this.nbClasses = nbClasses;
		}

		public override void SetLastODBCloseStatus(bool lastCloseStatus)
		{
			this.lastOdbCloseStatus = lastCloseStatus;
		}

		public override void SetCurrentIdBlockInfos(long currentBlockPosition, int currentBlockNumber
			, NeoDatis.Odb.OID maxId)
		{
			this.currentIdBlockPosition = currentBlockPosition;
			this.currentIdBlockNumber = currentBlockNumber;
			this.currentIdBlockMaxOid = maxId;
		}

		public override int GetCurrentIdBlockNumber()
		{
			return currentIdBlockNumber;
		}

		public override long GetCurrentIdBlockPosition()
		{
			return currentIdBlockPosition;
		}

		public override NeoDatis.Odb.DatabaseId GetDatabaseId()
		{
			return databaseId;
		}

		public override NeoDatis.Odb.OID GetCurrentIdBlockMaxOid()
		{
			return currentIdBlockMaxOid;
		}

		public override NeoDatis.Odb.OID GetMaxOid()
		{
			return objectWriter.GetIdManager().ConsultNextOid();
		}

		public override void SetMetaModel(NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel 
			metaModel2)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = null;
			NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel = new NeoDatis.Odb.Core.Layers.Layer2.Meta.SessionMetaModel
				();
			GetSession(true).SetMetaModel(metaModel);
			// Just add the classes
			System.Collections.IEnumerator iterator = metaModel2.GetAllClasses().GetEnumerator
				();
			while (iterator.MoveNext())
			{
				this.GetMetaModel().AddClass((NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo)iterator
					.Current);
			}
			// Now persists classes
			iterator = metaModel.GetAllClasses().GetEnumerator();
			int i = 0;
			while (iterator.MoveNext())
			{
				ci = (NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo)iterator.Current;
				if (ci.GetPosition() == -1)
				{
					objectWriter.PersistClass(ci, (i == 0 ? -2 : i - 1), false, false);
				}
				i++;
			}
		}

		public override bool IsClosed()
		{
			return isClosed;
		}

		public override int GetVersion()
		{
			return version;
		}

		public override NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification GetBaseIdentification
			()
		{
			return baseIdentification;
		}

		public override NeoDatis.Odb.OID WriteObjectInfo(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 aoi, long position, bool updatePointers)
		{
			// TODO check if it must be written in transaction
			return objectWriter.WriteNonNativeObjectInfo(oid, aoi, position, updatePointers, 
				true);
		}

		public override NeoDatis.Odb.OID UpdateObject(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi, bool forceUpdate)
		{
			return objectWriter.UpdateNonNativeObjectInfo(nnoi, forceUpdate);
		}

		public override NeoDatis.Odb.Values GetValues(NeoDatis.Odb.Core.Query.IValuesQuery
			 query, int startIndex, int endIndex)
		{
			if (isClosed)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbIsClosed
					.AddParameter(baseIdentification.GetIdentification()));
			}
			return objectReader.GetValues(query, startIndex, endIndex);
		}

		public override void AddCommitListener(NeoDatis.Odb.Core.Layers.Layer3.ICommitListener
			 commitListener)
		{
			this.commitListeners.Add(commitListener);
		}

		public override NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer3.ICommitListener
			> GetCommitListeners()
		{
			return commitListeners;
		}

		public override NeoDatis.Odb.Core.Layers.Layer3.IRefactorManager GetRefactorManager
			()
		{
			return provider.GetRefactorManager(this);
		}

		public override void ResetCommitListeners()
		{
			commitListeners.Clear();
		}

		public override bool IsLocal()
		{
			return isLocal;
		}

		public override NeoDatis.Odb.TransactionId GetCurrentTransactionId()
		{
			return currentTransactionId;
		}

		public override void SetCurrentTransactionId(NeoDatis.Odb.TransactionId transactionId
			)
		{
			currentTransactionId = transactionId;
		}

		public override void Disconnect(object @object)
		{
			GetSession(true).RemoveObjectFromCache(@object);
			//remove from cross session cache
			if (NeoDatis.Odb.OdbConfiguration.ReconnectObjectsToSession())
			{
				NeoDatis.Odb.Impl.Core.Transaction.CacheFactory.GetCrossSessionCache(GetBaseIdentification
					().GetIdentification()).RemoveObject(@object);
			}
		}

		/// <summary>Reconnect an object to the current session.</summary>
		/// <remarks>
		/// Reconnect an object to the current session. It connects the object and
		/// all the dependent objects (Objects accessible from the object graph of the
		/// root object
		/// </remarks>
		public override void Reconnect(object @object)
		{
			if (@object == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ReconnectCanReconnectNullObject
					);
			}
			NeoDatis.Odb.Core.Transaction.ICrossSessionCache crossSessionCache = NeoDatis.Odb.Impl.Core.Transaction.CacheFactory
				.GetCrossSessionCache(GetBaseIdentification().GetIdentification());
			NeoDatis.Odb.OID oid = crossSessionCache.GetOid(@object);
			//in some situation the user can control the disconnect and reconnect
			//so before throws an exception test if in the current session 
			//there is the object on the cache
			if (oid == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.CrossSessionCacheNullOidForObject
					.AddParameter(@object));
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = objectReader.ReadObjectInfoHeaderFromOid
				(oid, false);
			GetSession(true).AddObjectToCache(oid, @object, oih);
			// Retrieve Dependent Objects
			NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.GetDependentObjectIntrospectingCallback
				 getObjectsCallback = new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.GetDependentObjectIntrospectingCallback
				();
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = GetSession(true).GetMetaModel
				().GetClassInfoFromId(oih.GetClassInfoId());
			objectIntrospector.GetMetaRepresentation(@object, ci, true, null, getObjectsCallback
				);
			System.Collections.Generic.ICollection<object> dependentObjects = getObjectsCallback
				.GetObjects();
			System.Collections.Generic.IEnumerator<object> iterator = dependentObjects.GetEnumerator
				();
			while (iterator.MoveNext())
			{
				object o = iterator.Current;
				if (o != null)
				{
					oid = crossSessionCache.GetOid(o);
					if (oid == null)
					{
						throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.CrossSessionCacheNullOidForObject
							.AddParameter(o));
					}
					oih = objectReader.ReadObjectInfoHeaderFromOid(oid, false);
					GetSession(true).AddObjectToCache(oid, o, oih);
				}
			}
		}

		public override NeoDatis.Odb.Core.Trigger.ITriggerManager GetTriggerManager()
		{
			return triggerManager;
		}

		public override void AddDeleteTriggerFor(string className, NeoDatis.Odb.Core.Trigger.DeleteTrigger
			 trigger)
		{
			triggerManager.AddDeleteTriggerFor(className, trigger);
		}

		public override void AddInsertTriggerFor(string className, NeoDatis.Odb.Core.Trigger.InsertTrigger
			 trigger)
		{
			triggerManager.AddInsertTriggerFor(className, trigger);
		}

		public override void AddSelectTriggerFor(string className, NeoDatis.Odb.Core.Trigger.SelectTrigger
			 trigger)
		{
			triggerManager.AddSelectTriggerFor(className, trigger);
		}

		public override void AddUpdateTriggerFor(string className, NeoDatis.Odb.Core.Trigger.UpdateTrigger
			 trigger)
		{
			triggerManager.AddUpdateTriggerFor(className, trigger);
		}

		public override NeoDatis.Odb.Core.Layers.Layer1.Introspector.IObjectIntrospector 
			GetObjectIntrospector()
		{
			return objectIntrospector;
		}

		public override NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery CriteriaQuery
			(System.Type clazz, NeoDatis.Odb.Core.Query.Criteria.ICriterion criterion)
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

		public override NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery CriteriaQuery
			(System.Type clazz)
		{
			NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery q = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(clazz);
			q.SetStorageEngine(this);
			return q;
		}

        public abstract override NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList AddClasses(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList
			 arg1);

        public abstract override NeoDatis.Odb.Core.Transaction.ISession BuildDefaultSession();

        public abstract override NeoDatis.Odb.Core.Layers.Layer1.Introspector.IObjectIntrospector 
			BuildObjectIntrospector();

        public abstract override NeoDatis.Odb.Core.Layers.Layer3.IObjectReader BuildObjectReader();

        public abstract override NeoDatis.Odb.Core.Layers.Layer3.IObjectWriter BuildObjectWriter();

        public abstract override NeoDatis.Odb.Core.Trigger.ITriggerManager BuildTriggerManager();
	}
}
