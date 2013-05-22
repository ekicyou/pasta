namespace NeoDatis.Odb.Core.Server.Trigger
{
	public abstract class ServerInsertTrigger : NeoDatis.Odb.Core.Trigger.InsertTrigger
	{
		public override void AfterInsert(object @object, NeoDatis.Odb.OID oid)
		{
			AfterInsert((NeoDatis.Odb.ObjectRepresentation)@object, oid);
		}

		public override bool BeforeInsert(object @object)
		{
			return BeforeInsert((NeoDatis.Odb.ObjectRepresentation)@object);
		}

		public abstract bool BeforeInsert(NeoDatis.Odb.ObjectRepresentation objectRepresentation
			);

		public abstract void AfterInsert(NeoDatis.Odb.ObjectRepresentation objectRepresentation
			, NeoDatis.Odb.OID oid);
	}
}
