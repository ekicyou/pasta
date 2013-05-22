namespace NeoDatis.Odb.Impl.Main
{
	public class ODBForTrigger : NeoDatis.Odb.Impl.Main.ODBAdapter
	{
		public ODBForTrigger(NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine storageEngine
			) : base(storageEngine)
		{
		}

		public virtual void AddDeleteTrigger(NeoDatis.Odb.Core.Trigger.DeleteTrigger trigger
			)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotAllowedInTrigger
				);
		}

		public virtual void AddInsertTrigger(NeoDatis.Odb.Core.Trigger.InsertTrigger trigger
			)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotAllowedInTrigger
				);
		}

		public virtual void AddSelectTrigger(NeoDatis.Odb.Core.Trigger.SelectTrigger trigger
			)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotAllowedInTrigger
				);
		}

		public virtual void AddUpdateTrigger(NeoDatis.Odb.Core.Trigger.UpdateTrigger trigger
			)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotAllowedInTrigger
				);
		}

		public override void Close()
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotAllowedInTrigger
				);
		}

		public override void Commit()
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotAllowedInTrigger
				);
		}

		public override void CommitAndClose()
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotAllowedInTrigger
				);
		}

		public override void DefragmentTo(string newFileName)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotAllowedInTrigger
				);
		}

		public override void Disconnect(object @object)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotAllowedInTrigger
				);
		}

		public override NeoDatis.Odb.ClassRepresentation GetClassRepresentation(System.Type
			 clazz)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotAllowedInTrigger
				);
		}

		public override NeoDatis.Odb.ClassRepresentation GetClassRepresentation(string fullClassName
			)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotAllowedInTrigger
				);
		}

		public override NeoDatis.Odb.Core.Layers.Layer3.IRefactorManager GetRefactorManager
			()
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotAllowedInTrigger
				);
		}

		public override NeoDatis.Odb.Core.Transaction.ISession GetSession()
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotAllowedInTrigger
				);
		}

		public override void Reconnect(object @object)
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotAllowedInTrigger
				);
		}

		public override void Rollback()
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotAllowedInTrigger
				);
		}

		public override void Run()
		{
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.OperationNotAllowedInTrigger
				);
		}
	}
}
