namespace NeoDatis.Odb.Core.Server.Connection
{
	/// <summary>The abstract class that manages the client server connections.</summary>
	/// <remarks>
	/// The abstract class that manages the client server connections. It is message
	/// based and it manages all the client server messages.
	/// </remarks>
	/// <author>olivier s</author>
	public abstract class ClientServerConnection
	{
		private static readonly string LogId = "ClientServerConnection";

		private static int nbMessages = 0;

		protected bool connectionIsUp;

		protected string baseIdentifier;

		protected string connectionId;

		protected bool debug;

		protected bool automaticallyCreateDatabase;

		protected NeoDatis.Odb.Core.Server.Layers.Layer3.IODBServerExt server;

		protected NeoDatis.Odb.Core.Server.Transaction.ISessionManager sessionManager;

		public ClientServerConnection(NeoDatis.Odb.Core.Server.Layers.Layer3.IODBServerExt
			 server, bool automaticallyCreateDatabase)
		{
			this.debug = NeoDatis.Odb.OdbConfiguration.LogServerConnections();
			this.automaticallyCreateDatabase = automaticallyCreateDatabase;
			this.server = server;
			this.sessionManager = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClientServerSessionManager
				();
		}

		public abstract string GetName();

		/// <summary>The main method.</summary>
		/// <remarks>
		/// The main method. It is the message dispatcher. Checks the message type
		/// and calls the right message handler.
		/// </remarks>
		/// <param name="message"></param>
		/// <returns></returns>
		public virtual NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message ManageMessage
			(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message message)
		{
			long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			try
			{
				nbMessages++;
				int commandId = message.GetCommandId();
				switch (commandId)
				{
					case NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.Connect:
					{
						return ManageConnectCommand((NeoDatis.Odb.Core.Server.Message.ConnectMessage)message
							);
					}

					case NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.Get:
					{
						return ManageGetObjectsCommand((NeoDatis.Odb.Core.Server.Message.GetMessage)message
							);
					}

					case NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.GetObjectFromId:
					{
						return ManageGetObjectFromIdCommand((NeoDatis.Odb.Core.Server.Message.GetObjectFromIdMessage
							)message);
					}

					case NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.GetObjectHeaderFromId:
					{
						return ManageGetObjectHeaderFromIdCommand((NeoDatis.Odb.Core.Server.Message.GetObjectHeaderFromIdMessage
							)message);
					}

					case NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.Store:
					{
						return ManageStoreCommand((NeoDatis.Odb.Core.Server.Message.StoreMessage)message);
					}

					case NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.DeleteObject:
					{
						return ManageDeleteObjectCommand((NeoDatis.Odb.Core.Server.Message.DeleteObjectMessage
							)message);
					}

					case NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.Close:
					{
						return ManageCloseCommand((NeoDatis.Odb.Core.Server.Message.CloseMessage)message);
					}

					case NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.Commit:
					{
						return ManageCommitCommand((NeoDatis.Odb.Core.Server.Message.CommitMessage)message
							);
					}

					case NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.Rollback:
					{
						return ManageRollbackCommand((NeoDatis.Odb.Core.Server.Message.RollbackMessage)message
							);
					}

					case NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.DeleteBase:
					{
						return ManageDeleteBaseCommand((NeoDatis.Odb.Core.Server.Message.DeleteBaseMessage
							)message);
					}

					case NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.GetSessions:
					{
						return ManageGetSessionsCommand((NeoDatis.Odb.Core.Server.Message.GetSessionsMessage
							)message);
					}

					case NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.AddUniqueIndex:
					{
						return ManageAddIndexCommand((NeoDatis.Odb.Core.Server.Message.AddIndexMessage)message
							);
					}

					case NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.RebuildIndex:
					{
						return ManageRebuildIndexCommand((NeoDatis.Odb.Core.Server.Message.RebuildIndexMessage
							)message);
					}

					case NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.DeleteIndex:
					{
						return ManageDeleteIndexCommand((NeoDatis.Odb.Core.Server.Message.DeleteIndexMessage
							)message);
					}

					case NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.AddClassInfoList:
					{
						return ManageAddClassInfoListCommand((NeoDatis.Odb.Core.Server.Message.NewClassInfoListMessage
							)message);
					}

					case NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.Count:
					{
						return ManageCountCommand((NeoDatis.Odb.Core.Server.Message.CountMessage)message);
					}

					case NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.GetObjectValues:
					{
						return ManageGetObjectValuesCommand((NeoDatis.Odb.Core.Server.Message.GetObjectValuesMessage
							)message);
					}

					case NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Command.CheckMetaModelCompatibility
						:
					{
						return ManageCheckMetaModelCompatibilityCommand((NeoDatis.Odb.Core.Server.Message.CheckMetaModelCompatibilityMessage
							)message);
					}

					default:
					{
						break;
						break;
					}
				}
				System.Text.StringBuilder buffer = new System.Text.StringBuilder();
				buffer.Append("ODBServer.ConnectionThread:command ").Append(commandId).Append(" not implemented"
					);
				return new NeoDatis.Odb.Core.Server.Message.ErrorMessage("?", "?", buffer.ToString
					());
			}
			finally
			{
				long end = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
				if (debug)
				{
					System.Text.StringBuilder buffer = new System.Text.StringBuilder();
					buffer.Append("[").Append(nbMessages).Append("] ");
					buffer.Append(message.ToString()).Append(" - Thread=").Append(GetName()).Append(" - connectionId ="
						).Append(connectionId).Append(" - duration=").Append((end - start));
					NeoDatis.Tool.DLogger.Info(buffer);
				}
			}
		}

