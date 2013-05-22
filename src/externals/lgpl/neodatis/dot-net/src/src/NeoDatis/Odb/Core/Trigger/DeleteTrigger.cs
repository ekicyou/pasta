namespace NeoDatis.Odb.Core.Trigger
{
	public abstract class DeleteTrigger : NeoDatis.Odb.Core.Trigger.Trigger
	{
		public abstract bool BeforeDelete(object @object, NeoDatis.Odb.OID oid);

		public abstract void AfterDelete(object @object, NeoDatis.Odb.OID oid);
	}
}
