namespace NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Engine
{
	/// <summary>
	/// Client storage engine used when the client runs in the same Virtual machine
	/// than the client.
	/// </summary>
	/// <remarks>
	/// Client storage engine used when the client runs in the same Virtual machine
	/// than the client. In this case ODB will not execute remote call via IO but it
	/// will pass message (instead of sending them over the network. This can be very
	/// useful for Web Application where Server and client use to run on the same VM.
	/// </remarks>
	/// <author>osmadja</author>
	public class SameVmClientEngine : NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Engine.ClientStorageEngine
	{
		public static readonly string LogId = "SameVmClientEngine";

		protected NeoDatis.Odb.Core.Server.Connection.SameVmConnectionThread connectionThread;

		protected NeoDatis.Odb.Core.Server.Layers.Layer3.IODBServerExt server;

		public SameVmClientEngine(NeoDatis.Odb.Core.Server.Layers.Layer3.IODBServerExt server
			, string baseIdentifier) : base(server.GetParameters(baseIdentifier, true))
		{
			this.server = server;
			// Call super class init
			base.InitODBConnection();
		}

		public override NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message SendMessage
			(NeoDatis.Odb.Core.Server.Layers.Layer3.Engine.Message msg)
		{
			CheckConnectionThread();
			return connectionThread.ManageMessage(msg);
		}

		protected override void InitMessageStreamer()
		{
			this.messageStreamer = null;
		}

		protected override void InitODBConnection()
		{
		}

		// Do nothing here
		// This is called by super class. But it is too early as 'server' attribute is not set yet 
		private void CheckConnectionThread()
		{
			lock (this)
			{
				if (connectionThread == null)
				{
					connectionThread = new NeoDatis.Odb.Core.Server.Connection.SameVmConnectionThread
						(server, true);
				}
			}
		}

		public override void Close()
		{
			if (isClosed)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OdbIsClosed
					.AddParameter(parameters.GetIdentification()));
			}
			NeoDatis.Odb.Core.Server.Message.CloseMessage msg = new NeoDatis.Odb.Core.Server.Message.CloseMessage
				(parameters.GetBaseIdentifier(), connectionId);
			NeoDatis.Odb.Core.Server.Message.CloseMessageResponse rmsg = (NeoDatis.Odb.Core.Server.Message.CloseMessageResponse
				)SendMessage(msg);
			if (rmsg.HasError())
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ServerSideError
					.AddParameter("Error while closing database :").AddParameter(rmsg.GetError()));
			}
			isClosed = true;
			provider.RemoveLocalTriggerManager(this);
		}
	}
}