		/// <summary>Used to check if client classes meta model is compatible with the meta model persisted in the database
		/// 	</summary>
		/// <param name="message"></param>
		/// <returns></returns>
		private NeoDatis.Odb.Core.Server.Message.CheckMetaModelCompatibilityMessageResponse
			 ManageCheckMetaModelCompatibilityCommand(NeoDatis.Odb.Core.Server.Message.CheckMetaModelCompatibilityMessage
			 message)
		{
			// Gets the base identifier
			string baseIdentifier = message.GetBaseIdentifier();
			// Gets the connection manager for this base identifier
			NeoDatis.Odb.Core.Server.Connection.ConnectionManager connectionManager = null;
			NeoDatis.Odb.Core.Server.Connection.IConnection connection = null;
			try
			{
				// Gets the connection manager for this base identifier
				connectionManager = GetConnectionManager(baseIdentifier);
				if (connectionManager == null)
				{
					System.Text.StringBuilder buffer = new System.Text.StringBuilder();
					buffer.Append("ODBServer.ConnectionThread:Base ").Append(baseIdentifier).Append(" is not registered on this server!"
						);
					return new NeoDatis.Odb.Core.Server.Message.CheckMetaModelCompatibilityMessageResponse
						(baseIdentifier, message.GetConnectionId(), buffer.ToString());
				}
				connection = connectionManager.GetConnection(message.GetConnectionId());
				NeoDatis.Odb.Impl.Core.Server.Transaction.ServerSession session = (NeoDatis.Odb.Impl.Core.Server.Transaction.ServerSession
					)sessionManager.GetSession(baseIdentifier, true);
				NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = connection.GetStorageEngine
					();
				System.Collections.Generic.IDictionary<string, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
					> currentCIs = message.GetCurrentCIs();
				NeoDatis.Odb.Core.Layers.Layer3.Engine.CheckMetaModelResult result = engine.CheckMetaModelCompatibility
					(currentCIs);
				NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel updatedMetaModel = null;
				if (result.IsModelHasBeenUpdated())
				{
					updatedMetaModel = session.GetMetaModel().Duplicate();
				}
				// If meta model has been updated, returns it to clients
				return new NeoDatis.Odb.Core.Server.Message.CheckMetaModelCompatibilityMessageResponse
					(baseIdentifier, message.GetConnectionId(), result, updatedMetaModel);
			}
			catch (System.Exception e)
			{
				NeoDatis.Tool.DLogger.Error(baseIdentifier + ":Server error while closing", e);
				return new NeoDatis.Odb.Core.Server.Message.CheckMetaModelCompatibilityMessageResponse
					(baseIdentifier, message.GetConnectionId(), NeoDatis.Tool.Wrappers.OdbString.ExceptionToString
					(e, false));
			}
		}

		/// <summary>Manage Index Message</summary>
		/// <param name="message"></param>
		/// <returns></returns>
		private NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message ManageAddIndexCommand
			(NeoDatis.Odb.Core.Server.Message.AddIndexMessage message)
		{
			// Gets the base identifier
			string baseIdentifier = message.GetBaseIdentifier();
			// Gets the connection manager for this base identifier
			NeoDatis.Odb.Core.Server.Connection.ConnectionManager connectionManager = null;
			NeoDatis.Odb.Core.Server.Connection.IConnection connection = null;
			try
			{
				// Gets the connection manager for this base identifier
				connectionManager = GetConnectionManager(baseIdentifier);
				if (connectionManager == null)
				{
					System.Text.StringBuilder buffer = new System.Text.StringBuilder();
					buffer.Append("ODBServer.ConnectionThread:Base ").Append(baseIdentifier).Append(" is not registered on this server!"
						);
					return new NeoDatis.Odb.Core.Server.Message.AddIndexMessageResponse(baseIdentifier
						, message.GetConnectionId(), buffer.ToString());
				}
				connection = connectionManager.GetConnection(message.GetConnectionId());
				NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = connection.GetStorageEngine
					();
				engine.AddIndexOn(message.GetClassName(), message.GetIndexName(), message.GetIndexFieldNames
					(), message.IsVerbose(), message.AcceptMultipleValuesForSameKey());
			}
			catch (System.Exception e)
			{
				NeoDatis.Tool.DLogger.Error(baseIdentifier + ":Server error while closing", e);
				return new NeoDatis.Odb.Core.Server.Message.AddIndexMessageResponse(baseIdentifier
					, message.GetConnectionId(), NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(
					e, false));
			}
			return new NeoDatis.Odb.Core.Server.Message.AddIndexMessageResponse(baseIdentifier
				, message.GetConnectionId());
		}

		/// <summary>Rebuild an index Index Message</summary>
		/// <param name="message"></param>
		/// <returns></returns>
		private NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message ManageRebuildIndexCommand
			(NeoDatis.Odb.Core.Server.Message.RebuildIndexMessage message)
		{
			// Gets the base identifier
			string baseIdentifier = message.GetBaseIdentifier();
			// Gets the connection manager for this base identifier
			NeoDatis.Odb.Core.Server.Connection.ConnectionManager connectionManager = null;
			NeoDatis.Odb.Core.Server.Connection.IConnection connection = null;
			try
			{
				// Gets the connection manager for this base identifier
				connectionManager = GetConnectionManager(baseIdentifier);
				if (connectionManager == null)
				{
					System.Text.StringBuilder buffer = new System.Text.StringBuilder();
					buffer.Append("ODBServer.ConnectionThread:Base ").Append(baseIdentifier).Append(" is not registered on this server!"
						);
					return new NeoDatis.Odb.Core.Server.Message.RebuildIndexMessageResponse(baseIdentifier
						, message.GetConnectionId(), buffer.ToString());
				}
				connection = connectionManager.GetConnection(message.GetConnectionId());
				NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = connection.GetStorageEngine
					();
				engine.RebuildIndex(message.GetClassName(), message.GetIndexName(), message.IsVerbose
					());
			}
			catch (System.Exception e)
			{
				NeoDatis.Tool.DLogger.Error(baseIdentifier + ":Server error while closing", e);
				return new NeoDatis.Odb.Core.Server.Message.RebuildIndexMessageResponse(baseIdentifier
					, message.GetConnectionId(), NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(
					e, false));
			}
			return new NeoDatis.Odb.Core.Server.Message.RebuildIndexMessageResponse(baseIdentifier
				, message.GetConnectionId());
		}

