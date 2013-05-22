using NUnit.Framework;
namespace NeoDatis.Odb.Test.Trigger
{
	public class MySelectTrigger : NeoDatis.Odb.Core.Trigger.SelectTrigger
	{
		public int nbCalls;

		public override void AfterSelect(object @object, NeoDatis.Odb.OID oid)
		{
			nbCalls++;
			System.Console.Out.WriteLine("Select on object with oid " + oid);
		}
	}
}
