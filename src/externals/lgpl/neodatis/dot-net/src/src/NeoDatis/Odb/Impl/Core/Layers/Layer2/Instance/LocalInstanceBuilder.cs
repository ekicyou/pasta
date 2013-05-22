namespace NeoDatis.Odb.Impl.Core.Layers.Layer2.Instance
{
	public class LocalInstanceBuilder : NeoDatis.Odb.Impl.Core.Layers.Layer2.Instance.InstanceBuilder
	{
		private NeoDatis.Odb.Core.Transaction.ISession session;

		public LocalInstanceBuilder(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine
			) : base(engine)
		{
			this.session = engine.GetSession(true);
		}

		protected override NeoDatis.Odb.Core.Transaction.ISession GetSession()
		{
			return session;
		}
	}
}
