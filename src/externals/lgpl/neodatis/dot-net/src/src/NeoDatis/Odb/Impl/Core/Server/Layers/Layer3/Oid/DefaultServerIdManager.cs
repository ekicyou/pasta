namespace NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Oid
{
	public class DefaultServerIdManager : NeoDatis.Odb.Impl.Core.Layers.Layer3.Oid.DefaultIdManager
	{
		protected NeoDatis.Odb.Core.Server.Transaction.ISessionManager sessionManager;

		public DefaultServerIdManager(NeoDatis.Odb.Core.Layers.Layer3.IObjectWriter objectWriter
			, NeoDatis.Odb.Core.Layers.Layer3.IObjectReader objectReader, long currentBlockIdPosition
			, int currentBlockIdNumber, NeoDatis.Odb.OID currentMaxId) : base(objectWriter, 
			objectReader, currentBlockIdPosition, currentBlockIdNumber, currentMaxId)
		{
			sessionManager = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClientServerSessionManager
				();
		}

		protected override NeoDatis.Odb.Core.Transaction.ISession GetSession()
		{
			return sessionManager.GetSession(objectReader.GetBaseIdentification(), true);
		}
	}
}
