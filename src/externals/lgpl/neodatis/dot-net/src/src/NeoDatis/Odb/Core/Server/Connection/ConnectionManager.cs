namespace NeoDatis.Odb.Core.Server.Connection
{
	public class ConnectionManager
	{
		public static readonly string LogId = "IConnectionManager";

		private NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine;

		private System.Collections.Generic.IDictionary<string, NeoDatis.Odb.Core.Server.Connection.IConnection
			> connections;

		private System.Collections.Generic.IDictionary<NeoDatis.Odb.OID, NeoDatis.Odb.Core.Server.Connection.IConnection
			> lockedOids;

		public ConnectionManager(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine)
		{
			// A map that contains oids that are locked. The key is the oid, the value is the connection that hold the object
			this.storageEngine = engine;
			connections = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, NeoDatis.Odb.Core.Server.Connection.IConnection
				>();
			lockedOids = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<NeoDatis.Odb.OID, NeoDatis.Odb.Core.Server.Connection.IConnection
				>();
		}

		public virtual NeoDatis.Odb.Core.Server.Connection.IConnection NewConnection(string
			 ip, long dateTime, int sequence)
		{
			string connectionId = NeoDatis.Odb.Core.Server.Connection.ConnectionIdGenerator.NewId
				(ip, dateTime, sequence);
			NeoDatis.Odb.Core.Server.Connection.IConnection connection = new NeoDatis.Odb.Impl.Core.Server.Connection.DefaultServerConnection
				(this, connectionId, storageEngine);
			connections.Add(connectionId, connection);
			return connection;
		}

		public virtual NeoDatis.Odb.Core.Server.Connection.IConnection GetConnection(string
			 connectionId)
		{
			NeoDatis.Odb.Core.Server.Connection.IConnection c = (NeoDatis.Odb.Core.Server.Connection.IConnection
				)connections[connectionId];
			if (c == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ClientServerConnectionIsNull
					.AddParameter(connectionId).AddParameter(connections));
			}
			return c;
		}

		public virtual void RemoveConnection(NeoDatis.Odb.Core.Server.Connection.IConnection
			 connection)
		{
			connections.Remove(connection.GetId());
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine GetStorageEngine()
		{
			return storageEngine;
		}

		public virtual int GetNbConnections()
		{
			return connections.Count;
		}

		public virtual string GetConnectionDescriptions()
		{
			System.Collections.IEnumerator iterator = connections.Values.GetEnumerator();
			NeoDatis.Odb.Core.Server.Connection.IConnection connection = null;
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			while (iterator.MoveNext())
			{
				connection = (NeoDatis.Odb.Core.Server.Connection.IConnection)iterator.Current;
				buffer.Append("\n\t+ ").Append(connection.GetDescription()).Append("\n");
			}
			return buffer.ToString();
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void LockOidForConnection(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Server.Connection.IConnection
			 connection)
		{
			lock (this)
			{
				long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
					NeoDatis.Tool.DLogger.Debug("Trying to lock object with oid " + oid + " - id=" + 
						connection.GetId());
				}
				try
				{
					NeoDatis.Odb.Core.Server.Connection.IConnection c = lockedOids[oid];
					if (c == null)
					{
						lockedOids.Add(oid, connection);
						return;
					}
					// If oid is locked for by the passed connection, no problem, it is not considered as being locked
					if (c != null && c.Equals(connection))
					{
						return;
					}
					while (c != null)
					{
						NeoDatis.Tool.Wrappers.OdbThread.Sleep(10);
						c = lockedOids[oid];
					}
					lockedOids.Add(oid, connection);
				}
				finally
				{
					if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
					{
						NeoDatis.Tool.DLogger.Debug("Object with oid " + oid + " locked (" + (NeoDatis.Tool.Wrappers.OdbTime
							.GetCurrentTimeInMs() - start) + "ms) - " + connection.GetId());
					}
				}
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void UnlockOidForConnection(NeoDatis.Odb.OID oid, NeoDatis.Odb.Core.Server.Connection.IConnection
			 connection)
		{
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
			{
				start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				NeoDatis.Tool.DLogger.Debug("Trying to unlock lock object with oid " + oid + " - id="
					 + connection.GetId());
			}
			try
			{
				lockedOids.Remove(oid);
			}
			finally
			{
				if (NeoDatis.Odb.OdbConfiguration.IsDebugEnabled(LogId))
				{
					NeoDatis.Tool.DLogger.Debug("Object with oid " + oid + " unlocked (" + (NeoDatis.Tool.Wrappers.OdbTime
						.GetCurrentTimeInMs() - start) + "ms) - " + connection.GetId());
				}
			}
		}
	}
}
