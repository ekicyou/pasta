namespace NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Engine
{
	public class ServerObjectWriter : NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter
	{
		private NeoDatis.Odb.Core.Server.Transaction.ISessionManager sessionManager;

		public ServerObjectWriter(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine) : 
			base(engine)
		{
			this.sessionManager = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClientServerSessionManager
				();
		}

		internal virtual NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.IServerStorageEngine
			 GetEngine()
		{
			return (NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.IServerStorageEngine)storageEngine;
		}

		public virtual void InitIdManager()
		{
			this.idManager = new NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Oid.DefaultServerIdManager
				(this, objectReader, storageEngine.GetCurrentIdBlockPosition(), storageEngine.GetCurrentIdBlockNumber
				(), storageEngine.GetCurrentIdBlockMaxOid());
		}

		public override NeoDatis.Odb.OID WriteNonNativeObjectInfo(NeoDatis.Odb.OID existingOid
			, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo objectInfo, long position
			, bool writeDataInTransaction, bool isNewObject)
		{
			// To enable object auto-reconnect on the server side
			if (NeoDatis.Odb.OdbConfiguration.ReconnectObjectsToSession() && objectInfo.GetHeader
				().GetOid() != null)
			{
				NeoDatis.Odb.Impl.Core.Server.Transaction.ServerSession session = (NeoDatis.Odb.Impl.Core.Server.Transaction.ServerSession
					)sessionManager.GetSession(storageEngine.GetBaseIdentification().GetIdentification
					(), true);
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = session.GetCache().GetObjectInfoHeaderFromOid
					(objectInfo.GetOid(), false);
				// only add in th cache if object does not exist in the cache
				if (oih == null)
				{
					session.GetCache().AddObjectInfo(objectInfo.GetHeader());
				}
			}
			NeoDatis.Odb.OID roid = base.WriteNonNativeObjectInfo(existingOid, objectInfo, position
				, writeDataInTransaction, isNewObject);
			if (objectInfo is NeoDatis.Odb.Core.Server.Layers.Layer2.Meta.ClientNonNativeObjectInfo)
			{
				NeoDatis.Odb.Core.Server.Layers.Layer2.Meta.ClientNonNativeObjectInfo cnnoi = (NeoDatis.Odb.Core.Server.Layers.Layer2.Meta.ClientNonNativeObjectInfo
					)objectInfo;
				NeoDatis.Odb.Impl.Core.Server.Transaction.ServerSession session = (NeoDatis.Odb.Impl.Core.Server.Transaction.ServerSession
					)GetSession();
				session.AssociateIds(roid, cnnoi.GetLocalOid());
				// Adds the abstract Objectinfo in the cache
				session.GetCache().AddObjectInfo(cnnoi.GetHeader());
			}
			return roid;
		}

		public override NeoDatis.Odb.OID UpdateNonNativeObjectInfo(NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
			 nnoi, bool forceUpdate)
		{
			// To enable object auto-reconnect on the server side
			if (NeoDatis.Odb.OdbConfiguration.ReconnectObjectsToSession() && nnoi.GetHeader()
				.GetOid() != null)
			{
				NeoDatis.Odb.Impl.Core.Server.Transaction.ServerSession session = (NeoDatis.Odb.Impl.Core.Server.Transaction.ServerSession
					)sessionManager.GetSession(storageEngine.GetBaseIdentification().GetIdentification
					(), true);
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = session.GetCache().GetObjectInfoHeaderFromOid
					(nnoi.GetOid(), false);
				// only add in th cache if object does not exist in the cache
				if (oih == null)
				{
					session.GetCache().AddObjectInfo(nnoi.GetHeader());
				}
			}
			NeoDatis.Odb.OID roid = base.UpdateNonNativeObjectInfo(nnoi, forceUpdate);
			if (nnoi is NeoDatis.Odb.Core.Server.Layers.Layer2.Meta.ClientNonNativeObjectInfo)
			{
				NeoDatis.Odb.Core.Server.Layers.Layer2.Meta.ClientNonNativeObjectInfo cnnoi = (NeoDatis.Odb.Core.Server.Layers.Layer2.Meta.ClientNonNativeObjectInfo
					)nnoi;
				NeoDatis.Odb.Impl.Core.Server.Transaction.ServerSession session = (NeoDatis.Odb.Impl.Core.Server.Transaction.ServerSession
					)GetSession();
				session.AssociateIds(cnnoi.GetOid(), cnnoi.GetLocalOid());
			}
			return roid;
		}

		/// <summary>FIXME check using a class variable to keep the base identification</summary>
		public override NeoDatis.Odb.Core.Transaction.ISession GetSession()
		{
			return (NeoDatis.Odb.Impl.Core.Server.Transaction.ServerSession)sessionManager.GetSession
				(GetEngine().GetBaseIdentification().GetIdentification(), true);
		}

		public override NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface BuildFSI
			()
		{
			return new NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.ServerFileSystemInterface
				("server-data", storageEngine.GetBaseIdentification(), true, NeoDatis.Odb.OdbConfiguration
				.GetDefaultBufferSizeForData());
		}

		protected virtual NeoDatis.Odb.Core.Trigger.ITriggerManager BuildTriggerManager()
		{
			return NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetServerTriggerManager(storageEngine
				);
		}
	}
}
