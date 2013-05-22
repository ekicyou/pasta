namespace NeoDatis.Odb.Test.Trigger
{
	public class MyUpdateTriggerBefore : NeoDatis.Odb.Core.Trigger.UpdateTrigger
	{
		public virtual void AfterInsert(object @object, NeoDatis.Odb.OID oid)
		{
		}

		public virtual bool BeforeInsert(object @object)
		{
			NeoDatis.Odb.Test.Trigger.SimpleObject so = (NeoDatis.Odb.Test.Trigger.SimpleObject
				)@object;
			// just add 1
			so.SetId(so.GetId() + 1);
			return true;
		}

		public override void AfterUpdate(NeoDatis.Odb.ObjectRepresentation oldObjectRepresentation
			, object newObject, NeoDatis.Odb.OID oid)
		{
		}

		// TODO Auto-generated method stub
		public override bool BeforeUpdate(NeoDatis.Odb.ObjectRepresentation oldObjectRepresentation
			, object newObject, NeoDatis.Odb.OID oid)
		{
			NeoDatis.Odb.Test.Trigger.SimpleObject so = (NeoDatis.Odb.Test.Trigger.SimpleObject
				)newObject;
			// just add 1
			so.SetId(so.GetId() + 1);
			return true;
		}
	}
}
