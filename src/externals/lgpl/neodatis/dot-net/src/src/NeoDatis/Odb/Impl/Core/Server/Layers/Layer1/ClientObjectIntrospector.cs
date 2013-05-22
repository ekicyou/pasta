namespace NeoDatis.Odb.Impl.Core.Server.Layers.Layer1
{
	/// <summary>Not thread safe</summary>
	/// <author>osmadja</author>
	public class ClientObjectIntrospector : NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector.LocalObjectIntrospector
		, NeoDatis.Odb.Core.Server.Layers.Layer1.IClientObjectIntrospector
	{
		/// <summary>client oids are sequential ids created by the client side engine.</summary>
		/// <remarks>
		/// client oids are sequential ids created by the client side engine. When an object is sent to server, server ids are sent back from server
		/// and client engine replace all local(client) oids by the server oids.
		/// </remarks>
		protected NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.OID> clientOids;

		/// <summary>A map of abstract object info, keys are local ids</summary>
		protected System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, NeoDatis.Odb.Core.Server.Layers.Layer2.Meta.ClientNonNativeObjectInfo
			> aois;

		protected System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, object> objects;

		protected NeoDatis.Odb.Core.Server.Transaction.ISessionManager sessionManager;

		/// <summary>This represents the connection to the server</summary>
		protected string connectionId;

		public ClientObjectIntrospector(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine
			, string connectionId) : base(storageEngine)
		{
			clientOids = new NeoDatis.Tool.Wrappers.List.OdbArrayList<NeoDatis.Odb.OID>();
			aois = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<NeoDatis.Odb.OID, NeoDatis.Odb.Core.Server.Layers.Layer2.Meta.ClientNonNativeObjectInfo
				>();
			objects = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<NeoDatis.Odb.OID, object>();
			sessionManager = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClientServerSessionManager
				();
			this.connectionId = connectionId;
		}

		public virtual NeoDatis.Odb.Core.Transaction.ISession GetSession()
		{
			return sessionManager.GetSession(storageEngine.GetBaseIdentification().GetIdentification
				(), true);
		}

		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo BuildNnoi
			(object o, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo info, NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			[] values, long[] attributesIdentification, int[] attributeIds, System.Collections.Generic.IDictionary
			<object, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo> alreadyReadObjects
			)
		{
			NeoDatis.Odb.Core.Server.Layers.Layer2.Meta.ClientNonNativeObjectInfo cnnoi = new 
				NeoDatis.Odb.Core.Server.Layers.Layer2.Meta.ClientNonNativeObjectInfo(null, info
				, values, attributesIdentification, attributeIds);
			cnnoi.SetLocalOid(NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(alreadyReadObjects
				.Count + 1));
			NeoDatis.Odb.OID id = cnnoi.GetLocalOid();
			NeoDatis.Odb.Core.Transaction.ICache cache = GetSession().GetCache();
			// Check if object is in the cache, if so sets its id
			NeoDatis.Odb.OID oid = cache.GetOid(o, false);
			if (oid != null)
			{
				cnnoi.SetOid(oid);
			}
			clientOids.Add(id);
			aois.Add(id, cnnoi);
			objects.Add(id, o);
			return cnnoi;
		}

		public virtual NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.OID> GetClientOids
			()
		{
			return clientOids;
		}

		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo GetMetaRepresentation
			(object @object, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo ci, bool recursive
			, System.Collections.Generic.IDictionary<object, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			> alreadyReadObjects, NeoDatis.Odb.Core.Layers.Layer1.Introspector.IIntrospectionCallback
			 callback)
		{
			clientOids.Clear();
			aois.Clear();
			objects.Clear();
			return base.GetObjectInfo(@object, ci, recursive, alreadyReadObjects, callback);
		}

		/// <summary>This method is used to make sure that client oids and server oids are equal.
		/// 	</summary>
		/// <remarks>
		/// This method is used to make sure that client oids and server oids are equal.
		/// <pre>
		/// When storing an object, the client side does nt know the oid that each object will receive. So the client create
		/// temporary (sequencial) oids. These oids are sent to the server in the object meta-representations. On the server side,
		/// real OIDs are created and associated to the objects and to the client side ids. After calling the store on the server side
		/// The client use the the synchronizeIds method to replace client ids by the right server side ids.
		/// </pre>
		/// </remarks>
		public virtual void SynchronizeIds(NeoDatis.Odb.OID[] clientIds, NeoDatis.Odb.OID
			[] serverIds)
		{
			if (clientIds.Length != clientOids.Count)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ClientServerSynchronizeIds
					.AddParameter(clientOids.Count).AddParameter(clientIds.Length));
			}
			NeoDatis.Odb.Core.Server.Layers.Layer2.Meta.ClientNonNativeObjectInfo cnnoi = null;
			NeoDatis.Odb.Core.Transaction.ICache cache = GetSession().GetCache();
			object @object = null;
			NeoDatis.Odb.OID id = null;
			NeoDatis.Odb.Core.Transaction.ICrossSessionCache crossSessionCache = NeoDatis.Odb.Impl.Core.Transaction.CacheFactory
				.GetCrossSessionCache(storageEngine.GetBaseIdentification().GetIdentification());
			for (int i = 0; i < clientIds.Length; i++)
			{
				id = clientIds[i];
				cnnoi = aois[id];
				@object = objects[id];
				// Server ids may be null when an object or part of an object has been updated.
				// In these case local objects have already the correct ids
				if (serverIds[i] != null)
				{
					cnnoi.SetOid(serverIds[i]);
					cache.AddObject(serverIds[i], @object, cnnoi.GetHeader());
				}
				// As serverIds may be null, we need to check it
				if (NeoDatis.Odb.OdbConfiguration.ReconnectObjectsToSession() && serverIds[i] != 
					null)
				{
					crossSessionCache.AddObject(@object, serverIds[i]);
				}
			}
		}
	}
}
