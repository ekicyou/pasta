namespace NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine
{
	public class LocalObjectWriter : NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter
	{
		private NeoDatis.Odb.Core.Transaction.ISession session;

		public LocalObjectWriter(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine) : 
			base(engine)
		{
			this.session = engine.GetSession(true);
		}

		public override NeoDatis.Odb.Core.Transaction.ISession GetSession()
		{
			return session;
		}

		public override NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface BuildFSI
			()
		{
			return new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.LocalFileSystemInterface("local-data"
				, GetSession(), storageEngine.GetBaseIdentification(), true, NeoDatis.Odb.OdbConfiguration
				.GetDefaultBufferSizeForData());
		}

		protected virtual NeoDatis.Odb.Core.Trigger.ITriggerManager BuildTriggerManager()
		{
			return NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetLocalTriggerManager(storageEngine
				);
		}
	}
}
