namespace NeoDatis.Odb.Core.Server.Layers.Layer3.Engine
{
	public class ServerFileSystemInterface : NeoDatis.Odb.Core.Layers.Layer3.Engine.FileSystemInterface
	{
		private NeoDatis.Odb.Core.Server.Transaction.ISessionManager sessionManager;

		public ServerFileSystemInterface(string name, NeoDatis.Odb.Core.Layers.Layer3.IBaseIdentification
			 parameters, bool canLog, int bufferSize) : base(name, parameters, canLog, bufferSize
			)
		{
			sessionManager = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClientServerSessionManager
				();
		}

		public override NeoDatis.Odb.Core.Transaction.ISession GetSession()
		{
			return sessionManager.GetSession(parameters.GetIdentification(), true);
		}
	}
}
