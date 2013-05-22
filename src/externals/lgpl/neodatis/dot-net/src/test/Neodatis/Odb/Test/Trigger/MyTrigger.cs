using NUnit.Framework;
namespace NeoDatis.Odb.Test.Trigger
{
	public class MyTrigger : NeoDatis.Odb.Core.Trigger.InsertTrigger
	{
		public int nbInsertsBefore;

		public int nbInsertsAfter;

		public override void AfterInsert(object @object, NeoDatis.Odb.OID oid)
		{
			// println("after insert object with id "+oid+"("+object.getClass().getName()+")");
			nbInsertsAfter++;
		}

		public override bool BeforeInsert(object @object)
		{
			// System.out.println("trigger before inserting " + object);
			nbInsertsBefore++;
			return true;
		}
	}
}