		/// <summary>Delete an index Index Message</summary>
		/// <param name="message"></param>
		/// <returns></returns>
		private NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message ManageDeleteIndexCommand
			(NeoDatis.Odb.Core.Server.Message.DeleteIndexMessage message)
		{
			// Gets the base identifier
			string baseIdentifier = message.GetBaseIdentifier();
			// Gets the connection manager for this base identifier
			NeoDatis.Odb.Core.Server.Connection.ConnectionManager connectionManager = null;
			NeoDatis.Odb.Core.Server.Connection.IConnection connection = null;
			try
			{
				// Gets the connection manager for this base identifier
				connectionManager = GetConnectionManager(baseIdentifier);
				if (connectionManager == null)
				{
					System.Text.StringBuilder buffer = new System.Text.StringBuilder();
					buffer.Append("ODBServer.ConnectionThread:Base ").Append(baseIdentifier).Append(" is not registered on this server!"
						);
					return new NeoDatis.Odb.Core.Server.Message.DeleteIndexMessageResponse(baseIdentifier
						, message.GetConnectionId(), buffer.ToString());
				}
				connection = connectionManager.GetConnection(message.GetConnectionId());
				NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = connection.GetStorageEngine
					();
				engine.DeleteIndex(message.GetClassName(), message.GetIndexName(), message.IsVerbose
					());
			}
			catch (System.Exception e)
			{
				NeoDatis.Tool.DLogger.Error(baseIdentifier + ":Server error while closing", e);
				return new NeoDatis.Odb.Core.Server.Message.DeleteIndexMessageResponse(baseIdentifier
					, message.GetConnectionId(), NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(
					e, false));
			}
			return new NeoDatis.Odb.Core.Server.Message.DeleteIndexMessageResponse(baseIdentifier
				, message.GetConnectionId());
		}

		/// <exception cref="System.Exception"></exception>
		private NeoDatis.Odb.Core.Server.Connection.ConnectionManager GetConnectionManager
			(string baseIdentifier)
		{
			return GetConnectionManager(baseIdentifier, null, null, false);
		}

		/// <summary>Gets the connection manager for the base</summary>
		/// <param name="baseIdentifier"></param>
		/// <param name="user"></param>
		/// <param name="password"></param>
		/// <param name="returnNullIfDoesNotExit"></param>
		/// <returns></returns>
		/// <exception cref="System.Exception">System.Exception</exception>
		private NeoDatis.Odb.Core.Server.Connection.ConnectionManager GetConnectionManager
			(string baseIdentifier, string user, string password, bool returnNullIfDoesNotExit
			)
		{
			try
			{
				// Gets the connection manager for this base identifier
				NeoDatis.Odb.Core.Server.Connection.ConnectionManager connectionManager = (NeoDatis.Odb.Core.Server.Connection.ConnectionManager
					)server.GetConnectionManagers()[baseIdentifier];
				if (connectionManager == null && returnNullIfDoesNotExit)
				{
					return null;
				}
				if (connectionManager == null && automaticallyCreateDatabase)
				{
					server.AddBase(baseIdentifier, baseIdentifier, user, password);
					connectionManager = (NeoDatis.Odb.Core.Server.Connection.ConnectionManager)server
						.GetConnectionManagers()[baseIdentifier];
				}
				if (connectionManager == null && !automaticallyCreateDatabase)
				{
					System.Text.StringBuilder buffer = new System.Text.StringBuilder();
					buffer.Append("ODBServer.ConnectionThread:Base ").Append(baseIdentifier).Append(" is not registered on this server!"
						);
					return null;
				}
				return connectionManager;
			}
			finally
			{
			}
		}

		/// <summary>manages the Close Message</summary>
		/// <param name="message"></param>
		/// <returns></returns>
		private NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message ManageCloseCommand(
			NeoDatis.Odb.Core.Server.Message.CloseMessage message)
		{
			// Gets the base identifier
			string baseIdentifier = message.GetBaseIdentifier();
			// Gets the connection manager for this base identifier
			NeoDatis.Odb.Core.Server.Connection.ConnectionManager connectionManager = null;
			NeoDatis.Odb.Core.Server.Connection.IConnection connection = null;
			NeoDatis.Tool.Mutex.Mutex mutex = null;
			try
			{
				mutex = NeoDatis.Tool.Mutex.MutexFactory.Get(baseIdentifier).Acquire("close");
				// Gets the connection manager for this base identifier
				connectionManager = GetConnectionManager(baseIdentifier);
				if (connectionManager == null)
				{
					System.Text.StringBuilder buffer = new System.Text.StringBuilder();
					buffer.Append("ODBServer.ConnectionThread:Base ").Append(baseIdentifier).Append(" is not registered on this server!"
						);
					return new NeoDatis.Odb.Core.Server.Message.CloseMessageResponse(baseIdentifier, 
						message.GetConnectionId(), buffer.ToString());
				}
				connection = connectionManager.GetConnection(message.GetConnectionId());
				connection.SetCurrentAction(NeoDatis.Odb.Core.Server.Connection.ConnectionAction.
					ActionClose);
				connection.Close();
				connectionManager.RemoveConnection(connection);
				sessionManager.RemoveSession(baseIdentifier);
				connectionIsUp = false;
				return new NeoDatis.Odb.Core.Server.Message.CloseMessageResponse(baseIdentifier, 
					message.GetConnectionId());
			}
			catch (System.Exception e)
			{
				NeoDatis.Tool.DLogger.Error(baseIdentifier + ":Server error while closing", e);
				return new NeoDatis.Odb.Core.Server.Message.CloseMessageResponse(baseIdentifier, 
					message.GetConnectionId(), NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(e, 
					false));
			}
			finally
			{
				if (mutex != null)
				{
					mutex.Release("close");
				}
			}
		}

		private NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message ManageGetSessionsCommand
			(NeoDatis.Odb.Core.Server.Message.GetSessionsMessage message)
		{
			try
			{
				System.Collections.Generic.IList<string> descriptions = sessionManager.GetSessionDescriptions
					(server.GetConnectionManagers());
				return new NeoDatis.Odb.Core.Server.Message.GetSessionsMessageResponse(descriptions
					);
			}
			catch (System.Exception e)
			{
				NeoDatis.Tool.DLogger.Error("Server error while getting session descriptions", e);
				return new NeoDatis.Odb.Core.Server.Message.GetSessionsMessageResponse(NeoDatis.Tool.Wrappers.OdbString
					.ExceptionToString(e, false));
			}
		}

