namespace NeoDatis.Odb.Core.Trigger
{
	public abstract class UpdateTrigger : NeoDatis.Odb.Core.Trigger.Trigger
	{
		public abstract bool BeforeUpdate(NeoDatis.Odb.ObjectRepresentation oldObjectRepresentation
			, object newObject, NeoDatis.Odb.OID oid);

		public abstract void AfterUpdate(NeoDatis.Odb.ObjectRepresentation oldObjectRepresentation
			, object newObject, NeoDatis.Odb.OID oid);
	}
}
