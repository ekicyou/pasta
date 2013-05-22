namespace NeoDatis.Odb.Core.Server.Connection
{
	/// <summary>A thread to manage client connections via socket</summary>
	/// <author>olivier s</author>
	public class DefaultConnectionThread : NeoDatis.Odb.Core.Server.Connection.ClientServerConnection
		, NeoDatis.Tool.Wrappers.OdbRunnable
	{
		private static readonly string LogId = "DefaultConnectionThread";

		private System.Net.Sockets.TcpClient socketConnection;

		private string name;

		public DefaultConnectionThread(NeoDatis.Odb.Core.Server.Layers.Layer3.IODBServerExt
			 server, System.Net.Sockets.TcpClient connection, bool automaticallyCreateDatabase
			) : base(server, automaticallyCreateDatabase)
		{
			this.socketConnection = connection;
		}

		public virtual void Run()
		{
			
			/*
			System.IO.Stream @out = null;
			System.IO.Stream @in = null;
			System.IO.BinaryWriter oos = null;
			System.IO.BinaryReader ois = null;
			string messageType = null;
			try
			{
				// socketConnection.setKeepAlive(true);
				// socketConnection.setSoTimeout(0);
				//socketConnection.SetTcpNoDelay(true);
				connectionIsUp = true;
				@out = socketConnection.GetOutputStream();
				@in = socketConnection.GetInputStream();
				oos = new System.IO.BinaryWriter(@out);
				ois = new System.IO.BinaryReader(new Java.IO.BufferedInputStream(@in));
				NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Engine.MessageStreamer messageStreamer
					 = new NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Engine.MessageStreamer(@in, @out
					, ois, oos);
				NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message message = null;
				NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message rmessage = null;
				do
				{
					message = null;
					message = messageStreamer.Read();
					if (message != null)
					{
						messageType = message.GetType().FullName;
						rmessage = ManageMessage(message);
						messageStreamer.Write(rmessage);
					}
					else
					{
						messageType = "Null Message";
					}
				}
				while (connectionIsUp && message != null);
			}
			catch (Java.IO.EOFException)
			{
				// To force disconnection
				// connectionIsUp = false;
				NeoDatis.Tool.DLogger.Error("Thread " + NeoDatis.Tool.Wrappers.OdbThread.GetCurrentThreadName
					() + ": Error in connection thread baseId=" + baseIdentifier + " and cid=" + connectionId
					 + " for message of type " + messageType + " : Client has terminated the connection!"
					);
				connectionIsUp = false;
			}
			catch (System.IO.IOException e)
			{
				NeoDatis.Tool.DLogger.Error("Thread " + NeoDatis.Tool.Wrappers.OdbThread.GetCurrentThreadName
					() + ": Error in connection thread baseId=" + baseIdentifier + " and cid=" + connectionId
					 + " for message of type " + messageType + " : \n" + NeoDatis.Tool.StringUtils.ExceptionToString
					(e, false));
				connectionIsUp = false;
			}
			catch (System.TypeLoadException e)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.Error.NetSerialisationError
					, e);
			}
			try
			{
				oos.Flush();
				oos.Close();
				ois.Close();
				socketConnection.Close();
			}
			catch (System.IO.IOException e)
			{
				NeoDatis.Tool.DLogger.Error("Error while closing socket - connection thread baseId="
					 + baseIdentifier + " and cid=" + connectionId + ": \n" + NeoDatis.Tool.StringUtils
					.ExceptionToString(e, false));
			}
			if (debug)
			{
				NeoDatis.Tool.DLogger.Info("Exiting thread " + NeoDatis.Tool.Wrappers.OdbThread.GetCurrentThreadName
					());
			}
			*/
		}

		public override string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}
	}
}