		private NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message ManageCommitCommand
			(NeoDatis.Odb.Core.Server.Message.CommitMessage message)
		{
			// Gets the base identifier
			string baseIdentifier = message.GetBaseIdentifier();
			NeoDatis.Odb.Core.Server.Connection.ConnectionManager connectionManager = null;
			NeoDatis.Odb.Core.Server.Connection.IConnection connection = null;
			NeoDatis.Tool.Mutex.Mutex mutex = null;
			try
			{
				mutex = NeoDatis.Tool.Mutex.MutexFactory.Get(baseIdentifier).Acquire("commit");
				// Gets the connection manager for this base identifier
				connectionManager = GetConnectionManager(baseIdentifier);
				if (connectionManager == null)
				{
					System.Text.StringBuilder buffer = new System.Text.StringBuilder();
					buffer.Append("ODBServer.ConnectionThread:Base ").Append(baseIdentifier).Append(" is not registered on this server!"
						);
					return new NeoDatis.Odb.Core.Server.Message.CommitMessageResponse(baseIdentifier, 
						message.GetConnectionId(), buffer.ToString());
				}
				connection = connectionManager.GetConnection(message.GetConnectionId());
				connection.SetCurrentAction(NeoDatis.Odb.Core.Server.Connection.ConnectionAction.
					ActionCommit);
				connection.Commit();
				return new NeoDatis.Odb.Core.Server.Message.CommitMessageResponse(baseIdentifier, 
					message.GetConnectionId(), true);
			}
			catch (System.Exception e)
			{
				NeoDatis.Tool.DLogger.Error(baseIdentifier + ":Server error while commiting", e);
				return new NeoDatis.Odb.Core.Server.Message.CommitMessageResponse(baseIdentifier, 
					message.GetConnectionId(), NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(e, 
					false));
			}
			finally
			{
				if (mutex != null)
				{
					mutex.Release("commit");
				}
				connection.EndCurrentAction();
			}
		}

		private NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message ManageRollbackCommand
			(NeoDatis.Odb.Core.Server.Message.RollbackMessage message)
		{
			// Gets the base identifier
			string baseIdentifier = message.GetBaseIdentifier();
			NeoDatis.Odb.Core.Server.Connection.ConnectionManager connectionManager = null;
			NeoDatis.Odb.Core.Server.Connection.IConnection connection = null;
			NeoDatis.Tool.Mutex.Mutex mutex = null;
			try
			{
				mutex = NeoDatis.Tool.Mutex.MutexFactory.Get(baseIdentifier).Acquire("rollback");
				// Gets the connection manager for this base identifier
				connectionManager = GetConnectionManager(baseIdentifier);
				if (connectionManager == null)
				{
					System.Text.StringBuilder buffer = new System.Text.StringBuilder();
					buffer.Append("ODBServer.ConnectionThread:Base ").Append(baseIdentifier).Append(" is not registered on this server!"
						);
					return new NeoDatis.Odb.Core.Server.Message.RollbackMessageResponse(baseIdentifier
						, message.GetConnectionId(), buffer.ToString());
				}
				connection = connectionManager.GetConnection(message.GetConnectionId());
				connection.SetCurrentAction(NeoDatis.Odb.Core.Server.Connection.ConnectionAction.
					ActionRollback);
				connection.Rollback();
			}
			catch (System.Exception e)
			{
				NeoDatis.Tool.DLogger.Error(baseIdentifier + ":Server error while rollbacking", e
					);
				return new NeoDatis.Odb.Core.Server.Message.RollbackMessageResponse(baseIdentifier
					, message.GetConnectionId(), NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(
					e, false));
			}
			finally
			{
				if (mutex != null)
				{
					mutex.Release("rollback");
				}
				connection.EndCurrentAction();
			}
			return new NeoDatis.Odb.Core.Server.Message.RollbackMessageResponse(baseIdentifier
				, message.GetConnectionId(), true);
		}

		/// <summary>manage the store command.</summary>
		/// <remarks>
		/// manage the store command. The store command can be an insert(oid==null)
		/// or an update(oid!=null)
		/// If insert get the base mutex IF update, first get the mutex of the oid to
		/// update then get the base mutex, to avoid dead lock in case of concurrent
		/// update.
		/// </remarks>
		/// <param name="message"></param>
		/// <returns></returns>
		private NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message ManageStoreCommand(
			NeoDatis.Odb.Core.Server.Message.StoreMessage message)
		{
			// Gets the base identifier
			string baseIdentifier = message.GetBaseIdentifier();
			NeoDatis.Odb.Core.Server.Connection.ConnectionManager connectionManager = null;
			NeoDatis.Odb.Core.Server.Connection.IConnection connection = null;
			NeoDatis.Tool.Mutex.Mutex mutex = null;
			NeoDatis.Odb.OID oid = message.GetNnoi().GetOid();
			try
			{
				// Gets the connection manager for this base identifier
				connectionManager = GetConnectionManager(baseIdentifier);
				if (connectionManager == null)
				{
					System.Text.StringBuilder buffer = new System.Text.StringBuilder();
					buffer.Append("ODBServer.ConnectionThread:Base ").Append(baseIdentifier).Append(" is not registered on this server!"
						);
					return new NeoDatis.Odb.Core.Server.Message.StoreMessageResponse(baseIdentifier, 
						message.GetConnectionId(), buffer.ToString());
				}
				connection = connectionManager.GetConnection(message.GetConnectionId());
				NeoDatis.Odb.Impl.Core.Server.Transaction.ServerSession session = (NeoDatis.Odb.Impl.Core.Server.Transaction.ServerSession
					)sessionManager.GetSession(baseIdentifier, true);
				NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = connection.GetStorageEngine
					();
				session.SetClientIds(message.GetClientIds());
				bool objectIsNew = oid == NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
					.NullObjectId;
				if (objectIsNew)
				{
					connection.SetCurrentAction(NeoDatis.Odb.Core.Server.Connection.ConnectionAction.
						ActionInsert);
					mutex = NeoDatis.Tool.Mutex.MutexFactory.Get(baseIdentifier).Acquire("store");
					oid = engine.WriteObjectInfo(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
						.NullObjectId, message.GetNnoi(), NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
						.PositionNotInitialized, false);
				}
				else
				{
					connection.SetCurrentAction(NeoDatis.Odb.Core.Server.Connection.ConnectionAction.
						ActionUpdate);
					// If object is not new, ODB is going to execute an Update.
					// Here we must lock the object with the oid to avoid a
					// concurrent update
					// This is done by creating a special mutex with the base
					// identifier and the oid.
					// This mutex will be kept in the connection and only released
					// when committing
					// or rollbacking the connection
					connection.LockObjectWithOid(oid);
					// If object lock is ok, then get the mutex of the database
					mutex = NeoDatis.Tool.Mutex.MutexFactory.Get(baseIdentifier).Acquire("store");
					// If oid is not -1, the object already exist, we must update
					oid = engine.UpdateObject(message.GetNnoi(), false);
				}
				return new NeoDatis.Odb.Core.Server.Message.StoreMessageResponse(baseIdentifier, 
					message.GetConnectionId(), oid, objectIsNew, message.GetClientIds(), session.GetServerIds
					());
			}
			catch (System.Exception e)
			{
				if (oid != null)
				{
					try
					{
						connection.UnlockObjectWithOid(message.GetNnoi().GetOid());
					}
					catch (System.Exception e1)
					{
						NeoDatis.Tool.DLogger.Error("Error while unlocking object with oid " + oid + " : "
							 + NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(e1, true));
					}
				}
				string se = NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(e, false);
				string msg = baseIdentifier + ":Error while storing object " + message.GetNnoi();
				NeoDatis.Tool.DLogger.Error(msg, e);
				return new NeoDatis.Odb.Core.Server.Message.StoreMessageResponse(baseIdentifier, 
					message.GetConnectionId(), msg + ":\n" + se);
			}
			finally
			{
				if (mutex != null)
				{
					mutex.Release("store");
				}
				connection.EndCurrentAction();
			}
		}

