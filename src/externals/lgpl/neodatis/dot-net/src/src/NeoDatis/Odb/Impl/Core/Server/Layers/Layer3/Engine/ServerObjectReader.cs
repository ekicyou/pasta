namespace NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Engine
{
	public class ServerObjectReader : NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.ObjectReader
	{
		public ServerObjectReader(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine) : 
			base(engine)
		{
		}

		public override NeoDatis.Odb.Core.Transaction.ISession GetSession()
		{
			return storageEngine.GetSession(true);
		}

		protected override NeoDatis.Odb.Core.Layers.Layer2.Instance.IInstanceBuilder BuildInstanceBuilder
			()
		{
			return NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetServerInstanceBuilder(storageEngine
				);
		}
	}
}
