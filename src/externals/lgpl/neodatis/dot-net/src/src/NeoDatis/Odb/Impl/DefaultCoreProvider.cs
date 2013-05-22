using NeoDatis.Odb.Core.Layers.Layer3.Engine;
using NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine;
using NeoDatis.Odb.Core.Layers.Layer1.Introspector;
using NeoDatis.Tool.Wrappers.Map;
using NeoDatis.Odb.Core.Layers.Layer3;
using NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector;
using NeoDatis.Odb.Core.Trigger;
using NeoDatis.Odb.Core.Layers.Layer2.Instance;
using NeoDatis.Odb.Core.Server.Transaction;
using System.Collections.Generic;
using NeoDatis.Odb.Impl.Core.Trigger;
using NeoDatis.Odb.Impl.Core.Layers.Layer2.Instance;
using NeoDatis.Odb.Impl.Core.Server.Transaction;
namespace NeoDatis.Odb.Impl
{
	/// <summary>The is the default implementation of ODB</summary>
	/// <author>olivier</author>
	public class DefaultCoreProvider : NeoDatis.Odb.Core.ICoreProvider
	{
		private static IClassPool classPool = new ODBClassPool();

		private static IByteArrayConverter byteArrayConverter = new DefaultByteArrayConverter();

		private static IClassIntrospector classIntrospector = null;

		private static ISessionManager sessionManager = new SessionManager();

		private static IDictionary<IStorageEngine, ITriggerManager> triggerManagers = new OdbHashMap<IStorageEngine, ITriggerManager>();

		public virtual void Init2()
		{
			byteArrayConverter.Init2();
			if (OsIsAndroid())
			{
				// One feature is currently not supported on Android : dynamic empty
				// constructor creation
				classIntrospector = new AndroidClassIntrospector();
			}
			else
			{
				classIntrospector = new DefaultClassIntrospector();
			}
			classIntrospector.Init2();
			sessionManager.Init2();
		}

		private bool OsIsAndroid()
		{
			string javaVendor = NeoDatis.Tool.Wrappers.OdbSystem.GetProperty("java.vendor");
			if (javaVendor != null && javaVendor.Equals("The Android Project"))
			{
				return true;
			}
			return false;
		}

		public virtual void ResetClassDefinitions()
		{
			classIntrospector.Reset();
			classPool.Reset();
		}