		private NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message ManageAddClassInfoListCommand
			(NeoDatis.Odb.Core.Server.Message.NewClassInfoListMessage message)
		{
			// Gets the base identifier
			string baseIdentifier = message.GetBaseIdentifier();
			NeoDatis.Odb.Core.Server.Connection.ConnectionManager connectionManager = null;
			NeoDatis.Odb.Core.Server.Connection.IConnection connection = null;
			NeoDatis.Tool.Mutex.Mutex mutex = null;
			try
			{
				mutex = NeoDatis.Tool.Mutex.MutexFactory.Get(baseIdentifier).Acquire("addClassInfoList"
					);
				// Gets the connection manager for this base identifier
				connectionManager = GetConnectionManager(baseIdentifier);
				if (connectionManager == null)
				{
					System.Text.StringBuilder buffer = new System.Text.StringBuilder();
					buffer.Append("ODBServer.ConnectionThread:Base ").Append(baseIdentifier).Append(" is not registered on this server!"
						);
					return new NeoDatis.Odb.Core.Server.Message.StoreMessageResponse(baseIdentifier, 
						message.GetConnectionId(), buffer.ToString());
				}
				connection = connectionManager.GetConnection(message.GetConnectionId());
				NeoDatis.Odb.Impl.Core.Server.Transaction.ServerSession session = (NeoDatis.Odb.Impl.Core.Server.Transaction.ServerSession
					)sessionManager.GetSession(baseIdentifier, true);
				NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = connection.GetStorageEngine
					();
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoList ciList = message.GetClassInfoList
					();
				ciList = engine.GetObjectWriter().AddClasses(ciList);
				// here we must create a new list with all class info because
				// Serialization hold object references
				// In this case, it holds the reference of the previous class info
				// list. Serialization thinks object did not change so it will send the reference
				// instead of the new object. Creating the new list force the
				// serialization
				// mechanism to send object
				NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
					> allClassInfos = new NeoDatis.Tool.Wrappers.List.OdbArrayList<NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
					>();
				allClassInfos.AddAll(session.GetMetaModel().GetAllClasses());
				NeoDatis.Odb.Core.Server.Message.NewClassInfoListMessageResponse r = new NeoDatis.Odb.Core.Server.Message.NewClassInfoListMessageResponse
					(baseIdentifier, message.GetConnectionId(), allClassInfos);
				session.ResetClassInfoIds();
				return r;
			}
			catch (System.Exception e)
			{
				string se = NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(e, false);
				string msg = baseIdentifier + ":Error while adding new Class Info List" + message
					.GetClassInfoList();
				NeoDatis.Tool.DLogger.Error(msg, e);
				return new NeoDatis.Odb.Core.Server.Message.NewClassInfoListMessageResponse(baseIdentifier
					, message.GetConnectionId(), msg + ":\n" + se);
			}
			finally
			{
				if (mutex != null)
				{
					mutex.Release("addClassInfoList");
				}
			}
		}

		private NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message ManageDeleteObjectCommand
			(NeoDatis.Odb.Core.Server.Message.DeleteObjectMessage message)
		{
			// Gets the base identifier
			string baseIdentifier = message.GetBaseIdentifier();
			NeoDatis.Odb.Core.Server.Connection.ConnectionManager connectionManager = null;
			NeoDatis.Odb.Core.Server.Connection.IConnection connection = null;
			NeoDatis.Tool.Mutex.Mutex mutex = null;
			try
			{
				mutex = NeoDatis.Tool.Mutex.MutexFactory.Get(baseIdentifier).Acquire("deleteObject"
					);
				// Gets the connection manager for this base identifier
				connectionManager = GetConnectionManager(baseIdentifier, null, null, true);
				if (connectionManager == null)
				{
					System.Text.StringBuilder buffer = new System.Text.StringBuilder();
					buffer.Append("ODBServer.ConnectionThread:Base ").Append(baseIdentifier).Append(" is not registered on this server!"
						);
					return new NeoDatis.Odb.Core.Server.Message.StoreMessageResponse(baseIdentifier, 
						message.GetConnectionId(), buffer.ToString());
				}
				connection = connectionManager.GetConnection(message.GetConnectionId());
				connection.SetCurrentAction(NeoDatis.Odb.Core.Server.Connection.ConnectionAction.
					ActionDelete);
				NeoDatis.Odb.Impl.Core.Server.Transaction.ServerSession session = (NeoDatis.Odb.Impl.Core.Server.Transaction.ServerSession
					)sessionManager.GetSession(baseIdentifier, true);
				NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = connection.GetStorageEngine
					();
				engine.DeleteObjectWithOid(message.GetOid());
				return new NeoDatis.Odb.Core.Server.Message.DeleteObjectMessageResponse(baseIdentifier
					, message.GetConnectionId(), message.GetOid());
			}
			catch (System.Exception e)
			{
				string se = NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(e, false);
				string msg = baseIdentifier + ":Error while deleting object " + message.GetOid();
				NeoDatis.Tool.DLogger.Error(msg, e);
				return new NeoDatis.Odb.Core.Server.Message.DeleteObjectMessageResponse(baseIdentifier
					, message.GetConnectionId(), msg + ":\n" + se);
			}
			finally
			{
				if (mutex != null)
				{
					mutex.Release("deleteObject");
				}
				connection.EndCurrentAction();
			}
		}

