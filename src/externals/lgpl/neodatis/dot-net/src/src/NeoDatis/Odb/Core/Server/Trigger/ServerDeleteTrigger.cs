namespace NeoDatis.Odb.Core.Server.Trigger
{
	public abstract class ServerDeleteTrigger : NeoDatis.Odb.Core.Trigger.DeleteTrigger
	{
		public override void AfterDelete(object @object, NeoDatis.Odb.OID oid)
		{
			AfterDelete((NeoDatis.Odb.ObjectRepresentation)@object, oid);
		}

		public override bool BeforeDelete(object @object, NeoDatis.Odb.OID oid)
		{
			return BeforeDelete((NeoDatis.Odb.ObjectRepresentation)@object, oid);
		}

		public abstract bool BeforeDelete(NeoDatis.Odb.ObjectRepresentation objectRepresentation
			, NeoDatis.Odb.OID oid);

		public abstract void AfterDelete(NeoDatis.Odb.ObjectRepresentation objectRepresentation
			, NeoDatis.Odb.OID oid);
	}
}
