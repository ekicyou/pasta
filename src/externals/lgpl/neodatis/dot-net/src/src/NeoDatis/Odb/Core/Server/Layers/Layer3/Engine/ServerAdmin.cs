using NeoDatis.Odb.Core.Server.Layers.Layer3.Engine;

namespace NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Engine
{
	public class ServerAdmin
	{
		virtual public System.String Host
		{
			get
			{
				return host;
			}
			
		}
		virtual public int Port
		{
			get
			{
				return port;
			}
			
		}
		private System.IO.Stream outStream;
		
		private System.IO.Stream inStream;
		
		//UPGRADE_TODO: Class 'java.io.ObjectOutputStream' was converted to 'System.IO.BinaryWriter' which has a different behavior. "ms-help://MS.VSCC.v80/dv_commoner/local/redirect.htm?index='!DefaultContextWindowIndex'&keyword='jlca1073_javaioObjectOutputStream'"
		private System.IO.BinaryWriter oos;
		
		//UPGRADE_TODO: Class 'java.io.ObjectInputStream' was converted to 'System.IO.BinaryReader' which has a different behavior. "ms-help://MS.VSCC.v80/dv_commoner/local/redirect.htm?index='!DefaultContextWindowIndex'&keyword='jlca1073_javaioObjectInputStream'"
		private System.IO.BinaryReader ois;
		
		private System.String host;
		
		private int port;
		
		private System.Net.Sockets.TcpClient socket;
		
		public ServerAdmin(System.String host, int port)
		{
			this.host = host;
			this.port = port;
		}
		
		public virtual void  close()
		{
			closeSocket();
		}
		
		//UPGRADE_NOTE: Synchronized keyword was removed from method 'initSocket'. Lock expression was added. "ms-help://MS.VSCC.v80/dv_commoner/local/redirect.htm?index='!DefaultContextWindowIndex'&keyword='jlca1027'"
		private void  initSocket()
		{
			lock (this)
			{
				if (socket == null)
				{
					socket = new System.Net.Sockets.TcpClient(host, port);
					outStream = socket.GetStream();
					inStream = socket.GetStream();
					oos = new System.IO.BinaryWriter(outStream);
					ois = new System.IO.BinaryReader(inStream);
				}
			}
		}
		
		private void  closeSocket()
		{
			ois.Close();
			oos.Close();
			inStream.Close();
			outStream.Close();
			socket.Close();
		}
		
		public virtual Message sendMessage(Message msg)
		{
			if (socket == null)
			{
				initSocket();
			}
			/*
			MessageStreamer.write(oos, msg);
			Message rmsg = MessageStreamer.read(ois);
			// closeSocket();
			return rmsg;
			*/
			return null;
		}
	}
}
