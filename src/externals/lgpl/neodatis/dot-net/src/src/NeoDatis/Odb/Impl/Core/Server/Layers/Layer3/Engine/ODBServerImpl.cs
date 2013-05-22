namespace NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Engine
{
	/// <summary>The ODB implementation for Server mode</summary>
	/// <author>osmadja</author>
	public class ODBServerImpl : NeoDatis.Tool.Wrappers.OdbRunnable, NeoDatis.Odb.Core.Server.Layers.Layer3.IODBServerExt
	{
		public static readonly string LogId = "ODBServer";

		private int port;

		private NeoDatis.Tool.Wrappers.OdbThread thread;

		private bool serverIsUp;

		private System.Net.Sockets.TcpListener socketServer;

		private bool isRunning;

		private System.Collections.IDictionary bases;

		private System.Collections.IDictionary connectionManagers;

		private bool automaticallyCreateDatabase;

		public ODBServerImpl(int port)
		{
			NeoDatis.Odb.OdbConfiguration.SetCheckModelCompatibility(false);
			this.port = port;
			this.automaticallyCreateDatabase = true;
			InitServer();
		}

		private void InitServer()
		{
			this.bases = new System.Collections.Hashtable();
			this.connectionManagers = new System.Collections.Hashtable();
			try
			{
				socketServer = new System.Net.Sockets.TcpListener(port);
				isRunning = true;
			}
			catch (Java.Net.BindException e1)
			{
				isRunning = false;
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.Error.ClientServerPortIsBusy
					.AddParameter(port), e1);
			}
			catch (System.IO.IOException e2)
			{
				isRunning = false;
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.Error.ClientServerCanNotOpenOdbServerOnPort
					.AddParameter(port), e2);
			}
		}

		public virtual void AddBase(string baseIdentifier, string fileName)
		{
			AddBase(baseIdentifier, fileName, null, null);
		}

		public virtual void AddBase(string baseIdentifier, string fileName, string user, 
			string password)
		{
			NeoDatis.Odb.Core.Server.Layers.Layer3.ServerFileParameter fileParameter = new NeoDatis.Odb.Core.Server.Layers.Layer3.ServerFileParameter
				(baseIdentifier, fileName, true);
			NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = null;
			engine = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetServerStorageEngine(fileParameter
				, user, password);
			engine.Commit();
			bases.Add(baseIdentifier, engine);
			connectionManagers.Add(baseIdentifier, new NeoDatis.Odb.Core.Server.Connection.ConnectionManager
				(engine));
			if (NeoDatis.Odb.OdbConfiguration.IsInfoEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Info("ODBServer:Adding base : name=" + baseIdentifier + " (file="
					 + fileName + ") to server");
			}
		}

		public virtual void AddUserForBase(string baseIdentifier, string user, string password
			)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.Error.NotYetImplemented
				);
		}

		public virtual void StartServer(bool inThread)
		{
			if (inThread)
			{
				thread = new NeoDatis.Tool.Wrappers.OdbThread(this);
				thread.Start();
			}
			else
			{
				Run();
			}
		}

		public virtual void Run()
		{
			try
			{
				StartServer();
			}
			catch (System.IO.IOException e)
			{
				NeoDatis.Tool.DLogger.Error(NeoDatis.Tool.Wrappers.OdbString.ExceptionToString(e, 
					true));
			}
		}

		/// <exception cref="System.IO.IOException"></exception>
		public virtual void StartServer()
		{
			if (NeoDatis.Odb.OdbConfiguration.IsInfoEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Info("ODBServer: ODB Server on port " + port + " started!");
				NeoDatis.Tool.DLogger.Info("ODBServer: Managed bases: " + bases.Keys);
			}
			while (isRunning)
			{
				try
				{
					WaitForRemoteConnection();
				}
				catch (System.Net.Sockets.SocketException e)
				{
					if (isRunning)
					{
						NeoDatis.Tool.DLogger.Error("ODBServer:ODBServerImpl.startServer:" + NeoDatis.Tool.Wrappers.OdbString
							.ExceptionToString(e, true));
					}
				}
			}
		}

		/// <exception cref="System.IO.IOException"></exception>
		public virtual NeoDatis.Odb.Core.Server.Connection.ClientServerConnection WaitForRemoteConnection
			()
		{
			System.Net.Sockets.TcpClient connection = socketServer.Accept();
			connection.SetTcpNoDelay(true);
			NeoDatis.Odb.Core.Server.Connection.DefaultConnectionThread connectionThread = new 
				NeoDatis.Odb.Core.Server.Connection.DefaultConnectionThread(this, connection, automaticallyCreateDatabase
				);
			Java.Lang.Thread thread = new Java.Lang.Thread(connectionThread);
			connectionThread.SetName(thread.GetName());
			thread.Start();
			return connectionThread;
		}

		public virtual void Close()
		{
			if (NeoDatis.Odb.OdbConfiguration.IsInfoEnabled(LogId))
			{
				NeoDatis.Tool.DLogger.Info("ODBServer:Shutting down ODB Server");
			}
			try
			{
				isRunning = false;
				socketServer.Close();
				System.Collections.IEnumerator iterator = bases.Keys.GetEnumerator();
				string baseIdentifier = null;
				NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = null;
				while (iterator.MoveNext())
				{
					baseIdentifier = (string)iterator.Current;
					engine = (NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine)bases[baseIdentifier];
					if (NeoDatis.Odb.OdbConfiguration.IsInfoEnabled(LogId))
					{
						NeoDatis.Tool.DLogger.Info("ODBServer:Closing Base " + baseIdentifier);
					}
					engine.Close();
				}
				if (thread != null)
				{
					thread.Interrupt();
				}
			}
			catch (System.Exception e)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.Error.ServerError.AddParameter
					("While closing server"), e);
			}
		}

		public virtual void SetAutomaticallyCreateDatabase(bool yes)
		{
			automaticallyCreateDatabase = yes;
		}

		public virtual NeoDatis.Odb.ODB OpenClient(string baseIdentifier)
		{
			return new NeoDatis.Odb.Impl.Main.SameVMODBClient(this, baseIdentifier);
		}

		public virtual System.Collections.IDictionary GetConnectionManagers()
		{
			return connectionManagers;
		}

		public virtual NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter GetParameters(string
			 baseIdentifier, bool clientAndServerRunInSameVM)
		{
			try
			{
				return new NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter(Java.Net.InetAddress
					.GetLocalHost().GetHostName(), port, baseIdentifier, NeoDatis.Odb.Core.Layers.Layer3.IOSocketParameter
					.TypeDatabase, NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs(), null, null, 
					clientAndServerRunInSameVM);
			}
			catch (Java.Net.UnknownHostException e)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.Error.UnknownHost, e
					);
			}
		}

		public virtual void AddDeleteTrigger(string baseIdentifier, string className, NeoDatis.Odb.Core.Server.Trigger.ServerDeleteTrigger
			 trigger)
		{
			NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.IServerStorageEngine engine = (NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.IServerStorageEngine
				)bases[baseIdentifier];
			if (engine == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.Error.UnregisteredBaseOnServer
					.AddParameter(baseIdentifier));
			}
			engine.AddDeleteTriggerFor(className, trigger);
		}

		public virtual void AddInsertTrigger(string baseIdentifier, string className, NeoDatis.Odb.Core.Server.Trigger.ServerInsertTrigger
			 trigger)
		{
			NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.IServerStorageEngine engine = (NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.IServerStorageEngine
				)bases[baseIdentifier];
			if (engine == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.Error.UnregisteredBaseOnServer
					.AddParameter(baseIdentifier));
			}
			engine.AddInsertTriggerFor(className, trigger);
		}

		public virtual void AddSelectTrigger(string baseIdentifier, string className, NeoDatis.Odb.Core.Server.Trigger.ServerSelectTrigger
			 trigger)
		{
			NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.IServerStorageEngine engine = (NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.IServerStorageEngine
				)bases[baseIdentifier];
			if (engine == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.Error.UnregisteredBaseOnServer
					.AddParameter(baseIdentifier));
			}
			engine.AddSelectTriggerFor(className, trigger);
		}

		public virtual void AddUpdateTrigger(string baseIdentifier, string className, NeoDatis.Odb.Core.Server.Trigger.ServerUpdateTrigger
			 trigger)
		{
			NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.IServerStorageEngine engine = (NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.IServerStorageEngine
				)bases[baseIdentifier];
			if (engine == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.Error.UnregisteredBaseOnServer
					.AddParameter(baseIdentifier));
			}
			engine.AddUpdateTriggerFor(className, trigger);
		}
	}
}
