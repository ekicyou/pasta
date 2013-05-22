using NUnit.Framework;
namespace NeoDatis.Odb.Test.Intropector
{
	public class IntrospectorWithNestedClasses : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			DeleteBase("test-nested");
			NeoDatis.Odb.ODB odb = Open("test-nested");
			NeoDatis.Odb.Test.Intropector.Class1 c1 = new NeoDatis.Odb.Test.Intropector.Class1
				("name1", "name2", "name3");
			NeoDatis.Odb.OID oid = odb.Store(c1);
			odb.Close();
			odb = Open("test-nested");
			NeoDatis.Odb.Test.Intropector.Class1 c11 = (NeoDatis.Odb.Test.Intropector.Class1)
				odb.GetObjectFromId(oid);
			AssertEquals(c1.GetName1(), c11.GetName1());
			AssertEquals(c1.GetClass2().GetClass3().GetName3(), c11.GetClass2().GetClass3().GetName3
				());
		}
	}
}
