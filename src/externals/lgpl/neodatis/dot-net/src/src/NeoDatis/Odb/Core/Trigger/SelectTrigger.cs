namespace NeoDatis.Odb.Core.Trigger
{
	public abstract class SelectTrigger : NeoDatis.Odb.Core.Trigger.Trigger
	{
		public abstract void AfterSelect(object @object, NeoDatis.Odb.OID oid);
	}
}