		public virtual IStorageEngine GetClientStorageEngine(IBaseIdentification baseIdentification)
		{
			if (baseIdentification is NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter)
			{
				return new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalStorageEngine(baseIdentification
					);
			}
			if (baseIdentification is NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter)
			{
				NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter p = (NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter
					)baseIdentification;
				return new NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Engine.ClientStorageEngine
					(p);
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.UnsupportedIoType
				.AddParameter(baseIdentification.ToString()));
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer3.IObjectWriter GetClientObjectWriter
			(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine)
		{
			return new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalObjectWriter(engine);
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer3.IObjectReader GetClientObjectReader
			(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine)
		{
			return new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.ObjectReader(engine);
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer3.IObjectWriter GetServerObjectWriter
			(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine)
		{
			return new NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Engine.ServerObjectWriter(
				engine);
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer3.IObjectReader GetServerObjectReader
			(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine)
		{
			return new NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Engine.ServerObjectReader(
				engine);
		}

		public virtual NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.IServerStorageEngine
			 GetServerStorageEngine(NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification baseIdentification
			)
		{
			return new NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Engine.ServerStorageEngine
				(baseIdentification);
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer3.Engine.IByteArrayConverter GetByteArrayConverter
			()
		{
			return byteArrayConverter;
		}

		/// <summary>
		/// TODO Return a list of IO to enable replication or other IO mechanism Used
		/// by the FileSystemInterface to actual write/read byte to underlying
		/// storage
		/// </summary>
		/// <param name="name">The name of the buffered io</param>
		/// <param name="parameters">The parameters that define the buffer</param>
		/// <param name="bufferSize">The size of the buffers</param>
		/// <returns>The buffer implementation @</returns>
		public virtual NeoDatis.Odb.Core.Layers.Layer3.IBufferedIO GetIO(string name, NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification
			 parameters, int bufferSize)
		{
			if (parameters is NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter)
			{
				NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter fileParameters = (NeoDatis.Odb.Core.Layers.Layer3.IOFileParameter
					)parameters;
				// Guarantee that file directory structure exist
				NeoDatis.Tool.Wrappers.IO.OdbFile f = new NeoDatis.Tool.Wrappers.IO.OdbFile(fileParameters
					.GetFileName());
				NeoDatis.Tool.Wrappers.IO.OdbFile fparent = f.GetParentFile();
				if (fparent != null && !fparent.Exists())
				{
					fparent.Mkdirs();
				}
				return new NeoDatis.Odb.Impl.Core.Layers.Layer3.Buffer.MultiBufferedFileIO(NeoDatis.Odb.OdbConfiguration
					.GetNbBuffers(), name, fileParameters.GetFileName(), fileParameters.CanWrite(), 
					bufferSize);
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.UnsupportedIoType
				.AddParameter(parameters.ToString()));
		}

		/// <summary>Returns the Local Instance Builder</summary>
		public virtual NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder GetLocalInstanceBuilder
			(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine)
		{
			return new NeoDatis.Odb.Impl.Core.Layers.Layer2.Instance.LocalInstanceBuilder(engine
				);
		}

		/// <summary>Returns the Server Instance Builder</summary>
		public virtual NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder GetServerInstanceBuilder
			(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine)
		{
			return new NeoDatis.Odb.Impl.Core.Layers.Layer2.Instance.ServerInstanceBuilder(engine
				);
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer1.Introspector.IObjectIntrospector GetLocalObjectIntrospector
			(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine)
		{
			return new NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector
				(engine);
		}

		public virtual NeoDatis.Odb.Core.Server.Layers.Layer1.IClientObjectIntrospector GetClientObjectIntrospector
			(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine, string connectionId)
		{
			return new NeoDatis.Odb.Impl.Core.Server.Layers.Layer1.ClientObjectIntrospector(engine
				, connectionId);
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer1.Introspector.IObjectIntrospector GetServerObjectIntrospector
			(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine)
		{
			return new NeoDatis.Odb.Impl.Core.Server.Layers.Layer1.ServerObjectIntrospector(engine
				);
		}

		public virtual NeoDatis.Odb.Core.Trigger.ITriggerManager GetLocalTriggerManager(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine)
		{
			// First check if trigger manager has already been built for the engine
			ITriggerManager triggerManager = null;
            triggerManagers.TryGetValue(engine,out triggerManager);
			if (triggerManager != null)
			{
				return triggerManager;
			}
			triggerManager = new DefaultTriggerManager(engine);
			triggerManagers[engine] = triggerManager;
			return triggerManager;
		}

		public virtual void RemoveLocalTriggerManager(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine)
		{
			triggerManagers.Remove(engine);
		}

		public virtual NeoDatis.Odb.Core.Trigger.ITriggerManager GetServerTriggerManager(
			NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine)
		{
			return new NeoDatis.Odb.Impl.Core.Server.Trigger.DefaultServerTriggerManager(engine
				);
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer1.Introspector.IClassIntrospector GetClassIntrospector
			()
		{
			return classIntrospector;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer3.IIdManager GetClientIdManager(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine)
		{
			return new NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.DefaultIdManager(engine.GetObjectWriter
				(), engine.GetObjectReader(), engine.GetCurrentIdBlockPosition(), engine.GetCurrentIdBlockNumber
				(), engine.GetCurrentIdBlockMaxOid());
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer3.IIdManager GetServerIdManager(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine)
		{
			return new NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Oid.DefaultServerIdManager
				(engine.GetObjectWriter(), engine.GetObjectReader(), engine.GetCurrentIdBlockPosition
				(), engine.GetCurrentIdBlockNumber(), engine.GetCurrentIdBlockMaxOid());
		}

		// Transaction related
		public virtual NeoDatis.Odb.Core.Server.Transaction.ISessionManager GetClientServerSessionManager
			()
		{
			return sessionManager;
		}

		public virtual NeoDatis.Odb.Core.Transaction.IWriteAction GetWriteAction(long position
			, byte[] bytes)
		{
			return new NeoDatis.Odb.Impl.Core.Transaction.DefaultWriteAction(position, bytes);
		}

		public virtual NeoDatis.Odb.Core.Transaction.ITransaction GetTransaction(NeoDatis.Odb.Core.Transaction.ISession
			 session, NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsi)
		{
			return new NeoDatis.Odb.Impl.Core.Transaction.DefaultTransaction(session, fsi);
		}

		public virtual NeoDatis.Odb.Core.Transaction.ISession GetLocalSession(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine)
		{
			return new NeoDatis.Odb.Impl.Core.Transaction.LocalSession(engine);
		}

		public virtual NeoDatis.Odb.Core.Transaction.ISession GetClientSession(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine)
		{
			return new NeoDatis.Odb.Impl.Core.Transaction.ClientSession(engine);
		}

		public virtual NeoDatis.Odb.Core.Transaction.ISession GetServerSession(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine, string sessionId)
		{
			return new NeoDatis.Odb.Impl.Core.Server.Transaction.ServerSession(engine, sessionId
				);
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer3.IRefactorManager GetRefactorManager
			(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine)
		{
			return new NeoDatis.Odb.Impl.Core.Layers.Layer3.Refactor.DefaultRefactorManager(engine
				);
		}

		// For query result handler
		public virtual NeoDatis.Odb.Core.Query.Execution.IMatchingObjectAction GetCollectionQueryResultAction
			(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine, NeoDatis.Odb.Core.Query.IQuery
			 query, bool inMemory, bool returnObjects)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.CollectionQueryResultAction<object>(query
				, inMemory, engine, returnObjects, engine.GetObjectReader().GetInstanceBuilder()
				);
		}

		// OIDs
		public virtual NeoDatis.Odb.OID GetObjectOID(long objectOid, long classOid)
		{
			return new NeoDatis.Odb.Impl.Core.Oid.OdbObjectOID(objectOid);
		}

		public virtual NeoDatis.Odb.OID GetClassOID(long oid)
		{
			return new NeoDatis.Odb.Impl.Core.Oid.OdbClassOID(oid);
		}

		public virtual NeoDatis.Odb.OID GetExternalObjectOID(long objectOid, long classOid
			)
		{
			return new NeoDatis.Odb.Impl.Core.Oid.OdbObjectOID(objectOid);
		}

		public virtual NeoDatis.Odb.OID GetExternalClassOID(long oid)
		{
			return new NeoDatis.Odb.Impl.Core.Oid.OdbClassOID(oid);
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Instance.IClassPool GetClassPool()
		{
			return classPool;
		}

		/// <summary>(non-Javadoc)</summary>
		/// <seealso cref="NeoDatis.Odb.Core.ICoreProvider.GetMessageStreamer">NeoDatis.Odb.Core.ICoreProvider.GetMessageStreamer
		/// 	</seealso>
		public virtual NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.IMessageStreamer GetMessageStreamer
			(System.Net.Sockets.TcpClient socket)
		{
			return NeoDatis.Tool.Wrappers.IO.MessageStreamerBuilder.GetMessageStreamer(socket
				);
		}

		public virtual NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.IMessageStreamer GetMessageStreamer
			(string host, int port, string name)
		{
			return NeoDatis.Tool.Wrappers.IO.MessageStreamerBuilder.GetMessageStreamer(host, 
				port, name);
		}
	}
}
