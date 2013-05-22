namespace NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Engine
{
	public class ClientStorageEngine : NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineAdapter
	{
		public static readonly string LogId = "ClientStorageEngine";

		private System.Net.Sockets.TcpClient socket;

		public static int nbcalls = 0;

		public static int nbdiffcalls = 0;

		protected NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.IMessageStreamer messageStreamer;

		protected string connectionId;

		private NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel localMetaModel;

		private NeoDatis.Odb.Core.Server.Layers.Layer1.IClientObjectIntrospector objectIntrospector;

		private NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder instanceBuilder;

		private NeoDatis.Odb.Core.Layers.Layer1.Introspector.IClassIntrospector classIntrospector;

		private NeoDatis.Odb.Core.Transaction.ISession session;

		private bool isRollbacked;

		protected NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter parameters;

		protected NeoDatis.Odb.Core.ICoreProvider provider;

		/// <summary>
		/// This is a visitor used to execute some specific action(like calling
		/// 'Before Insert Trigger') when introspecting an object
		/// </summary>
		protected NeoDatis.Odb.Core.Layers.Layer1.Introspector.IIntrospectionCallback introspectionCallbackForInsert;

		/// <summary>
		/// This is a visitor used to execute some specific action when introspecting
		/// an object
		/// </summary>
		protected NeoDatis.Odb.Core.Layers.Layer1.Introspector.IIntrospectionCallback introspectionCallbackForUpdate;

		protected ClientStorageEngine(string hostName, int port, string baseId) : this(new 
			NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter(hostName, port, baseId, NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter
			.TypeDatabase, null, null))
		{
		}

		protected ClientStorageEngine(string hostName, int port, string baseId, string user
			, string password) : this(new NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter(
			hostName, port, baseId, NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter.TypeDatabase
			, user, password))
		{
		}

		public ClientStorageEngine(NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter parameters
			)
		{
			Init(parameters);
		}

		public override NeoDatis.Odb.Core.Transaction.ISession BuildDefaultSession()
		{
			session = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClientSession(this);
			return session;
		}

		private void Init(NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter parameter)
		{
			this.provider = NeoDatis.Odb.OdbConfiguration.GetCoreProvider();
			this.parameters = parameter;
			NeoDatis.Odb.Core.ICoreProvider provider = NeoDatis.Odb.OdbConfiguration.GetCoreProvider
				();
			// Class Introspector must be built before as it is used by the
			// initODBConnection
			classIntrospector = provider.GetClassIntrospector();
			BuildDefaultSession();
			InitODBConnection();
			provider.GetClientServerSessionManager().AddSession(session);
			objectIntrospector = (NeoDatis.Odb.Core.Server.Layers.Layer1.IClientObjectIntrospector
				)BuildObjectIntrospector();
			instanceBuilder = provider.GetLocalInstanceBuilder(this);
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("ODBRemote:Connected to " + parameters.GetDestinationHost
					() + ":" + parameters.GetPort() + " - connection id=" + connectionId);
			}
			triggerManager = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetLocalTriggerManager
				(this);
			this.introspectionCallbackForInsert = new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrumentationCallbackForStore
				(this, triggerManager, false);
			this.introspectionCallbackForUpdate = new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.DefaultInstrumentationCallbackForStore
				(this, triggerManager, true);
		}

		protected virtual void InitMessageStreamer()
		{
			this.messageStreamer = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetMessageStreamer
				(parameters.GetDestinationHost(), parameters.GetPort(), parameters.GetIdentification
				());
		}

		protected virtual void InitODBConnection()
		{
			string localhost = null;
			localhost = NeoDatis.Tool.Wrappers.Net.NeoDatisIpAddress.Get("localhost");
			InitMessageStreamer();
			NeoDatis.Odb.Core.Server.Message.ConnectMessage msg = new NeoDatis.Odb.Core.Server.Message.ConnectMessage
				(parameters.GetBaseIdentifier(), localhost, parameters.GetUserName(), parameters
				.GetPassword());
			NeoDatis.Odb.Core.Server.Message.ConnectMessageResponse rmsg = (NeoDatis.Odb.Core.Server.Message.ConnectMessageResponse
				)SendMessage(msg);
			if (rmsg.HasError())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ServerSideError
					.AddParameter("Error while getting a connection from ODB Server").AddParameter(rmsg
					.GetError()));
			}
			// The client server conenction id is the server session id.
			connectionId = rmsg.GetConnectionId();
			session.SetMetaModel(rmsg.GetMetaModel());
			session.SetId(connectionId);
			SetDatabaseId(rmsg.GetTransactionId().GetDatabaseId());
			SetCurrentTransactionId(rmsg.GetTransactionId());
			// Now we have to send back the meta model extracted from current java
			// classes to check the compatibility
			if (NeoDatis.Odb.OdbConfiguration.CheckModelCompatibility())
			{
				// retrieve the current version of meta-model, based on the client
				// classes
				System.Collections.Generic.IDictionary<string, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
					> currentCIs = classIntrospector.Instrospect(rmsg.GetMetaModel().GetAllClasses()
					);
				// Creates the message to be sent
				NeoDatis.Odb.Core.Server.Message.CheckMetaModelCompatibilityMessage compatibilityMessage
					 = new NeoDatis.Odb.Core.Server.Message.CheckMetaModelCompatibilityMessage(parameters
					.GetBaseIdentifier(), rmsg.GetConnectionId(), currentCIs);
				NeoDatis.Odb.Core.Server.Message.CheckMetaModelCompatibilityMessageResponse rmsg2
					 = (NeoDatis.Odb.Core.Server.Message.CheckMetaModelCompatibilityMessageResponse)
					SendMessage(compatibilityMessage);
				if (rmsg2.GetResult().IsModelHasBeenUpdated())
				{
					NeoDatis.Tool.DLogger.Info("Meta-model has changed:");
					NeoDatis.Tool.DLogger.Info(rmsg2.GetResult().GetResults());
					// Meta model has been updated
					session.SetMetaModel(rmsg2.GetUpdatedMetaModel());
				}
			}
		}

		/// <summary>Opens socket send message and close.</summary>
		/// <remarks>Opens socket send message and close.</remarks>
		/// <TODO>This is bad,should keep the socket alive..</TODO>
		/// <param name="msg"></param>
		/// <returns>The response message</returns>
		public virtual NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message SendMessage(
			NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message msg)
		{
			messageStreamer.Write(msg);
			NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message rmsg;
			try
			{
				rmsg = messageStreamer.Read();
			}
			catch (System.Exception e)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ClientNetError
					.AddParameter(NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(e, true)));
			}
			return rmsg;
		}

		public override void Commit()
		{
			NeoDatis.Odb.Core.Server.Message.CommitMessage msg = new NeoDatis.Odb.Core.Server.Message.CommitMessage
				(parameters.GetBaseIdentifier(), connectionId);
			NeoDatis.Odb.Core.Server.Message.CommitMessageResponse rmsg = (NeoDatis.Odb.Core.Server.Message.CommitMessageResponse
				)SendMessage(msg);
			if (rmsg.HasError())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ServerSideError
					.AddParameter("Error while committing database").AddParameter(rmsg.GetError()));
			}
		}

		public override void Close()
		{
			if (isClosed)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbIsClosed
					.AddParameter(parameters.GetIdentification()));
			}
			NeoDatis.Odb.Core.Server.Message.CloseMessage msg = new NeoDatis.Odb.Core.Server.Message.CloseMessage
				(parameters.GetBaseIdentifier(), connectionId);
			NeoDatis.Odb.Core.Server.Message.CloseMessageResponse rmsg = (NeoDatis.Odb.Core.Server.Message.CloseMessageResponse
				)SendMessage(msg);
			if (rmsg.HasError())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ServerSideError
					.AddParameter("Error while closing database").AddParameter(rmsg.GetError()));
			}
			messageStreamer.Close();
			isClosed = true;
			provider.RemoveLocalTriggerManager(this);
		}

		public override void Rollback()
		{
			NeoDatis.Odb.Core.Server.Message.RollbackMessage msg = new NeoDatis.Odb.Core.Server.Message.RollbackMessage
				(parameters.GetBaseIdentifier(), connectionId);
			NeoDatis.Odb.Core.Server.Message.RollbackMessageResponse rmsg = (NeoDatis.Odb.Core.Server.Message.RollbackMessageResponse
				)SendMessage(msg);
			if (rmsg.HasError())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ServerSideError
					.AddParameter("Error while executing rollback").AddParameter(rmsg.GetError()));
			}
			isRollbacked = true;
		}

		public override NeoDatis.Odb.OID Store(object @object)
		{
			return Store(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant.NullObjectId
				, @object);
		}

		public override NeoDatis.Odb.OID Store(NeoDatis.Odb.OID oid, object @object)
		{
			if (isClosed)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbIsClosed
					.AddParameter(parameters.GetBaseIdentifier()));
			}
			NeoDatis.Odb.Core.Server.Layers.Layer2.Meta.ClientNonNativeObjectInfo nnoi = ObjectToMetaRepresentation
				(@object);
			if (nnoi.GetOid() == NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.NullObjectId)
			{
				nnoi.SetOid(oid);
			}
			NeoDatis.Odb.Core.Server.Message.StoreMessage msg = new NeoDatis.Odb.Core.Server.Message.StoreMessage
				(parameters.GetBaseIdentifier(), connectionId, nnoi, ConvertToOIDArray(objectIntrospector
				.GetClientOids()));
			NeoDatis.Odb.Core.Server.Message.StoreMessageResponse rmsg = (NeoDatis.Odb.Core.Server.Message.StoreMessageResponse
				)SendMessage(msg);
			if (rmsg.HasError())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ServerSideError
					.AddParameter("Error while storing object").AddParameter(rmsg.GetError()));
			}
			nnoi.SetOid(rmsg.GetOid());
			if (NeoDatis.Odb.OdbConfiguration.ReconnectObjectsToSession())
			{
				NeoDatis.Odb.Core.Transaction.ICrossSessionCache crossSessionCache = NeoDatis.Odb.Impl.Core.Transaction.CacheFactory
					.GetCrossSessionCache(GetBaseIdentification().GetIdentification());
				crossSessionCache.AddObject(@object, rmsg.GetOid());
			}
			// FIXME We should synchronize ids even in SameVM Mode
			// FIXME When byte code is on, oid and header should be set for all
			// objects, not only root one
			// Store the object in cache?
			session.GetCache().AddObject(rmsg.GetOid(), @object, nnoi.GetHeader());
			// if (!parameters.clientAndServerRunInSameVM()) {
			objectIntrospector.SynchronizeIds(rmsg.GetClientIds(), rmsg.GetServerIds());
			// }
			return rmsg.GetOid();
		}

		private NeoDatis.Odb.OID[] ConvertToOIDArray(NeoDatis.Tool.Wrappers.List.IOdbList
			<NeoDatis.Odb.OID> localOids)
		{
			NeoDatis.Odb.OID[] array = new NeoDatis.Odb.OID[localOids.Count];
			System.Collections.Generic.IEnumerator<NeoDatis.Odb.OID> iterator = localOids.GetEnumerator
				();
			int i = 0;
			while (iterator.MoveNext())
			{
				array[i++] = iterator.Current;
			}
			return array;
		}

		private NeoDatis.Odb.Core.Server.Layers.Layer2.Meta.ClientNonNativeObjectInfo ObjectToMetaRepresentation
			(object @object)
		{
			if (@object == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbCanNotStoreNullObject
					);
			}
			if (@object.GetType().IsArray)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbCanNotStoreArrayDirectly
					);
			}
			if (NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.IsNative(@object.GetType()))
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbCanNotStoreNativeObjectDirectly
					);
			}
			// The object must be transformed into meta representation
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci = null;
			string className = @object.GetType().FullName;
			// first checks if the class of this object already exist in the
			// metamodel
			if (session.GetMetaModel().ExistClass(className))
			{
				ci = session.GetMetaModel().GetClassInfo(className, true);
			}
			else
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList ciList = classIntrospector.Introspect
					(@object.GetType(), true);
				AddClasses(ciList);
				ci = ciList.GetMainClassInfo();
			}
			bool mustUpdate = false;
			NeoDatis.Odb.OID oid = GetSession(true).GetCache().GetOid(@object, false);
			if (oid != null)
			{
				mustUpdate = true;
			}
			// The introspection callback is used to execute some specific task
			// (like calling trigger, for example) while introspecting the object
			NeoDatis.Odb.Core.Layers.Layer1.Introspector.IIntrospectionCallback callback = introspectionCallbackForInsert;
			if (mustUpdate)
			{
				callback = introspectionCallbackForUpdate;
			}
			// Transform the object into a ObjectInfo
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)objectIntrospector.GetMetaRepresentation(@object, ci, true, null, callback);
			if (mustUpdate)
			{
				nnoi.SetOid(oid);
			}
			return (NeoDatis.Odb.Core.Server.Layers.Layer2.Meta.ClientNonNativeObjectInfo)nnoi;
		}

		/// <summary>
		/// TODO Remove comment public ClassInfo addClass(ClassInfo newClassInfo,
		/// boolean addDependentClasses) { ClassInfoList ciList = new
		/// ClassInfoList(newClassInfo); ciList = addClasses(ciList); return
		/// session.getMetaModel
		/// ().getClassInfo(newClassInfo.getFullClassName(),true); }
		/// </summary>
		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList AddClasses(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList
			 classInfoList)
		{
			// Call server to add the class info list to the meta model on the
			// server and retrieve class info id from server
			NeoDatis.Odb.Core.Server.Message.NewClassInfoListMessage msg = new NeoDatis.Odb.Core.Server.Message.NewClassInfoListMessage
				(parameters.GetBaseIdentifier(), connectionId, classInfoList);
			NeoDatis.Odb.Core.Server.Message.NewClassInfoListMessageResponse rmsg = (NeoDatis.Odb.Core.Server.Message.NewClassInfoListMessageResponse
				)SendMessage(msg);
			if (rmsg.HasError())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ServerSideError
					.AddParameter("Error while adding  new Class Info list").AddParameter(rmsg.GetError
					()));
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel = NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel
				.FromClassInfos(rmsg.GetClassInfos());
			session.SetMetaModel(metaModel);
			// Updates the main class info from new meta model
			string mainClassName = classInfoList.GetMainClassInfo().GetFullClassName();
			classInfoList.SetMainClassInfo(metaModel.GetClassInfo(mainClassName, true));
			return classInfoList;
		}

		public virtual NeoDatis.Odb.Objects<T> GetObjects<T>(System.Type clazz)
		{
			return GetObjects<T>(clazz, true);
		}

		public virtual NeoDatis.Odb.Objects<T> GetObjects<T>(System.Type clazz, bool inMemory
			)
		{
			return GetObjects<T>(clazz, inMemory, -1, -1);
		}

		public override NeoDatis.Odb.Objects<T> GetObjects<T>(System.Type clazz, bool inMemory
			, int startIndex, int endIndex)
		{
			NeoDatis.Odb.Core.Query.IQuery query = new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
				(clazz);
			return GetObjects<T>(query, inMemory, startIndex, endIndex);
		}

		private NeoDatis.Odb.Objects<T> BuildInstances<T>(NeoDatis.Odb.Objects<NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			> metaObjects)
		{
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Debug("Building instances for " + metaObjects.Count + " meta objects"
					);
			}
			// FIXME Do we need to create the btree collection?
			NeoDatis.Odb.Objects<T> list = new NeoDatis.Odb.Impl.Core.Query.List.Objects.InMemoryBTreeCollection
				<T>(metaObjects.Count, NeoDatis.Odb.Core.OrderByConstants.OrderByAsc);
			System.Collections.Generic.IEnumerator<NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				> iterator = metaObjects.GetEnumerator();
			T o = default(T);
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = null;
			int i = 0;
			while (iterator.MoveNext())
			{
				nnoi = iterator.Current;
				object obj = instanceBuilder.BuildOneInstance(nnoi);
				o = (T)obj;
				list.Add(o);
				i++;
			}
			return list;
		}

		private object BuildOneInstance(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi)
		{
			object o = instanceBuilder.BuildOneInstance(nnoi);
			return o;
		}

		public override long Count(NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery query
			)
		{
			if (isClosed)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbIsClosed
					.AddParameter(parameters.GetBaseIdentifier()));
			}
			NeoDatis.Odb.Core.Server.Message.CountMessage msg = new NeoDatis.Odb.Core.Server.Message.CountMessage
				(parameters.GetBaseIdentifier(), connectionId, query);
			NeoDatis.Odb.Core.Server.Message.CountMessageResponse rmsg = (NeoDatis.Odb.Core.Server.Message.CountMessageResponse
				)SendMessage(msg);
			if (rmsg.HasError())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ServerSideError
					.AddParameter("Error while counting objects").AddParameter(rmsg.GetError()));
			}
			return rmsg.GetNbObjects();
		}

		public override NeoDatis.Odb.OID Delete(object @object)
		{
			if (isClosed)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbIsClosed
					.AddParameter(parameters.GetBaseIdentifier()));
			}
			if (@object == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbCanNotDeleteNullObject
					);
			}
			NeoDatis.Odb.OID oid = GetObjectId(@object, false);
			if (oid == null && NeoDatis.Odb.OdbConfiguration.ReconnectObjectsToSession())
			{
				oid = NeoDatis.Odb.Impl.Core.Transaction.CacheFactory.GetCrossSessionCache(GetBaseIdentification
					().GetIdentification()).GetOid(@object);
			}
			if (oid == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ObjectDoesNotExistInCacheForDelete
					.AddParameter(@object.GetType().FullName).AddParameter(@object));
			}
			InternalDeleteObjectWithOid(oid);
			if (NeoDatis.Odb.OdbConfiguration.ReconnectObjectsToSession())
			{
				NeoDatis.Odb.Impl.Core.Transaction.CacheFactory.GetCrossSessionCache(this.GetBaseIdentification
					().GetIdentification()).RemoveObject(@object);
			}
			return oid;
		}

		/// <summary>Delete an object from the database with the id</summary>
		/// <param name="oid">The object id to be deleted @</param>
		public override void DeleteObjectWithOid(NeoDatis.Odb.OID oid)
		{
			if (isClosed)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbIsClosed
					.AddParameter(parameters.GetBaseIdentifier()));
			}
			InternalDeleteObjectWithOid(oid);
			if (NeoDatis.Odb.OdbConfiguration.ReconnectObjectsToSession())
			{
				NeoDatis.Odb.Impl.Core.Transaction.CacheFactory.GetCrossSessionCache(this.GetBaseIdentification
					().GetIdentification()).RemoveOid(oid);
			}
		}

		/// <summary>Delete an object from the database with the id</summary>
		/// <param name="oid">The object id to be deleted @</param>
		public virtual void InternalDeleteObjectWithOid(NeoDatis.Odb.OID oid)
		{
			if (isClosed)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbIsClosed
					.AddParameter(parameters.GetBaseIdentifier()));
			}
			NeoDatis.Odb.Core.Server.Message.DeleteObjectMessage msg = new NeoDatis.Odb.Core.Server.Message.DeleteObjectMessage
				(parameters.GetBaseIdentifier(), connectionId, oid);
			NeoDatis.Odb.Core.Server.Message.DeleteObjectMessageResponse rmsg = (NeoDatis.Odb.Core.Server.Message.DeleteObjectMessageResponse
				)SendMessage(msg);
			if (rmsg.HasError())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ServerSideError
					.AddParameter("Error while deleting objects").AddParameter(rmsg.GetError()));
			}
			NeoDatis.Odb.Core.Transaction.ICache cache = GetSession(true).GetCache();
			cache.MarkIdAsDeleted(oid);
			cache.RemoveObjectWithOid(oid);
		}

		public virtual NeoDatis.Odb.Objects<T> GetObjects<T>(NeoDatis.Odb.Core.Query.IQuery
			 query)
		{
			return GetObjects<T>(query, true, -1, -1);
		}

		public virtual NeoDatis.Odb.Objects<T> GetObjects<T>(NeoDatis.Odb.Core.Query.IQuery
			 query, bool inMemory)
		{
			return GetObjects<T>(query, inMemory, -1, -1);
		}

		public override NeoDatis.Odb.Objects<T> GetObjects<T>(NeoDatis.Odb.Core.Query.IQuery
			 query, bool inMemory, int startIndex, int endIndex)
		{
			if (isRollbacked)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbHasBeenRollbacked
					);
			}
			if (isClosed)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbIsClosed
					.AddParameter(parameters.GetBaseIdentifier()));
			}
			NeoDatis.Odb.Core.Server.Message.GetMessage msg = new NeoDatis.Odb.Core.Server.Message.GetMessage
				(parameters.GetBaseIdentifier(), connectionId, query, inMemory, startIndex, endIndex
				);
			NeoDatis.Odb.Core.Server.Message.GetMessageResponse rmsg = (NeoDatis.Odb.Core.Server.Message.GetMessageResponse
				)SendMessage(msg);
			if (rmsg.HasError())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ServerSideError
					.AddParameter("Error while getting objects").AddParameter(rmsg.GetError()));
			}
			// Sets execution plan
			query.SetExecutionPlan(rmsg.GetPlan());
			return (NeoDatis.Odb.Objects<T>)BuildInstances<T>(rmsg.GetMetaObjects());
		}

		public override NeoDatis.Odb.Values GetValues(NeoDatis.Odb.Core.Query.IValuesQuery
			 query, int startIndex, int endIndex)
		{
			if (isRollbacked)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbHasBeenRollbacked
					);
			}
			if (isClosed)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbIsClosed
					.AddParameter(parameters.GetBaseIdentifier()));
			}
			NeoDatis.Odb.Core.Server.Message.GetObjectValuesMessage msg = new NeoDatis.Odb.Core.Server.Message.GetObjectValuesMessage
				(parameters.GetBaseIdentifier(), connectionId, query, startIndex, endIndex);
			object o = SendMessage(msg);
			NeoDatis.Odb.Core.Server.Message.GetObjectValuesMessageResponse rmsg = (NeoDatis.Odb.Core.Server.Message.GetObjectValuesMessageResponse
				)o;
			if (rmsg.HasError())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ServerSideError
					.AddParameter("Error while getting object values").AddParameter(rmsg.GetError())
					);
			}
			NeoDatis.Odb.Values values = rmsg.GetValues();
			// Sets execution plan
			query.SetExecutionPlan(rmsg.GetPlan());
			// When object values API is used, Lazy list have to get a reference to
			// the client storage engine
			// This reference is obtained via lookup
			NeoDatis.Odb.Core.Lookup.LookupFactory.Get(GetSession(true).GetId()).Set(NeoDatis.Odb.Impl.Core.Lookup.Lookups
				.InstanceBuilder, instanceBuilder);
			return values;
		}

		public override NeoDatis.Odb.OID GetObjectId(object @object, bool throwExceptionIfDoesNotExist
			)
		{
			if (@object == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbCanNotReturnOidOfNullObject
					);
			}
			// If byte code instrumentation is on, just check if current object has
			// the OID
			if (NeoDatis.Odb.OdbConfiguration.ReconnectObjectsToSession())
			{
				NeoDatis.Odb.Core.Transaction.ICrossSessionCache crossSessionCache = NeoDatis.Odb.Impl.Core.Transaction.CacheFactory
					.GetCrossSessionCache(GetBaseIdentification().GetIdentification());
				if (crossSessionCache.GetOid(@object) != null)
				{
					return crossSessionCache.GetOid(@object);
				}
			}
			NeoDatis.Odb.OID oid = GetSession(true).GetCache().GetOid(@object, false);
			if (oid == null && throwExceptionIfDoesNotExist)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.UnknownObjectToGetOid
					.AddParameter(@object.ToString()));
			}
			return oid;
		}

		public override object GetObjectFromOid(NeoDatis.Odb.OID oid)
		{
			if (oid == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.CanNotGetObjectFromNullOid
					);
			}
			NeoDatis.Odb.Core.Server.Message.GetObjectFromIdMessage message = new NeoDatis.Odb.Core.Server.Message.GetObjectFromIdMessage
				(parameters.GetBaseIdentifier(), connectionId, oid);
			NeoDatis.Odb.Core.Server.Message.GetObjectFromIdMessageResponse rmsg = (NeoDatis.Odb.Core.Server.Message.GetObjectFromIdMessageResponse
				)SendMessage(message);
			if (rmsg.HasError())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ClientServerError
					.AddParameter("Error while getting object from id :" + rmsg.GetError()));
			}
			return BuildOneInstance(rmsg.GetMetaRepresentation());
		}

		/// <summary>FIXME : not very efficient because it retrieves the full object</summary>
		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo GetMetaObjectFromOid
			(NeoDatis.Odb.OID oid)
		{
			NeoDatis.Odb.Core.Server.Message.GetObjectFromIdMessage message = new NeoDatis.Odb.Core.Server.Message.GetObjectFromIdMessage
				(parameters.GetBaseIdentifier(), connectionId, oid);
			NeoDatis.Odb.Core.Server.Message.GetObjectFromIdMessageResponse rmsg = (NeoDatis.Odb.Core.Server.Message.GetObjectFromIdMessageResponse
				)SendMessage(message);
			if (rmsg.HasError())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ClientServerError
					.AddParameter("Error while getting object from id :" + rmsg.GetError()));
			}
			return rmsg.GetMetaRepresentation();
		}

		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader GetObjectInfoHeaderFromOid
			(NeoDatis.Odb.OID oid)
		{
			NeoDatis.Odb.Core.Server.Message.GetObjectHeaderFromIdMessage message = new NeoDatis.Odb.Core.Server.Message.GetObjectHeaderFromIdMessage
				(parameters.GetBaseIdentifier(), connectionId, oid);
			NeoDatis.Odb.Core.Server.Message.GetObjectHeaderFromIdMessageResponse rmsg = (NeoDatis.Odb.Core.Server.Message.GetObjectHeaderFromIdMessageResponse
				)SendMessage(message);
			if (rmsg.HasError())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ClientServerError
					.AddParameter("Error while getting object header from id :" + rmsg.GetError()));
			}
			return rmsg.GetObjectInfoHeader();
		}

		public override void DefragmentTo(string newFileName)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NotYetImplemented
				.AddParameter("ClientStorageEngine.defragmentTo"));
		}

		public virtual NeoDatis.Odb.ClassRepresentation GetClassRepresentation(System.Type
			 clazz)
		{
			string fullClassName = clazz.FullName;
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo classInfo = session.GetMetaModel()
				.GetClassInfo(fullClassName, false);
			if (classInfo == null)
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList ciList = classIntrospector.Introspect
					(clazz, false);
				classInfo = ciList.GetMainClassInfo();
			}
			return new NeoDatis.Odb.Impl.Main.DefaultClassRepresentation(null, classInfo);
		}

		/// <summary>or shutdown hook</summary>
		public virtual void Run()
		{
			if (!isClosed)
			{
				NeoDatis.Tool.DLogger.Debug("ODBFactory has not been closed and VM is exiting : force ODBFactory close"
					);
				Close();
			}
		}

		public virtual void AddUpdateTrigger(NeoDatis.Odb.Core.Trigger.UpdateTrigger trigger
			)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NotYetImplemented
				.AddParameter("ClientStorageEngine.addUpdateTrigger"));
		}

		public virtual void AddInsertTrigger(NeoDatis.Odb.Core.Trigger.InsertTrigger trigger
			)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NotYetImplemented
				.AddParameter("ClientStorageEngine.addInsertTrigger"));
		}

		public virtual void AddDeleteTrigger(NeoDatis.Odb.Core.Trigger.DeleteTrigger trigger
			)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NotYetImplemented
				.AddParameter("ClientStorageEngine.addDeleteTrigger"));
		}

		public virtual void AddSelectTrigger(NeoDatis.Odb.Core.Trigger.SelectTrigger trigger
			)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NotYetImplemented
				.AddParameter("ClientStorageEngine.addSelectTrigger"));
		}

		public override NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification GetBaseIdentification
			()
		{
			return parameters;
		}

		public override NeoDatis.Odb.Objects<T> GetObjectInfos<T>(NeoDatis.Odb.Core.Query.IQuery
			 query, bool inMemory, int startIndex, int endIndex, bool returnOjects)
		{
			NeoDatis.Odb.Core.Server.Message.GetMessage msg = new NeoDatis.Odb.Core.Server.Message.GetMessage
				(parameters.GetBaseIdentifier(), connectionId, query, inMemory, startIndex, endIndex
				);
			NeoDatis.Odb.Core.Server.Message.GetMessageResponse rmsg = (NeoDatis.Odb.Core.Server.Message.GetMessageResponse
				)SendMessage(msg);
			if (rmsg.HasError())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ClientServerError
					.AddParameter("Error while getting objects :" + rmsg.GetError()));
			}
			return (NeoDatis.Odb.Objects<T>)rmsg.GetMetaObjects();
		}

		public virtual NeoDatis.Odb.Objects<T> GetObjectInfos<T>(string fullClassName, bool
			 inMemory, int startIndex, int endIndex, bool returnOjects)
		{
			return GetObjectInfos<T>(new NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery(fullClassName
				), inMemory, startIndex, endIndex, returnOjects);
		}

		public override NeoDatis.Odb.Core.Transaction.ISession GetSession(bool throwExceptionIfDoesNotExist
			)
		{
			return session;
		}

		public override NeoDatis.Odb.OID UpdateObject(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi, bool forceUpdate)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NotYetImplemented
				.AddParameter("updateObject from meta representation not implemented in ClientStorageEngine"
				));
		}

		public override NeoDatis.Odb.OID WriteObjectInfo(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 aoi, long position, bool updatePointers)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NotYetImplemented
				.AddParameter("writeObjectInfo from meta representation not implemented in ClientStorageEngine"
				));
		}

		public override void AddSession(NeoDatis.Odb.Core.Transaction.ISession session, bool
			 readMetamodel)
		{
			this.session = session;
		}

		public override void AddIndexOn(string className, string indexName, string[] indexFields
			, bool verbose, bool acceptMultipleValuesForSameKey)
		{
			NeoDatis.Odb.Core.Server.Message.AddIndexMessage message = new NeoDatis.Odb.Core.Server.Message.AddIndexMessage
				(parameters.GetBaseIdentifier(), connectionId, className, indexName, indexFields
				, acceptMultipleValuesForSameKey, verbose);
			NeoDatis.Odb.Core.Server.Message.AddIndexMessageResponse rmsg = (NeoDatis.Odb.Core.Server.Message.AddIndexMessageResponse
				)SendMessage(message);
			if (rmsg.HasError())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ClientServerError
					.AddParameter(indexName + ":" + rmsg.GetError()));
			}
		}

		public override void RebuildIndex(string className, string indexName, bool verbose
			)
		{
			NeoDatis.Odb.Core.Server.Message.RebuildIndexMessage message = new NeoDatis.Odb.Core.Server.Message.RebuildIndexMessage
				(parameters.GetBaseIdentifier(), connectionId, className, indexName, verbose);
			NeoDatis.Odb.Core.Server.Message.RebuildIndexMessageResponse rmsg = (NeoDatis.Odb.Core.Server.Message.RebuildIndexMessageResponse
				)SendMessage(message);
			if (rmsg.HasError())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ClientServerError
					.AddParameter(indexName + ":" + rmsg.GetError()));
			}
		}

		public override void DeleteIndex(string className, string indexName, bool verbose
			)
		{
			NeoDatis.Odb.Core.Server.Message.DeleteIndexMessage message = new NeoDatis.Odb.Core.Server.Message.DeleteIndexMessage
				(parameters.GetBaseIdentifier(), connectionId, className, indexName, verbose);
			NeoDatis.Odb.Core.Server.Message.DeleteIndexMessageResponse rmsg = (NeoDatis.Odb.Core.Server.Message.DeleteIndexMessageResponse
				)SendMessage(message);
			if (rmsg.HasError())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ClientServerError
					.AddParameter(indexName + ":" + rmsg.GetError()));
			}
		}

		public override void AddCommitListener(NeoDatis.Odb.Core.Layers.Layer3.ICommitListener
			 commitListener)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NotYetImplemented
				.AddParameter("ClientStorageEngine.addCommitListener"));
		}

		public override NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer3.ICommitListener
			> GetCommitListeners()
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NotYetImplemented
				.AddParameter("ClientStorageEngine.getCommitListeners"));
		}

		public override NeoDatis.Odb.Core.Layers.Layer3.IRefactorManager GetRefactorManager
			()
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NotYetImplemented
				.AddParameter("ClientStorageEngine.getRefactorManager"));
		}

		public override void ResetCommitListeners()
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NotYetImplemented
				.AddParameter("ClientStorageEngine.resetCommitListeners"));
		}

		public override bool IsLocal()
		{
			return false;
		}

		public override NeoDatis.Odb.Core.Trigger.ITriggerManager GetTriggerManager()
		{
			return triggerManager;
		}

		public override void Disconnect(object @object)
		{
			GetSession(true).RemoveObjectFromCache(@object);
			// remove from cross session cache
			if (NeoDatis.Odb.OdbConfiguration.ReconnectObjectsToSession())
			{
				NeoDatis.Odb.Impl.Core.Transaction.CacheFactory.GetCrossSessionCache(GetBaseIdentification
					().GetIdentification()).RemoveObject(@object);
			}
		}

		/// <summary>Reconnect an object to the current session.</summary>
		/// <remarks>
		/// Reconnect an object to the current session. It connects the object and
		/// all the dependent objects (Objects accessible from the object graph of
		/// the root object
		/// <pre>
		/// This code is duplicated here because we don't have ObjectReader on client side,
		/// so all needed object reader methods are implement in the ClientStorageEngine class
		/// </pre>
		/// </remarks>
		public virtual void Reconnect(object @object, NeoDatis.Odb.OID oid)
		{
			if (@object == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ReconnectCanReconnectNullObject
					);
			}
			if (!NeoDatis.Odb.OdbConfiguration.ReconnectObjectsToSession())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ReconnectOnlyWithByteCodeAgentConfigured
					);
			}
			NeoDatis.Odb.Core.Transaction.ICrossSessionCache crossSessionCache = NeoDatis.Odb.Impl.Core.Transaction.CacheFactory
				.GetCrossSessionCache(GetBaseIdentification().GetIdentification());
			NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = GetObjectInfoHeaderFromOid
				(oid);
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
					oih = GetObjectInfoHeaderFromOid(oid);
					GetSession(true).AddObjectToCache(oid, o, oih);
				}
			}
		}

		public override void AddDeleteTrigger(System.Type clazz, NeoDatis.Odb.Core.Trigger.DeleteTrigger
			 trigger)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NotYetImplemented
				.AddParameter("addDeleteTrigger not implemented in ClientStorageEngine"));
		}

		public override void AddInsertTrigger(System.Type clazz, NeoDatis.Odb.Core.Trigger.InsertTrigger
			 trigger)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NotYetImplemented
				.AddParameter("addInsertTrigger not implemented in ClientStorageEngine"));
		}

		public override void AddSelectTrigger(System.Type clazz, NeoDatis.Odb.Core.Trigger.SelectTrigger
			 trigger)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NotYetImplemented
				.AddParameter("addSelectTrigger not implemented in ClientStorageEngine"));
		}

		public override void AddUpdateTrigger(System.Type clazz, NeoDatis.Odb.Core.Trigger.UpdateTrigger
			 trigger)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NotYetImplemented
				.AddParameter("addUpdateTrigger not implemented in ClientStorageEngine"));
		}

		public override NeoDatis.Odb.Core.Layers.Layer1.Introspector.IObjectIntrospector 
			BuildObjectIntrospector()
		{
			return provider.GetClientObjectIntrospector(this, connectionId);
		}

		public override NeoDatis.Odb.Core.Layers.Layer3.IObjectReader BuildObjectReader()
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NotYetImplemented
				.AddParameter("buildObjectReader not implemented in ClientStorageEngine"));
		}

		public override NeoDatis.Odb.Core.Layers.Layer3.IObjectWriter BuildObjectWriter()
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NotYetImplemented
				.AddParameter("buildObjectWriter not implemented in ClientStorageEngine"));
		}

		public override NeoDatis.Odb.Core.Trigger.ITriggerManager BuildTriggerManager()
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NotYetImplemented
				.AddParameter("buildTriggerManager not implemented in ClientStorageEngine"));
		}

		public override NeoDatis.Odb.Core.Layers.Layer1.Introspector.IObjectIntrospector 
			GetObjectIntrospector()
		{
			return objectIntrospector;
		}
	}
}