		private NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message ManageDeleteBaseCommand
			(NeoDatis.Odb.Core.Server.Message.DeleteBaseMessage message)
		{
			// Gets the base identifier
			string baseIdentifier = message.GetBaseIdentifier();
			NeoDatis.Odb.Core.Server.Connection.ConnectionManager connectionManager = null;
			try
			{
				// Gets the connection manager for this base identifier
				connectionManager = GetConnectionManager(baseIdentifier, null, null, true);
				if (connectionManager == null)
				{
					string fileName = message.GetBaseIdentifier();
					NeoDatis.Tool.Wrappers.IO.OdbFile file = new NeoDatis.Tool.Wrappers.IO.OdbFile(fileName
						);
					System.Text.StringBuilder log = new System.Text.StringBuilder();
					try
					{
						if (debug)
						{
							log.Append("Deleting base " + file.GetFullPath()).Append(" | exists?").Append(file
								.Exists());
						}
						if (file.Exists())
						{
							bool b = NeoDatis.Tool.IOUtil.DeleteFile(file.GetFullPath());
							if (debug)
							{
								log.Append("| deleted=").Append(b);
							}
							b = !file.Exists();
							if (debug)
							{
								log.Append("| deleted=").Append(b);
							}
							if (b)
							{
								return new NeoDatis.Odb.Core.Server.Message.DeleteBaseMessageResponse(baseIdentifier
									);
							}
							return new NeoDatis.Odb.Core.Server.Message.DeleteBaseMessageResponse(baseIdentifier
								, "[1] could not delete base " + file.GetFullPath());
						}
						return new NeoDatis.Odb.Core.Server.Message.DeleteBaseMessageResponse(baseIdentifier
							);
					}
					finally
					{
						if (true || debug)
						{
							NeoDatis.Tool.DLogger.Info(log.ToString());
						}
					}
				}
				NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = connectionManager.GetStorageEngine
					();
				if (!engine.IsClosed())
				{
					// Simulate a session
					sessionManager.AddSession(NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetServerSession
						(engine, "temp"));
					// engine.rollback();
					engine.Close();
					sessionManager.RemoveSession(baseIdentifier);
				}
				if (NeoDatis.Tool.IOUtil.DeleteFile(message.GetBaseIdentifier()))
				{
					return new NeoDatis.Odb.Core.Server.Message.DeleteBaseMessageResponse(baseIdentifier
						);
				}
				return new NeoDatis.Odb.Core.Server.Message.DeleteBaseMessageResponse(baseIdentifier
					, "[2] could not delete base " + new NeoDatis.Tool.Wrappers.IO.OdbFile(message.GetBaseIdentifier
					()).GetFullPath());
			}
			catch (System.Exception e)
			{
				string se = NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(e, false);
				string msg = baseIdentifier + ":Error while deleting base " + message.GetBaseIdentifier
					();
				NeoDatis.Tool.DLogger.Error(msg, e);
				return new NeoDatis.Odb.Core.Server.Message.DeleteBaseMessageResponse(baseIdentifier
					, msg + ":\n" + se);
			}
			finally
			{
				sessionManager.RemoveSession(baseIdentifier);
				RemoveConnectionManager(baseIdentifier);
				connectionIsUp = false;
			}
		}

		private void RemoveConnectionManager(string baseId)
		{
			server.GetConnectionManagers().Remove(baseId);
		}

		private NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message ManageGetObjectsCommand
			(NeoDatis.Odb.Core.Server.Message.GetMessage message)
		{
			// Gets the base identifier
			string baseIdentifier = message.GetBaseIdentifier();
			// Gets the connection manager for this base identifier
			NeoDatis.Odb.Core.Server.Connection.ConnectionManager connectionManager = null;
			NeoDatis.Odb.Core.Server.Connection.IConnection connection = null;
			NeoDatis.Tool.Mutex.Mutex mutex = null;
			try
			{
				mutex = NeoDatis.Tool.Mutex.MutexFactory.Get(baseIdentifier).Acquire("getObjects"
					);
				// Gets the connection manager for this base identifier
				connectionManager = GetConnectionManager(baseIdentifier);
				if (connectionManager == null)
				{
					System.Text.StringBuilder buffer = new System.Text.StringBuilder();
					buffer.Append("ODBServer.ConnectionThread:Base ").Append(baseIdentifier).Append(" is not registered on this server!"
						);
					return new NeoDatis.Odb.Core.Server.Message.GetMessageResponse(baseIdentifier, message
						.GetConnectionId(), buffer.ToString());
				}
				connection = connectionManager.GetConnection(message.GetConnectionId());
				connection.SetCurrentAction(NeoDatis.Odb.Core.Server.Connection.ConnectionAction.
					ActionSelect);
				NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = connection.GetStorageEngine
					();
				NeoDatis.Odb.Core.Transaction.ISession session = engine.GetSession(true);
				NeoDatis.Odb.Objects<NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo> metaObjects
					 = null;
				metaObjects = engine.GetObjectInfos<NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo>(message.GetQuery(), true, message.GetStartIndex
					(), message.GetEndIndex(), false);
				// message.getQuery().setStorageEngine(null);
				return new NeoDatis.Odb.Core.Server.Message.GetMessageResponse(baseIdentifier, message
					.GetConnectionId(), metaObjects, message.GetQuery().GetExecutionPlan());
			}
			catch (System.Exception e)
			{
				string se = NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(e, false);
				string msg = baseIdentifier + ":Error while getting objects for query " + message
					.GetQuery();
				NeoDatis.Tool.DLogger.Error(msg, e);
				return new NeoDatis.Odb.Core.Server.Message.GetMessageResponse(baseIdentifier, message
					.GetConnectionId(), msg + ":\n" + se);
			}
			finally
			{
				if (mutex != null)
				{
					mutex.Release("getObjects");
				}
				connection.EndCurrentAction();
			}
		}

