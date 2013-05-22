namespace NeoDatis.Odb.Core.Server.Trigger
{
	public abstract class ServerUpdateTrigger : NeoDatis.Odb.Core.Trigger.UpdateTrigger
	{
		public override void AfterUpdate(NeoDatis.Odb.ObjectRepresentation oldObjectRepresentation
			, object newObject, NeoDatis.Odb.OID oid)
		{
			AfterUpdate(oldObjectRepresentation, (NeoDatis.Odb.ObjectRepresentation)newObject
				, oid);
		}

		public override bool BeforeUpdate(NeoDatis.Odb.ObjectRepresentation oldObjectRepresentation
			, object newObject, NeoDatis.Odb.OID oid)
		{
			return BeforeUpdate(oldObjectRepresentation, (NeoDatis.Odb.ObjectRepresentation)newObject
				, oid);
		}

		public abstract bool BeforeUpdate(NeoDatis.Odb.ObjectRepresentation oldObjectRepresentation
			, NeoDatis.Odb.ObjectRepresentation newObjectRepresentation, NeoDatis.Odb.OID oid
			);

		public abstract void AfterUpdate(NeoDatis.Odb.ObjectRepresentation oldObjectRepresentation
			, NeoDatis.Odb.ObjectRepresentation newObjectRepresentation, NeoDatis.Odb.OID oid
			);
	}
}
