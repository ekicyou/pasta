namespace NeoDatis.Odb.Core.Trigger
{
	public abstract class InsertTrigger : NeoDatis.Odb.Core.Trigger.Trigger
	{
		public abstract bool BeforeInsert(object @object);

		public abstract void AfterInsert(object @object, NeoDatis.Odb.OID oid);
	}
}