		private NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message ManageGetObjectValuesCommand
			(NeoDatis.Odb.Core.Server.Message.GetObjectValuesMessage message)
		{
			// Gets the base identifier
			string baseIdentifier = message.GetBaseIdentifier();
			// Gets the connection manager for this base identifier
			NeoDatis.Odb.Core.Server.Connection.ConnectionManager connectionManager = null;
			NeoDatis.Odb.Core.Server.Connection.IConnection connection = null;
			NeoDatis.Tool.Mutex.Mutex mutex = null;
			try
			{
				mutex = NeoDatis.Tool.Mutex.MutexFactory.Get(baseIdentifier).Acquire("getObjects"
					);
				// Gets the connection manager for this base identifier
				connectionManager = GetConnectionManager(baseIdentifier);
				if (connectionManager == null)
				{
					System.Text.StringBuilder buffer = new System.Text.StringBuilder();
					buffer.Append("ODBServer.ConnectionThread:Base ").Append(baseIdentifier).Append(" is not registered on this server!"
						);
					return new NeoDatis.Odb.Core.Server.Message.GetMessageResponse(baseIdentifier, message
						.GetConnectionId(), buffer.ToString());
				}
				connection = connectionManager.GetConnection(message.GetConnectionId());
				connection.SetCurrentAction(NeoDatis.Odb.Core.Server.Connection.ConnectionAction.
					ActionSelect);
				NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = connection.GetStorageEngine
					();
				NeoDatis.Odb.Values values = engine.GetValues(message.GetQuery(), message.GetStartIndex
					(), message.GetEndIndex());
				return new NeoDatis.Odb.Core.Server.Message.GetObjectValuesMessageResponse(baseIdentifier
					, message.GetConnectionId(), values, message.GetQuery().GetExecutionPlan());
			}
			catch (System.Exception e)
			{
				string se = NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(e, false);
				string msg = baseIdentifier + ":Error while getting objects for query " + message
					.GetQuery();
				NeoDatis.Tool.DLogger.Error(msg, e);
				return new NeoDatis.Odb.Core.Server.Message.GetObjectValuesMessageResponse(baseIdentifier
					, message.GetConnectionId(), msg + ":\n" + se);
			}
			finally
			{
				if (mutex != null)
				{
					mutex.Release("getObjects");
				}
				connection.EndCurrentAction();
			}
		}

		private NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message ManageGetObjectFromIdCommand
			(NeoDatis.Odb.Core.Server.Message.GetObjectFromIdMessage message)
		{
			// Gets the base identifier
			string baseIdentifier = message.GetBaseIdentifier();
			// Gets the connection manager for this base identifier
			NeoDatis.Odb.Core.Server.Connection.ConnectionManager connectionManager = null;
			NeoDatis.Odb.Core.Server.Connection.IConnection connection = null;
			NeoDatis.Odb.OID oid = null;
			NeoDatis.Tool.Mutex.Mutex mutex = null;
			try
			{
				mutex = NeoDatis.Tool.Mutex.MutexFactory.Get(baseIdentifier).Acquire("getObjectFromId"
					);
				// Gets the connection manager for this base identifier
				connectionManager = GetConnectionManager(baseIdentifier);
				if (connectionManager == null)
				{
					System.Text.StringBuilder buffer = new System.Text.StringBuilder();
					buffer.Append("ODBServer.ConnectionThread:Base ").Append(baseIdentifier).Append(" is not registered on this server!"
						);
					return new NeoDatis.Odb.Core.Server.Message.GetObjectFromIdMessageResponse(baseIdentifier
						, message.GetConnectionId(), buffer.ToString());
				}
				connection = connectionManager.GetConnection(message.GetConnectionId());
				connection.SetCurrentAction(NeoDatis.Odb.Core.Server.Connection.ConnectionAction.
					ActionSelect);
				NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = connection.GetStorageEngine
					();
				oid = message.GetOid();
				NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = engine.GetMetaObjectFromOid
					(oid);
				return new NeoDatis.Odb.Core.Server.Message.GetObjectFromIdMessageResponse(baseIdentifier
					, message.GetConnectionId(), nnoi);
			}
			catch (System.Exception e)
			{
				string se = NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(e, false);
				string msg = baseIdentifier + ":Error while getting object of id " + oid;
				NeoDatis.Tool.DLogger.Error(msg, e);
				return new NeoDatis.Odb.Core.Server.Message.GetObjectFromIdMessageResponse(baseIdentifier
					, message.GetConnectionId(), msg + ":\n" + se);
			}
			finally
			{
				if (mutex != null)
				{
					mutex.Release("getObjectFromId");
				}
				connection.EndCurrentAction();
			}
		}

		private NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message ManageGetObjectHeaderFromIdCommand
			(NeoDatis.Odb.Core.Server.Message.GetObjectHeaderFromIdMessage message)
		{
			// Gets the base identifier
			string baseIdentifier = message.GetBaseIdentifier();
			// Gets the connection manager for this base identifier
			NeoDatis.Odb.Core.Server.Connection.ConnectionManager connectionManager = null;
			NeoDatis.Odb.Core.Server.Connection.IConnection connection = null;
			NeoDatis.Odb.OID oid = null;
			NeoDatis.Tool.Mutex.Mutex mutex = null;
			try
			{
				mutex = NeoDatis.Tool.Mutex.MutexFactory.Get(baseIdentifier).Acquire(message.ToString
					());
				// Gets the connection manager for this base identifier
				connectionManager = GetConnectionManager(baseIdentifier);
				if (connectionManager == null)
				{
					System.Text.StringBuilder buffer = new System.Text.StringBuilder();
					buffer.Append("ODBServer.ConnectionThread:Base ").Append(baseIdentifier).Append(" is not registered on this server!"
						);
					return new NeoDatis.Odb.Core.Server.Message.GetObjectHeaderFromIdMessageResponse(
						baseIdentifier, message.GetConnectionId(), buffer.ToString());
				}
				connection = connectionManager.GetConnection(message.GetConnectionId());
				connection.SetCurrentAction(NeoDatis.Odb.Core.Server.Connection.ConnectionAction.
					ActionSelect);
				NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = connection.GetStorageEngine
					();
				oid = message.GetOid();
				NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader oih = engine.GetObjectInfoHeaderFromOid
					(oid);
				// the oih.duplicate method is called to create a new instance of the ObjectInfoHeader becasue of
				// the java Serialization problem : Serialization will check the reference of the object and only send the reference if the object has already
				// been changed. => creating a new will avoid this problem
				return new NeoDatis.Odb.Core.Server.Message.GetObjectHeaderFromIdMessageResponse(
					baseIdentifier, message.GetConnectionId(), oih.Duplicate());
			}
			catch (System.Exception e)
			{
				string se = NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(e, false);
				string msg = baseIdentifier + ":Error while getting object of id " + oid;
				NeoDatis.Tool.DLogger.Error(msg, e);
				return new NeoDatis.Odb.Core.Server.Message.GetObjectHeaderFromIdMessageResponse(
					baseIdentifier, message.GetConnectionId(), msg + ":\n" + se);
			}
			finally
			{
				if (mutex != null)
				{
					mutex.Release("getObjectFromId");
				}
				connection.EndCurrentAction();
			}
		}

