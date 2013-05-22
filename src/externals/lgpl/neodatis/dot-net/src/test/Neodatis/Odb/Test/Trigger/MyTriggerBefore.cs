using NUnit.Framework;
namespace NeoDatis.Odb.Test.Trigger
{
	public class MyTriggerBefore : NeoDatis.Odb.Core.Trigger.InsertTrigger
	{
		public override void AfterInsert(object @object, NeoDatis.Odb.OID oid)
		{
		}

		public override bool BeforeInsert(object @object)
		{
			NeoDatis.Odb.Test.Trigger.SimpleObject so = (NeoDatis.Odb.Test.Trigger.SimpleObject
				)@object;
			// just add 1
			so.SetId(so.GetId() + 1);
			return true;
		}
	}
}
