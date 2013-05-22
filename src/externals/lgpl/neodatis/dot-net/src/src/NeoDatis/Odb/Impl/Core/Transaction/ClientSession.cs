namespace NeoDatis.Odb.Impl.Core.Transaction
{
	/// <summary>The client session when ODB is used in client server mode</summary>
	/// <author>olivier s</author>
	public class ClientSession : NeoDatis.Odb.Impl.Core.Transaction.Session
	{
		protected NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine;

		public ClientSession(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine) : base
			("client", engine.GetBaseIdentification().GetIdentification())
		{
			this.engine = engine;
		}

		public override NeoDatis.Odb.Core.Transaction.ICache GetCache()
		{
			return cache;
		}

		public override void Commit()
		{
		}

		public override void Rollback()
		{
			base.Rollback();
		}

		public override void Close()
		{
			Clear();
		}

		public override void ClearCache()
		{
			cache.Clear(false);
		}

		public override bool IsRollbacked()
		{
			return rollbacked;
		}

		public override void Clear()
		{
			base.Clear();
		}

		public override NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine GetStorageEngine()
		{
			return engine;
		}

		public override bool TransactionIsPending()
		{
			// TODO do this right
			return false;
		}

		public override NeoDatis.Odb.Core.Transaction.ITransaction GetTransaction()
		{
			// TODO Auto-generated method stub
			return null;
		}

		public override void SetFileSystemInterfaceToApplyTransaction(NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface
			 fsi)
		{
		}

		// TODO Auto-generated method stub
		public override NeoDatis.Odb.Core.Transaction.ICache BuildCache()
		{
			return NeoDatis.Odb.Impl.Core.Transaction.CacheFactory.GetLocalCache(this, "permanent"
				);
		}

		public override NeoDatis.Odb.Core.Transaction.ITmpCache BuildTmpCache()
		{
			return NeoDatis.Odb.Impl.Core.Transaction.CacheFactory.GetLocalTmpCache(this, "tmp"
				);
		}
	}
}
