namespace NeoDatis.Odb.Impl.Core.Layers.Layer2.Instance
{
	public class ServerInstanceBuilder : NeoDatis.Odb.Impl.Core.Layers.Layer2.Instance.InstanceBuilder
	{
		public ServerInstanceBuilder(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine
			) : base(engine)
		{
		}

		protected override NeoDatis.Odb.Core.Transaction.ISession GetSession()
		{
			return engine.GetSession(true);
		}
	}
}
