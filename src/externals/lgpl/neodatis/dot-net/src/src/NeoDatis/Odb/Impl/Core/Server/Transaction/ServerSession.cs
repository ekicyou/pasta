namespace NeoDatis.Odb.Impl.Core.Server.Transaction
{
	public class ServerSession : NeoDatis.Odb.Impl.Core.Transaction.LocalSession
	{
		/// <summary>client object ids</summary>
		protected NeoDatis.Odb.OID[] clientIds;

		/// <summary>server object ids.</summary>
		/// <remarks>
		/// server object ids. The server ids are sent to client as a result of a
		/// store operation to enable client to synchronize ids with server
		/// </remarks>
		protected NeoDatis.Odb.OID[] serverIds;

		/// <summary>To keep track of class info creation on server.</summary>
		/// <remarks>
		/// To keep track of class info creation on server. The ids of class info are
		/// then sent to client to update their ci ids
		/// </remarks>
		protected System.Collections.Generic.IDictionary<string, NeoDatis.Odb.OID> classInfoIds;

		protected NeoDatis.Odb.Core.Server.Transaction.ISessionManager sessionManager;

		public ServerSession(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine, string
			 sessionId) : base(engine, sessionId)
		{
			classInfoIds = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, NeoDatis.Odb.OID
				>();
			this.sessionManager = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClientServerSessionManager
				();
		}

		public virtual NeoDatis.Odb.OID[] GetClientIds()
		{
			return clientIds;
		}

		public virtual void SetClientIds(NeoDatis.Odb.OID[] clientIds)
		{
			this.clientIds = clientIds;
			this.serverIds = new NeoDatis.Odb.OID[clientIds.Length];
		}

		public virtual NeoDatis.Odb.OID[] GetServerIds()
		{
			return serverIds;
		}

		public virtual void SetServerIds(NeoDatis.Odb.OID[] serverIds)
		{
			this.serverIds = serverIds;
		}

		public virtual void AssociateIds(NeoDatis.Odb.OID serverId, NeoDatis.Odb.OID clientOid
			)
		{
			for (int i = 0; i < clientIds.Length; i++)
			{
				if (clientOid.CompareTo(clientIds[i]) == 0)
				{
					serverIds[i] = serverId;
					return;
				}
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ClientServerCanNotAssociateOids
				.AddParameter(serverId).AddParameter(clientOid));
		}

		public override NeoDatis.Odb.Core.Transaction.ICache BuildCache()
		{
			return new NeoDatis.Odb.Impl.Core.Transaction.ServerCache(this);
		}

		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel GetMetaModel()
		{
			if (metaModel == null)
			{
				try
				{
					metaModel = new NeoDatis.Odb.Core.Layers.Layer2.Meta.SessionMetaModel();
					if (GetStorageEngine().GetObjectReader() != null)
					{
						this.metaModel = GetStorageEngine().GetObjectReader().ReadMetaModel(metaModel, true
							);
					}
				}
				catch (System.Exception e)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InternalError
						.AddParameter("in ServerSession.getMetaModel"), e);
				}
			}
			return metaModel;
		}

		public virtual void SetClassInfoId(string fullClassName, NeoDatis.Odb.OID id)
		{
			classInfoIds.Add(fullClassName, id);
		}

		public virtual System.Collections.Generic.IDictionary<string, NeoDatis.Odb.OID> GetClassInfoIds
			()
		{
			return classInfoIds;
		}

		public virtual void ResetClassInfoIds()
		{
			classInfoIds = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, NeoDatis.Odb.OID
				>();
		}

		public override void Clear()
		{
			base.Clear();
		}

		~ServerSession()
		{
		}
		//sessionManager.removeSession(baseIdentification);
	}
}
