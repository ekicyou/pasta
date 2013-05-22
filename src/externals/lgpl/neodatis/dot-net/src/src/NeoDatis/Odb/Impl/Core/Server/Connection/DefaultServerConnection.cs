namespace NeoDatis.Odb.Impl.Core.Server.Connection
{
	public class DefaultServerConnection : NeoDatis.Odb.Core.Server.Connection.IConnection
	{
		public static readonly string LogId = "Connection";

		private string id;

		private NeoDatis.Odb.Core.Server.Connection.ConnectionManager connectionManager;

		private NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine;

		private string baseIdentifier;

		/// <summary>To keep locked id for this session : key = oid, value=timestamp (Long)</summary>
		private System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, long> oidsLockedForUpdate;

		/// <summary>Current action being executed</summary>
		private string currentAction;

		private long currentActionStart;

		private long lastActionDuration;

		private string lastAction;

		private int[] actions;

		public DefaultServerConnection(NeoDatis.Odb.Core.Server.Connection.ConnectionManager
			 connectionManager, string connectionId, NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine
			 storageEngine)
		{
			this.connectionManager = connectionManager;
			this.id = connectionId;
			this.storageEngine = storageEngine;
			this.baseIdentifier = storageEngine.GetBaseIdentification().GetIdentification();
			this.oidsLockedForUpdate = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<NeoDatis.Odb.OID
				, long>();
			actions = new int[NeoDatis.Odb.Core.Server.Connection.ConnectionAction.GetNumberOfActions
				()];
		}

		public virtual string GetId()
		{
			return id;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine GetStorageEngine()
		{
			return storageEngine;
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Close()
		{
			Commit();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Commit()
		{
			storageEngine.Commit();
			NeoDatis.Odb.OID oid = null;
			// release update mutexes
			System.Collections.IEnumerator iterator = oidsLockedForUpdate.Keys.GetEnumerator(
				);
			while (iterator.MoveNext())
			{
				oid = (NeoDatis.Odb.OID)iterator.Current;
				connectionManager.UnlockOidForConnection(oid, this);
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug("Release object lock for " + oid);
				}
			}
			oidsLockedForUpdate.Clear();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void UnlockObjectWithOid(NeoDatis.Odb.OID oid)
		{
			lock (this)
			{
				connectionManager.UnlockOidForConnection(oid, this);
				oidsLockedForUpdate.Remove(oid);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void Rollback()
		{
			storageEngine.Rollback();
			NeoDatis.Odb.OID oid = null;
			// release update mutexes
			System.Collections.IEnumerator iterator = oidsLockedForUpdate.Keys.GetEnumerator(
				);
			while (iterator.MoveNext())
			{
				oid = (NeoDatis.Odb.OID)iterator.Current;
				connectionManager.UnlockOidForConnection(oid, this);
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug("Release object lock for " + oid);
				}
			}
			oidsLockedForUpdate.Clear();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual bool LockObjectWithOid(NeoDatis.Odb.OID oid)
		{
			lock (this)
			{
				connectionManager.LockOidForConnection(oid, this);
				oidsLockedForUpdate.Add(oid, NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs());
				return true;
			}
		}

		public virtual void SetCurrentAction(int action)
		{
			currentActionStart = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			currentAction = NeoDatis.Odb.Core.Server.Connection.ConnectionAction.GetActionLabel
				(action);
			actions[action] = actions[action] + 1;
		}

		public virtual void EndCurrentAction()
		{
			lastAction = currentAction;
			lastActionDuration = (NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs() - currentActionStart
				);
			currentAction = NeoDatis.Odb.Core.Server.Connection.ConnectionAction.ActionNoActionLabel;
		}

		public virtual string GetDescription()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append("cid=" + id).Append("\n\t\t+ Current action : ");
			buffer.Append(currentAction).Append("(").Append(NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs
				() - currentActionStart).Append("ms) | last action : ");
			buffer.Append(lastAction).Append("(").Append(lastActionDuration).Append("ms)");
			buffer.Append("\n\t\t+ Actions : ");
			for (int i = 0; i < actions.Length; i++)
			{
				buffer.Append(NeoDatis.Odb.Core.Server.Connection.ConnectionAction.GetActionLabel
					(i)).Append("=").Append(actions[i]).Append(" | ");
			}
			buffer.Append("\n\t\t+ Blocked Oid (").Append(oidsLockedForUpdate.Count).Append(") : "
				);
			buffer.Append(oidsLockedForUpdate.Keys);
			return buffer.ToString();
		}

		public override bool Equals(object obj)
		{
			if (obj == null || !(obj is NeoDatis.Odb.Impl.Core.Server.Connection.DefaultServerConnection
				))
			{
				return false;
			}
			NeoDatis.Odb.Impl.Core.Server.Connection.DefaultServerConnection c = (NeoDatis.Odb.Impl.Core.Server.Connection.DefaultServerConnection
				)obj;
			return id.Equals(c.id);
		}
	}
}
