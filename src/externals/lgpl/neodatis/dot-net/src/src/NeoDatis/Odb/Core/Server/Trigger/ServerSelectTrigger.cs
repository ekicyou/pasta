namespace NeoDatis.Odb.Core.Server.Trigger
{
	public abstract class ServerSelectTrigger : NeoDatis.Odb.Core.Trigger.SelectTrigger
	{
		public override void AfterSelect(object @object, NeoDatis.Odb.OID oid)
		{
			AfterSelect((NeoDatis.Odb.ObjectRepresentation)@object, oid);
		}

		public abstract void AfterSelect(NeoDatis.Odb.ObjectRepresentation objectRepresentation
			, NeoDatis.Odb.OID oid);
	}
}