		private NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message ManageCountCommand(
			NeoDatis.Odb.Core.Server.Message.CountMessage message)
		{
			// Gets the base identifier
			string baseIdentifier = message.GetBaseIdentifier();
			// Gets the connection manager for this base identifier
			NeoDatis.Odb.Core.Server.Connection.ConnectionManager connectionManager = null;
			NeoDatis.Odb.Core.Server.Connection.IConnection connection = null;
			NeoDatis.Tool.Mutex.Mutex mutex = null;
			try
			{
				mutex = NeoDatis.Tool.Mutex.MutexFactory.Get(baseIdentifier).Acquire("count");
				// Gets the connection manager for this base identifier
				connectionManager = GetConnectionManager(baseIdentifier);
				if (connectionManager == null)
				{
					System.Text.StringBuilder buffer = new System.Text.StringBuilder();
					buffer.Append("ODBServer.ConnectionThread:Base ").Append(baseIdentifier).Append(" is not registered on this server!"
						);
					return new NeoDatis.Odb.Core.Server.Message.GetObjectFromIdMessageResponse(baseIdentifier
						, message.GetConnectionId(), buffer.ToString());
				}
				connection = connectionManager.GetConnection(message.GetConnectionId());
				NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = connection.GetStorageEngine
					();
				NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery query = message.GetCriteriaQuery
					();
				long nbObjects = engine.Count(query);
				return new NeoDatis.Odb.Core.Server.Message.CountMessageResponse(baseIdentifier, 
					message.GetConnectionId(), nbObjects);
			}
			catch (System.Exception e)
			{
				string se = NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(e, false);
				string msg = baseIdentifier + ":Error while counting objects for " + message.GetCriteriaQuery
					();
				NeoDatis.Tool.DLogger.Error(msg, e);
				return new NeoDatis.Odb.Core.Server.Message.CountMessageResponse(baseIdentifier, 
					message.GetConnectionId(), msg + ":\n" + se);
			}
			finally
			{
				if (mutex != null)
				{
					mutex.Release("count");
				}
			}
		}

		private NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message ManageConnectCommand
			(NeoDatis.Odb.Core.Server.Message.ConnectMessage message)
		{
			// Gets the base identifier
			baseIdentifier = message.GetBaseIdentifier();
			// Gets the connection manager for this base identifier
			NeoDatis.Odb.Core.Server.Connection.ConnectionManager connectionManager = null;
			NeoDatis.Odb.Core.Server.Connection.IConnection connection = null;
			NeoDatis.Tool.Mutex.Mutex mutex = null;
			try
			{
				mutex = NeoDatis.Tool.Mutex.MutexFactory.Get(baseIdentifier).Acquire("connect");
				// Gets the connection manager for this base identifier
				connectionManager = GetConnectionManager(baseIdentifier, message.GetUser(), message
					.GetPassword(), false);
				if (connectionManager == null)
				{
					System.Text.StringBuilder buffer = new System.Text.StringBuilder();
					buffer.Append("Base ").Append(baseIdentifier).Append(" is not registered on this server!"
						);
					return new NeoDatis.Odb.Core.Server.Message.ConnectMessageResponse(baseIdentifier
						, "?", buffer.ToString());
				}
				string ip = message.GetIp();
				long dateTime = message.GetDateTime();
				connection = connectionManager.NewConnection(ip, dateTime, connectionManager.GetNbConnections
					());
				connection.SetCurrentAction(NeoDatis.Odb.Core.Server.Connection.ConnectionAction.
					ActionConnect);
				connectionId = connection.GetId();
				// Creates a new session for this connection
				NeoDatis.Odb.Core.Transaction.ISession session = new NeoDatis.Odb.Impl.Core.Server.Transaction.ServerSession
					(connection.GetStorageEngine(), connectionId);
				NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = connection.GetStorageEngine
					();
				// adds the session to the storage engine (it will be associated to
				// the current thread
				// The add session sets the correct meta model
				engine.AddSession(session, true);
				NeoDatis.Odb.TransactionId transactionId = engine.GetCurrentTransactionId();
				if (debug)
				{
					NeoDatis.Tool.DLogger.Info(new System.Text.StringBuilder("Connection from ").Append
						(ip).Append(" - cid=").Append(connection.GetId()).Append(" - session=").Append(session
						.GetId()).Append(" - Base Id=").Append(baseIdentifier).ToString());
				}
				// Returns the meta-model to the client
				NeoDatis.Odb.Core.Layers.Layer2.Meta.MetaModel metaModel = engine.GetSession(true
					).GetMetaModel();
				NeoDatis.Odb.Core.Server.Message.ConnectMessageResponse cmr = new NeoDatis.Odb.Core.Server.Message.ConnectMessageResponse
					(baseIdentifier, connection.GetId(), metaModel, transactionId);
				return cmr;
			}
			catch (System.Exception e)
			{
				string se = NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(e, false);
				string msg = baseIdentifier + ":Error while connecting to  " + message.GetBaseIdentifier
					();
				NeoDatis.Tool.DLogger.Error(msg, e);
				return new NeoDatis.Odb.Core.Server.Message.ConnectMessageResponse(baseIdentifier
					, message.GetConnectionId(), msg + ":\n" + se);
			}
			finally
			{
				if (mutex != null)
				{
					mutex.Release("connect");
				}
				if (connection != null)
				{
					connection.EndCurrentAction();
				}
			}
		}
	}
}
