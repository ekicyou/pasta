namespace NeoDatis.Odb.Core
{
	/// <summary>This is the default Core Object Provider.</summary>
	/// <remarks>This is the default Core Object Provider.</remarks>
	/// <author>olivier</author>
	public interface ICoreProvider : NeoDatis.Odb.Core.ITwoPhaseInit
	{
		NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine GetClientStorageEngine(NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification
			 baseIdentification);

		NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.IServerStorageEngine GetServerStorageEngine
			(NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification baseIdentification);

		NeoDatis.Odb.Core.Layers.Layer3.Engine.IByteArrayConverter GetByteArrayConverter(
			);

		/// <summary>
		/// TODO Return a list of IO to enable replication or other IO mechanism
		/// Used by the FileSystemInterface to actual write/read byte to underlying storage
		/// </summary>
		/// <param name="name">The name of the buffered io</param>
		/// <param name="parameters">The parameters that define the buffer</param>
		/// <param name="bufferSize">The size of the buffers</param>
		/// <returns>The buffer implementation</returns>
		/// <></>
		NeoDatis.Odb.Core.Layers.Layer3.IBufferedIO GetIO(string name, NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification
			 parameters, int bufferSize);

		/// <summary>Returns the Local Instance Builder</summary>
		NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder GetLocalInstanceBuilder
			(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine);

		NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder GetServerInstanceBuilder
			(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine);

		NeoDatis.Odb.Core.Layers.Layer1.Introspector.IObjectIntrospector GetLocalObjectIntrospector
			(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine);

		NeoDatis.Odb.Core.Server.Layers.Layer1.IClientObjectIntrospector GetClientObjectIntrospector
			(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine, string connectionId);

		NeoDatis.Odb.Core.Layers.Layer1.Introspector.IObjectIntrospector GetServerObjectIntrospector
			(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine);

		NeoDatis.Odb.Core.Layers.Layer3.IObjectWriter GetClientObjectWriter(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine);

		NeoDatis.Odb.Core.Layers.Layer3.IObjectReader GetClientObjectReader(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine);

		NeoDatis.Odb.Core.Trigger.ITriggerManager GetLocalTriggerManager(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine);

		NeoDatis.Odb.Core.Trigger.ITriggerManager GetServerTriggerManager(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine);

		NeoDatis.Odb.Core.Layers.Layer1.Introspector.IClassIntrospector GetClassIntrospector
			();

		NeoDatis.Odb.Core.Layers.Layer3.IIdManager GetClientIdManager(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine);

		NeoDatis.Odb.Core.Layers.Layer3.IIdManager GetServerIdManager(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine);

		NeoDatis.Odb.Core.Layers.Layer3.IObjectWriter GetServerObjectWriter(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine);

		NeoDatis.Odb.Core.Layers.Layer3.IObjectReader GetServerObjectReader(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine);

		// Transaction
		NeoDatis.Odb.Core.Server.Transaction.ISessionManager GetClientServerSessionManager
			();

		NeoDatis.Odb.Core.Transaction.ITransaction GetTransaction(NeoDatis.Odb.Core.Transaction.ISession
			 session, NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsi);

		NeoDatis.Odb.Core.Transaction.IWriteAction GetWriteAction(long position, byte[] bytes
			);

		NeoDatis.Odb.Core.Transaction.ISession GetLocalSession(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine);

		NeoDatis.Odb.Core.Transaction.ISession GetClientSession(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine);

		NeoDatis.Odb.Core.Transaction.ISession GetServerSession(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine, string sessionId);

		NeoDatis.Odb.Core.Layers.Layer3.IRefactorManager GetRefactorManager(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 engine);

		// For query result handler
		/// <summary>Returns the query result handler for normal query result (that return a collection of objects)
		/// 	</summary>
		NeoDatis.Odb.Core.Query.Execution.IMatchingObjectAction GetCollectionQueryResultAction
			(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine, NeoDatis.Odb.Core.Query.IQuery
			 query, bool inMemory, bool returnObjects);

		// OIDs
		NeoDatis.Odb.OID GetObjectOID(long objectOid, long classOid);

		NeoDatis.Odb.OID GetClassOID(long oid);

		NeoDatis.Odb.OID GetExternalObjectOID(long objectOid, long classOid);

		NeoDatis.Odb.OID GetExternalClassOID(long oid);

		NeoDatis.Odb.Core.Layers.Layer2.Instance.IClassPool GetClassPool();

		void ResetClassDefinitions();

		/// <summary>To retrieve the message streamer.</summary>
		/// <remarks>To retrieve the message streamer. used for client server communication</remarks>
		/// <param name="socket"></param>
		/// <returns></returns>
		/// <exception cref="System.IO.IOException"></exception>
		NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.IMessageStreamer GetMessageStreamer
			(System.Net.Sockets.TcpClient socket);

		NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.IMessageStreamer GetMessageStreamer
			(string host, int port, string name);

		void RemoveLocalTriggerManager(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine
			);
	}
}
