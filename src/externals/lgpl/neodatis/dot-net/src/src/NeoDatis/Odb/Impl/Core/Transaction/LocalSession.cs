namespace NeoDatis.Odb.Impl.Core.Transaction
{
	/// <summary>The session object used when ODB is used in local/embedded mode</summary>
	/// <author>olivier s</author>
	public class LocalSession : NeoDatis.Odb.Impl.Core.Transaction.Session
	{
		private NeoDatis.Odb.Core.Transaction.ITransaction transaction;

		private NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface fsiToApplyTransaction;

		private NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine;

		public LocalSession(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine, string
			 sessionId) : base(sessionId, engine.GetBaseIdentification().GetIdentification()
			)
		{
			this.storageEngine = engine;
		}

		public LocalSession(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine) : this
			(engine, "local " + NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs() + NeoDatis.Tool.Wrappers.OdbRandom
			.GetRandomInteger())
		{
		}

		public override void SetFileSystemInterfaceToApplyTransaction(NeoDatis.Odb.Core.Layers.Layer3.Engine.IFileSystemInterface
			 fsi)
		{
			fsiToApplyTransaction = fsi;
			if (transaction != null)
			{
				transaction.SetFsiToApplyWriteActions(fsiToApplyTransaction);
			}
		}

		public override NeoDatis.Odb.Core.Transaction.ITransaction GetTransaction()
		{
			if (transaction == null)
			{
				transaction = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetTransaction(this
					, fsiToApplyTransaction);
			}
			return transaction;
		}

		public override bool TransactionIsPending()
		{
			if (transaction == null)
			{
				return false;
			}
			return transaction.GetNumberOfWriteActions() != 0;
		}

		private void ResetTranstion()
		{
			if (transaction != null)
			{
				transaction.Clear();
				transaction = null;
			}
		}

		public override void Commit()
		{
			if (transaction != null)
			{
				transaction.Commit();
				transaction.Reset();
			}
		}

		public override void Rollback()
		{
			if (transaction != null)
			{
				transaction.Rollback();
				ResetTranstion();
			}
			base.Rollback();
		}

		public override NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine GetStorageEngine()
		{
			return storageEngine;
		}

		public override void Clear()
		{
			base.Clear();
			if (transaction != null)
			{
				transaction.Clear();
			}
			storageEngine = null;
		}

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
